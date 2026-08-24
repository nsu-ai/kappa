// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Model registry + inference-schema HTTP helpers (no card / no publish).

use pyo3::prelude::*;

use crate::model_artifacts::ModelArtifactsApi;
use crate::traits::ApiClient;

/// The server refuses oversized sync uploads and tells the caller to use this SDK instead.
fn is_sync_too_large(message: &str) -> bool {
    message.contains("413") || message.to_ascii_uppercase().contains("USE_KAPPA_APK")
}

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

    /// `PUT /models/inferences/{modelId}/{inferenceId}/pipeline` — draft on the inference
    /// (Kappa ≥ 2.14). Older backends 404; callers should skip.
    pub fn put_inference_pipeline<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let endpoint = format!(
            "/model-micro-services/v2/models/inferences/{}/{}/pipeline",
            model_id, inference_id
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

    // --- inference / version artifacts ---

    /// Upload or replace an inference artifact file.
    ///
    /// `file_category`: 1 Training, 2 Inference (default), 3 Model, 4 Data, 5 Other,
    /// 6 Prediction output (Kappa ≥ 2.14; pass `entity_id` + `field_name`).
    /// `replace`: when true uses PATCH, otherwise POST.
    /// `use_session`: `None` picks the transport automatically — a multipart upload session
    /// for files past the server's sync cap, otherwise the plain multipart POST/PATCH with a
    /// session retry if the server rejects the size (`413 USE_KAPPA_APK`). Sessions always
    /// upsert by file name, so `replace` does not apply to them.
    #[allow(clippy::too_many_arguments)]
    pub fn upload_model_inference_file<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        file_path: &str,
        file_category: Option<i32>,
        replace: bool,
        use_session: Option<bool>,
        entity_id: Option<&str>,
        field_name: Option<&str>,
    ) -> PyResult<PyObject> {
        let category = file_category.unwrap_or(2);
        crate::model_artifacts::validate_file_category(category)?;
        if category == crate::model_artifacts::FILE_CATEGORY_PREDICTION_OUTPUT
            && (entity_id.map(str::trim).unwrap_or("").is_empty()
                || field_name.map(str::trim).unwrap_or("").is_empty())
        {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "file_category=6 (prediction output) requires entity_id and field_name",
            ));
        }
        let byte_len = std::fs::metadata(file_path)
            .map(|m| m.len())
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Cannot read file {}: {}",
                    file_path, e
                ))
            })?;
        let session_upload = || {
            ModelArtifactsApi::upload_model_artifact_session(
                client,
                model_id,
                inference_id,
                file_path,
                Some(category),
                None,
            )
        };
        match use_session {
            Some(true) => return session_upload(),
            None if ModelArtifactsApi::requires_upload_session(byte_len) => {
                return session_upload();
            }
            _ => {}
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
        let endpoint = crate::model_artifacts::inference_file_upload_endpoint(
            model_id,
            inference_id,
            category,
            entity_id,
            field_name,
        );
        let method = if replace { "PATCH" } else { "POST" };
        let result = client.submit_named_file(
            method,
            endpoint,
            "file",
            bytes,
            fname,
            Some(client.require_token()?),
        );
        match result {
            Err(e) if use_session.is_none() && is_sync_too_large(&e.to_string()) => {
                session_upload()
            }
            other => other,
        }
    }

    pub fn download_model_inference_artifacts<T: ApiClient>(
        client: &T,
        model_id: &str,
        inference_id: i32,
        dest_path: &str,
    ) -> PyResult<String> {
        ModelArtifactsApi::download_inference_artifacts_zip(
            client,
            model_id,
            inference_id,
            dest_path,
        )
    }

    pub fn download_model_version_artifacts<T: ApiClient>(
        client: &T,
        model_id: &str,
        version_id: i32,
        dest_path: &str,
    ) -> PyResult<String> {
        ModelArtifactsApi::download_version_artifacts_zip(client, model_id, version_id, dest_path)
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

