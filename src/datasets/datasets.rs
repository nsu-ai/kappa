// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PySequence};
use urlencoding;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::traits::ApiClient;
use crate::datasets::kappa_dataloader::KappaDataLoader;
use crate::models::datasets_model::{
    Dataset,
    DatasetVersionDetails,
    DatasetDownloadDetails,
    DatasetItem,
    ItemFile,
    AnnotationValue,
    DatasetLabel,
};

/// Default implementation of Datasets trait
pub struct Datasets;

/// Unwrap common API envelopes so label records deserialize as a JSON array.
fn extract_labels_json_array(value: &serde_json::Value) -> serde_json::Value {
    if value.is_array() {
        return value.clone();
    }
    if let Some(obj) = value.as_object() {
        for key in ["data", "labels", "items", "result"] {
            if let Some(inner) = obj.get(key) {
                if inner.is_array() {
                    return inner.clone();
                }
                if let Some(nested) = inner.as_object() {
                    for nested_key in ["data", "labels", "items"] {
                        if let Some(arr) = nested.get(nested_key)
                            && arr.is_array()
                        {
                            return arr.clone();
                        }
                    }
                }
            }
        }
    }
    value.clone()
}

/// Parse label records from heterogeneous API payloads (full objects, partial dicts, or strings).
fn parse_dataset_labels_from_value(
    value: &serde_json::Value,
    dataset_id: i32,
) -> PyResult<Vec<DatasetLabel>> {
    let array_value = extract_labels_json_array(value);
    let items = array_value.as_array().ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Expected array response for dataset labels",
        )
    })?;

    let mut out = Vec::new();
    for item in items {
        if let Some(label_str) = item.as_str() {
            let label = label_str.trim();
            if !label.is_empty() {
                out.push(DatasetLabel {
                    label_id: 0,
                    dataset_id,
                    label: label.to_string(),
                    created_on: None,
                    modified_on: None,
                });
            }
            continue;
        }

        if let Some(obj) = item.as_object() {
            if let Ok(mut parsed) = serde_json::from_value::<DatasetLabel>(item.clone()) {
                if parsed.dataset_id == 0 {
                    parsed.dataset_id = dataset_id;
                }
                if !parsed.label.trim().is_empty() {
                    out.push(parsed);
                    continue;
                }
            }

            let label_text = ["label", "name", "labelName", "label_name"]
                .iter()
                .find_map(|key| obj.get(*key).and_then(|v| v.as_str()))
                .map(str::trim)
                .filter(|s| !s.is_empty());

            if let Some(label) = label_text {
                let label_id = obj
                    .get("labelId")
                    .or_else(|| obj.get("label_id"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                let ds_id = obj
                    .get("datasetId")
                    .or_else(|| obj.get("dataset_id"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .unwrap_or(dataset_id);
                out.push(DatasetLabel {
                    label_id,
                    dataset_id: ds_id,
                    label: label.to_string(),
                    created_on: obj
                        .get("createdOn")
                        .or_else(|| obj.get("created_on"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    modified_on: obj
                        .get("modifiedOn")
                        .or_else(|| obj.get("modified_on"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                });
            }
        }
    }

    Ok(out)
}

impl Datasets {
    /// List datasets for the current user (paginated).
    /// 
    /// By default returns up to 200 datasets on page 1. You can override `page`, `size`,
    /// `order_by`, and `order_keyword` if desired.
    /// 
    /// # Python Example
    /// 
    /// ```python
    /// # Default: size=200, page=1
    /// result = client.list_datasets(None, None, None, None)
    /// 
    /// # Custom page/size
    /// result = client.list_datasets(1, 50, None, None)
    /// 
    /// # result is a dict mirroring server JSON (paginated list)
    /// ```
    pub fn list_datasets_json<T: ApiClient>(
        client: &T,
        page: Option<i32>,
        size: Option<i32>,
        order_by: Option<String>,
        order_keyword: Option<String>,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let page = page.unwrap_or(1);
        let size = size.unwrap_or(200);
        let order_by = order_by.unwrap_or_else(|| "datasetId".to_string());
        let order_keyword = order_keyword.unwrap_or_else(|| "DESC".to_string());

        let endpoint = format!(
            "/data-micro-services/v2/datasets/filter?page={}&size={}&orderBy={}&orderKeyword={}",
            page,
            size,
            urlencoding::encode(&order_by),
            urlencoding::encode(&order_keyword)
        );

        client.make_request("GET".to_string(), endpoint, None, Some(token))
    }

    /// List datasets as typed `Dataset` objects.
    /// 
    /// Same as `list_datasets` but returns a Python `list[Dataset]` for convenience.
    /// Defaults: page=1, size=200.
    ///
    /// # Python Example
    /// ```python
    /// datasets = client.list_datasets_typed(None, None, None, None)
    /// for ds in datasets:
    ///     print(ds.dataset_id, ds.dataset_name)
    /// ```
    pub fn list_datasets<T: ApiClient>(
        client: &T,
        page: Option<i32>,
        size: Option<i32>,
        order_by: Option<String>,
        order_keyword: Option<String>,
    ) -> PyResult<PyObject> {
        // Reuse JSON variant and map to typed objects
        let json_obj = Self::list_datasets_json(client, page, size, order_by, order_keyword)?;
        let typed_list = Python::with_gil(|py| -> PyResult<PyObject> {
            let json_module = py.import("json")?;
            let dumps = json_module.getattr("dumps")?;
            let json_str_obj = dumps.call1((json_obj.clone_ref(py),))?;
            let json_str: String = json_str_obj.extract()?;

            // Parse with serde to extract items array; assume standard pagination shape
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse datasets JSON: {}", e)))?;

            let items = value.get("items").or_else(|| value.get("data")).cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));

            let datasets: Vec<Dataset> = serde_json::from_value(items)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to decode datasets: {}", e)))?;

            // Convert Vec<Dataset> into Python list of pyclass objects
            let py_vec: Vec<Py<Dataset>> = datasets
                .into_iter()
                .map(|ds| Py::new(py, ds))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to create Python Dataset objects: {}", e)))?;

            let py_list = PyList::new(py, &py_vec)?;
            let owned = py_list.unbind();
            Ok(owned.into())
        })?;

        Ok(typed_list)
    }

    /// Get dataset details
    ///
    /// Returns a Python dict mirroring server JSON
    ///
    /// # Python Example
    /// ```python
    /// dataset = client.get_dataset_details(1)
    /// print(dataset.dataset_id, dataset.dataset_name)
    /// ```
    pub fn get_dataset_details<T: ApiClient>(
        client: &T,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
    ) -> PyResult<Dataset> {
        // Decide which identifier to use:
        // - Only id  provided  -> query by id
        // - Only name provided -> query by name
        // - Both provided      -> prefer id, ignore name
        // - Neither provided   -> error
        let (id_opt, name_opt) = match (dataset_id, dataset_name) {
            (Some(id), None) => (Some(id), None),
            (None, Some(name)) => (None, Some(name)),
            (Some(id), Some(_)) => (Some(id), None),
            (None, None) => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "dataset_id or dataset_name is required.",
                ))
            }
        };

        let endpoint = if let Some(id) = id_opt {
            format!(
                "/data-micro-services/v2/datasets/filter?page=1&size=1&datasetId={}&orderBy=modifiedOn&orderKeyword=DESC",
                id
            )
        } else if let Some(name) = name_opt {
            format!(
                "/data-micro-services/v2/datasets/filter?page=1&size=1&datasetName={}&orderBy=modifiedOn&orderKeyword=DESC",
                urlencoding::encode(&name)
            )
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "dataset_id or dataset_name is required.",
            ));
        };

        let json_obj =
            client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))?;
        // 200 Response
        // {
        //     "items": [
        //       {
        //         "datasetId": 11,
        //         "datasetName": "Animal-bird test dataset",
        //         "datasetType": 1,
        //         "datasetTypeInterp": "Computer Vision",
        //         "datasetShortInfo": "Animal-bird dataset to test",
        //         "datasetStatus": 1,
        //         "datasetStatusInterp": "Active",
        //         "datasetTags": "Object Detection",
        //         "publishType": 2,
        //         "createdOn": "2025-08-14T08:33:40.823929Z",
        //         "modifiedOn": "2025-08-14T08:59:59.458187Z",
        //         "versionNo": null
        //       }
        //     ],
        //     "total": 1,
        //     "page": 1,
        //     "size": 1,
        //     "pages": 1
        //   }
        let typed_item = Python::with_gil(|py| -> PyResult<Dataset> {
            let json_module = py.import("json")?;
            let dumps = json_module.getattr("dumps")?;
            let json_str_obj = dumps.call1((json_obj.clone_ref(py),))?;
            let json_str: String = json_str_obj.extract()?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse datasets JSON: {}", e)))?;
            // Extract items array and decode
            let items_val = value.get("items").or_else(|| value.get("data")).cloned()
                .unwrap_or(serde_json::Value::Array(vec![]));
            let datasets: Vec<Dataset> = serde_json::from_value(items_val)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to decode datasets: {}", e)))?;
            if datasets.is_empty() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("No record found."));
            }
            Ok(datasets[0].clone())
        })?;
        Ok(typed_item)
    }

    /// Get dataset version details
    /// 
    /// Returns a Python dict mirroring server JSON.
    /// Search by dataset_id or dataset_name; if both are given, dataset_id is used.
    ///
    /// # Python Example
    /// ```python
    /// version = client.get_dataset_version_details(dataset_id=1, version_id=1)
    /// version = client.get_dataset_version_details(dataset_name="My Dataset", version_no="1.0.0")
    /// print(version.id, version.version_no)
    /// ```
    pub fn get_dataset_version_details<T: ApiClient>(
        client: &T,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
    ) -> PyResult<DatasetVersionDetails> {
        // Resolve dataset: require one of id or name; prefer id when both given
        let resolved_dataset_id: i32 = match (dataset_id, dataset_name) {
            (Some(id), None) | (Some(id), Some(_)) => id,
            (None, Some(name)) => {
                let details = Self::get_dataset_details(client, None, Some(name))?;
                details.dataset_id
            }
            (None, None) => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "dataset_id or dataset_name is required.",
                ));
            }
        };

        let version_id = version_id.unwrap_or(0);
        let version_no = version_no.unwrap_or_default();
        if version_id == 0 && version_no.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "version_id or version_no is required.",
            ));
        }

        if version_no.is_empty() {
            let endpoint = "/data-micro-services/v2/datasets/versions/details/list".to_string();
            let data = serde_json::json!([version_id]);
            let json_obj = client.make_request("POST".to_string(), endpoint, Some(data.to_string()), Some(client.require_token()?))?;
            Python::with_gil(|py| -> PyResult<DatasetVersionDetails> {
                let json_module = py.import("json")?;
                let dumps = json_module.getattr("dumps")?;
                let json_str_obj = dumps.call1((json_obj.clone_ref(py),))?;
                let json_str: String = json_str_obj.extract()?;
                let value: serde_json::Value = serde_json::from_str(&json_str)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse datasets version details JSON: {}", e)))?;

                let versions_array = value.as_array()
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Expected array response for version details"))?;

                if versions_array.is_empty() {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("No record found."));
                }

                let first_version = &versions_array[0];
                let dataset_version_details: DatasetVersionDetails = serde_json::from_value(first_version.clone())
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to decode datasets version details: {}", e)))?;
                Ok(dataset_version_details)
            })
        } else {
            let endpoint = format!(
                "/data-micro-services/v2/datasets/versions/{}/{}",
                resolved_dataset_id,
                version_no
            );
            let json_obj = client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))?;
            Python::with_gil(|py| -> PyResult<DatasetVersionDetails> {
                let json_module = py.import("json")?;
                let dumps = json_module.getattr("dumps")?;
                let json_str_obj = dumps.call1((json_obj.clone_ref(py),))?;
                let json_str: String = json_str_obj.extract()?;
                let value: serde_json::Value = serde_json::from_str(&json_str)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse datasets version details JSON: {}", e)))?;
                let dataset_version_details: DatasetVersionDetails = serde_json::from_value(value)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to decode datasets version details: {}", e)))?;
                Ok(dataset_version_details)
            })
        }
    }

    /// Download dataset version archive file
    /// 
    /// Downloads the zip file from the server and extracts it to the cache directory or the provided path.
    /// If the directory already exists, it skips the download.
    /// 
    /// # Python Example
    /// ```python
    /// cache_path = client.download_dataset_version_archive(1, 8, "1.0.0", "path/to/dataset")
    /// print(f"Dataset extracted to: {cache_path}")
    /// ```
    pub fn download_dataset_version_archive<T: ApiClient>(
        client: &T,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
    ) -> PyResult<DatasetDownloadDetails> {
        let token = client.require_token()?;
        let dataset_path = dataset_path.unwrap_or_default();

        // Resolve dataset either by ID or by name (preferring ID when both given).
        let dataset_details = Self::get_dataset_details(client, dataset_id, dataset_name)?;
        let resolved_dataset_id = dataset_details.dataset_id;
        let resolved_dataset_name = dataset_details.dataset_name.clone();

        // Resolve version information.
        let resolved_version_id = version_id.unwrap_or(0);
        let mut resolved_version_no = version_no.unwrap_or_default();
        if resolved_version_id == 0 && resolved_version_no.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "version_id or version_no is required.",
            ));
        }

        let dataset_version_details = Self::get_dataset_version_details(
            client,
            Some(resolved_dataset_id),
            None,
            Some(resolved_version_id),
            Some(resolved_version_no.clone()),
        )?;
        if resolved_version_no.is_empty() {
            resolved_version_no = dataset_version_details.version_no.clone();
        }

        let data_dir = if dataset_path.is_empty() {
            let home_dir = dirs::home_dir().ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Could not determine home directory")
            })?;
            home_dir
                .join("cache")
                .join("kappa-framework")
                .join("datasets")
                .join(format!("{}_{}", resolved_dataset_name, resolved_version_no))
        } else {
            Path::new(&dataset_path)
                .join(format!("{}_{}", resolved_dataset_name, resolved_version_no))
        };

        // Skip if already cached.
        if data_dir.exists() {
            return Ok(DatasetDownloadDetails {
                dataset_id: resolved_dataset_id,
                data_path: data_dir.to_string_lossy().to_string(),
                version_no: resolved_version_no,
                download_status: true,
            });
        }

        fs::create_dir_all(&data_dir).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to create cache directory: {}", e),
            )
        })?;

        let endpoint = format!(
            "/data-micro-services/v2/datasets/versions/archive/{}/{}",
            resolved_dataset_id,
            resolved_version_no
        );
        let url = format!("{}{}", client.get_base_url(), endpoint);
        let http = client.get_http_client();
        let runtime = client.get_runtime();

        let result = runtime.block_on(async {
            let response = http
                .get(&url)
                .header("accept", "*/*")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyConnectionError, _>(
                        format!("Failed to download archive: {}", e),
                    )
                })?;

            if !response.status().is_success() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Download failed with status: {}", response.status()),
                ));
            }

            // Stream response body directly to a temp file — avoids loading the entire
            // archive into memory before writing (important for large datasets).
            let temp_zip_path = data_dir.join("temp_archive.zip");
            {
                let mut file = tokio::fs::File::create(&temp_zip_path).await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Failed to create temporary file: {}", e),
                    )
                })?;
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Download error: {}", e),
                        )
                    })?;
                    file.write_all(&chunk).await.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Write error: {}", e),
                        )
                    })?;
                }
                file.flush().await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Flush error: {}", e),
                    )
                })?;
            } // file is dropped and closed here

            // Extract synchronously — zip crate does not have async support.
            let zip_file = fs::File::open(&temp_zip_path).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to open zip file: {}", e),
                )
            })?;
            let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to read zip archive: {}", e),
                )
            })?;

            for i in 0..archive.len() {
                let mut entry = archive.by_index(i).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Failed to access entry in zip: {}", e),
                    )
                })?;

                let outpath = data_dir.join(entry.name());

                // Zip-slip prevention: reject any path that escapes the target directory.
                if !outpath.starts_with(&data_dir) {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Zip entry '{}' would escape the extraction directory",
                        entry.name()
                    )));
                }

                if entry.name().ends_with('/') {
                    fs::create_dir_all(&outpath).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to create directory: {}", e),
                        )
                    })?;
                } else {
                    if let Some(parent) = outpath.parent()
                        && !parent.exists()
                    {
                        fs::create_dir_all(parent).map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                                format!("Failed to create parent directory: {}", e),
                            )
                        })?;
                    }
                    let mut outfile = fs::File::create(&outpath).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to create file: {}", e),
                        )
                    })?;
                    std::io::copy(&mut entry, &mut outfile).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to write file: {}", e),
                        )
                    })?;
                }
            }

            fs::remove_file(&temp_zip_path).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to delete temporary zip file: {}", e),
                )
            })?;

            Ok(DatasetDownloadDetails {
                dataset_id: resolved_dataset_id,
                data_path: data_dir.to_string_lossy().to_string(),
                version_no: resolved_version_no,
                download_status: true,
            })
        });
        result
    }

    /// Load dataset from cache directory
    /// 
    /// Loads the dataset from the cache directory.
    /// 
    /// # Python Example
    /// ```python
    /// dataset = client.load_dataset(1, "1.0.0")
    /// print(dataset.dataset_id, dataset.dataset_name)
    /// ```
    pub fn load<T: ApiClient>(
        client: &T,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
    ) -> PyResult<Vec<DatasetItem>> {
        // Ensure the dataset version archive is downloaded (or already cached)
        let dataset_download_details = Self::download_dataset_version_archive(
            client,
            dataset_id,
            dataset_name,
            version_id,
            version_no,
            dataset_path,
        )?;
        let data_dir = Path::new(&dataset_download_details.data_path);
        let entries = Self::read_dataset_directory(&data_dir)?;
        let items = Self::parse_dataset_items(&entries, &data_dir)?;

        if items.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No dataset items found in directory."
            ));
        }

        Ok(items)
    }

    fn read_dataset_directory(data_dir: &Path) -> PyResult<Vec<fs::DirEntry>> {
        let entries: Vec<fs::DirEntry> = fs::read_dir(data_dir)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to read dataset directory: {}",
                e
            )))?
            .filter_map(|res| res.ok())
            .collect();
        Ok(entries)
    }

    fn parse_dataset_items(
        entries: &[fs::DirEntry],
        data_dir: &Path,
    ) -> PyResult<Vec<DatasetItem>> {
        let mut items = Vec::new();
        for entry in entries {
            if let Some(dataset_item) = Self::parse_dataset_item(entry, data_dir)? {
                items.push(dataset_item);
            }
        }
        Ok(items)
    }

    fn parse_dataset_item(
        entry: &fs::DirEntry,
        data_dir: &Path,
    ) -> PyResult<Option<DatasetItem>> {
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };
        if !file_type.is_file() {
            return Ok(None);
        }

        let file_name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => return Ok(None),
        };
        if !file_name.ends_with("_info.json") {
            return Ok(None);
        }

        let entity_id = file_name.trim_end_matches("_info.json").to_string();
        let info_path = entry.path();
        let (annotations, files, entity_info) =
            Self::parse_info_file(&info_path, &entity_id, data_dir)?;

        Ok(Some(DatasetItem {
            entity_id,
            files,
            annotations,
            entity_info,
        }))
    }

    fn parse_info_file(
        info_path: &Path,
        entity_id: &str,
        data_dir: &Path,
    ) -> PyResult<(
        Option<Vec<std::collections::HashMap<String, AnnotationValue>>>,
        Option<Vec<ItemFile>>,
        Option<HashMap<String, serde_json::Value>>,
    )> {
        let content = fs::read_to_string(info_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to read info file: {}",
                e
            ))
        })?;
        let value: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse info file JSON: {}",
                e
            ))
        })?;

        let annotations = Self::extract_annotations(&value)?;
        let files = Self::extract_files(&value, entity_id, data_dir)?;
        let entity_info = Self::extract_entity_info(&value)?;
        Ok((annotations, files, entity_info))
    }

    fn extract_annotations(
        value: &serde_json::Value,
    ) -> PyResult<Option<Vec<std::collections::HashMap<String, AnnotationValue>>>> {
        if let Some(annotations_value) = value.get("annotations") {
            return Self::parse_annotations_array(annotations_value);
        }

        if let Some(json_information_value) = value
            .get("jsonInformation")
            .or_else(|| value.get("json_information"))
            .or_else(|| value.get("jsoninfo"))
            .or_else(|| value.get("json"))
        {
            return Self::extract_annotations_from_json_information(json_information_value);
        }

        Ok(None)
    }

    fn parse_annotations_array(
        annotations_value: &serde_json::Value,
    ) -> PyResult<Option<Vec<std::collections::HashMap<String, AnnotationValue>>>> {
        if annotations_value.is_null() {
            return Ok(None);
        }

        let annotations: Vec<std::collections::HashMap<String, AnnotationValue>> =
            serde_json::from_value(annotations_value.clone()).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse annotations: {}",
                    e
                ))
            })?;
        Ok(Some(annotations))
    }

    fn extract_annotations_from_json_information(
        json_information_value: &serde_json::Value,
    ) -> PyResult<Option<Vec<std::collections::HashMap<String, AnnotationValue>>>> {
        if json_information_value.is_null() {
            return Ok(None);
        }

        let normalized_value = if let Some(json_str) = json_information_value.as_str() {
            let trimmed = json_str.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            serde_json::from_str::<serde_json::Value>(trimmed).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse json information: {}",
                    e
                ))
            })?
        } else {
            json_information_value.clone()
        };

        if normalized_value.is_null() {
            return Ok(None);
        }

        if let Some(array_value) = normalized_value.as_array() {
            return Self::parse_annotations_array(&serde_json::Value::Array(array_value.clone()));
        }

        if let Some(object_value) = normalized_value.as_object() {
            if let Some(annotations_value) = object_value.get("annotations") {
                return Self::parse_annotations_array(annotations_value);
            }

            let map: std::collections::HashMap<String, AnnotationValue> =
                serde_json::from_value(serde_json::Value::Object(object_value.clone())).map_err(
                    |e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Failed to decode json information annotations: {}",
                            e
                        ))
                    },
                )?;
            return Ok(Some(vec![map]));
        }

        Ok(None)
    }

    fn extract_entity_info(
        value: &serde_json::Value,
    ) -> PyResult<Option<HashMap<String, serde_json::Value>>> {
        if let Some(object_value) = value.as_object() {
            let mut info = object_value.clone();
            info.remove("annotations");
            info.remove("files");

            if info.is_empty() {
                return Ok(None);
            }

            Ok(Some(info.into_iter().collect()))
        } else {
            Ok(None)
        }
    }

    fn extract_files(
        value: &serde_json::Value,
        entity_id: &str,
        data_dir: &Path,
    ) -> PyResult<Option<Vec<ItemFile>>> {
        if let Some(files_value) = value.get("files") {
            let file_names: Vec<String> = serde_json::from_value(files_value.clone()).map_err(
                |e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to parse files: {}",
                        e
                    ))
                },
            )?;

            let files: Vec<ItemFile> = file_names
                .into_iter()
                .map(|name| ItemFile {
                    file_id: entity_id.to_string(),
                    file_name: name.clone(),
                    file: data_dir.join(name),
                })
                .collect();
            Ok(Some(files))
        } else {
            Ok(None)
        }
    }
}

