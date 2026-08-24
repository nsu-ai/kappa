// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Model artifact upload/download helpers.
//!
//! Kappa caps GUI/sync artifact uploads and rejects oversized ones with
//! `413 USE_KAPPA_APK`; large files go through an S3 multipart upload session that this SDK
//! drives. On the download side a package manifest lists every artifact, and the legacy
//! single ZIP is refused with `409 PACKAGE_TOO_LARGE_FOR_ZIP` once a package outgrows it.
//! Every call falls back to the older ZIP-only behaviour when a backend has no package or
//! upload-session routes.

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::traits::ApiClient;

const MODELS_ROOT: &str = "/model-micro-services/v2/models";

/// Sync uploads above this size go straight to an upload session
/// (`MODEL_ARTIFACT_SYNC_MAX_BYTES`, 512 MiB by default server-side).
const SYNC_UPLOAD_MAX_BYTES: u64 = 512 * 1024 * 1024;

/// Presigned URLs are fetched in blocks so a 100 GB file costs ~16 calls instead of ~1600.
const PART_URL_BATCH: usize = 100;

/// Retries per part before the whole upload fails; each retry re-presigns the URL.
const DEFAULT_PART_RETRIES: u32 = 5;

/// Attempts to wait out `429 MODEL_ARTIFACT_UPLOAD_ADMISSION_LIMIT` when creating a session.
const DEFAULT_ADMISSION_RETRIES: u32 = 5;

/// Only hash artifacts up to this size by default — hashing re-reads the whole file.
const DEFAULT_CHECKSUM_MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// 6 = Prediction output (Kappa ≥ 2.14); 1–5 are the original artifact categories.
pub const FILE_CATEGORY_PREDICTION_OUTPUT: i32 = 6;

pub fn validate_file_category(category: i32) -> PyResult<()> {
    if (1..=6).contains(&category) {
        Ok(())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "file_category must be 1–6 (1 Training, 2 Inference, 3 Model, 4 Data, 5 Other, 6 Prediction output)",
        ))
    }
}

pub fn inference_file_upload_endpoint(
    model_id: &str,
    inference_id: i32,
    category: i32,
    entity_id: Option<&str>,
    field_name: Option<&str>,
) -> String {
    let mut url = format!(
        "/model-micro-services/v2/models/inferences/files/{}/{}?file_category={}",
        model_id, inference_id, category
    );
    if let Some(id) = entity_id.map(str::trim).filter(|s| !s.is_empty()) {
        url.push_str("&entityId=");
        url.push_str(&urlencoding::encode(id));
    }
    if let Some(name) = field_name.map(str::trim).filter(|s| !s.is_empty()) {
        url.push_str("&fieldName=");
        url.push_str(&urlencoding::encode(name));
    }
    url
}

/// Options for a single session upload; defaults come from [`SessionUploadOptions::default`].
pub struct SessionUploadOptions {
    /// 1 Training, 2 Inference, 3 Model, 4 Data, 5 Other, 6 Prediction output (server default 3).
    pub file_category: Option<i32>,
    /// Called after each part with `(bytes_sent, total_bytes, percent)`.
    pub on_progress: Option<PyObject>,
    /// Send `checksumSha256` so the server can record the artifact digest.
    pub checksum: bool,
    /// Reuse a still-pending session recorded by an earlier interrupted run.
    pub resume: bool,
    /// Retries per part.
    pub max_retries: u32,
    /// Wait and retry when the server is at its concurrent-upload limit.
    pub wait_for_slot: bool,
}

impl Default for SessionUploadOptions {
    fn default() -> Self {
        Self {
            file_category: None,
            on_progress: None,
            checksum: false,
            resume: true,
            max_retries: DEFAULT_PART_RETRIES,
            wait_for_slot: true,
        }
    }
}

/// Resume record for an interrupted multipart upload.
#[derive(Serialize, Deserialize)]
struct UploadState {
    upload_id: String,
    file_name: String,
    bytes: u64,
    mtime: u64,
    part_size: u64,
    parts: Vec<(i32, String)>,
}

pub struct ModelArtifactsApi;

impl ModelArtifactsApi {
    // --- download ---

