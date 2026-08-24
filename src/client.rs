// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::Bound;
use reqwest::multipart;
use reqwest::Client;
use std::sync::Arc;

use crate::traits::ApiClient;
use crate::datasets::datasets::{Datasets, KappaDataset};
use crate::benchmarks::benchmarks::Benchmarks;
use crate::benchmarks::benchmarks_api::BenchmarksApi;
use crate::model_artifacts::ModelArtifactsApi;
use crate::models::users_model::User;
use crate::models::login_models::LoginRequest;
use crate::models::datasets_model::{
    BulkMutationJob, BulkUploadJob, DatasetDownloadDetails, DatasetLabel, DatasetVersionDetails,
    NewDataset, NewDatasetEntity, NewDatasetVersion, UpdateDatasetEntity, UpdateDatasetRequest,
    VersionBuildJob,
};
use crate::utils::python_json::{json_value_to_pyobject, rust_value_to_pyobject};
use crate::users::users::Users;
use zeroize::Zeroizing;

/// HTTP client for authentication and API calls to the Kappa-Apk framework.
///
/// # Python Example
///
/// ```python
/// from kappa_apk import KappaApkClient
///
/// client = KappaApkClient("https://api.example.com", "user@example.com", "password")
/// response = client.connect()
/// token = response['token']
///
/// data = client.make_request("GET", "/api/v2/datasets", None, token)
/// ```
#[pyclass]
pub struct KappaApkClient {
    client: Client,
    runtime: Arc<tokio::runtime::Runtime>,
    base_url: String,
    login_id: String,
    passwd: Zeroizing<String>,
    login_response: Option<User>,
}

impl Clone for KappaApkClient {
    fn clone(&self) -> Self {
        KappaApkClient {
            client: self.client.clone(),
            runtime: Arc::clone(&self.runtime),
            base_url: self.base_url.clone(),
            login_id: self.login_id.clone(),
            passwd: Zeroizing::new(String::new()),
            login_response: self.login_response.clone(),
        }
    }
}

impl ApiClient for KappaApkClient {
    fn get_base_url(&self) -> String {
        self.base_url.clone()
    }

    fn get_user_id(&self) -> Option<i32> {
        self.login_response.as_ref().map(|r| r.user_id)
    }

    fn get_user_type_id(&self) -> Option<i32> {
        self.login_response.as_ref().map(|r| r.user_type_id)
    }

    fn get_token(&self) -> Option<String> {
        self.login_response.as_ref().and_then(|r| r.token.clone())
    }

    fn get_http_client(&self) -> Client {
        self.client.clone()
    }

    fn get_runtime(&self) -> Arc<tokio::runtime::Runtime> {
        Arc::clone(&self.runtime)
    }

    fn make_request(
        &self,
        method: String,
        endpoint: String,
        data: Option<String>,
        token: Option<String>,
    ) -> PyResult<PyObject> {
        self.runtime.block_on(async {
            let url = format!("{}{}", self.base_url, endpoint);

            let mut request = match method.to_uppercase().as_str() {
                "GET"    => self.client.get(&url),
                "POST"   => self.client.post(&url),
                "PUT"    => self.client.put(&url),
                "DELETE" => self.client.delete(&url),
                other => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Unsupported HTTP method: {}", other),
                )),
            };

            request = request
                .header("accept", "application/json")
                .header("Content-Type", "application/json");

            if let Some(t) = token {
                request = request.header("Authorization", format!("Bearer {}", t));
            }
            if let Some(body) = data {
                request = request.body(body);
            }

            let response = request.send().await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyConnectionError, _>(
                    format!("Request failed: {}", e),
                )
            })?;

            if response.status().is_success() {
                let json_data = response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to parse response: {}", e),
                        )
                    })?;
                Python::with_gil(|py| json_value_to_pyobject(py, &json_data))
            } else {
                Err(http_error_to_pyerr(response.status(), response.text().await.unwrap_or_default()))
            }
        })
    }

    fn fetch_url_bytes(&self, url: &str) -> PyResult<Vec<u8>> {
        let url = url.to_string();
        self.runtime.block_on(async {
            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyConnectionError, _>(
                        format!("Failed to fetch URL {}: {}", url, e),
                    )
                })?;
            if !response.status().is_success() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("URL returned HTTP {}: {}", response.status(), url),
                ));
            }
            response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to read response body from {}: {}", url, e),
                )
            })
        })
    }

    fn submit_dataset_entity_request(
        &self,
        method: &str,
        endpoint: String,
        json_field_name: &str,
        json_value: String,
        file_parts: Vec<(Vec<u8>, String)>,
        token: Option<String>,
    ) -> PyResult<PyObject> {
        let url = format!("{}{}", self.base_url, endpoint);
        let http = self.client.clone();
        let method_owned = method.to_uppercase();
        let jfield = json_field_name.to_string();

        self.runtime.block_on(async move {
            // Always multipart so FastAPI `files: List[UploadFile] = File(...)` accepts
            // file-less (tabular) creates the same way as the React FormData path.
            let mut form = multipart::Form::new().text(jfield.clone(), json_value);
            for (bytes, fname) in file_parts {
                let part = multipart::Part::bytes(bytes)
                    .file_name(fname)
                    .mime_str("application/octet-stream")
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
                    })?;
                form = form.part("files", part);
            }

            let mut request = match method_owned.as_str() {
                "POST" => http.post(&url).multipart(form),
                "PUT" => http.put(&url).multipart(form),
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Unsupported method for entity request: {}",
                        method_owned
                    )));
                }
            };

            request = request.header("accept", "application/json");
            if let Some(ref t) = token {
                request = request.header("Authorization", format!("Bearer {}", t));
            }

            let response = request.send().await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyConnectionError, _>(
                    format!("Request failed: {}", e),
                )
            })?;

            if response.status().is_success() {
                let json_data: serde_json::Value = response.json().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Failed to parse response: {}", e),
                    )
                })?;
                Python::with_gil(|py| json_value_to_pyobject(py, &json_data))
            } else {
                Err(http_error_to_pyerr(response.status(), response.text().await.unwrap_or_default()))
            }
        })
    }

    fn submit_multipart_files(
        &self,
        method: &str,
        endpoint: String,
        file_parts: Vec<(Vec<u8>, String)>,
        token: Option<String>,
    ) -> PyResult<PyObject> {
        if file_parts.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "At least one file is required for multipart upload",
            ));
        }
        let url = format!("{}{}", self.base_url, endpoint);
        let http = self.client.clone();
        let method_owned = method.to_uppercase();

        self.runtime.block_on(async move {
            let mut form = multipart::Form::new();
            for (bytes, fname) in file_parts {
                let part = multipart::Part::bytes(bytes)
                    .file_name(fname)
                    .mime_str("application/octet-stream")
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
                    })?;
                form = form.part("files", part);
            }

            let mut request = match method_owned.as_str() {
                "POST" => http.post(&url).multipart(form),
                "PUT" => http.put(&url).multipart(form),
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Unsupported method for multipart upload: {}",
                        method_owned
                    )));
                }
            };

            request = request.header("accept", "application/json");
            if let Some(ref t) = token {
                request = request.header("Authorization", format!("Bearer {}", t));
            }

            let response = request.send().await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyConnectionError, _>(
                    format!("Request failed: {}", e),
                )
            })?;

            if response.status().is_success() {
                let json_data: serde_json::Value = response.json().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Failed to parse response: {}", e),
                    )
                })?;
                Python::with_gil(|py| json_value_to_pyobject(py, &json_data))
            } else {
                Err(http_error_to_pyerr(response.status(), response.text().await.unwrap_or_default()))
            }
        })
    }

    fn submit_bulk_upload(
        &self,
        endpoint: String,
        sources_json: String,
        file_path: String,
        headers: Vec<(String, String)>,
        token: Option<String>,
        on_upload_progress: Option<PyObject>,
    ) -> PyResult<PyObject> {
        use futures_util::StreamExt;
        use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
        use std::sync::Arc;
        use tokio_util::io::ReaderStream;

        let url = format!("{}{}", self.base_url, endpoint);
        let http = self.client.clone();
        let runtime = Arc::clone(&self.runtime);
        let path = std::path::PathBuf::from(&file_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload.bin")
            .to_string();
        let total = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let progress_cb = on_upload_progress.map(Arc::new);

        // Release the GIL before block_on. Progress callbacks re-acquire it via
        // Python::with_gil from Tokio worker threads; holding the GIL across
        // block_on deadlocks as soon as the first progress tick runs.
        Python::with_gil(|py| {
            py.allow_threads(move || {
                runtime.block_on(async move {
                    let file = tokio::fs::File::open(&path).await.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                            "Cannot open {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    let reader = ReaderStream::new(file);
                    let sent = Arc::new(AtomicU64::new(0));
                    let last_pct = Arc::new(AtomicI32::new(-1));
                    let sent2 = sent.clone();
                    let last2 = last_pct.clone();
                    let cb2 = progress_cb.clone();
                    let total2 = total;

                    let stream = reader.map(move |chunk| match chunk {
                        Ok(bytes) => {
                            let n = bytes.len() as u64;
                            let cur = sent2.fetch_add(n, Ordering::Relaxed) + n;
                            let pct = if total2 > 0 {
                                (((cur as f64) * 100.0) / (total2 as f64)).round() as i32
                            } else {
                                0
                            }
                            .clamp(0, 100);
                            let prev_pct = last2.swap(pct, Ordering::Relaxed);
                            if (pct != prev_pct || cur >= total2)
                                && let Some(ref cb) = cb2
                            {
                                Python::with_gil(|py| {
                                    let _ = cb.bind(py).call1((cur, total2, pct));
                                });
                            }
                            Ok::<_, std::io::Error>(bytes)
                        }
                        Err(e) => Err(e),
                    });

                    let body = reqwest::Body::wrap_stream(stream);
                    let file_part = multipart::Part::stream(body)
                        .file_name(file_name)
                        .mime_str("application/octet-stream")
                        .map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
                        })?;
                    let form = multipart::Form::new()
                        .text("sources", sources_json)
                        .part("file", file_part);

                    let mut request = http.post(&url).multipart(form);
                    request = request.header("accept", "application/json");
                    if let Some(ref t) = token {
                        request = request.header("Authorization", format!("Bearer {}", t));
                    }
                    for (k, v) in headers {
                        request = request.header(k, v);
                    }

                    let response = request.send().await.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!(
                            "Request failed: {}",
                            e
                        ))
                    })?;

                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    if status.is_success() {
                        let json_data: serde_json::Value = serde_json::from_str(&text)
                            .map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                    "Failed to parse response: {}",
                                    e
                                ))
                            })?;
                        return Python::with_gil(|py| json_value_to_pyobject(py, &json_data));
                    }
                    // BE returns HTTP 400 for layout preflight with status=needs_correction
                    // and a jobId (staging kept for retry). Surface that body like a start
                    // response so callers can save jobId and call retry_bulk_upload_job.
                    if status.as_u16() == 400
                        && let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&text)
                    {
                        let has_job = json_data
                            .get("jobId")
                            .or_else(|| json_data.get("job_id"))
                            .and_then(|v| v.as_str())
                            .map(|s| !s.is_empty())
                            .unwrap_or(false);
                        let job_status = json_data
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        if has_job
                            && (job_status == "needs_correction"
                                || json_data
                                    .get("retryable")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false))
                        {
                            return Python::with_gil(|py| {
                                json_value_to_pyobject(py, &json_data)
                            });
                        }
                    }
                    Err(http_error_to_pyerr(status, text))
                })
            })
        })
    }

    fn download_bytes(&self, endpoint: String, token: Option<String>) -> PyResult<Vec<u8>> {
        let url = format!("{}{}", self.base_url, endpoint);
        let http = self.client.clone();
        self.runtime.block_on(async move {
            let mut request = http.get(&url).header("accept", "*/*");
            if let Some(ref t) = token {
                request = request.header("Authorization", format!("Bearer {}", t));
            }
            let response = request.send().await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!(
                    "Download failed: {}",
                    e
                ))
            })?;
            if response.status().is_success() {
                response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to read download body: {}",
                        e
                    ))
                })
            } else {
                Err(http_error_to_pyerr(
                    response.status(),
                    response.text().await.unwrap_or_default(),
                ))
            }
        })
    }

    fn submit_named_file(
        &self,
        method: &str,
        endpoint: String,
        field_name: &str,
        file_bytes: Vec<u8>,
        file_name: String,
        token: Option<String>,
    ) -> PyResult<PyObject> {
        let url = format!("{}{}", self.base_url, endpoint);
        let http = self.client.clone();
        let method_owned = method.to_uppercase();
        let field = field_name.to_string();

        self.runtime.block_on(async move {
            let part = multipart::Part::bytes(file_bytes)
                .file_name(file_name)
                .mime_str("application/octet-stream")
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            let form = multipart::Form::new().part(field, part);

            let mut request = match method_owned.as_str() {
                "POST" => http.post(&url).multipart(form),
                "PUT" => http.put(&url).multipart(form),
                "PATCH" => http.patch(&url).multipart(form),
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Unsupported method for named file upload: {}",
                        method_owned
                    )));
                }
            };
            request = request.header("accept", "application/json");
            if let Some(ref t) = token {
                request = request.header("Authorization", format!("Bearer {}", t));
            }

            let response = request.send().await.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!(
                    "Request failed: {}",
                    e
                ))
            })?;

            if response.status().is_success() {
                let json_data: serde_json::Value = response.json().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to parse response: {}",
                        e
                    ))
                })?;
                Python::with_gil(|py| json_value_to_pyobject(py, &json_data))
            } else {
                Err(http_error_to_pyerr(
                    response.status(),
                    response.text().await.unwrap_or_default(),
                ))
            }
        })
    }
}

