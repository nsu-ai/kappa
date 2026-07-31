// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Model registry + inference-schema HTTP helpers (no card / no publish).

use pyo3::prelude::*;

use crate::traits::ApiClient;

pub struct ModelsApi;

impl ModelsApi {
    // --- models core ---

    pub fn filter_models<T: ApiClient>(
        client: &T,
        page: Option<i32>,
        size: Option<i32>,
        search: Option<String>,
    ) -> PyResult<PyObject> {
        let page = page.unwrap_or(1);
        let size = size.unwrap_or(20);
        let mut endpoint = format!(
            "/model-micro-services/v2/models/filter?page={}&size={}",
            page, size
        );
        if let Some(s) = search {
            endpoint.push_str(&format!("&search={}", urlencoding::encode(&s)));
        }
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn get_model<T: ApiClient>(client: &T, model_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/{}", model_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn create_model<T: ApiClient>(client: &T, body_json: String) -> PyResult<PyObject> {
        let endpoint = "/model-micro-services/v2/models".to_string();
        client.make_request(
            "POST".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn update_model<T: ApiClient>(
        client: &T,
        model_id: &str,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/{}", model_id);
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn delete_model<T: ApiClient>(
        client: &T,
        model_id: &str,
        _remark: Option<String>,
    ) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/{}", model_id);
        client.make_request(
            "DELETE".to_string(),
            endpoint,
            None,
            Some(client.require_token()?),
        )
    }

    pub fn get_model_history<T: ApiClient>(client: &T, model_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/history/{}", model_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    // --- versions (no publish) ---

    pub fn create_model_version<T: ApiClient>(
        client: &T,
        model_id: &str,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/versions/{}", model_id);
        client.make_request(
            "POST".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn list_model_versions<T: ApiClient>(client: &T, model_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/versions/{}", model_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn get_model_version<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/versions/{}/{}",
            model_id, version_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn update_model_version<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/versions/{}/{}",
            model_id, version_id
        );
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn delete_model_version<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/versions/{}/{}",
            model_id, version_id
        );
        client.make_request(
            "DELETE".to_string(),
            endpoint,
            None,
            Some(client.require_token()?),
        )
    }

    // --- inferences ---

    pub fn create_model_inference<T: ApiClient>(
        client: &T,
        model_id: &str,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/inferences/{}", model_id);
        client.make_request(
            "POST".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn list_model_inferences<T: ApiClient>(client: &T, model_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/inferences/{}", model_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn update_model_inference<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/inferences/{}/{}",
            model_id, inference_id
        );
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    // --- inference schema ---

    pub fn get_model_inference_schema<T: ApiClient>(
        client: &T,
        model_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/{}/inference-schema",
            model_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn update_model_inference_schema<T: ApiClient>(
        client: &T,
        model_id: &str,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/{}/inference-schema",
            model_id
        );
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn delete_model_inference_schema<T: ApiClient>(
        client: &T,
        model_id: &str,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/{}/inference-schema",
            model_id
        );
        client.make_request(
            "DELETE".to_string(),
            endpoint,
            None,
            Some(client.require_token()?),
        )
    }

    pub fn validate_inference_result<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_result_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/{}/inference-schema/validate",
            model_id
        );
        let result: serde_json::Value = serde_json::from_str(&inference_result_json).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "inference_result must be JSON: {}",
                e
            ))
        })?;
        let body = serde_json::json!({ "inferenceResult": result }).to_string();
        client.make_request(
            "POST".to_string(),
            endpoint,
            Some(body),
            Some(client.require_token()?),
        )
    }

    pub fn list_inference_metrics<T: ApiClient>(client: &T) -> PyResult<PyObject> {
        let endpoint = "/model-micro-services/v2/models/inference-metrics".to_string();
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn list_inference_schema_types<T: ApiClient>(client: &T) -> PyResult<PyObject> {
        let endpoint = "/model-micro-services/v2/models/inference-schemas/types".to_string();
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    // --- pipelines ---

    pub fn list_model_pipelines<T: ApiClient>(client: &T, model_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/models/pipelines/{}", model_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn create_model_pipeline<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/pipelines/{}/{}",
            model_id, version_id
        );
        client.make_request(
            "POST".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn get_model_pipeline<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/pipelines/{}/{}",
            model_id, version_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn update_model_pipeline<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/pipelines/{}/{}",
            model_id, version_id
        );
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn delete_model_pipeline<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/pipelines/{}/{}",
            model_id, version_id
        );
        client.make_request(
            "DELETE".to_string(),
            endpoint,
            None,
            Some(client.require_token()?),
        )
    }