    /// `GET /models/inferences/{model_id}/{inference_id}/package` — artifact manifest.
    pub fn get_model_inference_artifacts_package<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/inferences/{}/{}/package",
            MODELS_ROOT, model_id, inference_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /models/versions/{model_id}/{version_id}/artifacts/package` — artifact manifest
    /// of the inference linked to a model version.
    pub fn get_model_version_artifacts_package<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/versions/{}/{}/artifacts/package",
            MODELS_ROOT, model_id, version_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /models/inferences/files/{model_id}/{inference_id}/{file_id}` — write one
    /// artifact to `dest_path`.
    ///
    /// `redirect=true` asks for a presigned URL (302) instead of streaming through the
    /// gateway; only use it when the object storage is reachable from this machine.
    pub fn download_model_inference_artifact_file<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        file_id: &str,
        dest_path: &str,
        redirect: bool,
    ) -> PyResult<String> {
        let mut endpoint = format!(
            "{}/inferences/files/{}/{}/{}",
            MODELS_ROOT, model_id, inference_id, file_id
        );
        if redirect {
            endpoint.push_str("?redirect=true");
        }
        let bytes = client.download_bytes(endpoint, Some(client.require_token()?))?;
        write_bytes(dest_path, bytes)
    }

    /// Download every artifact of an inference into `dest_dir`, one file per manifest entry.
    ///
    /// Falls back to the single ZIP when the manifest says `legacySingleZip` or the backend
    /// has no package route; returns the paths written.
    pub fn download_model_inference_artifacts_package<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        dest_dir: &str,
        redirect: bool,
    ) -> PyResult<Vec<String>> {
        let manifest = match Self::get_model_inference_artifacts_package(
            client,
            model_id,
            inference_id,
        ) {
            Ok(obj) => crate::utils::python_json::pyobject_to_rust_value(
                &obj,
                "model artifact package manifest",
            )?,
            Err(e) => {
                let msg = e.to_string();
                if is_missing_route(&msg) {
                    // Pre-package backends only expose the ZIP.
                    let dest = Path::new(dest_dir)
                        .join(format!("inference_{}_artifacts.zip", inference_id));
                    let path = Self::download_inference_artifacts_zip(
                        client,
                        model_id,
                        inference_id,
                        &dest.to_string_lossy(),
                    )?;
                    return Ok(vec![path]);
                }
                return Err(e);
            }
        };

        if manifest
            .get("legacySingleZip")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            let dest =
                Path::new(dest_dir).join(format!("inference_{}_artifacts.zip", inference_id));
            let path = Self::download_inference_artifacts_zip(
                client,
                model_id,
                inference_id,
                &dest.to_string_lossy(),
            )?;
            return Ok(vec![path]);
        }

        let files = manifest
            .get("files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if files.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Artifact package manifest lists no files",
            ));
        }

        fs::create_dir_all(dest_dir).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create {}: {}",
                dest_dir, e
            ))
        })?;

        let mut written = Vec::new();
        for file in &files {
            let file_id = file.get("fileId").and_then(|v| v.as_str()).ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Artifact package entry is missing fileId",
                )
            })?;
            let file_name = file
                .get("fileName")
                .and_then(|v| v.as_str())
                .unwrap_or(file_id);
            // Manifest names may carry directories; keep only the leaf.
            let leaf = Path::new(file_name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(file_id);
            let dest = Path::new(dest_dir).join(leaf);
            written.push(Self::download_model_inference_artifact_file(
                client,
                model_id,
                inference_id,
                file_id,
                &dest.to_string_lossy(),
                redirect,
            )?);
        }
        Ok(written)
    }

    /// `GET /models/inferences/files/{model_id}/{inference_id}/zip`.
    ///
    /// Errors with a pointer to the package flow when the package is too large to zip.
    pub fn download_inference_artifacts_zip<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        dest_path: &str,
    ) -> PyResult<String> {
        let endpoint = format!(
            "{}/inferences/files/{}/{}/zip",
            MODELS_ROOT, model_id, inference_id
        );
        let bytes = client
            .download_bytes(endpoint, Some(client.require_token()?))
            .map_err(annotate_zip_error)?;
        write_bytes(dest_path, bytes)
    }

    /// `GET /models/versions/{model_id}/{version_id}/artifacts/zip`.
    pub fn download_version_artifacts_zip<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
        dest_path: &str,
    ) -> PyResult<String> {
        let endpoint = format!(
            "{}/versions/{}/{}/artifacts/zip",
            MODELS_ROOT, model_id, version_id
        );
        let bytes = client
            .download_bytes(endpoint, Some(client.require_token()?))
            .map_err(annotate_zip_error)?;
        write_bytes(dest_path, bytes)
    }

    // --- upload session (large artifacts) ---

    /// `POST /models/inferences/files/{model_id}/{inference_id}/upload-session`.
    ///
    /// Note the server picks the part size; read `partSize` / `partCount` off the response.
    /// `file_category` defaults to 3 (Model) here, unlike the sync upload's 2 (Inference).
    #[allow(clippy::too_many_arguments)]
    pub fn create_model_artifact_upload_session<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        file_name: &str,
        bytes_expected: u64,
        content_type: Option<String>,
        file_category: Option<i32>,
        checksum_sha256: Option<String>,
    ) -> PyResult<PyObject> {
        if file_name.is_empty()
            || file_name.contains('/')
            || file_name.contains('\\')
            || file_name.contains("..")
        {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "file_name must be a bare name without path separators or '..'",
            ));
        }
        if bytes_expected == 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "bytes_expected must be greater than 0",
            ));
        }
        if let Some(category) = file_category {
            validate_file_category(category)?;
        }

        let mut body = serde_json::Map::new();
        body.insert("fileName".to_string(), serde_json::json!(file_name));
        body.insert("bytesExpected".to_string(), serde_json::json!(bytes_expected));
        if let Some(ct) = content_type {
            body.insert("contentType".to_string(), serde_json::json!(ct));
        }
        if let Some(category) = file_category {
            body.insert("fileCategory".to_string(), serde_json::json!(category));
        }
        if let Some(sum) = checksum_sha256 {
            body.insert("checksumSha256".to_string(), serde_json::json!(sum));
        }
        let endpoint = format!(
            "{}/inferences/files/{}/{}/upload-session",
            MODELS_ROOT, model_id, inference_id
        );
        client.make_request(
            "POST".to_string(),
            endpoint,
            Some(serde_json::Value::Object(body).to_string()),
            Some(client.require_token()?),
        )
    }

    pub fn get_model_artifact_upload_session<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        upload_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/inferences/files/{}/{}/upload-session/{}",
            MODELS_ROOT, model_id, inference_id, upload_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `POST .../upload-session/{upload_id}/part-urls` — presigned PUT URLs for part numbers.
    pub fn get_model_artifact_upload_part_urls<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        upload_id: &str,
        part_numbers: Vec<i32>,
    ) -> PyResult<PyObject> {
        if part_numbers.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "part_numbers must not be empty",
            ));
        }
        let endpoint = format!(
            "{}/inferences/files/{}/{}/upload-session/{}/part-urls",
            MODELS_ROOT, model_id, inference_id, upload_id
        );
        let body = serde_json::json!({ "partNumbers": part_numbers }).to_string();
        client.make_request("POST".to_string(), endpoint, Some(body), Some(client.require_token()?))
    }

    /// `POST .../upload-session/{upload_id}/complete` — finish the multipart upload.
    ///
    /// `parts` is a list of `(part_number, etag)` in any order.
    pub fn complete_model_artifact_upload_session<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        upload_id: &str,
        parts: Vec<(i32, String)>,
    ) -> PyResult<PyObject> {
        if parts.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "parts must not be empty",
            ));
        }
        let parts_json: Vec<serde_json::Value> = parts
            .into_iter()
            .map(|(number, etag)| serde_json::json!({ "partNumber": number, "etag": etag }))
            .collect();
        let endpoint = format!(
            "{}/inferences/files/{}/{}/upload-session/{}/complete",
            MODELS_ROOT, model_id, inference_id, upload_id
        );
        let body = serde_json::json!({ "parts": parts_json }).to_string();
        client.make_request("POST".to_string(), endpoint, Some(body), Some(client.require_token()?))
    }

    /// `DELETE .../upload-session/{upload_id}` — abort a pending session.
    pub fn abort_model_artifact_upload_session<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        upload_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/inferences/files/{}/{}/upload-session/{}",
            MODELS_ROOT, model_id, inference_id, upload_id
        );
        client.make_request("DELETE".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// Upload a large artifact through an upload session, part by part.
    ///
    /// Creates the session, uploads each part to its presigned URL using the server's part
    /// size, then completes it. A failure aborts the session before returning.
    /// `on_progress` is called after each part with `(bytes_sent, total_bytes, percent)`.
    pub fn upload_model_artifact_session<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        file_path: &str,
        file_category: Option<i32>,
        on_progress: Option<PyObject>,
    ) -> PyResult<PyObject> {
        Self::upload_model_artifact_session_with(
            client,
            model_id,
            inference_id,
            file_path,
            SessionUploadOptions {
                file_category,
                on_progress,
                ..SessionUploadOptions::default()
            },
        )
    }

    /// Session upload with retry / resume / admission control.
    ///
    /// Parts are retried with backoff against a freshly presigned URL, and the upload id plus
    /// part ETags are checkpointed to the cache dir so an interrupted 100 GB shard resumes
    /// where it stopped instead of starting over. The session is aborted (and its checkpoint
    /// dropped) when the upload finally fails.
    pub fn upload_model_artifact_session_with<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        file_path: &str,
        options: SessionUploadOptions,
    ) -> PyResult<PyObject> {
        let metadata = fs::metadata(file_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Cannot read file {}: {}",
                file_path, e
            ))
        })?;
        let total_bytes = metadata.len();
        let mtime = file_mtime(&metadata);
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("artifact.bin")
            .to_string();

        let state_path = state_file_path(model_id, inference_id, file_path, total_bytes, mtime);
        let resumed = if options.resume {
            state_path.as_ref().and_then(|path| {
                Self::resume_session(client, model_id, inference_id, path, total_bytes, mtime)
            })
        } else {
            None
        };

        let (upload_id, part_size, done_parts) = match resumed {
            Some(state) => {
                let parts: HashMap<i32, String> = state.parts.iter().cloned().collect();
                (state.upload_id, state.part_size, parts)
            }
            None => {
                let checksum = if options.checksum {
                    Some(sha256_file(file_path)?)
                } else {
                    None
                };
                let session = Self::create_session_with_admission(
                    client,
                    model_id,
                    inference_id,
                    &file_name,
                    total_bytes,
                    options.file_category,
                    checksum,
                    options.wait_for_slot,
                )?;
                let session_value =
                    crate::utils::python_json::pyobject_to_rust_value(&session, "upload session")?;
                let upload_id = session_value
                    .get("uploadId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Upload session response has no uploadId",
                        )
                    })?
                    .to_string();
                let part_size = session_value
                    .get("partSize")
                    .and_then(|v| v.as_u64())
                    .filter(|v| *v > 0)
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Upload session response has no usable partSize",
                        )
                    })?;
                (upload_id, part_size, HashMap::new())
            }
        };

        let part_count = total_bytes.div_ceil(part_size).max(1);
        let outcome = Self::upload_parts(
            client,
            model_id,
            inference_id,
            &upload_id,
            file_path,
            part_size,
            part_count,
            total_bytes,
            done_parts,
            &options,
            state_path.as_deref().map(|path| StateSink {
                path,
                upload_id: &upload_id,
                file_name: &file_name,
                bytes: total_bytes,
                mtime,
                part_size,
            }),
        );
        match outcome {
            Ok(parts) => {
                let completed = Self::complete_model_artifact_upload_session(
                    client,
                    model_id,
                    inference_id,
                    &upload_id,
                    parts,
                );
                if let Some(path) = state_path.as_ref() {
                    let _ = fs::remove_file(path);
                }
                completed
            }
            Err(e) => {
                let _ = Self::abort_model_artifact_upload_session(
                    client,
                    model_id,
                    inference_id,
                    &upload_id,
                );
                if let Some(path) = state_path.as_ref() {
                    let _ = fs::remove_file(path);
                }
                Err(e)
            }
        }
    }

    /// Create a session, waiting out the server's concurrent-upload admission limit.
    #[allow(clippy::too_many_arguments)]
    fn create_session_with_admission<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        file_name: &str,
        total_bytes: u64,
        file_category: Option<i32>,
        checksum: Option<String>,
        wait_for_slot: bool,
    ) -> PyResult<PyObject> {
        let attempts = if wait_for_slot {
            DEFAULT_ADMISSION_RETRIES
        } else {
            0
        };
        let mut last: Option<PyErr> = None;
        for attempt in 0..=attempts {
            match Self::create_model_artifact_upload_session(
                client,
                model_id,
                inference_id,
                file_name,
                total_bytes,
                None,
                file_category,
                checksum.clone(),
            ) {
                Ok(session) => return Ok(session),
                Err(e) if is_admission_limit(&e.to_string()) && attempt < attempts => {
                    sleep_backoff(attempt, 5);
                    last = Some(e);
                }
                Err(e) if is_admission_limit(&e.to_string()) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "{} Hint: the server allows only a couple of concurrent large uploads per model — \
                         upload shards one at a time, or abort stale sessions with abort_model_artifact_upload_session().",
                        e
                    )));
                }
                Err(e) => return Err(e),
            }
        }
        Err(last.unwrap_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to create upload session")
        }))
    }

    /// Load a checkpoint and confirm the server still has that session pending.
    fn resume_session<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        state_path: &Path,
        total_bytes: u64,
        mtime: u64,
    ) -> Option<UploadState> {
        let raw = fs::read_to_string(state_path).ok()?;
        let state: UploadState = serde_json::from_str(&raw).ok()?;
        if state.bytes != total_bytes || state.mtime != mtime || state.part_size == 0 {
            let _ = fs::remove_file(state_path);
            return None;
        }
        let session = Self::get_model_artifact_upload_session(
            client,
            model_id,
            inference_id,
            &state.upload_id,
        )
        .ok()?;
        let value =
            crate::utils::python_json::pyobject_to_rust_value(&session, "upload session").ok()?;
        let pending = value
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("pending"))
            .unwrap_or(false);
        let same_layout = value
            .get("partSize")
            .and_then(|v| v.as_u64())
            .map(|size| size == state.part_size)
            .unwrap_or(false);
        if pending && same_layout {
            Some(state)
        } else {
            let _ = fs::remove_file(state_path);
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn upload_parts<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        upload_id: &str,
        file_path: &str,
        part_size: u64,
        part_count: u64,
        total_bytes: u64,
        done_parts: HashMap<i32, String>,
        options: &SessionUploadOptions,
        state: Option<StateSink<'_>>,
    ) -> PyResult<Vec<(i32, String)>> {
        let mut file = fs::File::open(file_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Cannot open file {}: {}",
                file_path, e
            ))
        })?;
        let http = client.get_http_client();
        let runtime = client.get_runtime();

        let mut parts = done_parts;
        let mut urls: HashMap<i32, String> = HashMap::new();
        let mut bytes_sent: u64 = parts
            .keys()
            .map(|n| part_len(*n as u64, part_size, total_bytes))
            .sum();

        for part_number in 1..=part_count {
            let number = part_number as i32;
            if parts.contains_key(&number) {
                continue;
            }
            let offset = (part_number - 1) * part_size;
            let chunk_len = part_len(part_number, part_size, total_bytes) as usize;
            if chunk_len == 0 {
                continue;
            }
            file.seek(SeekFrom::Start(offset)).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "Failed to seek to part {} of {}: {}",
                    part_number, file_path, e
                ))
            })?;
            let mut buffer = vec![0_u8; chunk_len];
            file.read_exact(&mut buffer).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "Failed to read part {} of {}: {}",
                    part_number, file_path, e
                ))
            })?;

            let mut attempt = 0;
            let etag = loop {
                if !urls.contains_key(&number) {
                    Self::presign_batch(
                        client,
                        model_id,
                        inference_id,
                        upload_id,
                        number,
                        part_count,
                        &parts,
                        &mut urls,
                    )?;
                }
                let url = urls.get(&number).cloned().ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "No presigned URL returned for part {}",
                        part_number
                    ))
                })?;

                match put_part(&http, &runtime, &url, buffer.clone()) {
                    Ok(etag) => break etag,
                    Err(_) if attempt < options.max_retries => {
                        // The URL may simply have expired mid-upload; drop it and back off.
                        urls.remove(&number);
                        sleep_backoff(attempt, 1);
                        attempt += 1;
                    }
                    Err(e) => {
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                            "Part {} of {} failed after {} attempts: {}",
                            part_number,
                            part_count,
                            options.max_retries + 1,
                            e
                        )));
                    }
                }
            };

            parts.insert(number, etag);
            urls.remove(&number);
            bytes_sent += chunk_len as u64;
            if let Some(sink) = state.as_ref() {
                sink.save(&parts);
            }

            if let Some(callback) = options.on_progress.as_ref() {
                let percent = if total_bytes == 0 {
                    100
                } else {
                    ((bytes_sent as f64 / total_bytes as f64) * 100.0).round() as i64
                };
                Python::with_gil(|py| -> PyResult<()> {
                    callback.bind(py).call1((bytes_sent, total_bytes, percent))?;
                    Ok(())
                })?;
            }
        }

        let mut ordered: Vec<(i32, String)> = parts.into_iter().collect();
        ordered.sort_by_key(|(number, _)| *number);
        Ok(ordered)
    }

    /// Presign the next block of outstanding parts, starting at `from`.
    #[allow(clippy::too_many_arguments)]
    fn presign_batch<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        upload_id: &str,
        from: i32,
        part_count: u64,
        done: &HashMap<i32, String>,
        urls: &mut HashMap<i32, String>,
    ) -> PyResult<()> {
        let wanted: Vec<i32> = (from..=part_count as i32)
            .filter(|n| !done.contains_key(n) && !urls.contains_key(n))
            .take(PART_URL_BATCH)
            .collect();
        if wanted.is_empty() {
            return Ok(());
        }
        let response = Self::get_model_artifact_upload_part_urls(
            client,
            model_id,
            inference_id,
            upload_id,
            wanted,
        )?;
        let value =
            crate::utils::python_json::pyobject_to_rust_value(&response, "upload part urls")?;
        for part in value
            .get("parts")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
        {
            let number = part.get("partNumber").and_then(|v| v.as_i64());
            let url = part.get("url").and_then(|v| v.as_str());
            if let (Some(number), Some(url)) = (number, url) {
                urls.insert(number as i32, url.to_string());
            }
        }
        Ok(())
    }

    /// True when a file must use the upload session instead of the sync endpoint.
    pub fn requires_upload_session(byte_len: u64) -> bool {
        byte_len > SYNC_UPLOAD_MAX_BYTES
    }

    /// Default policy for hashing an artifact when the caller did not decide.
    pub fn should_checksum(byte_len: u64) -> bool {
        byte_len <= DEFAULT_CHECKSUM_MAX_BYTES
    }

    /// Upload a whole artifact set — shard files, a directory, or a mix — to one inference.
    ///
    /// Files go up one at a time because the server admits only a couple of concurrent large
    /// uploads per model. Each entry picks its own transport: the sync POST for sidecars, an
    /// upload session for anything past the sync cap. With `skip_existing` the package
    /// manifest is read first and same-name/same-size artifacts are left alone, which makes
    /// a re-run after a failure cheap.
    ///
    /// `on_progress` is called as `(file_name, bytes_sent, total_bytes, percent)`.
    #[allow(clippy::too_many_arguments)]
    pub fn upload_model_artifacts<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        paths: Vec<String>,
        file_category: Option<i32>,
        use_session: Option<bool>,
        skip_existing: bool,
        checksum: Option<bool>,
        on_progress: Option<PyObject>,
    ) -> PyResult<serde_json::Value> {
        let files = expand_artifact_paths(&paths)?;
        if files.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No artifact files found in the given paths",
            ));
        }

        let existing: HashMap<String, u64> = if skip_existing {
            Self::existing_artifact_sizes(client, model_id, inference_id)
        } else {
            HashMap::new()
        };

        let category = file_category.unwrap_or(3);
        let mut uploaded = Vec::new();
        for path in files {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            let display = path.to_string_lossy().to_string();
            let bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

            if existing.get(&name) == Some(&bytes) {
                uploaded.push(serde_json::json!({
                    "fileName": name,
                    "path": display,
                    "bytes": bytes,
                    "transport": "skipped",
                    "skipped": true,
                }));
                continue;
            }

            let session = use_session.unwrap_or_else(|| Self::requires_upload_session(bytes));
            let response = if session {
                let progress = on_progress
                    .as_ref()
                    .map(|cb| Python::with_gil(|py| bind_file_progress(py, cb, &name)))
                    .transpose()?;
                Self::upload_model_artifact_session_with(
                    client,
                    model_id,
                    inference_id,
                    &display,
                    SessionUploadOptions {
                        file_category: Some(category),
                        on_progress: progress,
                        checksum: checksum.unwrap_or_else(|| Self::should_checksum(bytes)),
                        ..SessionUploadOptions::default()
                    },
                )?
            } else {
                // Auto mode keeps the 413 fallback so a server with a lower sync cap than
                // ours still lands the file; an explicit use_session=False forbids it.
                crate::models_api::ModelsApi::upload_model_inference_file(
                    client,
                    model_id,
                    inference_id,
                    &display,
                    Some(category),
                    false,
                    use_session,
                    None,
                    None,
                )?
            };

            let value = crate::utils::python_json::pyobject_to_rust_value(
                &response,
                "artifact upload response",
            )
            .unwrap_or(serde_json::Value::Null);
            uploaded.push(serde_json::json!({
                "fileName": name,
                "path": display,
                "bytes": bytes,
                "transport": if session { "session" } else { "sync" },
                "skipped": false,
                "fileId": value.get("fileId").cloned().unwrap_or(serde_json::Value::Null),
                "response": value,
            }));
        }
        Ok(serde_json::Value::Array(uploaded))
    }

    /// Artifact name → byte size for what the inference already holds.
    fn existing_artifact_sizes<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
    ) -> HashMap<String, u64> {
        let manifest = match Self::get_model_inference_artifacts_package(
            client,
            model_id,
            inference_id,
        ) {
            Ok(obj) => crate::utils::python_json::pyobject_to_rust_value(&obj, "package manifest")
                .unwrap_or(serde_json::Value::Null),
            Err(_) => return HashMap::new(),
        };
        manifest
            .get("files")
            .and_then(|v| v.as_array())
            .map(|files| {
                files
                    .iter()
                    .filter_map(|file| {
                        let name = file.get("fileName").and_then(|v| v.as_str())?;
                        let bytes = file.get("bytes").and_then(|v| v.as_u64())?;
                        Some((name.to_string(), bytes))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Wrap a 4-arg queue callback so the session uploader can call it with 3 args.
fn bind_file_progress(py: Python<'_>, callback: &PyObject, file_name: &str) -> PyResult<PyObject> {
    let functools = py.import("functools")?;
    let partial = functools.getattr("partial")?;
    Ok(partial.call1((callback.bind(py), file_name))?.into())
}

/// Expand files and directories into a stable, de-duplicated file list.
fn expand_artifact_paths(paths: &[String]) -> PyResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for raw in paths {
        let path = Path::new(raw);
        if path.is_dir() {
            let mut found: Vec<PathBuf> = walkdir::WalkDir::new(path)
                .sort_by_file_name()
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .map(|entry| entry.into_path())
                .collect();
            files.append(&mut found);
        } else if path.is_file() {
            files.push(path.to_path_buf());
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Artifact path does not exist: {}",
                raw
            )));
        }
    }
    // The server stores artifacts by bare file name, so duplicates would overwrite each other.
    let mut seen = std::collections::HashSet::new();
    files.retain(|path| {
        path.file_name()
            .and_then(|s| s.to_str())
            .map(|name| seen.insert(name.to_string()))
            .unwrap_or(false)
    });
    Ok(files)
}

/// Checkpoint writer for a resumable upload.
struct StateSink<'a> {
    path: &'a Path,
    upload_id: &'a str,
    file_name: &'a str,
    bytes: u64,
    mtime: u64,
    part_size: u64,
}

