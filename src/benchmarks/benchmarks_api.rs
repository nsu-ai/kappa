// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Benchmark registry, review and evaluation-set download endpoints
//! (`/model-micro-services/v2/benchmarks`).

use pyo3::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::traits::ApiClient;
use crate::utils::cache_paths;
use crate::utils::package_download::{self, PackagePlan};
use crate::utils::zip_utils::{self, cache_is_complete};

const BENCHMARKS_ROOT: &str = "/model-micro-services/v2/benchmarks";
const DATASETS_ROOT: &str = "/data-micro-services/v2/datasets";

pub struct BenchmarksApi;

impl BenchmarksApi {
    // --- registry ---

    /// `GET /benchmarks` — paginated benchmark list with optional filters.
    ///
    /// Sort direction is `order` (`ASC`/`DESC`); the response is
    /// `{items, total, page, size, pages}`.
    #[allow(clippy::too_many_arguments)]
    pub fn filter_benchmarks<T: ApiClient>(
        client: &T,
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
        let mut endpoint = format!(
            "{}?page={}&size={}&orderBy={}&order={}",
            BENCHMARKS_ROOT,
            page.unwrap_or(0).max(0),
            size.unwrap_or(10),
            urlencoding::encode(&order_by.unwrap_or_else(|| "modifiedOn".to_string())),
            urlencoding::encode(&order.unwrap_or_else(|| "DESC".to_string())),
        );
        if let Some(id) = benchmark_id {
            endpoint.push_str(&format!("&benchmarkId={}", urlencoding::encode(&id)));
        }
        if let Some(id) = model_id {
            endpoint.push_str(&format!("&mlModelId={}", urlencoding::encode(&id)));
        }
        if let Some(mt) = model_type {
            endpoint.push_str(&format!("&mlModelType={}", mt));
        }
        if let Some(id) = dataset_id {
            endpoint.push_str(&format!("&datasetId={}", id));
        }
        if let Some(id) = dataset_version_id {
            endpoint.push_str(&format!("&datasetVersionId={}", id));
        }
        if let Some(id) = model_version_id {
            endpoint.push_str(&format!("&mlmodelVersionId={}", id));
        }
        if let Some(status) = benchmark_status {
            endpoint.push_str(&format!("&benchmarkStatus={}", status));
        }
        if let Some(id) = user_id {
            endpoint.push_str(&format!("&userId={}", id));
        }
        if let Some(id) = report_id {
            endpoint.push_str(&format!("&reportId={}", id));
        }
        if let Some(sd) = start_date {
            endpoint.push_str(&format!("&startDate={}", urlencoding::encode(&sd)));
        }
        if let Some(ed) = end_date {
            endpoint.push_str(&format!("&endDate={}", urlencoding::encode(&ed)));
        }
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /benchmarks/{benchmark_id}` — full benchmark detail as a dict.
    pub fn get_benchmark<T: ApiClient>(client: &T, benchmark_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("{}/{}", BENCHMARKS_ROOT, benchmark_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn create_benchmark<T: ApiClient>(client: &T, body_json: String) -> PyResult<PyObject> {
        client.make_request(
            "POST".to_string(),
            BENCHMARKS_ROOT.to_string(),
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn update_benchmark<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!("{}/{}", BENCHMARKS_ROOT, benchmark_id);
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn delete_benchmark<T: ApiClient>(client: &T, benchmark_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("{}/{}", BENCHMARKS_ROOT, benchmark_id);
        client.make_request("DELETE".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `POST /benchmarks/inferences/{benchmark_id}/{model_version_id}` — attach the model
    /// version's inference results to the benchmark (moves status 4 → 5).
    pub fn complete_benchmark_inference<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        model_version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/inferences/{}/{}",
            BENCHMARKS_ROOT, benchmark_id, model_version_id
        );
        client.make_request("POST".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /benchmarks/flow-schema` — stage/step definitions used by the web stepper.
    pub fn get_benchmark_flow_schema<T: ApiClient>(client: &T) -> PyResult<PyObject> {
        let endpoint = format!("{}/flow-schema", BENCHMARKS_ROOT);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    // --- remarks (benchmark chat) ---

    pub fn list_benchmark_remarks<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!("{}/{}/remarks", BENCHMARKS_ROOT, benchmark_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn add_benchmark_remark<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        message: &str,
    ) -> PyResult<PyObject> {
        if message.is_empty() || message.chars().count() > 4000 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "message must be 1–4000 characters",
            ));
        }
        let endpoint = format!("{}/{}/remarks", BENCHMARKS_ROOT, benchmark_id);
        let body = serde_json::json!({ "msg": message }).to_string();
        client.make_request("POST".to_string(), endpoint, Some(body), Some(client.require_token()?))
    }

    // --- expert assignment & dataset selection ---

    /// `POST /benchmarks/management/expert/{benchmark_id}/{status}` — accept (2) or reject (0)
    /// an expert request as the assigned expert.
    pub fn respond_to_benchmark_expert_request<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        accept: bool,
    ) -> PyResult<PyObject> {
        let status = if accept { 2 } else { 0 };
        let endpoint = format!(
            "{}/management/expert/{}/{}",
            BENCHMARKS_ROOT, benchmark_id, status
        );
        client.make_request("POST".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `POST /benchmarks/management/{benchmark_id}/{expert_id}` — admin assigns an expert.
    pub fn assign_benchmark_expert<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        expert_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/management/{}/{}",
            BENCHMARKS_ROOT, benchmark_id, expert_id
        );
        client.make_request("POST".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `POST /benchmarks/management/expert/{benchmark_id}/{dataset_id}/{version_id}` —
    /// expert proposes the evaluation dataset.
    pub fn propose_benchmark_dataset<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        dataset_id: i32,
        dataset_version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/management/expert/{}/{}/{}",
            BENCHMARKS_ROOT, benchmark_id, dataset_id, dataset_version_id
        );
        client.make_request("POST".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `POST /benchmarks/management/expert/{benchmark_id}/dataset/confirm`.
    pub fn confirm_benchmark_dataset<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/management/expert/{}/dataset/confirm",
            BENCHMARKS_ROOT, benchmark_id
        );
        client.make_request("POST".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `POST /benchmarks/management/expert/{benchmark_id}/dataset/reject-proposal`.
    pub fn reject_benchmark_dataset<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/management/expert/{}/dataset/reject-proposal",
            BENCHMARKS_ROOT, benchmark_id
        );
        client.make_request("POST".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /benchmarks/datasets/attachments/{dataset_id}/{version_id}` — which benchmarks
    /// already use a dataset version.
    pub fn get_benchmark_dataset_attachments<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        dataset_version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/datasets/attachments/{}/{}",
            BENCHMARKS_ROOT, dataset_id, dataset_version_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    // --- expert review & report ---

    /// `GET /benchmarks/review/{expert_id}/{benchmark_id}` — inference results plus saved
    /// expert reviews. The first call moves the benchmark from status 5 to 6.
    pub fn get_benchmark_review<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        expert_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/review/{}/{}",
            BENCHMARKS_ROOT, expert_id, benchmark_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `PUT /benchmarks/review/{expert_id}/{benchmark_id}` — save review progress.
    pub fn save_benchmark_review<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        expert_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/review/{}/{}",
            BENCHMARKS_ROOT, expert_id, benchmark_id
        );
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    /// `POST /benchmarks/review/{expert_id}/{benchmark_id}` — finalize the review and
    /// schedule report generation (202). Save with `save_benchmark_review` first.
    pub fn finalize_benchmark_review<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        expert_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "{}/review/{}/{}",
            BENCHMARKS_ROOT, expert_id, benchmark_id
        );
        client.make_request("POST".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `PATCH /benchmarks/review/report/{benchmark_id}` — schedule report regeneration (202).
    pub fn regenerate_benchmark_report<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!("{}/review/report/{}", BENCHMARKS_ROOT, benchmark_id);
        client.make_request("PATCH".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /benchmarks/report/{benchmark_id}` — report metadata (id, name, status).
    pub fn get_benchmark_report<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!("{}/report/{}", BENCHMARKS_ROOT, benchmark_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /benchmarks/report/download/{benchmark_id}/{lang}` — write the report PDF.
    ///
    /// `lang` is `"en"` or `"ru"`.
    pub fn download_benchmark_report<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        dest_path: &str,
        lang: Option<String>,
    ) -> PyResult<String> {
        let lang = lang.unwrap_or_else(|| "en".to_string());
        if lang != "en" && lang != "ru" {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "lang must be 'en' or 'ru'",
            ));
        }
        let endpoint = format!(
            "{}/report/download/{}/{}",
            BENCHMARKS_ROOT, benchmark_id, lang
        );
        let bytes = client.download_bytes(endpoint, Some(client.require_token()?))?;
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

    // --- evaluation dataset download ---

    /// `GET /benchmarks/datasets/{benchmark_id}/package` — benchmark-proxied package manifest.
    pub fn get_benchmark_dataset_package_manifest<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!("{}/datasets/{}/package", BENCHMARKS_ROOT, benchmark_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// Download the benchmark evaluation set, following the same order as the web client.
    ///
    /// 1. Dataset-services package for `datasetId` / `datasetVersionNo` (needs `dataset.read`,
    ///    which is granted when the dataset is attached to the benchmark).
    /// 2. On 401/403/404, the benchmark proxy package under
    ///    `/benchmarks/datasets/{benchmark_id}/package`.
    /// 3. Either path falls back to its legacy single zip when the manifest says
    ///    `legacySingleZip` or lists at most one shard.
    ///
    /// Returns the extraction directory. A completed download is cached, so repeat calls
    /// skip the network entirely.
    pub fn download_benchmark_dataset_package<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        dataset_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
    ) -> PyResult<String> {
        let data_dir = Self::benchmark_cache_dir(benchmark_id, dataset_path)?;
        if cache_is_complete(&data_dir) {
            return Ok(data_dir.to_string_lossy().to_string());
        }

        // The web client reads both coordinates off the benchmark detail response.
        let (dataset_id, version_no) = match (dataset_id, version_no) {
            (Some(id), Some(no)) if id > 0 && !no.is_empty() => (Some(id), Some(no)),
            (id, no) => {
                let details = Self::get_benchmark(client, benchmark_id)?;
                let value = crate::utils::python_json::pyobject_to_rust_value(
                    &details,
                    "benchmark details",
                )?;
                let resolved_id = id.or_else(|| {
                    value
                        .get("datasetId")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32)
                });
                let resolved_no = no.or_else(|| {
                    value
                        .get("datasetVersionNo")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string())
                });
                (resolved_id, resolved_no)
            }
        };

        let direct_ready = matches!((dataset_id, version_no.as_deref()), (Some(id), Some(no)) if id > 0 && !no.is_empty());
        if direct_ready {
            let id = dataset_id.unwrap_or_default();
            let no = version_no.clone().unwrap_or_default();
            match Self::download_dataset_package_into(client, id, &no, &data_dir) {
                Ok(()) => return Ok(data_dir.to_string_lossy().to_string()),
                Err(e) => {
                    let msg = e.to_string();
                    if package_download::is_pending_approval_error(&msg) {
                        return Err(PyErr::new::<pyo3::exceptions::PyPermissionError, _>(format!(
                            "Dataset download approval is still pending for dataset {}: {}",
                            id, msg
                        )));
                    }
                    // Missing dataset.read is expected for benchmark-only viewers.
                    if !package_download::is_fallback_error(&msg) {
                        return Err(e);
                    }
                }
            }
        }

        Self::download_benchmark_proxy_package_into(client, benchmark_id, &data_dir)?;
        Ok(data_dir.to_string_lossy().to_string())
    }

    fn benchmark_cache_dir(benchmark_id: &str, dataset_path: Option<String>) -> PyResult<PathBuf> {
        cache_paths::benchmark_cache_dir(dataset_path.as_deref(), benchmark_id)
    }

    /// Direct dataset-services package download into `data_dir`.
    fn download_dataset_package_into<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        version_no: &str,
        data_dir: &Path,
    ) -> PyResult<()> {
        let token = client.require_token()?;
        let encoded_version = urlencoding::encode(version_no).into_owned();
        let manifest_endpoint = format!(
            "{}/versions/{}/{}/package",
            DATASETS_ROOT, dataset_id, encoded_version
        );
        let manifest_obj =
            client.make_request("GET".to_string(), manifest_endpoint, None, Some(token.clone()))?;
        let manifest = crate::utils::python_json::pyobject_to_rust_value(
            &manifest_obj,
            "version package manifest",
        )?;

        let base = client.get_base_url().trim_end_matches('/').to_string();
        let http = client.get_http_client();
        let runtime = client.get_runtime();
        let data_dir = data_dir.to_path_buf();

        match package_download::plan_from_manifest(&manifest)? {
            PackagePlan::LegacyZip => {
                let url = format!(
                    "{}{}/versions/archive/{}/{}",
                    base, DATASETS_ROOT, dataset_id, encoded_version
                );
                runtime.block_on(async move {
                    zip_utils::download_and_extract_zip(&http, &url, Some(&token), &data_dir)
                        .await
                        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
                })
            }
            PackagePlan::Shards(shards) => runtime.block_on(async move {
                package_download::download_shards(&http, &token, &data_dir, &shards, |name| {
                    format!(
                        "{}{}/versions/{}/{}/package/shards/{}",
                        base,
                        DATASETS_ROOT,
                        dataset_id,
                        encoded_version,
                        package_download::encode_shard_name(name)
                    )
                })
                .await
            }),
        }
    }

    /// Benchmark-proxied package download into `data_dir` (needs only `benchmark.read`).
    fn download_benchmark_proxy_package_into<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        data_dir: &Path,
    ) -> PyResult<()> {
        let token = client.require_token()?;
        let base = client.get_base_url().trim_end_matches('/').to_string();
        let http = client.get_http_client();
        let runtime = client.get_runtime();
        let legacy_url = format!("{}{}/datasets/download/{}", base, BENCHMARKS_ROOT, benchmark_id);

        let manifest = match Self::get_benchmark_dataset_package_manifest(client, benchmark_id) {
            Ok(obj) => Some(crate::utils::python_json::pyobject_to_rust_value(
                &obj,
                "benchmark package manifest",
            )?),
            Err(e) => {
                let msg = e.to_string();
                // A 409 means the archive build is still running; anything else just means
                // this backend has no manifest route, so try the legacy zip.
                if msg.contains("409") || msg.to_ascii_uppercase().contains("NOT_READY") {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Benchmark dataset archive is not ready yet: {}",
                        msg
                    )));
                }
                None
            }
        };

        let plan = match &manifest {
            Some(value) => package_download::plan_from_manifest(value)?,
            None => PackagePlan::LegacyZip,
        };

        let data_dir = data_dir.to_path_buf();
        match plan {
            PackagePlan::LegacyZip => runtime.block_on(async move {
                zip_utils::download_and_extract_zip(&http, &legacy_url, Some(&token), &data_dir)
                    .await
                    .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
            }),
            PackagePlan::Shards(shards) => runtime.block_on(async move {
                package_download::download_shards(&http, &token, &data_dir, &shards, |name| {
                    format!(
                        "{}{}/datasets/{}/package/shards/{}",
                        base,
                        BENCHMARKS_ROOT,
                        benchmark_id,
                        package_download::encode_shard_name(name)
                    )
                })
                .await
            }),
        }
    }
}