fn load_content_from_raw_sample<'py>(py: Python<'py>, raw: &Bound<'py, PyDict>) -> PyResult<PyObject> {
    let files_value = raw
        .get_item("files")?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing key 'files' in sample"))?;

    if files_value.is_none() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Sample['files'] is None; content loading requires at least one file",
        ));
    }

    let files_sequence: &Bound<'_, PySequence> = files_value.downcast::<PySequence>().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Sample['files'] must be a sequence of ItemFile objects",
        )
    })?;

    let first = files_sequence.get_item(0).map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Sample['files'] is empty; content loading requires at least one file",
        )
    })?;

    let file_path: String = first
        .getattr("file")?
        .extract::<String>()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("ItemFile.file must be a string"))?;

    let file_name: String = first
        .getattr("file_name")?
        .extract::<String>()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("ItemFile.file_name must be a string"))?;

    let ext = file_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    let image_exts = ["jpg", "jpeg", "png", "bmp", "gif", "webp"];
    if image_exts.contains(&ext.as_str()) {
        let pil = py.import("PIL.Image").map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Pillow is required for image content loading (pip install pillow)",
            )
        })?;
        let image = pil.call_method1("open", (file_path,))?;
        let rgb_image = image.call_method1("convert", ("RGB",))?;
        return Ok(rgb_image.unbind().into());
    }

    let audio_exts = ["wav", "mp3", "flac", "ogg", "m4a", "aac"];
    if audio_exts.contains(&ext.as_str()) {
        let torchaudio = py.import("torchaudio").map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "torchaudio is required for audio content loading (pip install torchaudio)",
            )
        })?;

        // torchaudio.load returns (waveform, sample_rate)
        let loaded = torchaudio.call_method1("load", (file_path,))?;
        let waveform = loaded.get_item(0)?;
        return Ok(waveform.unbind().into());
    }

    let text_exts = ["txt", "text", "md", "csv"];
    if text_exts.contains(&ext.as_str()) {
        let builtins = py.import("builtins")?;
        let open_fn = builtins.getattr("open")?;
        let f = open_fn.call1((file_path, "r"))?;
        let content: String = f.call_method0("read")?.extract()?;
        let s = builtins.call_method1("str", (content,))?;
        return Ok(s.unbind().into());
    }

    // Fallback: return the file path string.
    let builtins = py.import("builtins")?;
    let s = builtins.call_method1("str", (file_path,))?;
    Ok(s.unbind().into())
}