impl StateSink<'_> {
    fn save(&self, parts: &HashMap<i32, String>) {
        let mut ordered: Vec<(i32, String)> =
            parts.iter().map(|(n, etag)| (*n, etag.clone())).collect();
        ordered.sort_by_key(|(number, _)| *number);
        let state = UploadState {
            upload_id: self.upload_id.to_string(),
            file_name: self.file_name.to_string(),
            bytes: self.bytes,
            mtime: self.mtime,
            part_size: self.part_size,
            parts: ordered,
        };
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(&state) {
            let _ = fs::write(self.path, json);
        }
    }
}

/// PUT one part to object storage and return its ETag, releasing the GIL while it flies.
fn put_part(
    http: &reqwest::Client,
    runtime: &std::sync::Arc<tokio::runtime::Runtime>,
    url: &str,
    body: Vec<u8>,
) -> PyResult<String> {
    let http = http.clone();
    let url = url.to_string();
    Python::with_gil(|py| {
        py.allow_threads(move || {
            runtime.block_on(async move {
                let response = http.put(&url).body(body).send().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Part upload request failed: {}",
                        e
                    ))
                })?;
                let status = response.status();
                if !status.is_success() {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Part upload failed with HTTP {}",
                        status.as_u16()
                    )));
                }
                response
                    .headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.trim_matches('"').to_string())
                    .ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                            "Object storage did not return an ETag for the uploaded part",
                        )
                    })
            })
        })
    })
}