/// Map HTTP error bodies to Python exceptions, preserving status for 409 conflicts etc.
fn http_error_to_pyerr(status: reqwest::StatusCode, text: String) -> PyErr {
    let detail = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        json.get("detail")
            .map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else {
                    v.to_string()
                }
            })
            .unwrap_or_else(|| text.clone())
    } else {
        text.clone()
    };
    let msg = if detail.is_empty() {
        format!("HTTP {}", status)
    } else {
        format!("HTTP {}: {}", status, detail)
    };
    let upper = detail.to_ascii_uppercase();
    if status.as_u16() == 409 {
        let extra = if upper.contains("DATASET_PERMANENTLY_DELETED") {
            " Dataset was permanently deleted after the recover window (entity files purged)."
        } else if upper.contains("DATASET_DELETED") {
            " Dataset is soft-deleted (status 0); recover first or omit short-info edits."
        } else if upper.contains("PRIMARY_TAG_IMMUTABLE") {
            " Cannot remove or replace the primary ML tag via PATCH tags."
        } else {
            ""
        };
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}{}", msg, extra))
    } else if status.as_u16() == 403 {
        if upper.contains("PRIVATE_VERSION_ORG_ONLY") {
            PyErr::new::<pyo3::exceptions::PyPermissionError, _>(
                "HTTP 403: This version is private to the owning organization.",
            )
        } else {
            PyErr::new::<pyo3::exceptions::PyPermissionError, _>(format!(
                "{}. Hint: call client.get_my_permissions(dataset_id=...) or has_permission('dataset.write', dataset_id=...).",
                msg
            ))
        }
    } else if status.as_u16() == 401 {
        PyErr::new::<pyo3::exceptions::PyPermissionError, _>(msg)
    } else if status.as_u16() == 429 {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "{} (server busy — retry after Retry-After / retryAfterSeconds)",
            msg
        ))
    } else {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(msg)
    }
}

// ---------------------------------------------------------------------------
// Payload encoding helpers
// ---------------------------------------------------------------------------

fn py_json_dumps(v: &Bound<'_, PyAny>) -> PyResult<String> {
    let py = v.py();
    py.import("json")?.getattr("dumps")?.call1((v,))?.extract()
}

/// Convert an optional Python object into JSON, treating `None` as absent.
fn py_json_value(v: Option<&Bound<'_, PyAny>>) -> PyResult<Option<serde_json::Value>> {
    let Some(value) = v else { return Ok(None) };
    if value.is_none() {
        return Ok(None);
    }
    let json: String = py_json_dumps(value)?;
    serde_json::from_str(&json)
        .map(Some)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid JSON: {}", e)))
}

/// Accept a single path or a sequence of paths.
fn py_path_list(v: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<String>> {
    let Some(value) = v else {
        return Ok(Vec::new());
    };
    if value.is_none() {
        return Ok(Vec::new());
    }
    if let Ok(single) = value.extract::<String>() {
        return Ok(vec![single]);
    }
    value.extract::<Vec<String>>().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "artifact paths must be a string or a list of strings",
        )
    })
}

fn prediction_file_ref_from_upload(
    py: Python<'_>,
    upload: &Bound<'_, PyAny>,
    file_name: Option<String>,
    content_type: Option<String>,
) -> PyResult<PyObject> {
    let value = crate::utils::python_json::pyobject_to_rust_value(
        &upload.clone().unbind(),
        "prediction file upload",
    )?;
    let artifact_id = value
        .get("fileId")
        .or_else(|| value.get("artifactId"))
        .or_else(|| value.get("file_id"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let name = file_name
        .map(serde_json::Value::String)
        .or_else(|| value.get("fileName").or_else(|| value.get("file_name")).cloned())
        .unwrap_or(serde_json::Value::Null);
    let mut obj = serde_json::Map::new();
    obj.insert("artifactId".to_string(), artifact_id);
    obj.insert("fileName".to_string(), name);
    if let Some(ct) = content_type.filter(|s| !s.is_empty()) {
        obj.insert("contentType".to_string(), serde_json::Value::String(ct));
    } else if let Some(ct) = value.get("contentType").or_else(|| value.get("content_type")) {
        obj.insert("contentType".to_string(), ct.clone());
    }
    crate::utils::python_json::json_value_to_pyobject(py, &serde_json::Value::Object(obj))
}

pub(crate) fn attach_inference_pipeline<T: crate::traits::ApiClient>(
    py: Python<'_>,
    client: &T,
    model_id: &str,
    inference_id: i32,
    artifact_paths: &[String],
    explicit: Option<&Bound<'_, PyAny>>,
    pipeline_type: i32,
    model: Option<&Bound<'_, PyAny>>,
    entrypoint: Option<&str>,
    project_files: &[String],
    endpoint_url: Option<&str>,
) -> serde_json::Value {
    if inference_id == 0 {
        return crate::pipeline_detect::PipelineDetect::skipped("no inferenceId").to_status_json();
    }
    let mut detect = if let Some(body) = explicit {
        match py_json_value(Some(body)) {
            Ok(Some(value)) => {
                let draft = crate::pipeline_detect::manual_draft(value, pipeline_type);
                crate::pipeline_detect::PipelineDetect {
                    draft: Some(draft),
                    attached: true,
                    reason: "manual".to_string(),
                    candidates: Vec::new(),
                }
            }
            _ => crate::pipeline_detect::PipelineDetect::skipped("invalid pipeline body"),
        }
    } else {
        crate::pipeline_detect::detect(
            py,
            artifact_paths,
            model,
            entrypoint,
            project_files,
            pipeline_type,
        )
    };
    if let Some(url) = endpoint_url.map(str::trim).filter(|s| !s.is_empty())
        && let Some(serde_json::Value::Object(obj)) = detect.draft.as_mut()
    {
        obj.insert("endpointUrl".to_string(), serde_json::json!(url));
        obj.insert(
            "pipelineType".to_string(),
            serde_json::json!(crate::pipeline_detect::PIPELINE_TYPE_ONLINE),
        );
    }
    crate::pipeline_detect::put_or_skip(client, model_id, inference_id, detect).to_status_json()
}

fn encode_new_dataset_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    let json = if let Ok(d) = v.extract::<PyRef<NewDataset>>() {
        d.to_api_json()?
    } else {
        py_json_dumps(v)?
    };
    validate_dataset_short_info_in_json(&json)?;
    Ok(json)
}

fn encode_update_dataset_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    let json = if let Ok(d) = v.extract::<PyRef<UpdateDatasetRequest>>() {
        d.to_api_json()?
    } else {
        py_json_dumps(v)?
    };
    validate_dataset_short_info_in_json(&json)?;
    Ok(json)
}

fn validate_dataset_short_info_in_json(json: &str) -> PyResult<()> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid dataset JSON: {}", e))
    })?;
    if let Some(s) = json_str_field(&value, &["datasetShortInfo", "dataset_short_info"])
        && s.chars().count() > crate::models::datasets_model::DATASET_SHORT_INFO_MAX
    {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "datasetShortInfo must be at most {} characters",
            crate::models::datasets_model::DATASET_SHORT_INFO_MAX
        )));
    }
    Ok(())
}

fn encode_new_entity_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    let json = if let Ok(e) = v.extract::<PyRef<NewDatasetEntity>>() {
        e.to_api_json(v.py())?
    } else {
        py_json_dumps(v)?
    };
    validate_entity_labeling_algo_in_json(&json)?;
    validate_entity_source_in_json(&json)?;
    Ok(json)
}

/// Backend 2.9 rejects empty / `"none"` labelingAlgo on entity create.
fn validate_entity_labeling_algo_in_json(json: &str) -> PyResult<()> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid entity JSON: {}", e))
    })?;
    let algo = value
        .get("labelingAlgo")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if algo.is_empty() || algo.eq_ignore_ascii_case("none") {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "labelingAlgo is required and cannot be empty or 'none'",
        ));
    }
    Ok(())
}

fn encode_update_entity_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    let json = if let Ok(e) = v.extract::<PyRef<UpdateDatasetEntity>>() {
        e.to_api_json(v.py())?
    } else {
        py_json_dumps(v)?
    };
    validate_entity_source_in_json(&json)?;
    Ok(json)
}

fn validate_entity_source_in_json(json: &str) -> PyResult<()> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid entity JSON: {}", e))
    })?;
    if let Some(s) = json_str_field(&value, &["entitySource", "entity_source", "source"]) {
        crate::upload_limits::validate_source(s)?;
    }
    Ok(())
}

fn encode_new_version_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(nv) = v.extract::<PyRef<NewDatasetVersion>>() {
        return nv.to_api_json();
    }
    py_json_dumps(v)
}

fn json_i32_field(value: &serde_json::Value, keys: &[&str]) -> Option<i32> {
    for key in keys {
        if let Some(v) = value.get(*key) {
            if let Some(i) = v.as_i64() {
                return Some(i as i32);
            }
            if let Some(u) = v.as_u64() {
                return Some(u as i32);
            }
            if let Some(s) = v.as_str()
                && let Ok(i) = s.trim().parse::<i32>()
            {
                return Some(i);
            }
        }
    }
    None
}

fn json_str_field<'a>(value: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    for key in keys {
        if let Some(s) = value.get(*key).and_then(|v| v.as_str()) {
            return Some(s);
        }
    }
    None
}