/// Framework-agnostic in-memory dataset for Kappa datasets.
///
/// This dataset keeps the original `entity_id` for every sample and exposes
/// a PyTorch-style `__len__`/`__getitem__` interface so it can be wrapped by
/// different loaders (Kappa, PyTorch, TensorFlow via generators, etc.).
#[pyclass]
pub struct KappaDataset {
    pub(crate) items: Vec<DatasetItem>,
    pub(crate) transform: Option<Py<PyAny>>,
    pub(crate) target_transform: Option<Py<PyAny>>,
    pub(crate) transform_input_mode: String,
}

impl KappaDataset {
    /// Internal helper to build the canonical sample representation used by all loaders.
    ///
    /// Returns a dict:
    /// {
    ///   "entity_id": str,
    ///   "files": list[ItemFile] | None,
    ///   "annotations": Any (JSON-like)
    /// }
    fn build_raw_sample<'a>(&'a self, index: usize, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        if index >= self.items.len() {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>("Index out of range"));
        }

        let item = self.items[index].clone();

        let raw = PyDict::new(py);
        raw.set_item("entity_id", item.entity_id.clone())?;

        if let Some(files) = &item.files {
            // Clone the vector so it can be converted into a Python list
            raw.set_item("files", files.clone())?;
        } else {
            raw.set_item("files", py.None())?;
        }

        let annotations_py: PyObject = if let Some(annotations) = &item.annotations {
            let json_str = serde_json::to_string(annotations)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to serialize annotations: {}", e)
                ))?;
            let json_module = py.import("json")?;
            let parsed = json_module.call_method1("loads", (json_str,))?;
            parsed.into()
        } else {
            py.None().into()
        };
        raw.set_item("annotations", annotations_py)?;

        Ok(raw)
    }

    /// Internal helper used by both the Kappa loader and PyTorch loader.
    ///
    /// It applies `transform` and `target_transform` (if provided) over the
    /// raw sample while always preserving `entity_id` in the output.
    pub(crate) fn build_transformed_sample(&self, index: usize, py: Python<'_>) -> PyResult<PyObject> {
        let raw = self.build_raw_sample(index, py)?;

        let entity_id_obj: PyObject = raw
            .get_item("entity_id")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Missing entity_id in sample")
            })?
            .into();

        // Prepare cloned handles for transformations
        let raw_for_x = raw.clone();
        let raw_for_y = raw.clone();

        // Features (x)
        let x_obj: PyObject = if let Some(t) = &self.transform {
            let t_bound = t.bind(py);
            if self.transform_input_mode == "content" {
                let content = load_content_from_raw_sample(py, &raw_for_x)?;
                t_bound.call1((content,))?.into()
            } else {
                t_bound.call1((raw_for_x,))?.into()
            }
        } else {
            raw.clone().into()
        };

        // Target (y), optional
        let y_obj: Option<PyObject> = if let Some(tt) = &self.target_transform {
            let tt_bound = tt.bind(py);
            Some(tt_bound.call1((raw_for_y,))?.into())
        } else {
            None
        };

        let out = PyDict::new(py);
        out.set_item("x", x_obj)?;
        if let Some(y) = y_obj {
            out.set_item("y", y)?;
        }
        out.set_item("entity_id", entity_id_obj)?;

        Ok(out.into())
    }
}

