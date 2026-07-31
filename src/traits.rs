// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use std::sync::Arc;

pub trait ApiClient {
    fn get_base_url(&self) -> String;
    fn get_user_id(&self) -> Option<i32>;
    fn get_user_type_id(&self) -> Option<i32>;
    fn get_token(&self) -> Option<String>;
    fn get_http_client(&self) -> reqwest::Client;
    fn get_runtime(&self) -> Arc<tokio::runtime::Runtime>;
    fn make_request(
        &self,
        method: String,
        endpoint: String,
        data: Option<String>,
        token: Option<String>,
    ) -> PyResult<PyObject>;

    fn fetch_url_bytes(&self, _url: &str) -> PyResult<Vec<u8>> {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "This ApiClient does not support URL file sources",
        ))
    }

    fn submit_dataset_entity_request(
        &self,
        _method: &str,
        _endpoint: String,
        _json_field_name: &str,
        _json_value: String,
        _file_parts: Vec<(Vec<u8>, String)>,
        _token: Option<String>,
    ) -> PyResult<PyObject> {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "This ApiClient does not support dataset entity upload",
        ))
    }

    /// Multipart upload of `files` parts only (optional query string already in endpoint).
    fn submit_multipart_files(
        &self,
        _method: &str,
        _endpoint: String,
        _file_parts: Vec<(Vec<u8>, String)>,
        _token: Option<String>,
    ) -> PyResult<PyObject> {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "This ApiClient does not support multipart file upload",
        ))
    }

    /// Bulk entity upload: multipart `sources` (JSON text) + streamed `file` from disk.
    ///
    /// `on_upload_progress`: optional callable `(bytes_sent: int, total_bytes: int, percent: int)`.
    fn submit_bulk_upload(
        &self,
        _endpoint: String,
        _sources_json: String,
        _file_path: String,
        _headers: Vec<(String, String)>,
        _token: Option<String>,
        _on_upload_progress: Option<PyObject>,
    ) -> PyResult<PyObject> {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "This ApiClient does not support bulk upload",
        ))
    }

    /// Authenticated binary download (artifact ZIP, entity file, etc.).
    fn download_bytes(
        &self,
        _endpoint: String,
        _token: Option<String>,
    ) -> PyResult<Vec<u8>> {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "This ApiClient does not support binary download",
        ))
    }

    /// Multipart upload with a single named file field (e.g. model inference `file`).
    fn submit_named_file(
        &self,
        _method: &str,
        _endpoint: String,
        _field_name: &str,
        _file_bytes: Vec<u8>,
        _file_name: String,
        _token: Option<String>,
    ) -> PyResult<PyObject> {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "This ApiClient does not support named file upload",
        ))
    }

    fn is_authenticated(&self) -> bool {
        self.get_token().is_some()
    }

    // v2 APIs derive identity from JWT; these helpers read cached login state only.
    fn user_id(&self) -> i32 {
        self.get_user_id().unwrap_or(0)
    }

    fn user_type_id(&self) -> i32 {
        self.get_user_type_id().unwrap_or(0)
    }

    // Token is always required for authenticated API calls.
    fn require_token(&self) -> PyResult<String> {
        self.get_token().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyPermissionError, _>(
                "Not authenticated. Call connect() first.",
            )
        })
    }
}