/// Bytes covered by `part_number` (the last part is short).
fn part_len(part_number: u64, part_size: u64, total_bytes: u64) -> u64 {
    let offset = (part_number - 1).saturating_mul(part_size);
    total_bytes.saturating_sub(offset).min(part_size)
}

fn file_mtime(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Checkpoint location, keyed by everything that would invalidate a resume.
fn state_file_path(
    model_id: &str,
    inference_id: i32,
    file_path: &str,
    bytes: u64,
    mtime: u64,
) -> Option<PathBuf> {
    let absolute = fs::canonicalize(file_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.to_string());
    let key = format!(
        "{}|{}|{}|{}|{}",
        model_id, inference_id, absolute, bytes, mtime
    );
    let digest = format!("{:x}", Sha256::digest(key.as_bytes()));
    let base = dirs::cache_dir().or_else(|| crate::utils::cache_paths::ensure_temp_dir().ok())?;
    Some(
        base.join("kappa-apk")
            .join("uploads")
            .join(format!("{}.json", &digest[..32])),
    )
}

fn sha256_file(file_path: &str) -> PyResult<String> {
    let mut file = fs::File::open(file_path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Cannot open file {}: {}",
            file_path, e
        ))
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 8 * 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to hash {}: {}",
                file_path, e
            ))
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sleep_backoff(attempt: u32, base_secs: u64) {
    let seconds = base_secs.saturating_mul(1_u64 << attempt.min(5));
    std::thread::sleep(Duration::from_secs(seconds.min(60)));
}