#[pymethods]
impl KappaDataset {
    /// Create a new `KappaDataset` from a list of `DatasetItem` objects.
    ///
    /// Normally this is constructed internally via `KappaApkClient`, but it is
    /// exposed to Python for flexibility and testing.
    #[new]
    #[pyo3(signature = (items, transform=None, target_transform=None, transform_input_mode=None))]
    pub fn new(
        items: Vec<DatasetItem>,
        transform: Option<Py<PyAny>>,
        target_transform: Option<Py<PyAny>>,
        transform_input_mode: Option<String>,
    ) -> Self {
        let transform_input_mode = transform_input_mode.unwrap_or_else(|| "content".to_string());
        KappaDataset {
            items,
            transform,
            target_transform,
            transform_input_mode,
        }
    }

    /// Number of entities (samples) in the dataset.
    fn __len__(&self) -> usize {
        self.items.len()
    }

    /// Get a single transformed sample at the given index.
    ///
    /// Returns a dict like:
    /// {
    ///   "x": Any,            # output of `transform` or the raw sample
    ///   "y": Any | missing,  # output of `target_transform` if provided
    ///   "entity_id": str,    # original entity identifier
    /// }
    fn __getitem__(&self, idx: isize, py: Python<'_>) -> PyResult<PyObject> {
        let len = self.items.len() as isize;
        let index = if idx < 0 { len + idx } else { idx };
        if index < 0 || index >= len {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>("Index out of range"));
        }
        self.build_transformed_sample(index as usize, py)
    }
}