    pub fn validate_model_pipeline<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/pipelines/{}/{}/validate",
            model_id, version_id
        );
        client.make_request(
            "POST".to_string(),
            endpoint,
            None,
            Some(client.require_token()?),
        )
    }

    // --- benchmarks registry ---

    pub fn list_benchmarks<T: ApiClient>(client: &T) -> PyResult<PyObject> {
        let endpoint = "/model-micro-services/v2/benchmarks".to_string();
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn create_benchmark<T: ApiClient>(client: &T, body_json: String) -> PyResult<PyObject> {
        let endpoint = "/model-micro-services/v2/benchmarks".to_string();
        client.make_request(
            "POST".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn update_benchmark<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/benchmarks/{}", benchmark_id);
        client.make_request(
            "PUT".to_string(),
            endpoint,
            Some(body_json),
            Some(client.require_token()?),
        )
    }

    pub fn delete_benchmark<T: ApiClient>(client: &T, benchmark_id: &str) -> PyResult<PyObject> {
        let endpoint = format!("/model-micro-services/v2/benchmarks/{}", benchmark_id);
        client.make_request(
            "DELETE".to_string(),
            endpoint,
            None,
            Some(client.require_token()?),
        )
    }

    /// Mark benchmark inference complete for a model version (distinct from model inference POST).
    pub fn complete_benchmark_inference<T: ApiClient>(
        client: &T,
        benchmark_id: &str,
        model_version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/benchmarks/inferences/{}/{}",
            benchmark_id, model_version_id
        );
        client.make_request(
            "POST".to_string(),
            endpoint,
            None,
            Some(client.require_token()?),
        )
    }

    // --- inference / version artifacts ---

    /// Upload or replace an inference artifact file.
    ///
    /// `file_category`: 1 Training, 2 Inference (default), 3 Model, 4 Data, 5 Other.
    /// `replace`: when true uses PATCH, otherwise POST.
    pub fn upload_model_inference_file<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        file_path: &str,
        file_category: Option<i32>,
        replace: bool,
    ) -> PyResult<PyObject> {
        let category = file_category.unwrap_or(2);
        if !(1..=5).contains(&category) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "file_category must be 1–5 (1 Training, 2 Inference, 3 Model, 4 Data, 5 Other)",
            ));
        }
        let bytes = std::fs::read(file_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Cannot read file {}: {}",
                file_path, e
            ))
        })?;
        let fname = std::path::Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("artifact.bin")
            .to_string();
        let endpoint = format!(
            "/model-micro-services/v2/models/inferences/files/{}/{}?file_category={}",
            model_id, inference_id, category
        );
        let method = if replace { "PATCH" } else { "POST" };
        client.submit_named_file(
            method,
            endpoint,
            "file",
            bytes,
            fname,
            Some(client.require_token()?),
        )
    }

    pub fn download_model_inference_artifacts<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        dest_path: &str,
    ) -> PyResult<String> {
        let endpoint = format!(
            "/model-micro-services/v2/models/inferences/files/{}/{}/zip",
            model_id, inference_id
        );
        write_download(client, endpoint, dest_path)
    }

    pub fn download_model_version_artifacts<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
        dest_path: &str,
    ) -> PyResult<String> {
        let endpoint = format!(
            "/model-micro-services/v2/models/versions/{}/{}/artifacts/zip",
            model_id, version_id
        );
        write_download(client, endpoint, dest_path)
    }

    pub fn get_model_version_inference<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/versions/inference/{}/{}",
            model_id, version_id
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn get_model_inference_schema_history<T: ApiClient>(
        client: &T,
        model_id: &str,
        limit: Option<i32>,
    ) -> PyResult<PyObject> {
        let limit = limit.unwrap_or(50);
        let endpoint = format!(
            "/model-micro-services/v2/models/{}/inference-schema/history?limit={}",
            model_id, limit
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    pub fn get_inference_schema_type<T: ApiClient>(
        client: &T,
        model_type: i32,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/inference-schemas/types/{}",
            model_type
        );
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }
}

fn write_download<T: ApiClient>(
    client: &T,
    endpoint: String,
    dest_path: &str,
) -> PyResult<String> {
    let bytes = client.download_bytes(endpoint, Some(client.require_token()?))?;
    if let Some(parent) = std::path::Path::new(dest_path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create parent dir for {}: {}",
                dest_path, e
            ))
        })?;
    }
    std::fs::write(dest_path, bytes).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to write {}: {}",
            dest_path, e
        ))
    })?;
    Ok(dest_path.to_string())
}