fn is_admission_limit(message: &str) -> bool {
    message.contains("429") || message.contains("MODEL_ARTIFACT_UPLOAD_ADMISSION_LIMIT")
}

fn write_bytes(dest_path: &str, bytes: Vec<u8>) -> PyResult<String> {
    if let Some(parent) = Path::new(dest_path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create parent dir for {}: {}",
                dest_path, e
            ))
        })?;
    }
    fs::write(dest_path, bytes).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to write {}: {}",
            dest_path, e
        ))
    })?;
    Ok(dest_path.to_string())
}

/// Point ZIP callers at the package flow when the archive is refused for being too large.
fn annotate_zip_error(err: PyErr) -> PyErr {
    let msg = err.to_string();
    if msg.contains("409") || msg.to_ascii_uppercase().contains("PACKAGE_TOO_LARGE_FOR_ZIP") {
        return PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "{} Hint: this package is too large for a single zip — use download_model_inference_artifacts_package().",
            msg
        ));
    }
    err
}

/// True when the backend simply has no such route (older deployment).
fn is_missing_route(message: &str) -> bool {
    message.contains("404") || message.to_ascii_lowercase().contains("not found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_limit_matches_backend_cap() {
        assert!(!ModelArtifactsApi::requires_upload_session(SYNC_UPLOAD_MAX_BYTES));
        assert!(ModelArtifactsApi::requires_upload_session(
            SYNC_UPLOAD_MAX_BYTES + 1
        ));
    }

    #[test]
    fn missing_route_detection() {
        assert!(is_missing_route("HTTP 404 Not Found"));
        assert!(!is_missing_route("HTTP 409 PACKAGE_TOO_LARGE_FOR_ZIP"));
    }

    #[test]
    fn file_category_six_is_prediction_output() {
        assert!(validate_file_category(6).is_ok());
        assert!(validate_file_category(7).is_err());
        let url = inference_file_upload_endpoint("m", 1, 6, Some("ent-1"), Some("output_image"));
        assert!(url.contains("file_category=6"));
        assert!(url.contains("entityId=ent-1"));
        assert!(url.contains("fieldName=output_image"));
    }

    #[test]
    fn admission_limit_detection() {
        assert!(is_admission_limit(
            "HTTP 429: MODEL_ARTIFACT_UPLOAD_ADMISSION_LIMIT"
        ));
        assert!(!is_admission_limit("HTTP 413: USE_KAPPA_APK"));
    }

    #[test]
    fn last_part_is_short() {
        // 3 parts over 250 bytes with a 100-byte part size.
        assert_eq!(part_len(1, 100, 250), 100);
        assert_eq!(part_len(3, 100, 250), 50);
        assert_eq!(part_len(4, 100, 250), 0);
    }

    #[test]
    fn resumed_bytes_match_uploaded_parts() {
        let done: Vec<u64> = [1, 2].into_iter().map(|n| part_len(n, 100, 250)).collect();
        assert_eq!(done.iter().sum::<u64>(), 200);
    }

    #[test]
    fn artifact_paths_expand_and_dedupe() {
        let dir = std::env::temp_dir().join(format!("kappa-artifacts-{}", std::process::id()));
        let nested = dir.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.join("config.json"), b"{}").unwrap();
        fs::write(nested.join("shard-1.bin"), b"weights").unwrap();

        let files = expand_artifact_paths(&[dir.to_string_lossy().to_string()]).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["config.json", "shard-1.bin"]);

        // The same leaf name twice would collide server-side, so it is kept once.
        let repeated = expand_artifact_paths(&[
            dir.join("config.json").to_string_lossy().to_string(),
            dir.join("config.json").to_string_lossy().to_string(),
        ])
        .unwrap();
        assert_eq!(repeated.len(), 1);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_artifact_path_is_rejected() {
        // PyErr text needs an interpreter; the error itself is the assertion here.
        assert!(expand_artifact_paths(&["/definitely/not/here.bin".to_string()]).is_err());
    }

    #[test]
    fn checksum_is_skipped_for_huge_files() {
        assert!(ModelArtifactsApi::should_checksum(1024));
        assert!(!ModelArtifactsApi::should_checksum(
            DEFAULT_CHECKSUM_MAX_BYTES + 1
        ));
    }
}