impl Datasets {
    /// Load a dataset version and wrap it as a `KappaDataset` instance.
    ///
    /// This is the core building block used by higher-level loader helpers.
    pub fn load_kappa_dataset<T: ApiClient>(
        client: &T,
        dataset_id: Option<i32>,
        dataset_name: Option<String>,
        version_id: Option<i32>,
        version_no: Option<String>,
        dataset_path: Option<String>,
        transform: Option<Py<PyAny>>,
        target_transform: Option<Py<PyAny>>,
        transform_input_mode: String,
    ) -> PyResult<Py<KappaDataset>> {
        // Enforce that exactly one of dataset_id or dataset_name is provided
        let (id_opt, name_opt) = match (dataset_id, dataset_name) {
            // Only id provided
            (Some(id), None) => (Some(id), None),
            // Only name provided
            (None, Some(name)) => (None, Some(name)),
            // Both provided: prefer id, ignore name
            (Some(id), Some(_)) => (Some(id), None),
            (None, None) => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Either dataset_id or dataset_name is required",
                ))
            }
        };

        let items = Self::load(
            client,
            id_opt,
            name_opt,
            version_id,
            version_no,
            dataset_path,
        )?;

        Python::with_gil(|py| {
            Py::new(py, KappaDataset::new(items, transform, target_transform, Some(transform_input_mode)))
        })
    }

    pub fn get_dataset_loader<T: ApiClient>(
        client: &T,
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
        transform_input_mode: String,
    ) -> PyResult<PyObject> {
        let loader_kind = loader_type
            .unwrap_or_else(|| "kappa".to_string())
            .to_lowercase();
        let batch_size = batch_size.unwrap_or(32);
        let shuffle = shuffle.unwrap_or(true);
        let drop_last = drop_last.unwrap_or(false);

        let items = Self::load(
            client,
            dataset_id,
            dataset_name,
            version_id,
            version_no,
            dataset_path,
        )?;

        Python::with_gil(|py| {
            let dataset = Py::new(
                py,
                KappaDataset::new(items, transform, target_transform, Some(transform_input_mode)),
            )?;

            match loader_kind.as_str() {
                "kappa" => {
                    let loader = KappaDataLoader::new_internal(
                        dataset.clone_ref(py),
                        batch_size,
                        shuffle,
                        drop_last,
                        py,
                    )?;
                    let py_loader = Py::new(py, loader)?;
                    Ok(py_loader.into())
                }
                "pytorch" => {
                    let torch = py.import("torch").map_err(|_| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "PyTorch is required for loader_type='pytorch' but could not be imported",
                        )
                    })?;
                    let utils = torch.getattr("utils")?;
                    let data = utils.getattr("data")?;
                    let dataloader_cls = data.getattr("DataLoader")?;

                    let kwargs = PyDict::new(py);
                    kwargs.set_item("batch_size", batch_size)?;
                    kwargs.set_item("shuffle", shuffle)?;
                    kwargs.set_item("drop_last", drop_last)?;

                    let dataloader = dataloader_cls.call((dataset,), Some(&kwargs))?;
                    Ok(dataloader.into())
                }
                "transformers" => {
                    let torch = py.import("torch").map_err(|_| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "PyTorch is required for loader_type='transformers' but could not be imported",
                        )
                    })?;
                    let transformers = py.import("transformers").map_err(|_| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "transformers is required for loader_type='transformers' but could not be imported",
                        )
                    })?;

                    let data = torch.getattr("utils")?.getattr("data")?;
                    let dataloader_cls = data.getattr("DataLoader")?;
                    let default_collator = transformers.getattr("default_data_collator")?;

                    let kwargs = PyDict::new(py);
                    kwargs.set_item("batch_size", batch_size)?;
                    kwargs.set_item("shuffle", shuffle)?;
                    kwargs.set_item("drop_last", drop_last)?;
                    kwargs.set_item("collate_fn", default_collator)?;

                    let dataloader = dataloader_cls.call((dataset,), Some(&kwargs))?;
                    Ok(dataloader.into())
                }
                "tensorflow" => {
                    let tf = py.import("tensorflow").map_err(|_| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "TensorFlow is required for loader_type='tensorflow' but could not be imported",
                        )
                    })?;

                    let output_signature = tf_output_signature.ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "loader_type='tensorflow' requires tf_output_signature, e.g. a nested tf.TensorSpec structure",
                        )
                    })?;

                    // Reuse KappaDataLoader epoch semantics and feed TensorFlow from_generator.
                    let loader = KappaDataLoader::new_internal(
                        dataset.clone_ref(py),
                        batch_size,
                        shuffle,
                        drop_last,
                        py,
                    )?;
                    let py_loader = Py::new(py, loader)?;

                    let locals = PyDict::new(py);
                    locals.set_item("loader", py_loader.bind(py))?;
                    locals.set_item("tf", tf)?;
                    locals.set_item("output_signature", output_signature.bind(py))?;

                    py.run(
                        pyo3::ffi::c_str!(
                            r#"
import numpy as _np

def _kappa_tf_gen(_loader):
    for _batch in _loader:
        # KappaDataLoader yields list[dict]. Convert to dict-of-arrays for tf.data.
        if isinstance(_batch, list) and len(_batch) > 0 and isinstance(_batch[0], dict):
            _keys = list(_batch[0].keys())
            _out = {}
            for _k in _keys:
                _vals = [item.get(_k) for item in _batch]
                if _k == "entity_id":
                    _out[_k] = _np.array(_vals, dtype=_np.str_)
                else:
                    try:
                        _out[_k] = _np.array(_vals)
                    except Exception:
                        # Keep as python list if heterogeneous; signature must match.
                        _out[_k] = _vals
            yield _out
        else:
            yield _batch

tf_dataset = tf.data.Dataset.from_generator(
    lambda: _kappa_tf_gen(loader),
    output_signature=output_signature,
)
"#
                        ),
                        None,
                        Some(&locals),
                    )?;

                    let dataset_obj = locals
                        .get_item("tf_dataset")?
                        .ok_or_else(|| {
                            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                                "Failed to create tf.data.Dataset",
                            )
                        })?;
                    Ok(dataset_obj.into())
                }
                other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!(
                        "Unsupported loader_type: {}. Expected 'kappa', 'pytorch', 'tensorflow', or 'transformers'.",
                        other
                    ),
                )),
            }
        })
    }

    /// POST `/datasets/new` — JSON body matches `NewDataset` (camelCase keys).
    pub fn add_dataset<T: ApiClient>(client: &T, body_json: String) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = "/data-micro-services/v2/datasets/new".to_string();
        client.make_request("POST".to_string(), endpoint, Some(body_json), Some(token))
    }

    /// PUT `/datasets/{dataset_id}` — JSON body matches `UpdateDatasetRequest`.
    pub fn update_dataset<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!("/data-micro-services/v2/datasets/{}", dataset_id);
        client.make_request("PUT".to_string(), endpoint, Some(body_json), Some(token))
    }

    /// POST `/datasets/datasetEntities/new/...` via [`crate::traits::ApiClient::submit_dataset_entity_request`].
    ///
    /// `file_paths`: each entry is a local file path, a local directory (non-recursive: immediate
    /// child files only), or an `http://` / `https://` URL. All resolved payloads are sent as
    /// multipart parts named `files`.
    pub fn add_dataset_entity<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        entity_json: String,
        file_paths: Vec<String>,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!(
            "/data-micro-services/v2/datasets/datasetEntities/new/{}",
            dataset_id
        );
        let parts = resolve_entity_file_sources(client, &file_paths)?;
        client.submit_dataset_entity_request(
            "POST",
            endpoint,
            "new_dataset_entity",
            entity_json,
            parts,
            Some(token),
        )
    }

    /// PUT `/datasets/datasetEntities/.../{entity_id}` via [`crate::traits::ApiClient::submit_dataset_entity_request`].
    ///
    /// `file_paths` are resolved the same way as [`Self::add_dataset_entity`].
    pub fn update_dataset_entity<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        entity_id: &str,
        update_json: String,
        file_paths: Vec<String>,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!(
            "/data-micro-services/v2/datasets/datasetEntities/{}/{}",
            dataset_id, entity_id
        );
        let parts = resolve_entity_file_sources(client, &file_paths)?;
        client.submit_dataset_entity_request(
            "PUT",
            endpoint,
            "update_dataset_entity",
            update_json,
            parts,
            Some(token),
        )
    }

    // -----------------------------------------------------------------------
    // Dataset filtering & structure
    // -----------------------------------------------------------------------

    /// `GET /datasets/filter` with rich query params (search, tags, type, status, publish_type).
    ///
    /// Extends the basic `list_datasets` with all server-side filter options.
    pub fn filter_datasets<T: ApiClient>(
        client: &T,
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
        let page = page.unwrap_or(1);
        let size = size.unwrap_or(20);
        let order_by = order_by.unwrap_or_else(|| "modifiedOn".to_string());
        let order_keyword = order_keyword.unwrap_or_else(|| "DESC".to_string());

        let mut endpoint = format!(
            "/data-micro-services/v2/datasets/filter?page={}&size={}&orderBy={}&orderKeyword={}",
            page, size,
            urlencoding::encode(&order_by),
            urlencoding::encode(&order_keyword),
        );
        if let Some(s) = search {
            endpoint.push_str(&format!("&search={}", urlencoding::encode(&s)));
        }
        if let Some(id) = dataset_id {
            endpoint.push_str(&format!("&datasetId={}", id));
        }
        if let Some(n) = dataset_name {
            endpoint.push_str(&format!("&datasetName={}", urlencoding::encode(&n)));
        }
        if let Some(dt) = dataset_type {
            endpoint.push_str(&format!("&datasetType={}", dt));
        }
        if let Some(tags) = dataset_tags {
            endpoint.push_str(&format!("&datasetTags={}", urlencoding::encode(&tags)));
        }
        if let Some(ds) = dataset_status {
            endpoint.push_str(&format!("&datasetStatus={}", ds));
        }
        if let Some(pt) = publish_type {
            endpoint.push_str(&format!("&publishType={}", pt));
        }
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /datasets/fields/{dataset_id}` — return dataset input-field schema.
    pub fn get_dataset_fields<T: ApiClient>(client: &T, dataset_id: i32) -> PyResult<PyObject> {
        let endpoint = format!("/data-micro-services/v2/datasets/fields/{}", dataset_id);
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// Soft-delete a dataset by setting `datasetStatus = 0`.
    ///
    /// The service uses soft-deletes; deleted datasets can be recovered via the
    /// `/datasets/recover` endpoint.
    pub fn delete_dataset<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        remark: Option<String>,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!("/data-micro-services/v2/datasets/{}", dataset_id);
        let body = serde_json::json!({
            "datasetStatus": 0,
            "remark": remark.unwrap_or_default()
        })
        .to_string();
        client.make_request("PUT".to_string(), endpoint, Some(body), Some(token))
    }

    // -----------------------------------------------------------------------
    // Label management
    // -----------------------------------------------------------------------

    /// `POST /datasets/labels/{dataset_id}` — add one or more label strings.
    pub fn add_dataset_labels<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        labels: Vec<String>,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!("/data-micro-services/v2/datasets/labels/{}", dataset_id);
        let body = serde_json::to_string(&labels).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize labels: {}", e
            ))
        })?;
        client.make_request("POST".to_string(), endpoint, Some(body), Some(token))
    }

    /// `GET /datasets/labels/{dataset_id}` — list all labels for a dataset.
    pub fn get_dataset_labels<T: ApiClient>(
        client: &T,
        dataset_id: i32,
    ) -> PyResult<Vec<DatasetLabel>> {
        let json_obj = client.make_request(
            "GET".to_string(),
            format!("/data-micro-services/v2/datasets/labels/{}", dataset_id),
            None,
            Some(client.require_token()?),
        )?;
        Python::with_gil(|py| -> PyResult<Vec<DatasetLabel>> {
            let json_module = py.import("json")?;
            let json_str: String = json_module
                .getattr("dumps")?
                .call1((json_obj.clone_ref(py),))?
                .extract()?;
            let value: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse labels JSON: {}", e
                ))
            })?;
            parse_dataset_labels_from_value(&value, dataset_id)
        })
    }

    /// `GET /datasets/labels/{dataset_id}` — label strings in API order.
    pub fn get_dataset_label_names<T: ApiClient>(
        client: &T,
        dataset_id: i32,
    ) -> PyResult<Vec<String>> {
        Ok(Self::get_dataset_labels(client, dataset_id)?
            .into_iter()
            .map(|record| record.label)
            .filter(|name| !name.trim().is_empty())
            .collect())
    }

    /// `PUT /datasets/labels/{dataset_id}` — rename an existing label by ID.
    pub fn update_dataset_label<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        label_id: i32,
        label: String,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!("/data-micro-services/v2/datasets/labels/{}", dataset_id);
        let body = serde_json::json!({ "labelId": label_id, "label": label }).to_string();
        client.make_request("PUT".to_string(), endpoint, Some(body), Some(token))
    }

    // -----------------------------------------------------------------------
    // Entity read operations
    // -----------------------------------------------------------------------

    /// `GET /datasets/datasetEntities/{dataset_id}` — list entities (raw JSON list).
    pub fn list_dataset_entities<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        let mut endpoint = format!(
            "/data-micro-services/v2/datasets/datasetEntities/{}",
            dataset_id
        );
        if let Some(vid) = version_id {
            endpoint.push_str(&format!("?version_id={}", vid));
        }
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /datasets/datasetEntities/{dataset_id}/{entity_id}` — get a single entity.
    pub fn get_dataset_entity<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        entity_id: &str,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        let mut endpoint = format!(
            "/data-micro-services/v2/datasets/datasetEntities/{}/{}",
            dataset_id, entity_id
        );
        if let Some(vid) = version_id {
            endpoint.push_str(&format!("?version_id={}", vid));
        }
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `GET /datasets/datasetEntities/filter/{dataset_id}` — paginated entity search.
    pub fn filter_dataset_entities<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        entity_name: Option<String>,
        entity_status: Option<i32>,
        version_id: Option<i32>,
        page: Option<i32>,
        size: Option<i32>,
        order_by: Option<String>,
        order: Option<String>,
    ) -> PyResult<PyObject> {
        let page = page.unwrap_or(0);
        let size = size.unwrap_or(10);
        let order_by = order_by.unwrap_or_else(|| "modifiedOn".to_string());
        let order = order.unwrap_or_else(|| "DESC".to_string());

        let mut endpoint = format!(
            "/data-micro-services/v2/datasets/datasetEntities/filter/{}?page={}&size={}&orderBy={}&order={}",
            dataset_id, page, size,
            urlencoding::encode(&order_by),
            urlencoding::encode(&order),
        );
        if let Some(name) = entity_name {
            endpoint.push_str(&format!("&dsEntityName={}", urlencoding::encode(&name)));
        }
        if let Some(status) = entity_status {
            endpoint.push_str(&format!("&entityStatus={}", status));
        }
        if let Some(vid) = version_id {
            endpoint.push_str(&format!("&versionId={}", vid));
        }
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `DELETE /datasets/datasetEntities` — bulk soft-delete entities by ID list.
    pub fn delete_dataset_entities<T: ApiClient>(
        client: &T,
        dataset_entity_ids: Vec<String>,
        remark: String,
        version_id: Option<i32>,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = "/data-micro-services/v2/datasets/datasetEntities".to_string();
        let mut body_map = serde_json::Map::new();
        body_map.insert(
            "datasetEntityIds".to_string(),
            serde_json::json!(dataset_entity_ids),
        );
        body_map.insert("remark".to_string(), serde_json::json!(remark));
        if let Some(vid) = version_id {
            body_map.insert("versionId".to_string(), serde_json::json!(vid));
        }
        let body = serde_json::Value::Object(body_map).to_string();
        client.make_request("DELETE".to_string(), endpoint, Some(body), Some(token))
    }

    // -----------------------------------------------------------------------
    // Dataset version management
    // -----------------------------------------------------------------------

    /// `POST /datasets/versions/new/{dataset_id}` — create a new dataset version.
    pub fn create_dataset_version<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        body_json: String,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!("/data-micro-services/v2/datasets/versions/new/{}", dataset_id);
        client.make_request("POST".to_string(), endpoint, Some(body_json), Some(token))
    }

    /// `GET /datasets/versions/{dataset_id}` — list all versions for a dataset.
    pub fn list_dataset_versions<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        version_availability: Option<i32>,
    ) -> PyResult<PyObject> {
        let mut endpoint =
            format!("/data-micro-services/v2/datasets/versions/{}", dataset_id);
        if let Some(va) = version_availability {
            endpoint.push_str(&format!("?version_availability={}", va));
        }
        client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
    }

    /// `DELETE /datasets/versions/{dataset_id}/{version_no}` — delete a dataset version.
    pub fn delete_dataset_version<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        version_no: &str,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!(
            "/data-micro-services/v2/datasets/versions/{}/{}",
            dataset_id, version_no
        );
        client.make_request("DELETE".to_string(), endpoint, None, Some(token))
    }

    /// `POST /datasets/versions/publish/{dataset_id}/{version_no}` — publish a version.
    ///
    /// `publish_type`: 0 = Private, 1 = Internal, 2 = Public.
    pub fn publish_dataset_version<T: ApiClient>(
        client: &T,
        dataset_id: i32,
        version_no: &str,
        publish_type: i32,
    ) -> PyResult<PyObject> {
        let token = client.require_token()?;
        let endpoint = format!(
            "/data-micro-services/v2/datasets/versions/publish/{}/{}?publish_type={}",
            dataset_id, version_no, publish_type
        );
        client.make_request("POST".to_string(), endpoint, None, Some(token))
    }
}

