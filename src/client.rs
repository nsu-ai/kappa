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
use crate::models::users_model::User;
use crate::models::login_models::LoginRequest;
use crate::models::datasets_model::{
    DatasetDownloadDetails, DatasetLabel, DatasetVersionDetails, NewDataset, NewDatasetEntity,
    NewDatasetVersion, UpdateDatasetEntity, UpdateDatasetRequest,
};

/// Serialize a `serde_json::Value` to a Python object via `json.loads`.
/// Extracted to avoid repeating the same `.unwrap()` pattern everywhere.
fn json_value_to_pyobject(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    let json_str = serde_json::to_string(value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialization error: {}", e))
    })?;
    let parsed = py.import("json")?.call_method1("loads", (json_str,))?;
    Ok(parsed.into())
}

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
    passwd: String,
    login_response: Option<User>,
}

impl Clone for KappaApkClient {
    fn clone(&self) -> Self {
        KappaApkClient {
            client: self.client.clone(),
            runtime: Arc::clone(&self.runtime),
            base_url: self.base_url.clone(),
            login_id: self.login_id.clone(),
            passwd: self.passwd.clone(),
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
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
                    && let Some(detail) = json.get("detail").and_then(|v| v.as_str())
                {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        detail.to_string(),
                    ));
                }
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    if text.is_empty() {
                        format!("HTTP error: {}", status)
                    } else {
                        text
                    },
                ))
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
            let empty = file_parts.is_empty();
            let mut request = match (method_owned.as_str(), empty) {
                ("POST", true) => http
                    .post(&url)
                    .form(&[(jfield.as_str(), json_value.as_str())]),
                ("PUT", true) => http
                    .put(&url)
                    .form(&[(jfield.as_str(), json_value.as_str())]),
                ("POST", false) => {
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
                    http.post(&url).multipart(form)
                }
                ("PUT", false) => {
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
                    http.put(&url).multipart(form)
                }
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
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
                    && let Some(detail) = json.get("detail").and_then(|v| v.as_str())
                {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        detail.to_string(),
                    ));
                }
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    if text.is_empty() {
                        format!("HTTP error: {}", status)
                    } else {
                        text
                    },
                ))
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Payload encoding helpers
// ---------------------------------------------------------------------------

fn py_json_dumps(v: &Bound<'_, PyAny>) -> PyResult<String> {
    let py = v.py();
    py.import("json")?.getattr("dumps")?.call1((v,))?.extract()
}

fn encode_new_dataset_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(d) = v.extract::<PyRef<NewDataset>>() {
        return d.to_api_json();
    }
    py_json_dumps(v)
}

fn encode_update_dataset_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(d) = v.extract::<PyRef<UpdateDatasetRequest>>() {
        return d.to_api_json();
    }
    py_json_dumps(v)
}

fn encode_new_entity_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(e) = v.extract::<PyRef<NewDatasetEntity>>() {
        return e.to_api_json(v.py());
    }
    py_json_dumps(v)
}

fn encode_update_entity_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(e) = v.extract::<PyRef<UpdateDatasetEntity>>() {
        return e.to_api_json(v.py());
    }
    py_json_dumps(v)
}