/// Soft-validate create payloads: ≥1 predefined tag for type, and that tag is first.
fn validate_create_tags_payload(
    client: &KappaApkClient,
    body_json: &str,
    type_keys: &[&str],
    tags_keys: &[&str],
) -> PyResult<()> {
    let value: serde_json::Value = serde_json::from_str(body_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid JSON body: {}", e))
    })?;
    let type_id = json_i32_field(&value, type_keys).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Missing type field (tried {:?}) for ML tag validation",
            type_keys
        ))
    })?;
    let tags = json_str_field(&value, tags_keys).unwrap_or("").to_string();
    let catalog = client.list_predefined_ml_tags(type_id)?;
    if catalog.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "No predefined ML tags found for type_id={} (system config dataset_tags_{})",
            type_id, type_id
        )));
    }
    crate::ml_tags::validate_ml_tags(tags, catalog, true)
}

// ---------------------------------------------------------------------------
// Python-exposed methods
// ---------------------------------------------------------------------------

#[pymethods]
impl KappaApkClient {
    /// Create a new KappaApkClient instance.
    ///
    /// # Python Example
    ///
    /// ```python
    /// client = KappaApkClient("https://api.example.com", "user@example.com", "password")
    /// ```
    #[new]
    pub fn new(base_url: String, login_id: String, passwd: String) -> PyResult<Self> {
        // Connect timeout only — bulk uploads can run for a long time with no overall deadline.
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to create HTTP client: {}",
                    e
                ))
            })?;
        let runtime = Arc::new(tokio::runtime::Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {}",
                e
            ))
        })?);
        Ok(KappaApkClient {
            client,
            runtime,
            base_url,
            login_id,
            passwd: Zeroizing::new(passwd),
            login_response: None,
        })
    }

    /// Authenticate with the server and return user info with token.
    ///
    /// # Python Example
    ///
    /// ```python
    /// response = client.connect()
    /// token = response['token']
    /// print(f"Logged in as: {response['user_name']}")
    /// ```
    pub fn connect(&mut self) -> PyResult<PyObject> {
        if self.passwd.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Password unavailable (cleared after login). Create a new KappaApkClient to reconnect.",
            ));
        }
        let login_request = LoginRequest {
            login_id: self.login_id.clone(),
            passwd: self.passwd.to_string(),
        };
        let login_url = format!("{}/user-micro-services/v2/session/new", self.base_url);
        let client = self.client.clone();
        let runtime = Arc::clone(&self.runtime);

        let login_data: User = Python::with_gil(|py| {
            py.allow_threads(move || {
                runtime.block_on(async move {
                    let response = client
                        .post(&login_url)
                        .header("accept", "application/json")
                        .header("Content-Type", "application/json")
                        .json(&login_request)
                        .send()
                        .await
                        .map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!(
                                "Request failed: {}",
                                e
                            ))
                        })?;

                    if response.status().is_success() {
                        response.json::<User>().await.map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                "Failed to parse login response: {}",
                                e
                            ))
                        })
                    } else {
                        let status = response.status();
                        let msg = response.text().await.unwrap_or_default();
                        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Login failed (HTTP {}): {}",
                            status, msg
                        )))
                    }
                })
            })
        })?;

        self.login_response = Some(login_data.clone());
        // Clear password from memory after successful authentication.
        self.passwd = Zeroizing::new(String::new());

        // Return snake_case dict for Python consumers (User uses camelCase serde internally).
        Python::with_gil(|py| rust_value_to_pyobject(py, &serde_json::json!({
            "user_id": login_data.user_id,
            "user_name": login_data.user_name,
            "first_name": login_data.first_name,
            "middle_name": login_data.middle_name,
            "last_name": login_data.last_name,
            "email": login_data.email,
            "user_type_id": login_data.user_type_id,
            "org_id": login_data.org_id,
            "token": login_data.token,
            "token_expiry_date": login_data.token_expiry_date,
            "profile_pic": login_data.profile_pic,
            "user_type_details": {
                "user_type_id": login_data.user_type_details.user_type_id,
                "user_type": login_data.user_type_details.user_type
            },
            "org_details": login_data.org_details.as_ref().map(|org| serde_json::json!({
                "org_id": org.org_id,
                "org_name": org.org_name
            }))
        })))
    }

    /// Make HTTP request with optional authentication.
    ///
    /// # Python Example
    ///
    /// ```python
    /// # GET request
    /// data = client.make_request("GET", "/api/v2/datasets", None, token)
    ///
    /// # POST request
    /// import json
    /// result = client.make_request("POST", "/api/v2/datasets", json.dumps({"name": "ds"}), token)
    /// ```
    pub fn make_request(
        &self,
        method: String,
        endpoint: String,
        data: Option<String>,
        token: Option<String>,
    ) -> PyResult<PyObject> {
        ApiClient::make_request(self, method, endpoint, data, token)
    }

    /// Get the current base URL.
    pub fn get_base_url(&self) -> String {
        ApiClient::get_base_url(self)
    }

    /// Set a new base URL.
    pub fn set_base_url(&mut self, new_url: String) {
        self.base_url = new_url;
    }

    /// Close the current session and logout.
    ///
    /// Clears the stored authentication state so `is_authenticated()` returns
    /// `False` after this call. The session token is also invalidated on the server.
    ///
    /// # Python Example
    ///
    /// ```python
    /// client.close()
    /// ```
    pub fn close(&mut self) -> PyResult<()> {
        let login_response = self.login_response.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyPermissionError, _>(
                "Not logged in. Call connect() first.",
            )
        })?;

        // v2: DELETE /session — identity from Bearer token only (no path params).
        let logout_url = format!("{}/user-micro-services/v2/session", self.base_url);
        let token = login_response.token.clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Missing bearer token in login response",
            )
        })?;
        let client = self.client.clone();

        self.runtime.block_on(async move {
            let response = client
                .delete(&logout_url)
                .header("accept", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyConnectionError, _>(
                        format!("Request failed: {}", e),
                    )
                })?;
            if response.status().is_success() {
                Ok(())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to close session: HTTP {}", response.status()),
                ))
            }
        })?;

        // Clear stored credentials only after a successful server-side logout.
        self.login_response = None;
        self.passwd = Zeroizing::new(String::new());
        Ok(())
    }

    /// Context manager entry — automatically connects.
    ///
    /// # Python Example
    ///
    /// ```python
    /// with KappaApkClient("https://api.example.com", "user@example.com", "password") as client:
    ///     data = client.make_request("GET", "/api/v2/datasets", None, None)
    /// ```
    pub fn __enter__(mut slf: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
        slf.connect()?;
        Ok(slf)
    }

    /// Context manager exit — automatically closes the session.
    pub fn __exit__(
        mut slf: PyRefMut<'_, Self>,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_val: Option<&Bound<'_, PyAny>>,
        _exc_tb: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        // Ignore close errors so they don't mask the original exception.
        let _ = slf.close();
        Ok(false)
    }

    /// Get the current authentication token.
    pub fn get_token(&self) -> Option<String> {
        ApiClient::get_token(self)
    }

    /// Return `True` if the client has a valid session token.
    pub fn is_authenticated(&self) -> bool {
        ApiClient::is_authenticated(self)
    }

    /// Get the authenticated user's profile (`GET /user-micro-services/v2/users/me`).
    pub fn get_user_profile(&self) -> PyResult<User> {
        Users::get_user_profile(self)
    }

    /// Effective permissions (`GET /users/me/permissions`), optionally scoped.
    #[pyo3(signature = (dataset_id=None, org_id=None))]
    pub fn get_my_permissions(
        &self,
        dataset_id: Option<i32>,
        org_id: Option<i32>,
    ) -> PyResult<PyObject> {
        Users::get_my_permissions(self, dataset_id, org_id)
    }

    /// Whether `code` (e.g. ``dataset.write``) is granted for the optional scopes.
    #[pyo3(signature = (code, dataset_id=None, org_id=None))]
    pub fn has_permission(
        &self,
        code: String,
        dataset_id: Option<i32>,
        org_id: Option<i32>,
    ) -> PyResult<bool> {
        Users::has_permission(self, &code, dataset_id, org_id)
    }

    /// System configuration rows (`GET /user-micro-services/v2/system/config/{tag}`).
    ///
    /// Examples: ``"dataset_type"``, ``"dataset_tags_1"`` (CV predefined ML tags).
    pub fn get_system_config(&self, tag: String) -> PyResult<PyObject> {
        let token = self.require_token()?;
        let endpoint = format!("/user-micro-services/v2/system/config/{}", tag.trim());
        self.make_request("GET".to_string(), endpoint, None, Some(token))
    }

    /// Display values of predefined ML tags for a dataset/model type id
    /// (`dataset_tags_{type_id}` — same catalog the React create dialogs use).
    pub fn list_predefined_ml_tags(&self, type_id: i32) -> PyResult<Vec<String>> {
        if type_id <= 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "type_id must be a positive dataset_type / model_type id",
            ));
        }
        let rows = self.get_system_config(format!("dataset_tags_{}", type_id))?;
        Python::with_gil(|py| {
            let list = rows.bind(py);
            let seq = list
                .downcast::<pyo3::types::PyList>()
                .map_err(|_| {
                    PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "system config response must be a list",
                    )
                })?;
            let mut out = Vec::with_capacity(seq.len());
            for item in seq.iter() {
                let display = item
                    .get_item("displayValue")
                    .or_else(|_| item.get_item("display_value"))
                    .ok()
                    .and_then(|v| v.extract::<String>().ok())
                    .unwrap_or_default();
                let trimmed = display.trim().to_string();
                if !trimmed.is_empty() {
                    out.push(trimmed);
                }
            }
            Ok(out)
        })
    }

    /// List datasets for the current user (paginated, raw JSON dict).
    #[pyo3(signature = (page=None, size=None, order_by=None, order_keyword=None))]
    pub fn list_datasets(
        &self,
        page: Option<i32>,
        size: Option<i32>,
        order_by: Option<String>,
        order_keyword: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::list_datasets_json(self, page, size, order_by, order_keyword)
    }

    /// List datasets as typed `Dataset` objects.
    #[pyo3(signature = (page=None, size=None, order_by=None, order_keyword=None))]
    pub fn list_datasets_typed(
        &self,
        page: Option<i32>,
        size: Option<i32>,
        order_by: Option<String>,
        order_keyword: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::list_datasets(self, page, size, order_by, order_keyword)
    }

    /// Get dataset version details.
    #[pyo3(signature = (dataset_id=None, dataset_name=None, version_id=None, version_no=None))]
    pub fn get_dataset_version_details(
        &self,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
    ) -> PyResult<DatasetVersionDetails> {
        Datasets::get_dataset_version_details(self, dataset_id, dataset_name, version_id, version_no)
    }

    /// Download and extract a dataset version archive to the local cache.
    #[pyo3(signature = (dataset_id=None, dataset_name=None, version_id=None, version_no=None, dataset_path=None))]
    pub fn download_dataset_version_archive(
        &self,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
    ) -> PyResult<DatasetDownloadDetails> {
        Datasets::download_dataset_version_archive(
            self,
            dataset_id,
            dataset_name,
            version_id,
            version_no,
            dataset_path,
        )
    }

    /// Load a dataset version into a `KappaDataset` instance.
    ///
    /// `splits`: optional list of split names to keep (e.g. `["train"]`). Missing
    /// `entity_info.split` is treated as `"train"`.
    #[pyo3(signature = (dataset_id=None, dataset_name=None, version_id=None, version_no=None, dataset_path=None, transform=None, target_transform=None, transform_input_mode=None, splits=None))]
    pub fn load_kappa_dataset(
        &self,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
        transform: Option<Py<PyAny>>,
        target_transform: Option<Py<PyAny>>,
        transform_input_mode: Option<String>,
        splits: Option<Vec<String>>,
    ) -> PyResult<Py<KappaDataset>> {
        let transform_input_mode = transform_input_mode.unwrap_or_else(|| "content".to_string());
        Datasets::load_kappa_dataset(
            self,
            dataset_id,
            dataset_name,
            version_id,
            version_no,
            dataset_path,
            transform,
            target_transform,
            transform_input_mode,
            splits,
        )
    }

    /// Get a dataset loader in the requested format (`"kappa"`, `"pytorch"`, `"transformers"`, `"tensorflow"`).
    ///
    /// `splits`: optional list of split names to keep (see [`Self::load_kappa_dataset`]).
    #[pyo3(signature = (
        dataset_id=None,
        dataset_name=None,
        version_id=None,
        version_no=None,
        dataset_path=None,
        loader_type=None,
        batch_size=None,
        shuffle=None,
        drop_last=None,
        tf_output_signature=None,
        transform=None,
        target_transform=None,
        transform_input_mode=None,
        splits=None
    ))]
    pub fn get_dataset_loader(
        &self,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
        loader_type: Option<String>,
        batch_size: Option<usize>,
        shuffle: Option<bool>,
        drop_last: Option<bool>,
        tf_output_signature: Option<Py<PyAny>>,
        transform: Option<Py<PyAny>>,
        target_transform: Option<Py<PyAny>>,
        transform_input_mode: Option<String>,
        splits: Option<Vec<String>>,
    ) -> PyResult<PyObject> {
        let transform_input_mode = transform_input_mode.unwrap_or_else(|| "content".to_string());
        Datasets::get_dataset_loader(
            self,
            dataset_id,
            dataset_name,
            version_id,
            version_no,
            dataset_path,
            loader_type,
            batch_size,
            shuffle,
            drop_last,
            tf_output_signature,
            transform,
            target_transform,
            transform_input_mode,
            splits,
        )
    }

    /// Create a dataset (`POST /data-micro-services/v2/datasets/new`).
    ///
    /// When ``check_tags`` is true (default), loads predefined tags for
    /// ``dataset_type`` from system config and requires the **first**
    /// ``dataset_tags`` entry to be one of them (custom tags may follow).
    #[pyo3(signature = (dataset, check_tags=true))]
    pub fn add_dataset(
        &self,
        dataset: &Bound<'_, PyAny>,
        check_tags: bool,
    ) -> PyResult<PyObject> {
        let body = encode_new_dataset_payload(dataset)?;
        if check_tags {
            validate_create_tags_payload(
                self,
                &body,
                &["datasetType", "dataset_type"],
                &["datasetTags", "dataset_tags"],
            )?;
        }
        Datasets::add_dataset(self, body)
    }

    /// Update dataset metadata (`PUT /data-micro-services/v2/datasets/{dataset_id}`).
    ///
    /// Kappa ≥ 2.13: pass `datasetShortInfo` (max 10 000) to edit the blurb; omit to leave it.
    /// Soft-deleted / permanently deleted datasets reject short-info edits (`409`).
    #[pyo3(signature = (dataset_id, update))]
    pub fn update_dataset(
        &self,
        dataset_id: i32,
        update: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let body = encode_update_dataset_payload(update)?;
        Datasets::update_dataset(self, dataset_id, body)
    }

    /// Add/remove non-primary dataset tags (`PATCH …/datasets/{id}/tags`, Kappa ≥ 2.13).
    ///
    /// Cannot drop the primary ML tag (`409 PRIMARY_TAG_IMMUTABLE`). Custom tags are allowed.
    #[pyo3(signature = (dataset_id, add=None, remove=None))]
    pub fn patch_dataset_tags(
        &self,
        dataset_id: i32,
        add: Option<Vec<String>>,
        remove: Option<Vec<String>>,
    ) -> PyResult<PyObject> {
        Datasets::patch_dataset_tags(
            self,
            dataset_id,
            add.unwrap_or_default(),
            remove.unwrap_or_default(),
        )
    }

    /// Add a dataset entity with optional file attachments.
    ///
    /// `file_category`: ``"input"`` (default) or ``"output"`` when attaching files — builds
    /// `filesCategory` for resolved filenames unless the entity already sets it.
    /// `split`: optional train/validation/test (or schema-allowed) value for `dsEntityInfo.split`.
    #[pyo3(signature = (dataset_id, entity, file_paths=None, file_category=None, split=None))]
    pub fn add_dataset_entity(
        &self,
        dataset_id: i32,
        entity: &Bound<'_, PyAny>,
        file_paths: Option<Vec<String>>,
        file_category: Option<String>,
        split: Option<String>,
    ) -> PyResult<PyObject> {
        let entity_json = encode_new_entity_payload(entity)?;
        Datasets::add_dataset_entity(
            self,
            dataset_id,
            entity_json,
            file_paths.unwrap_or_default(),
            file_category,
            split,
        )
    }

    /// Update a dataset entity with optional file attachments.
    ///
    /// See [`Self::add_dataset_entity`] for `file_category` / `split`.
    #[pyo3(signature = (dataset_id, entity_id, update, file_paths=None, file_category=None, split=None))]
    pub fn update_dataset_entity(
        &self,
        dataset_id: i32,
        entity_id: String,
        update: &Bound<'_, PyAny>,
        file_paths: Option<Vec<String>>,
        file_category: Option<String>,
        split: Option<String>,
    ) -> PyResult<PyObject> {
        let update_json = encode_update_entity_payload(update)?;
        Datasets::update_dataset_entity(
            self,
            dataset_id,
            &entity_id,
            update_json,
            file_paths.unwrap_or_default(),
            file_category,
            split,
        )
    }

    /// Get full metadata for a dataset by ID or name.
    #[pyo3(signature = (dataset_id=None, dataset_name=None))]
    pub fn get_dataset_details(
        &self,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
    ) -> PyResult<crate::models::datasets_model::Dataset> {
        Datasets::get_dataset_details(self, dataset_id, dataset_name)
    }

    /// Filter / search datasets with rich query params.
    ///
    /// All parameters are optional; omit any you don't need. Pages are **1-based**;
    /// the response is `{items, total, page, size, pages}`.
    /// `dataset_tags` is a comma-separated tag string (e.g. `"vision,classification"`).
    /// `publish_type`: 0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand, 4 Purchase.
    /// `query_all=False` limits results to datasets you own, are assigned to, or that are
    /// shared with you; the backend default (`True`) also lists the public catalogue.
    /// `selected_version_id` / `selected_version_no` add `selectedVersionNo` and
    /// `selectedVersionBuildStatus` to each item — handy to check `ready` before download.
    #[pyo3(signature = (search=None, dataset_id=None, dataset_name=None, dataset_type=None, dataset_tags=None, dataset_status=None, publish_type=None, page=None, size=None, order_by=None, order_keyword=None, query_all=None, start_date=None, end_date=None, selected_version_id=None, selected_version_no=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn filter_datasets(
        &self,
        search: Option<String>,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        dataset_type: Option<i32>,
        dataset_tags: Option<String>,
        dataset_status: Option<i32>,
        publish_type: Option<i32>,
        page: Option<i32>,
        size: Option<i32>,
        order_by: Option<String>,
        order_keyword: Option<String>,
        query_all: Option<bool>,
        start_date: Option<String>,
        end_date: Option<String>,
        selected_version_id: Option<i32>,
        selected_version_no: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::filter_datasets(
            self, search, dataset_id, dataset_name, dataset_type,
            dataset_tags, dataset_status, publish_type, page, size,
            order_by, order_keyword, query_all, start_date, end_date,
            selected_version_id, selected_version_no,
        )
    }

    /// Return the field/schema definition for a dataset.
    ///
    /// Kappa ≥ 2.13 merges `dataset_outputs` (strictest `nullable`). Create still allows
    /// missing outputs; Labelled / Verified require them.
    pub fn get_dataset_fields(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::get_dataset_fields(self, dataset_id)
    }

    /// Soft-delete a dataset (sets `datasetStatus = 0`).
    ///
    /// Recover with [`Self::recover_datasets`] while still inside the window.
    /// Kappa ≥ 2.13: after expiry the dataset is status **5** and recover is `409`.
    #[pyo3(signature = (dataset_id, remark=None))]
    pub fn delete_dataset(
        &self,
        dataset_id: i32,
        remark: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::delete_dataset(self, dataset_id, remark)
    }

    /// Recover soft-deleted datasets (`POST .../datasets/recover`).
    ///
    /// Kappa ≥ 2.13: `409 DATASET_PERMANENTLY_DELETED` after the expiry window.
    pub fn recover_datasets(&self, dataset_ids: Vec<i32>) -> PyResult<PyObject> {
        Datasets::recover_datasets(self, dataset_ids)
    }

    /// Check whether a dataset name is available (`GET .../nameAvailability`).
    pub fn check_dataset_name_availability(&self, dataset_name: String) -> PyResult<PyObject> {
        Datasets::check_dataset_name_availability(self, &dataset_name)
    }

    // --- label management ---

    /// Add one or more label strings to a dataset.
    pub fn add_dataset_labels(
        &self,
        dataset_id: i32,
        labels: Vec<String>,
    ) -> PyResult<PyObject> {
        Datasets::add_dataset_labels(self, dataset_id, labels)
    }

    /// List all labels for a dataset as typed `DatasetLabel` objects.
    pub fn get_dataset_labels(&self, dataset_id: i32) -> PyResult<Vec<DatasetLabel>> {
        Datasets::get_dataset_labels(self, dataset_id)
    }

    /// List label strings for a dataset in API order (convenience for training scripts).
    pub fn get_dataset_label_names(&self, dataset_id: i32) -> PyResult<Vec<String>> {
        Datasets::get_dataset_label_names(self, dataset_id)
    }

    /// Rename an existing label by ID.
    pub fn update_dataset_label(
        &self,
        dataset_id: i32,
        label_id: i32,
        label: String,
    ) -> PyResult<PyObject> {
        Datasets::update_dataset_label(self, dataset_id, label_id, label)
    }

    // --- entity read operations ---

    /// List all entities (samples) in a dataset version.
    ///
    /// .. deprecated::
    ///    Backend unpaginated ``GET .../datasetEntities/{id}`` is deprecated (2.9).
    ///    Prefer :meth:`filter_dataset_entities`.
    #[pyo3(signature = (dataset_id, version_id=None))]
    pub fn list_dataset_entities(
        &self,
        dataset_id: i32,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        Python::with_gil(|py| -> PyResult<()> {
            let warnings = py.import("warnings")?;
            warnings.call_method1(
                "warn",
                (
                    "list_dataset_entities uses a deprecated unpaginated API; prefer filter_dataset_entities",
                    py.import("builtins")?.getattr("DeprecationWarning")?,
                ),
            )?;
            Ok(())
        })?;
        Datasets::list_dataset_entities(self, dataset_id, version_id)
    }

    /// Get a single entity by ID.
    #[pyo3(signature = (dataset_id, entity_id, version_id=None))]
    pub fn get_dataset_entity(
        &self,
        dataset_id: i32,
        entity_id: String,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        Datasets::get_dataset_entity(self, dataset_id, &entity_id, version_id)
    }

    /// Paginated entity search with optional name / status / version filters.
    ///
    /// Pages are **1-based** (default `page=1`), matching the backend's
    /// `offset = (page - 1) * size`. Sort direction is `order` here (dataset filters use
    /// `order_keyword`). `assignment_filter` is `"assigned"` or `"not_assigned"` (default).
    /// There is no server-side `split` filter — read `dsEntityInfo.split` from each item.
    #[pyo3(signature = (dataset_id, entity_name=None, entity_status=None, version_id=None, page=None, size=None, order_by=None, order=None, entity_id=None, location_id=None, assignment_filter=None, start_date=None, end_date=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn filter_dataset_entities(
        &self,
        dataset_id: i32,
        entity_name: Option<String>,
        entity_status: Option<i32>,
        version_id: Option<i32>,
        page: Option<i32>,
        size: Option<i32>,
        order_by: Option<String>,
        order: Option<String>,
        entity_id: Option<String>,
        location_id: Option<i32>,
        assignment_filter: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::filter_dataset_entities(
            self, dataset_id, entity_name, entity_status,
            version_id, page, size, order_by, order,
            entity_id, location_id, assignment_filter, start_date, end_date,
        )
    }

    /// Cheap filtered entity count (`GET …/datasetEntities/count/{id}`, Kappa ≥ 2.13).
    ///
    /// Same filters as [`Self::filter_dataset_entities`] without pagination. Returns `{total: N}`.
    #[pyo3(signature = (dataset_id, entity_name=None, entity_status=None, version_id=None, entity_id=None, location_id=None, assignment_filter=None, start_date=None, end_date=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn count_dataset_entities(
        &self,
        dataset_id: i32,
        entity_name: Option<String>,
        entity_status: Option<i32>,
        version_id: Option<i32>,
        entity_id: Option<String>,
        location_id: Option<i32>,
        assignment_filter: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::count_dataset_entities(
            self,
            dataset_id,
            entity_name,
            entity_status,
            version_id,
            entity_id,
            location_id,
            assignment_filter,
            start_date,
            end_date,
        )
    }

    /// Bulk soft-delete entities by their string IDs (async job on Kappa ≥ 2.11 — returns `jobId`).
    #[pyo3(signature = (dataset_entity_ids, remark, version_id=None))]
    pub fn delete_dataset_entities(
        &self,
        dataset_entity_ids: Vec<String>,
        remark: String,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        Datasets::delete_dataset_entities(self, dataset_entity_ids, remark, version_id)
    }

    /// Recover soft-deleted entities (async job on Kappa ≥ 2.11 — returns `jobId`).
    #[pyo3(signature = (dataset_entity_ids, version_id=None))]
    pub fn recover_dataset_entities(
        &self,
        dataset_entity_ids: Vec<String>,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        Datasets::recover_dataset_entities(self, dataset_entity_ids, version_id)
    }

    /// Upload additional files onto an existing entity.
    ///
    /// `file_category`: ``"input"`` (default) or ``"output"``. Max 2 GB per file.
    #[pyo3(signature = (dataset_id, entity_id, file_paths, file_category=None, check_permission=false))]
    pub fn upload_dataset_entity_files(
        &self,
        dataset_id: i32,
        entity_id: String,
        file_paths: Vec<String>,
        file_category: Option<String>,
        check_permission: bool,
    ) -> PyResult<PyObject> {
        Datasets::upload_dataset_entity_files(
            self,
            dataset_id,
            &entity_id,
            file_paths,
            file_category,
            check_permission,
        )
    }

    /// Soft-delete entity files by file ID list (async job on Kappa ≥ 2.11 — returns `jobId`).
    pub fn delete_dataset_entity_files(
        &self,
        entity_file_ids: Vec<String>,
    ) -> PyResult<PyObject> {
        Datasets::delete_dataset_entity_files(self, entity_file_ids)
    }

    /// Start an async bulk entity upload (`archive` zip or `csv`).
    ///
    /// * CSV max **2 GB**; archive `.zip` max **50 GB** (streamed from disk).
    /// * `archive_layout` required for archive: ``input_output`` | ``classes``.
    /// * `on_upload_progress(bytes_sent, total_bytes, percent)` — transfer % (FE parity).
    /// * Returns start response dict with ``jobId``; poll with [`Self::get_bulk_upload_job`].
    #[pyo3(signature = (
        dataset_id,
        file_path,
        upload_type,
        labeling_algo,
        source=None,
        dataset_schema=None,
        bulk_split=None,
        archive_layout=None,
        strict=true,
        idempotency_key=None,
        on_upload_progress=None,
        check_permission=false
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn bulk_upload_dataset_entities(
        &self,
        dataset_id: i32,
        file_path: String,
        upload_type: String,
        labeling_algo: String,
        source: Option<String>,
        dataset_schema: Option<&Bound<'_, PyAny>>,
        bulk_split: Option<String>,
        archive_layout: Option<String>,
        strict: bool,
        idempotency_key: Option<String>,
        on_upload_progress: Option<PyObject>,
        check_permission: bool,
    ) -> PyResult<PyObject> {
        let schema = match dataset_schema {
            Some(v) => {
                let s = py_json_dumps(v)?;
                Some(serde_json::from_str(&s).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "dataset_schema must be JSON-serializable: {}",
                        e
                    ))
                })?)
            }
            None => None,
        };
        Datasets::bulk_upload_dataset_entities(
            self,
            dataset_id,
            &file_path,
            &upload_type,
            &labeling_algo,
            source,
            schema,
            bulk_split,
            archive_layout,
            strict,
            idempotency_key,
            on_upload_progress,
            check_permission,
        )
    }

    /// Get bulk upload job status as a [`BulkUploadJob`] (use `.percent`, `.status`, `.phase`).
    pub fn get_bulk_upload_job(
        &self,
        dataset_id: i32,
        job_id: String,
    ) -> PyResult<Py<BulkUploadJob>> {
        Datasets::get_bulk_upload_job(self, dataset_id, &job_id)
    }

    /// List bulk upload jobs for a dataset.
    pub fn list_bulk_upload_jobs(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::list_bulk_upload_jobs(self, dataset_id)
    }

    /// Cancel a bulk upload job.
    pub fn cancel_bulk_upload_job(&self, dataset_id: i32, job_id: String) -> PyResult<PyObject> {
        Datasets::cancel_bulk_upload_job(self, dataset_id, &job_id)
    }

    /// Cancel stale bulk upload jobs for a dataset.
    pub fn cancel_stale_bulk_upload_jobs(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::cancel_stale_bulk_upload_jobs(self, dataset_id)
    }

    /// Poll until bulk upload job finishes or times out.
    ///
    /// `on_progress(job: BulkUploadJob)` is called each poll (job-side %, like the React panel).
    #[pyo3(signature = (dataset_id, job_id, poll_interval_secs=None, timeout_secs=None, on_progress=None))]
    pub fn wait_for_bulk_upload_job(
        &self,
        dataset_id: i32,
        job_id: String,
        poll_interval_secs: Option<f64>,
        timeout_secs: Option<f64>,
        on_progress: Option<PyObject>,
    ) -> PyResult<Py<BulkUploadJob>> {
        Datasets::wait_for_bulk_upload_job(
            self,
            dataset_id,
            &job_id,
            poll_interval_secs,
            timeout_secs,
            on_progress,
        )
    }

    // --- bulk mutations (Kappa ≥ 2.11) ---

    /// Mark default-algorithm entities as labeled.
    ///
    /// Pass entity IDs or `all_eligible=True`. **Poll only if the response has `jobId`:**
    /// that covers Kappa 2.11–2.12 (any size) and 2.13 multi-ID / `allEligible` (`202`).
    /// Kappa ≥ 2.13 with **one** ID may return a sync `200` result (no `jobId`) or `422`.
    #[pyo3(signature = (dataset_id, dataset_entity_ids=None, remark=None, all_eligible=None))]
    pub fn mark_dataset_entities_labeled(
        &self,
        dataset_id: i32,
        dataset_entity_ids: Option<Vec<String>>,
        remark: Option<String>,
        all_eligible: Option<bool>,
    ) -> PyResult<PyObject> {
        Datasets::mark_dataset_entities_labeled(
            self,
            dataset_id,
            dataset_entity_ids,
            remark,
            all_eligible,
        )
    }

    /// Eligible entity count for mark-labeled (`GET …/mark-labeled-stats/{id}`).
    pub fn get_mark_labeled_stats(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::get_mark_labeled_stats(self, dataset_id)
    }

    /// Self-verify eligibility counters (`GET …/self-verify-stats/{id}`).
    pub fn get_self_verify_stats(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::get_self_verify_stats(self, dataset_id)
    }

    /// Enqueue bulk self-verify (`status` 1=Pass, 3=Needs Modification).
    #[pyo3(signature = (dataset_id, status, comment=None, job_corrections_by_entity=None))]
    pub fn bulk_self_verify_dataset_entities(
        &self,
        dataset_id: i32,
        status: i32,
        comment: Option<String>,
        job_corrections_by_entity: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyObject> {
        let corrections = match job_corrections_by_entity {
            Some(v) => Some(py_json_dumps(v)?),
            None => None,
        };
        Datasets::bulk_self_verify_dataset_entities(
            self,
            dataset_id,
            status,
            comment,
            corrections,
        )
    }

    /// Enqueue auto-verify for the dataset (`dataset.manage`).
    pub fn auto_verify_dataset_entities(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::auto_verify_dataset_entities(self, dataset_id)
    }

    /// Poll a bulk-mutation job (`BulkMutationJob`).
    pub fn get_bulk_mutation_job(
        &self,
        dataset_id: i32,
        job_id: String,
    ) -> PyResult<Py<BulkMutationJob>> {
        Datasets::get_bulk_mutation_job(self, dataset_id, &job_id)
    }

    /// List recent bulk-mutation jobs for a dataset.
    pub fn list_bulk_mutation_jobs(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::list_bulk_mutation_jobs(self, dataset_id)
    }

    /// Cancel a queued/running bulk-mutation job.
    pub fn cancel_bulk_mutation_job(
        &self,
        dataset_id: i32,
        job_id: String,
    ) -> PyResult<PyObject> {
        Datasets::cancel_bulk_mutation_job(self, dataset_id, &job_id)
    }

    /// Poll until a bulk-mutation job is terminal (default timeout 1 hour).
    ///
    /// `succeeded` with 0 processed is still OK on Kappa 2.11–2.12. On ≥ 2.13 use
    /// `job.mutation_failed()` (`status=failed` or `failed_count > 0`).
    #[pyo3(signature = (dataset_id, job_id, poll_interval_secs=None, timeout_secs=None, on_progress=None))]
    pub fn wait_for_bulk_mutation_job(
        &self,
        dataset_id: i32,
        job_id: String,
        poll_interval_secs: Option<f64>,
        timeout_secs: Option<f64>,
        on_progress: Option<PyObject>,
    ) -> PyResult<Py<BulkMutationJob>> {
        Datasets::wait_for_bulk_mutation_job(
            self,
            dataset_id,
            &job_id,
            poll_interval_secs,
            timeout_secs,
            on_progress,
        )
    }

    // --- dataset version management ---

    /// Create a new dataset version (accepts `NewDatasetVersion` or a plain dict).
    ///
    /// On Kappa ≥ 2.11 the response includes `jobId` / `buildStatus`; wait with
    /// [`Self::wait_for_version_build_job`] before publish/download.
    pub fn create_dataset_version(
        &self,
        dataset_id: i32,
        version: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let body = encode_new_version_payload(version)?;
        Datasets::create_dataset_version(self, dataset_id, body)
    }

    /// List all versions for a dataset, optionally filtered by `version_availability`.
    #[pyo3(signature = (dataset_id, version_availability=None))]
    pub fn list_dataset_versions(
        &self,
        dataset_id: i32,
        version_availability: Option<i32>,
    ) -> PyResult<PyObject> {
        Datasets::list_dataset_versions(self, dataset_id, version_availability)
    }

    /// Delete a specific dataset version by version number string.
    pub fn delete_dataset_version(
        &self,
        dataset_id: i32,
        version_no: String,
    ) -> PyResult<PyObject> {
        Datasets::delete_dataset_version(self, dataset_id, &version_no)
    }

    /// Publish a dataset version.
    ///
    /// `publish_type`: 0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand, 4 Purchase.
    /// On Kappa ≥ 2.11 the archive must be `buildStatus=ready` or the API returns 409.
    pub fn publish_dataset_version(
        &self,
        dataset_id: i32,
        version_no: String,
        publish_type: i32,
    ) -> PyResult<PyObject> {
        Datasets::publish_dataset_version(self, dataset_id, &version_no, publish_type)
    }

    /// Recover a soft-deleted dataset version.
    pub fn recover_dataset_version(
        &self,
        dataset_id: i32,
        version_no: String,
    ) -> PyResult<PyObject> {
        Datasets::recover_dataset_version(self, dataset_id, &version_no)
    }

    /// Patch-release / refresh a version (enqueues archive build on Kappa ≥ 2.11).
    pub fn refresh_dataset_version(
        &self,
        dataset_id: i32,
        version_no: String,
    ) -> PyResult<PyObject> {
        Datasets::refresh_dataset_version(self, dataset_id, &version_no)
    }

    /// Poll a version archive build job.
    pub fn get_version_build_job(&self, job_id: String) -> PyResult<Py<VersionBuildJob>> {
        Datasets::get_version_build_job(self, &job_id)
    }

    /// Retry a failed/cancelled version build job.
    pub fn retry_version_build_job(&self, job_id: String) -> PyResult<PyObject> {
        Datasets::retry_version_build_job(self, &job_id)
    }

    /// Poll until a version build job is terminal (default timeout 1 hour).
    #[pyo3(signature = (job_id, poll_interval_secs=None, timeout_secs=None, on_progress=None))]
    pub fn wait_for_version_build_job(
        &self,
        job_id: String,
        poll_interval_secs: Option<f64>,
        timeout_secs: Option<f64>,
        on_progress: Option<PyObject>,
    ) -> PyResult<Py<VersionBuildJob>> {
        Datasets::wait_for_version_build_job(
            self,
            &job_id,
            poll_interval_secs,
            timeout_secs,
            on_progress,
        )
    }

    /// Fetch the sharded version package manifest (`GET …/package`).
    pub fn get_dataset_version_package_manifest(
        &self,
        dataset_id: i32,
        version_no: String,
    ) -> PyResult<PyObject> {
        Datasets::get_dataset_version_package_manifest(self, dataset_id, &version_no)
    }

    /// Download a sharded version package into the local cache (falls back to legacy zip).
    #[pyo3(signature = (dataset_id=None, dataset_name=None, version_id=None, version_no=None, dataset_path=None))]
    pub fn download_dataset_version_package(
        &self,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
    ) -> PyResult<DatasetDownloadDetails> {
        Datasets::download_dataset_version_package(
            self,
            dataset_id,
            dataset_name,
            version_id,
            version_no,
            dataset_path,
        )
    }

    /// Retry a bulk upload job (`POST …/bulk/jobs/{id}/retry`).
    #[pyo3(signature = (dataset_id, job_id, sources=None))]
    pub fn retry_bulk_upload_job(
        &self,
        dataset_id: i32,
        job_id: String,
        sources: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyObject> {
        let body = match sources {
            Some(s) => py_json_dumps(s)?,
            None => "{}".to_string(),
        };
        Datasets::retry_bulk_upload_job(self, dataset_id, &job_id, body)
    }

    /// Download an entity file to `dest_path`.
    #[pyo3(signature = (dataset_id, file_id, dest_path, as_attachment=false))]
    pub fn download_dataset_entity_file(
        &self,
        dataset_id: i32,
        file_id: String,
        dest_path: String,
        as_attachment: bool,
    ) -> PyResult<String> {
        Datasets::download_dataset_entity_file(
            self,
            dataset_id,
            &file_id,
            &dest_path,
            as_attachment,
        )
    }

    // --- tabular custom schema ---

    #[pyo3(signature = (dataset_id, schema_kind=None))]
    pub fn get_dataset_custom_schema(
        &self,
        dataset_id: i32,
        schema_kind: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::get_dataset_custom_schema(self, dataset_id, schema_kind)
    }

    pub fn put_dataset_custom_schema(
        &self,
        dataset_id: i32,
        schema: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        Datasets::put_dataset_custom_schema(self, dataset_id, py_json_dumps(schema)?)
    }

    #[pyo3(signature = (dataset_id, schema_kind=None))]
    pub fn lock_dataset_custom_schema(
        &self,
        dataset_id: i32,
        schema_kind: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::lock_dataset_custom_schema(self, dataset_id, schema_kind)
    }

    #[pyo3(signature = (dataset_id, schema_kind=None))]
    pub fn unlock_dataset_custom_schema(
        &self,
        dataset_id: i32,
        schema_kind: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::unlock_dataset_custom_schema(self, dataset_id, schema_kind)
    }

    #[pyo3(signature = (dataset_id, csv_content, sample_rows=None))]
    pub fn infer_dataset_custom_schema(
        &self,
        dataset_id: i32,
        csv_content: String,
        sample_rows: Option<i32>,
    ) -> PyResult<PyObject> {
        Datasets::infer_dataset_custom_schema(self, dataset_id, csv_content, sample_rows)
    }

    #[pyo3(signature = (dataset_id, column, schema_kind=None))]
    pub fn add_dataset_custom_schema_column(
        &self,
        dataset_id: i32,
        column: &Bound<'_, PyAny>,
        schema_kind: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::add_dataset_custom_schema_column(
            self,
            dataset_id,
            py_json_dumps(column)?,
            schema_kind,
        )
    }

    /// Load a benchmark and return a reusable `Benchmarks` object.
    pub fn load_benchmark(&self, benchmark_id: String) -> PyResult<Py<Benchmarks>> {
        Python::with_gil(|py| {
            let myself = Py::new(py, self.clone())?;
            Py::new(py, Benchmarks::new(benchmark_id, myself))
        })
    }

    // --- model registry (no card / no publish) ---

    #[pyo3(signature = (page=None, size=None, search=None))]
    pub fn filter_models(
        &self,
        page: Option<i32>,
        size: Option<i32>,
        search: Option<String>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::filter_models(self, page, size, search)
    }

    pub fn get_model(&self, model_id: String) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_model(self, &model_id)
    }

    /// Create a model. When ``check_tags`` is true (default), requires the first
    /// ``mlModelTags`` / ``modelTags`` entry to be a predefined tag for ``mlModelType``
    /// (same ``dataset_tags_{type}`` catalog as datasets).
    #[pyo3(signature = (model, check_tags=true))]
    pub fn create_model(
        &self,
        model: &Bound<'_, PyAny>,
        check_tags: bool,
    ) -> PyResult<PyObject> {
        let body = py_json_dumps(model)?;
        if check_tags {
            validate_create_tags_payload(
                self,
                &body,
                &["mlModelType", "modelType", "ml_model_type", "model_type"],
                &["mlModelTags", "modelTags", "ml_model_tags", "model_tags"],
            )?;
        }
        crate::models_api::ModelsApi::create_model(self, body)
    }

    pub fn update_model(&self, model_id: String, update: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::update_model(self, &model_id, py_json_dumps(update)?)
    }

    #[pyo3(signature = (model_id, remark=None))]
    pub fn delete_model(&self, model_id: String, remark: Option<String>) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::delete_model(self, &model_id, remark)
    }

    pub fn get_model_history(&self, model_id: String) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_model_history(self, &model_id)
    }

    pub fn create_model_version(
        &self,
        model_id: String,
        version: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::create_model_version(self, &model_id, py_json_dumps(version)?)
    }

    pub fn list_model_versions(&self, model_id: String) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::list_model_versions(self, &model_id)
    }

    pub fn get_model_version(&self, model_id: String, version_id: i32) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_model_version(self, &model_id, version_id)
    }

    pub fn update_model_version(
        &self,
        model_id: String,
        version_id: i32,
        update: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::update_model_version(
            self,
            &model_id,
            version_id,
            py_json_dumps(update)?,
        )
    }

    pub fn delete_model_version(&self, model_id: String, version_id: i32) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::delete_model_version(self, &model_id, version_id)
    }

    pub fn create_model_inference(
        &self,
        model_id: String,
        inference: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::create_model_inference(
            self,
            &model_id,
            py_json_dumps(inference)?,
        )
    }

    pub fn list_model_inferences(&self, model_id: String) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::list_model_inferences(self, &model_id)
    }

    pub fn update_model_inference(
        &self,
        model_id: String,
        inference_id: i32,
        update: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::update_model_inference(
            self,
            &model_id,
            inference_id,
            py_json_dumps(update)?,
        )
    }

    /// Upload an inference artifact file.
    ///
    /// `file_category`: 1 Training, 2 Inference (default), 3 Model, 4 Data, 5 Other,
    /// 6 Prediction output (Kappa ≥ 2.14; requires `entity_id` + `field_name`).
    /// Set `replace=True` to PATCH an existing artifact.
    ///
    /// Large files use a multipart upload session automatically: pass `use_session=True` to
    /// force it, `False` to insist on the plain upload (which the server rejects with
    /// `413 USE_KAPPA_APK` past its sync cap). Sessions upsert by file name, so `replace`
    /// has no effect on them.
    #[pyo3(signature = (
        model_id,
        inference_id,
        file_path,
        file_category=None,
        replace=false,
        use_session=None,
        entity_id=None,
        field_name=None
    ))]
    pub fn upload_model_inference_file(
        &self,
        model_id: String,
        inference_id: i32,
        file_path: String,
        file_category: Option<i32>,
        replace: bool,
        use_session: Option<bool>,
        entity_id: Option<String>,
        field_name: Option<String>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::upload_model_inference_file(
            self,
            &model_id,
            inference_id,
            &file_path,
            file_category,
            replace,
            use_session,
            entity_id.as_deref(),
            field_name.as_deref(),
        )
    }

    /// Upload a prediction output file (`file_category=6`) and return a predicted file-ref.
    ///
    /// Kappa ≥ 2.14. Use the returned dict as `predicted[field_name]` (e.g. `output_image`).
    /// Older backends reject category 6.
    #[pyo3(signature = (model_id, inference_id, file_path, entity_id, field_name, content_type=None))]
    pub fn upload_prediction_output_file(
        &self,
        py: Python<'_>,
        model_id: String,
        inference_id: i32,
        file_path: String,
        entity_id: String,
        field_name: String,
        content_type: Option<String>,
    ) -> PyResult<PyObject> {
        let uploaded = crate::models_api::ModelsApi::upload_model_inference_file(
            self,
            &model_id,
            inference_id,
            &file_path,
            Some(crate::model_artifacts::FILE_CATEGORY_PREDICTION_OUTPUT),
            false,
            Some(false),
            Some(&entity_id),
            Some(&field_name),
        )?;
        prediction_file_ref_from_upload(py, uploaded.bind(py), None, content_type)
    }

    /// Write an inference result and its artifacts in one call, following the model's schema.
    ///
    /// Reads the model's effective inference schema, shapes `predictions` / `metrics` into the
    /// `results.predictions[]` document it requires, validates the payload server-side, creates
    /// the inference and then uploads `artifacts` — files, directories of weight shards, or
    /// both. Large files automatically take a resumable multipart upload session; artifacts
    /// already attached with the same name and size are skipped.
    ///
    /// Pass `inference_result` to send a document you built yourself; `predictions` and
    /// `metrics` are then ignored.
    ///
    /// Returns `{"modelId", "inferenceId", "schema", "validation", "artifacts",
    /// "inferenceResult", "pipeline"}`.
    ///
    /// A bare `predicted` string is wrapped into the schema's single required string key
    /// (`class_name` on older templates, `label` / `output_text` on Kappa ≥ 2.14). Extra
    /// predicted keys are kept. `original` is optional and not used as ground truth.
    ///
    /// After artifacts upload, the client auto-detects an inference pipeline from the
    /// running program + files and PUTs it on the inference (Kappa ≥ 2.14; 404 is skipped).
    ///
    /// # Python Example
    /// ```python
    /// written = client.write_model_inference(
    ///     model_id,
    ///     predictions=[{"entityId": e.entity_id, "predicted": {"label": "pizza"}}],
    ///     metrics={"accuracy": 0.93},
    ///     artifacts=["./checkpoints"],
    ///     on_progress=lambda name, sent, total, pct: print(name, pct),
    /// )
    /// print(written["inferenceId"])
    /// ```
    #[pyo3(signature = (
        model_id,
        predictions=None,
        metrics=None,
        inference_result=None,
        artifacts=None,
        benchmark_id=None,
        file_category=None,
        validate=true,
        use_session=None,
        skip_existing=true,
        on_progress=None,
        attach_pipeline=true,
        pipeline=None,
        pipeline_type=None,
        model=None,
        entrypoint=None,
        endpoint_url=None
    ))]
    pub fn write_model_inference(
        &self,
        py: Python<'_>,
        model_id: String,
        predictions: Option<&Bound<'_, PyAny>>,
        metrics: Option<&Bound<'_, PyAny>>,
        inference_result: Option<&Bound<'_, PyAny>>,
        artifacts: Option<&Bound<'_, PyAny>>,
        benchmark_id: Option<String>,
        file_category: Option<i32>,
        validate: bool,
        use_session: Option<bool>,
        skip_existing: bool,
        on_progress: Option<PyObject>,
        attach_pipeline: bool,
        pipeline: Option<&Bound<'_, PyAny>>,
        pipeline_type: Option<i32>,
        model: Option<&Bound<'_, PyAny>>,
        entrypoint: Option<String>,
        endpoint_url: Option<String>,
    ) -> PyResult<PyObject> {
        let artifact_paths = py_path_list(artifacts)?;
        let request = crate::inference_writer::InferenceWrite {
            predictions: py_json_value(predictions)?,
            metrics: py_json_value(metrics)?,
            inference_result: py_json_value(inference_result)?,
            benchmark_id: benchmark_id.clone(),
            artifacts: artifact_paths.clone(),
            file_category,
            validate,
            use_session,
            skip_existing,
            on_progress,
        };
        let mut written = crate::inference_writer::InferenceWriter::write(self, &model_id, request)?;
        if attach_pipeline {
            let inference_id = written
                .get("inferenceId")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let default_type = if endpoint_url.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_some() {
                crate::pipeline_detect::PIPELINE_TYPE_ONLINE
            } else if benchmark_id.is_some() {
                crate::pipeline_detect::PIPELINE_TYPE_BENCHMARK
            } else {
                crate::pipeline_detect::PIPELINE_TYPE_BATCH
            };
            let ptype = pipeline_type.unwrap_or(default_type);
            let status = attach_inference_pipeline(
                py,
                self,
                &model_id,
                inference_id,
                &artifact_paths,
                pipeline,
                ptype,
                model,
                entrypoint.as_deref(),
                &[],
                endpoint_url.as_deref(),
            );
            written
                .as_object_mut()
                .map(|obj| obj.insert("pipeline".to_string(), status));
        }
        crate::utils::python_json::json_value_to_pyobject(py, &written)
    }

    /// Upload a set of artifacts (files, directories, weight shards) to one inference.
    ///
    /// Files go up one at a time, since the server admits only a couple of concurrent large
    /// uploads per model, and each picks its own transport: the plain upload for sidecars, a
    /// resumable multipart session past the sync cap. `skip_existing` compares against the
    /// package manifest so a re-run after a failure only sends what is missing.
    ///
    /// `on_progress` receives `(file_name, bytes_sent, total_bytes, percent)`.
    #[pyo3(signature = (
        model_id,
        inference_id,
        paths,
        file_category=None,
        use_session=None,
        skip_existing=true,
        checksum=None,
        on_progress=None
    ))]
    pub fn upload_model_artifacts(
        &self,
        py: Python<'_>,
        model_id: String,
        inference_id: i32,
        paths: &Bound<'_, PyAny>,
        file_category: Option<i32>,
        use_session: Option<bool>,
        skip_existing: bool,
        checksum: Option<bool>,
        on_progress: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let uploaded = ModelArtifactsApi::upload_model_artifacts(
            self,
            &model_id,
            inference_id,
            py_path_list(Some(paths))?,
            file_category,
            use_session,
            skip_existing,
            checksum,
            on_progress,
        )?;
        crate::utils::python_json::json_value_to_pyobject(py, &uploaded)
    }

    /// Upload a large artifact through an S3 multipart session with progress callbacks.
    ///
    /// `on_progress` receives `(bytes_sent, total_bytes, percent)` after each part. Parts are
    /// retried against a freshly presigned URL, and the session is checkpointed so an
    /// interrupted run resumes where it stopped (`resume=False` to always start over).
    /// `file_category` defaults to 3 (Model) on this route, unlike the plain upload's 2.
    #[pyo3(signature = (
        model_id,
        inference_id,
        file_path,
        file_category=None,
        on_progress=None,
        checksum=false,
        resume=true,
        max_retries=5,
        wait_for_slot=true
    ))]
    pub fn upload_model_artifact_session(
        &self,
        model_id: String,
        inference_id: i32,
        file_path: String,
        file_category: Option<i32>,
        on_progress: Option<PyObject>,
        checksum: bool,
        resume: bool,
        max_retries: u32,
        wait_for_slot: bool,
    ) -> PyResult<PyObject> {
        ModelArtifactsApi::upload_model_artifact_session_with(
            self,
            &model_id,
            inference_id,
            &file_path,
            crate::model_artifacts::SessionUploadOptions {
                file_category,
                on_progress,
                checksum,
                resume,
                max_retries,
                wait_for_slot,
            },
        )
    }

    /// Inspect a pending artifact upload session.
    pub fn get_model_artifact_upload_session(
        &self,
        model_id: String,
        inference_id: i32,
        upload_id: String,
    ) -> PyResult<PyObject> {
        ModelArtifactsApi::get_model_artifact_upload_session(
            self,
            &model_id,
            inference_id,
            &upload_id,
        )
    }

    /// Abort a pending artifact upload session (frees an admission slot).
    pub fn abort_model_artifact_upload_session(
        &self,
        model_id: String,
        inference_id: i32,
        upload_id: String,
    ) -> PyResult<PyObject> {
        ModelArtifactsApi::abort_model_artifact_upload_session(
            self,
            &model_id,
            inference_id,
            &upload_id,
        )
    }

    /// Artifact manifest for an inference (`files[]` with sizes, categories, download URLs).
    pub fn get_model_inference_artifacts_package(
        &self,
        model_id: String,
        inference_id: i32,
    ) -> PyResult<PyObject> {
        ModelArtifactsApi::get_model_inference_artifacts_package(self, &model_id, inference_id)
    }

    /// Artifact manifest for the inference linked to a model version.
    pub fn get_model_version_artifacts_package(
        &self,
        model_id: String,
        version_id: i32,
    ) -> PyResult<PyObject> {
        ModelArtifactsApi::get_model_version_artifacts_package(self, &model_id, version_id)
    }

    /// Download one artifact by file ID.
    ///
    /// `redirect=True` follows a presigned object-storage URL instead of streaming through
    /// the gateway; it only works where that storage is reachable.
    #[pyo3(signature = (model_id, inference_id, file_id, dest_path, redirect=false))]
    pub fn download_model_inference_artifact_file(
        &self,
        model_id: String,
        inference_id: i32,
        file_id: String,
        dest_path: String,
        redirect: bool,
    ) -> PyResult<String> {
        ModelArtifactsApi::download_model_inference_artifact_file(
            self,
            &model_id,
            inference_id,
            &file_id,
            &dest_path,
            redirect,
        )
    }

    /// Download every artifact of an inference into `dest_dir`, file by file.
    ///
    /// Prefer this over [`Self::download_model_inference_artifacts`] for big packages: the
    /// single zip is refused with `409 PACKAGE_TOO_LARGE_FOR_ZIP` once it grows past the
    /// server's zip ceiling. Falls back to the zip on backends without package routes.
    #[pyo3(signature = (model_id, inference_id, dest_dir, redirect=false))]
    pub fn download_model_inference_artifacts_package(
        &self,
        model_id: String,
        inference_id: i32,
        dest_dir: String,
        redirect: bool,
    ) -> PyResult<Vec<String>> {
        ModelArtifactsApi::download_model_inference_artifacts_package(
            self,
            &model_id,
            inference_id,
            &dest_dir,
            redirect,
        )
    }

    /// Download all inference artifacts as one zip (small packages only).
    pub fn download_model_inference_artifacts(
        &self,
        model_id: String,
        inference_id: i32,
        dest_path: String,
    ) -> PyResult<String> {
        crate::models_api::ModelsApi::download_model_inference_artifacts(
            self,
            &model_id,
            inference_id,
            &dest_path,
        )
    }

    /// Download a model version's artifacts as one zip (small packages only).
    pub fn download_model_version_artifacts(
        &self,
        model_id: String,
        version_id: i32,
        dest_path: String,
    ) -> PyResult<String> {
        crate::models_api::ModelsApi::download_model_version_artifacts(
            self,
            &model_id,
            version_id,
            &dest_path,
        )
    }

    pub fn get_model_version_inference(
        &self,
        model_id: String,
        version_id: i32,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_model_version_inference(self, &model_id, version_id)
    }

    pub fn get_model_inference_schema(&self, model_id: String) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_model_inference_schema(self, &model_id)
    }

    pub fn update_model_inference_schema(
        &self,
        model_id: String,
        schema: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::update_model_inference_schema(
            self,
            &model_id,
            py_json_dumps(schema)?,
        )
    }

    pub fn delete_model_inference_schema(&self, model_id: String) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::delete_model_inference_schema(self, &model_id)
    }

    pub fn validate_inference_result(
        &self,
        model_id: String,
        inference_result: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::validate_inference_result(
            self,
            &model_id,
            py_json_dumps(inference_result)?,
        )
    }

    pub fn list_inference_metrics(&self) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::list_inference_metrics(self)
    }

    pub fn list_inference_schema_types(&self) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::list_inference_schema_types(self)
    }

    #[pyo3(signature = (model_id, limit=None))]
    pub fn get_model_inference_schema_history(
        &self,
        model_id: String,
        limit: Option<i32>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_model_inference_schema_history(self, &model_id, limit)
    }

    pub fn get_inference_schema_type(&self, model_type: i32) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_inference_schema_type(self, model_type)
    }

    pub fn list_model_pipelines(&self, model_id: String) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::list_model_pipelines(self, &model_id)
    }

    pub fn create_model_pipeline(
        &self,
        model_id: String,
        version_id: i32,
        pipeline: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::create_model_pipeline(
            self,
            &model_id,
            version_id,
            py_json_dumps(pipeline)?,
        )
    }

    pub fn get_model_pipeline(&self, model_id: String, version_id: i32) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::get_model_pipeline(self, &model_id, version_id)
    }

    pub fn update_model_pipeline(
        &self,
        model_id: String,
        version_id: i32,
        pipeline: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::update_model_pipeline(
            self,
            &model_id,
            version_id,
            py_json_dumps(pipeline)?,
        )
    }

    pub fn delete_model_pipeline(&self, model_id: String, version_id: i32) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::delete_model_pipeline(self, &model_id, version_id)
    }

    pub fn validate_model_pipeline(&self, model_id: String, version_id: i32) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::validate_model_pipeline(self, &model_id, version_id)
    }

    /// Detect a pipeline draft from the running program and local artifact paths (no HTTP).
    #[pyo3(signature = (artifact_paths=None, model=None, entrypoint=None, pipeline_type=None))]
    pub fn detect_model_pipeline(
        &self,
        py: Python<'_>,
        artifact_paths: Option<&Bound<'_, PyAny>>,
        model: Option<&Bound<'_, PyAny>>,
        entrypoint: Option<String>,
        pipeline_type: Option<i32>,
    ) -> PyResult<PyObject> {
        let paths = py_path_list(artifact_paths)?;
        let detected = crate::pipeline_detect::detect(
            py,
            &paths,
            model,
            entrypoint.as_deref(),
            &[],
            pipeline_type.unwrap_or(crate::pipeline_detect::PIPELINE_TYPE_BATCH),
        );
        crate::utils::python_json::json_value_to_pyobject(py, &detected.to_status_json())
    }

    /// `PUT /models/inferences/{modelId}/{inferenceId}/pipeline` (Kappa ≥ 2.14).
    pub fn put_inference_pipeline(
        &self,
        model_id: String,
        inference_id: i32,
        pipeline: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        crate::models_api::ModelsApi::put_inference_pipeline(
            self,
            &model_id,
            inference_id,
            py_json_dumps(pipeline)?,
        )
    }

    /// Build `{artifactId, fileName, contentType}` from an upload response.
    #[pyo3(signature = (upload, file_name=None, content_type=None))]
    pub fn prediction_file_ref(
        &self,
        py: Python<'_>,
        upload: &Bound<'_, PyAny>,
        file_name: Option<String>,
        content_type: Option<String>,
    ) -> PyResult<PyObject> {
        prediction_file_ref_from_upload(py, upload, file_name, content_type)
    }

    // --- benchmark registry ---

    /// List benchmarks (unfiltered first page). Use [`Self::filter_benchmarks`] for queries.
    pub fn list_benchmarks(&self) -> PyResult<PyObject> {
        BenchmarksApi::filter_benchmarks(
            self, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        )
    }

    /// Paginated benchmark search. Sort direction is `order` (`"ASC"` / `"DESC"`).
    #[pyo3(signature = (benchmark_id=None, model_id=None, model_type=None, dataset_id=None, dataset_version_id=None, model_version_id=None, benchmark_status=None, user_id=None, report_id=None, start_date=None, end_date=None, order_by=None, order=None, page=None, size=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn filter_benchmarks(
        &self,
        benchmark_id: Option<String>,
        model_id: Option<String>,
        model_type: Option<i32>,
        dataset_id: Option<i32>,
        dataset_version_id: Option<i32>,
        model_version_id: Option<i32>,
        benchmark_status: Option<i32>,
        user_id: Option<i32>,
        report_id: Option<i32>,
        start_date: Option<String>,
        end_date: Option<String>,
        order_by: Option<String>,
        order: Option<String>,
        page: Option<i32>,
        size: Option<i32>,
    ) -> PyResult<PyObject> {
        BenchmarksApi::filter_benchmarks(
            self,
            benchmark_id,
            model_id,
            model_type,
            dataset_id,
            dataset_version_id,
            model_version_id,
            benchmark_status,
            user_id,
            report_id,
            start_date,
            end_date,
            order_by,
            order,
            page,
            size,
        )
    }

    /// Full benchmark detail as a dict (`datasetId`, `datasetVersionNo`, `benchmarkStatus`, …).
    pub fn get_benchmark(&self, benchmark_id: String) -> PyResult<PyObject> {
        BenchmarksApi::get_benchmark(self, &benchmark_id)
    }

    pub fn create_benchmark(&self, benchmark: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        BenchmarksApi::create_benchmark(self, py_json_dumps(benchmark)?)
    }

    pub fn update_benchmark(
        &self,
        benchmark_id: String,
        update: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        BenchmarksApi::update_benchmark(self, &benchmark_id, py_json_dumps(update)?)
    }

    pub fn delete_benchmark(&self, benchmark_id: String) -> PyResult<PyObject> {
        BenchmarksApi::delete_benchmark(self, &benchmark_id)
    }

    /// Attach a model version's inference results to the benchmark (status 4 → 5).
    pub fn complete_benchmark_inference(
        &self,
        benchmark_id: String,
        model_version_id: i32,
    ) -> PyResult<PyObject> {
        BenchmarksApi::complete_benchmark_inference(self, &benchmark_id, model_version_id)
    }

    /// Stage/step definitions behind the benchmark progress UI.
    pub fn get_benchmark_flow_schema(&self) -> PyResult<PyObject> {
        BenchmarksApi::get_benchmark_flow_schema(self)
    }

    // --- benchmark remarks ---

    /// Read the benchmark remark thread (owner, expert and admin chat).
    pub fn list_benchmark_remarks(&self, benchmark_id: String) -> PyResult<PyObject> {
        BenchmarksApi::list_benchmark_remarks(self, &benchmark_id)
    }

    /// Post a remark (1–4000 characters). Rejected once the benchmark is closed.
    pub fn add_benchmark_remark(
        &self,
        benchmark_id: String,
        message: String,
    ) -> PyResult<PyObject> {
        BenchmarksApi::add_benchmark_remark(self, &benchmark_id, &message)
    }

    // --- benchmark expert & dataset selection ---

    /// Accept or reject an expert request as the assigned expert.
    pub fn respond_to_benchmark_expert_request(
        &self,
        benchmark_id: String,
        accept: bool,
    ) -> PyResult<PyObject> {
        BenchmarksApi::respond_to_benchmark_expert_request(self, &benchmark_id, accept)
    }

    /// Assign an expert to a benchmark (requires `benchmark.manage`).
    pub fn assign_benchmark_expert(
        &self,
        benchmark_id: String,
        expert_id: i32,
    ) -> PyResult<PyObject> {
        BenchmarksApi::assign_benchmark_expert(self, &benchmark_id, expert_id)
    }

    /// Propose an evaluation dataset version for a benchmark (expert action).
    pub fn propose_benchmark_dataset(
        &self,
        benchmark_id: String,
        dataset_id: i32,
        dataset_version_id: i32,
    ) -> PyResult<PyObject> {
        BenchmarksApi::propose_benchmark_dataset(self, &benchmark_id, dataset_id, dataset_version_id)
    }

    /// Confirm the proposed evaluation dataset. Requires the expert to have accepted first.
    pub fn confirm_benchmark_dataset(&self, benchmark_id: String) -> PyResult<PyObject> {
        BenchmarksApi::confirm_benchmark_dataset(self, &benchmark_id)
    }

    /// Reject the proposed evaluation dataset.
    pub fn reject_benchmark_dataset(&self, benchmark_id: String) -> PyResult<PyObject> {
        BenchmarksApi::reject_benchmark_dataset(self, &benchmark_id)
    }

    /// Which benchmarks are already attached to a dataset version.
    pub fn get_benchmark_dataset_attachments(
        &self,
        dataset_id: i32,
        dataset_version_id: i32,
    ) -> PyResult<PyObject> {
        BenchmarksApi::get_benchmark_dataset_attachments(self, dataset_id, dataset_version_id)
    }

    // --- benchmark review & report ---

    /// Inference results plus saved expert reviews. The first call moves status 5 → 6.
    pub fn get_benchmark_review(
        &self,
        benchmark_id: String,
        expert_id: i32,
    ) -> PyResult<PyObject> {
        BenchmarksApi::get_benchmark_review(self, &benchmark_id, expert_id)
    }

    /// Save expert review progress (`{"reviews": {entityUuid: {...}}, "finalScore": ...}`).
    pub fn save_benchmark_review(
        &self,
        benchmark_id: String,
        expert_id: i32,
        review: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        BenchmarksApi::save_benchmark_review(
            self,
            &benchmark_id,
            expert_id,
            py_json_dumps(review)?,
        )
    }

    /// Finalize the review and schedule report generation. Save the review first.
    pub fn finalize_benchmark_review(
        &self,
        benchmark_id: String,
        expert_id: i32,
    ) -> PyResult<PyObject> {
        BenchmarksApi::finalize_benchmark_review(self, &benchmark_id, expert_id)
    }

    /// Schedule report regeneration (requires `benchmark.manage`).
    pub fn regenerate_benchmark_report(&self, benchmark_id: String) -> PyResult<PyObject> {
        BenchmarksApi::regenerate_benchmark_report(self, &benchmark_id)
    }

    /// Report metadata (`reportId`, `reportName`, `reportStatus`).
    pub fn get_benchmark_report(&self, benchmark_id: String) -> PyResult<PyObject> {
        BenchmarksApi::get_benchmark_report(self, &benchmark_id)
    }

    /// Write the benchmark report PDF to `dest_path`. `lang` is `"en"` (default) or `"ru"`.
    #[pyo3(signature = (benchmark_id, dest_path, lang=None))]
    pub fn download_benchmark_report(
        &self,
        benchmark_id: String,
        dest_path: String,
        lang: Option<String>,
    ) -> PyResult<String> {
        BenchmarksApi::download_benchmark_report(self, &benchmark_id, &dest_path, lang)
    }

    // --- benchmark evaluation dataset ---

    /// Package manifest for the benchmark evaluation set (benchmark-proxied route).
    pub fn get_benchmark_dataset_package_manifest(
        &self,
        benchmark_id: String,
    ) -> PyResult<PyObject> {
        BenchmarksApi::get_benchmark_dataset_package_manifest(self, &benchmark_id)
    }

    /// Download the benchmark evaluation set the same way the web client does.
    ///
    /// Tries the dataset-services package for the benchmark's `datasetId` /
    /// `datasetVersionNo` first, then falls back to the benchmark proxy when you only hold
    /// `benchmark.read`; either path uses the legacy single zip when the manifest says so.
    /// Both IDs are read from the benchmark detail when omitted. Returns the extraction
    /// directory, cached under the OS cache dir `kappa-framework/benchmarks/{id}/`.
    #[pyo3(signature = (benchmark_id, dataset_id=None, version_no=None, dataset_path=None))]
    pub fn download_benchmark_dataset_package(
        &self,
        benchmark_id: String,
        dataset_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
    ) -> PyResult<String> {
        BenchmarksApi::download_benchmark_dataset_package(
            self,
            &benchmark_id,
            dataset_id,
            version_no,
            dataset_path,
        )
    }
}