/// Collect immediate child files of a directory (non-recursive), sorted by path.
fn collect_directory_files(dir: &Path) -> PyResult<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Cannot read directory {:?}: {}",
                dir, e
            ))
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    Ok(paths)
}

fn guess_filename_from_url(url: &str) -> String {
    url.rsplit('/')
        .next()
        .and_then(|s| {
            let s = s.split('?').next().unwrap_or(s);
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
        .unwrap_or_else(|| "download".to_string())
}

/// Resolve user-supplied paths and URLs into `(bytes, filename)` pairs for multipart `files` parts.
fn resolve_entity_file_sources<T: ApiClient>(
    client: &T,
    sources: &[String],
) -> PyResult<Vec<(Vec<u8>, String)>> {
    let mut out = Vec::new();
    for s in sources {
        let t = s.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with("http://") || t.starts_with("https://") {
            let bytes = client.fetch_url_bytes(t)?;
            let fname = guess_filename_from_url(t);
            out.push((bytes, fname));
            continue;
        }
        let p = Path::new(t);
        let md = std::fs::metadata(p).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Invalid file or directory {:?}: {}",
                t, e
            ))
        })?;
        if md.is_dir() {
            for f in collect_directory_files(p)? {
                let bytes = std::fs::read(&f).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to read {:?}: {}",
                        f, e
                    ))
                })?;
                let name = f
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
                    .to_string();
                out.push((bytes, name));
            }
        } else if md.is_file() {
            let bytes = std::fs::read(p).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to read {:?}: {}",
                    t, e
                ))
            })?;
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            out.push((bytes, name));
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Not a file or directory: {:?}",
                t
            )));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod labels_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn load_fixture(name: &str) -> serde_json::Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        let content = fs::read_to_string(path).expect("read fixture");
        serde_json::from_str(&content).expect("parse fixture JSON")
    }

    #[test]
    fn parse_string_labels() {
        let value = load_fixture("dataset_labels_string.json");
        let labels = parse_dataset_labels_from_value(&value, 7).unwrap();
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[0].label, "cat");
        assert_eq!(labels[0].dataset_id, 7);
    }

    #[test]
    fn parse_wrapped_full_objects() {
        let value = load_fixture("dataset_labels_wrapped.json");
        let labels = parse_dataset_labels_from_value(&value, 7).unwrap();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].label_id, 1);
        assert_eq!(labels[1].label, "Trouser");
    }

    #[test]
    fn parse_partial_label_dicts() {
        let value = serde_json::json!([
            {"label": "Pizza"},
            {"name": "Steak"},
            {"labelName": "Sushi"}
        ]);
        let labels = parse_dataset_labels_from_value(&value, 3).unwrap();
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[2].label, "Sushi");
    }
}