fn encode_new_version_payload(v: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(nv) = v.extract::<PyRef<NewDatasetVersion>>() {
        return nv.to_api_json();
    }
    py_json_dumps(v)
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
        let client = Client::new();
        let runtime = Arc::new(tokio::runtime::Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to create async runtime: {}", e),
            )
        })?);
        Ok(KappaApkClient {
            client,
            runtime,
            base_url,
            login_id,
            passwd,
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
        let login_request = LoginRequest {
            login_id: self.login_id.clone(),
            passwd: self.passwd.clone(),
        };
        let login_url = format!("{}/user-micro-services/v2/session/new", self.base_url);
        let client = self.client.clone();

        let login_data: User = self.runtime.block_on(async move {
            let response = client
                .post(&login_url)
                .header("accept", "application/json")
                .header("Content-Type", "application/json")
                .json(&login_request)
                .send()
                .await
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyConnectionError, _>(
                        format!("Request failed: {}", e),
                    )
                })?;

            if response.status().is_success() {
                response.json::<User>().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Failed to parse login response: {}", e),
                    )
                })
            } else {
                let status = response.status();
                let msg = response.text().await.unwrap_or_default();
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Login failed (HTTP {}): {}", status, msg),
                ))
            }
        })?;

        self.login_response = Some(login_data.clone());

        // Return snake_case dict for Python consumers (User uses camelCase serde internally).
        let json_value = serde_json::json!({
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
            "org_details": {
                "org_id": login_data.org_details.org_id,
                "org_name": login_data.org_details.org_name
            }
        });
        Python::with_gil(|py| json_value_to_pyobject(py, &json_value))
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

        let logout_url = format!("{}/user-micro-services/v2/session/", self.base_url);
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
    #[pyo3(signature = (dataset_id=None, dataset_name=None, version_id=None, version_no=None, dataset_path=None, transform=None, target_transform=None, transform_input_mode=None))]
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
        )
    }

    /// Get a dataset loader in the requested format (`"kappa"`, `"pytorch"`, `"transformers"`, `"tensorflow"`).
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
        transform_input_mode=None
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
        )
    }

    /// Create a dataset (`POST /data-micro-services/v2/datasets/new`).
    pub fn add_dataset(&self, dataset: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let body = encode_new_dataset_payload(dataset)?;
        Datasets::add_dataset(self, body)
    }

    /// Update dataset metadata (`PUT /data-micro-services/v2/datasets/{dataset_id}`).
    #[pyo3(signature = (dataset_id, update))]
    pub fn update_dataset(
        &self,
        dataset_id: i32,
        update: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let body = encode_update_dataset_payload(update)?;
        Datasets::update_dataset(self, dataset_id, body)
    }

    /// Add a dataset entity with optional file attachments.
    #[pyo3(signature = (dataset_id, entity, file_paths=None))]
    pub fn add_dataset_entity(
        &self,
        dataset_id: i32,
        entity: &Bound<'_, PyAny>,
        file_paths: Option<Vec<String>>,
    ) -> PyResult<PyObject> {
        let entity_json = encode_new_entity_payload(entity)?;
        Datasets::add_dataset_entity(
            self,
            dataset_id,
            entity_json,
            file_paths.unwrap_or_default(),
        )
    }

    /// Update a dataset entity with optional file attachments.
    #[pyo3(signature = (dataset_id, entity_id, update, file_paths=None))]
    pub fn update_dataset_entity(
        &self,
        dataset_id: i32,
        entity_id: String,
        update: &Bound<'_, PyAny>,
        file_paths: Option<Vec<String>>,
    ) -> PyResult<PyObject> {
        let update_json = encode_update_entity_payload(update)?;
        Datasets::update_dataset_entity(
            self,
            dataset_id,
            &entity_id,
            update_json,
            file_paths.unwrap_or_default(),
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
    /// All parameters are optional; omit any you don't need.
    /// `dataset_tags` is a comma-separated tag string (e.g. `"vision,classification"`).
    #[pyo3(signature = (search=None, dataset_id=None, dataset_name=None, dataset_type=None, dataset_tags=None, dataset_status=None, publish_type=None, page=None, size=None, order_by=None, order_keyword=None))]
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
    ) -> PyResult<PyObject> {
        Datasets::filter_datasets(
            self, search, dataset_id, dataset_name, dataset_type,
            dataset_tags, dataset_status, publish_type, page, size,
            order_by, order_keyword,
        )
    }

    /// Return the field/schema definition for a dataset.
    pub fn get_dataset_fields(&self, dataset_id: i32) -> PyResult<PyObject> {
        Datasets::get_dataset_fields(self, dataset_id)
    }

    /// Soft-delete a dataset (sets `datasetStatus = 0`).
    ///
    /// Deleted datasets can be recovered via the server's `/datasets/recover` endpoint.
    #[pyo3(signature = (dataset_id, remark=None))]
    pub fn delete_dataset(
        &self,
        dataset_id: i32,
        remark: Option<String>,
    ) -> PyResult<PyObject> {
        Datasets::delete_dataset(self, dataset_id, remark)
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
    #[pyo3(signature = (dataset_id, version_id=None))]
    pub fn list_dataset_entities(
        &self,
        dataset_id: i32,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
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
    #[pyo3(signature = (dataset_id, entity_name=None, entity_status=None, version_id=None, page=None, size=None, order_by=None, order=None))]
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
    ) -> PyResult<PyObject> {
        Datasets::filter_dataset_entities(
            self, dataset_id, entity_name, entity_status,
            version_id, page, size, order_by, order,
        )
    }

    /// Bulk soft-delete entities by their string IDs.
    #[pyo3(signature = (dataset_entity_ids, remark, version_id=None))]
    pub fn delete_dataset_entities(
        &self,
        dataset_entity_ids: Vec<String>,
        remark: String,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        Datasets::delete_dataset_entities(self, dataset_entity_ids, remark, version_id)
    }

    // --- dataset version management ---

    /// Create a new dataset version (accepts `NewDatasetVersion` or a plain dict).
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
    /// `publish_type`: 0 = Private, 1 = Internal, 2 = Public.
    pub fn publish_dataset_version(
        &self,
        dataset_id: i32,
        version_no: String,
        publish_type: i32,
    ) -> PyResult<PyObject> {
        Datasets::publish_dataset_version(self, dataset_id, &version_no, publish_type)
    }

    /// Load a benchmark and return a reusable `Benchmarks` object.
    pub fn load_benchmark(&self, benchmark_id: String) -> PyResult<Py<Benchmarks>> {
        Python::with_gil(|py| {
            let myself = Py::new(py, self.clone())?;
            Py::new(py, Benchmarks::new(benchmark_id, myself))
        })
    }
}
