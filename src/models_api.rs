// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Model registry + inference-schema HTTP helpers (no card / no publish).

use pyo3::prelude::*;
use urlencoding;

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
}
