// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Dataset model exposed to Python.
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub dataset_id: i32,
    pub dataset_name: String,
    pub dataset_type: i32,
    pub dataset_type_interp: String,
    pub dataset_short_info: String,
    pub dataset_status: i32,
    pub dataset_status_interp: String,
    pub dataset_tags: String,
    pub publish_type: i32,
    pub created_on: String,
    pub modified_on: String,
    pub version_no: Option<String>,
}

#[pymethods]
impl Dataset {
    #[getter] pub fn dataset_id(&self) -> i32 { self.dataset_id }
    #[getter] pub fn dataset_name(&self) -> &str { &self.dataset_name }
    #[getter] pub fn dataset_type(&self) -> i32 { self.dataset_type }
    #[getter] pub fn dataset_type_interp(&self) -> &str { &self.dataset_type_interp }
    #[getter] pub fn dataset_short_info(&self) -> &str { &self.dataset_short_info }
    #[getter] pub fn dataset_status(&self) -> i32 { self.dataset_status }
    #[getter] pub fn dataset_status_interp(&self) -> &str { &self.dataset_status_interp }
    #[getter] pub fn dataset_tags(&self) -> &str { &self.dataset_tags }
    #[getter] pub fn publish_type(&self) -> i32 { self.publish_type }
    #[getter] pub fn created_on(&self) -> &str { &self.created_on }
    #[getter] pub fn modified_on(&self) -> &str { &self.modified_on }
    #[getter] pub fn version_no(&self) -> Option<&str> { self.version_no.as_deref() }
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetVersionDetails {
    pub id: i32,
    pub user_id: i32,
    pub dataset_id: i32,
    pub version_availability: i32,
    pub version_no: String,
    pub version_remark: String,
    pub publish_type: i32,
    pub created_on: String,
    pub modified_on: String,
}

#[pymethods]
impl DatasetVersionDetails {
    #[getter] pub fn id(&self) -> i32 { self.id }
    #[getter] pub fn user_id(&self) -> i32 { self.user_id }
    #[getter] pub fn dataset_id(&self) -> i32 { self.dataset_id }
    #[getter] pub fn version_availability(&self) -> i32 { self.version_availability }
    #[getter] pub fn version_no(&self) -> &str { &self.version_no }
    #[getter] pub fn version_remark(&self) -> &str { &self.version_remark }
    #[getter] pub fn publish_type(&self) -> i32 { self.publish_type }
    #[getter] pub fn created_on(&self) -> &str { &self.created_on }
    #[getter] pub fn modified_on(&self) -> &str { &self.modified_on }
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetDownloadDetails {
    pub dataset_id: i32,
    pub version_no: String,
    pub data_path: String,
    pub download_status: bool,
}

#[pymethods]
impl DatasetDownloadDetails {
    #[getter] pub fn dataset_id(&self) -> i32 { self.dataset_id }
    #[getter] pub fn version_no(&self) -> &str { &self.version_no }
    #[getter] pub fn data_path(&self) -> &str { &self.data_path }
    #[getter] pub fn download_status(&self) -> bool { self.download_status }
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFile {
    pub file_id: String,
    pub file_name: String,
    pub file: PathBuf,
}

#[pymethods]
impl ItemFile {
    /// Get the file ID associated with this file.
    #[getter]
    pub fn file_id(&self) -> String { 
        self.file_id.clone() 
    }
    
    /// Get the file name.
    #[getter]
    pub fn file_name(&self) -> String { 
        self.file_name.clone() 
    }
    
    /// Get the full file path as a string.
    #[getter]
    pub fn file(&self) -> String { 
        self.file.to_string_lossy().to_string() 
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnnotationValue {
    Str(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    VecString(Vec<String>),
    VecInt(Vec<i32>),
    VecFloat(Vec<f64>),
    VecBool(Vec<bool>),
    Map(HashMap<String, AnnotationValue>),
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetItem {
    pub entity_id: String,
    pub files: Option<Vec<ItemFile>>,
    pub annotations: Option<Vec<HashMap<String, AnnotationValue>>>,
    #[serde(default)]
    pub entity_info: Option<HashMap<String, serde_json::Value>>,
}

#[pymethods]
impl DatasetItem {
    /// Get the entity ID for this dataset item.
    #[getter]
    pub fn entity_id(&self) -> String { 
        self.entity_id.clone() 
    }
    
    /// Get the files associated with this dataset item.
    #[getter]
    pub fn files(&self) -> Option<Vec<ItemFile>> { 
        self.files.clone() 
    }
    
    /// Get the annotations for this dataset item as a Python object.
    /// 
    /// # Returns
    /// 
    /// A Python object containing the annotations, or None if no annotations exist.
    #[getter]
    pub fn annotations(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if let Some(annotations) = &self.annotations {
                let json_str = serde_json::to_string(annotations)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize annotations: {}", e
                    )))?;
                let json_module = py.import("json")?;
                let parsed = json_module.call_method1("loads", (json_str,))?;
                Ok(parsed.into())
            } else {
                Ok(py.None().into())
            }
        })
    }

    /// Get the full entity info payload (excluding file paths) as a Python object.
    #[getter]
    pub fn entity_info(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if let Some(entity_info) = &self.entity_info {
                let json_str = serde_json::to_string(entity_info)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize entity info: {}", e
                    )))?;
                let json_module = py.import("json")?;
                let parsed = json_module.call_method1("loads", (json_str,))?;
                Ok(parsed.into())
            } else {
                Ok(py.None().into())
            }
        })
    }

    /// Entity split from `dsEntityInfo.split` (defaults to `"train"` when absent).
    ///
    /// Common values: `train`, `validation`, `test` (custom schemas may allow more).
    #[getter]
    pub fn split(&self) -> String {
        entity_split_from_info(self.entity_info.as_ref())
    }
}

/// Resolve split for a dataset item (`dsEntityInfo.split`, default `"train"`).
pub fn entity_split_from_info(info: Option<&HashMap<String, serde_json::Value>>) -> String {
    info.and_then(|m| m.get("split"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "train".to_string())
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

fn pyany_to_json_value(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    let json_mod = obj.py().import("json")?;
    let dumps = json_mod.getattr("dumps")?;
    let s: String = dumps.call1((obj,))?.extract()?;
    serde_json::from_str(&s).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Value is not JSON-serializable: {}",
            e
        ))
    })
}

/// Mirrors the data service `NewDataset` model. Use snake_case attributes in Python; the client
/// sends camelCase JSON to the API.
#[pyclass]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewDataset {
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    #[pyo3(get, set)]
    pub user_id: i32,
    #[pyo3(get, set)]
    pub dataset_name: String,
    #[pyo3(get, set)]
    pub dataset_type: i32,
    #[pyo3(get, set)]
    pub dataset_short_info: String,
    #[pyo3(get, set)]
    pub dataset_tags: String,
    #[pyo3(get, set)]
    pub dataset_verification_type: i32,
}

#[pymethods]
impl NewDataset {
    #[new]
    #[pyo3(signature = (dataset_name, dataset_type, dataset_short_info, dataset_tags, user_id=0, dataset_verification_type=1))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dataset_name: String,
        dataset_type: i32,
        dataset_short_info: String,
        dataset_tags: String,
        user_id: i32,
        dataset_verification_type: i32,
    ) -> Self {
        NewDataset {
            user_id,
            dataset_name,
            dataset_type,
            dataset_short_info,
            dataset_tags,
            dataset_verification_type,
        }
    }

    /// Serialize to the JSON string expected by the dataset service (camelCase keys).
    pub fn to_api_json(&self) -> PyResult<String> {
        serde_json::to_string(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize NewDataset: {}",
                e
            ))
        })
    }
}

/// Mirrors `UpdateDatasetRequest` for `PUT .../datasets/{user}/{user_type}/{dataset_id}`.
#[pyclass]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatasetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub dataset_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub dataset_status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub dataset_verification_type: Option<i32>,
}

#[pymethods]
impl UpdateDatasetRequest {
    #[new]
    #[pyo3(signature = (dataset_name=None, dataset_status=None, remark=None, dataset_verification_type=None))]
    pub fn new(
        dataset_name: Option<String>,
        dataset_status: Option<i32>,
        remark: Option<String>,
        dataset_verification_type: Option<i32>,
    ) -> Self {
        UpdateDatasetRequest {
            dataset_name,
            dataset_status,
            remark,
            dataset_verification_type,
        }
    }

    pub fn to_api_json(&self) -> PyResult<String> {
        serde_json::to_string(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize UpdateDatasetRequest: {}",
                e
            ))
        })
    }
}

/// Mirrors `NewDatasetEntity`. `ds_entity_info` and optional `files_category` are Python dicts.
///
/// Optional `split` is merged into `dsEntityInfo.split` on serialize (`train`/`validation`/`test`).
#[pyclass]
pub struct NewDatasetEntity {
    #[pyo3(get, set)]
    pub dataset_id: i32,
    #[pyo3(get, set)]
    pub user_id: i32,
    #[pyo3(get, set)]
    pub ds_entity_name: String,
    #[pyo3(get, set)]
    pub entity_source: Option<String>,
    #[pyo3(get, set)]
    pub collected_on: String,
    #[pyo3(get, set)]
    pub labeling_algo: String,
    ds_entity_info: Py<PyAny>,
    #[pyo3(get, set)]
    pub location_id: Option<i32>,
    files_category: Option<Py<PyAny>>,
    /// Optional train/validation/test (or schema-allowed) split written into `dsEntityInfo`.
    #[pyo3(get, set)]
    pub split: Option<String>,
}

#[pymethods]
impl NewDatasetEntity {
    #[new]
    #[pyo3(signature = (ds_entity_name, collected_on, labeling_algo, ds_entity_info, dataset_id=0, user_id=0, entity_source=None, location_id=None, files_category=None, split=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ds_entity_name: String,
        collected_on: String,
        labeling_algo: String,
        ds_entity_info: Py<PyAny>,
        dataset_id: i32,
        user_id: i32,
        entity_source: Option<String>,
        location_id: Option<i32>,
        files_category: Option<Py<PyAny>>,
        split: Option<String>,
    ) -> Self {
        NewDatasetEntity {
            dataset_id,
            user_id,
            ds_entity_name,
            entity_source,
            collected_on,
            labeling_algo,
            ds_entity_info,
            location_id,
            files_category,
            split,
        }
    }

    #[getter]
    fn ds_entity_info(&self, py: Python<'_>) -> Py<PyAny> {
        self.ds_entity_info.clone_ref(py)
    }

    #[setter]
    fn set_ds_entity_info(&mut self, value: Py<PyAny>) {
        self.ds_entity_info = value;
    }

    #[getter]
    fn files_category<'py>(&self, py: Python<'py>) -> PyObject {
        match &self.files_category {
            Some(p) => p.clone_ref(py).into(),
            None => py.None().into(),
        }
    }

    #[setter]
    fn set_files_category(&mut self, value: Option<Py<PyAny>>) {
        self.files_category = value;
    }

    pub fn to_api_json(&self, py: Python<'_>) -> PyResult<String> {
        use serde_json::{json, Map, Value};
        let mut info = pyany_to_json_value(&self.ds_entity_info.bind(py))?;
        if let Some(ref split) = self.split {
            let normalized = normalize_entity_split(split)?;
            if let Some(obj) = info.as_object_mut() {
                obj.insert("split".to_string(), json!(normalized));
            } else {
                info = json!({ "split": normalized });
            }
        }
        let mut map = Map::new();
        if self.dataset_id != 0 {
            map.insert("datasetId".to_string(), json!(self.dataset_id));
        }
        if self.user_id != 0 {
            map.insert("userId".to_string(), json!(self.user_id));
        }
        map.insert("dsEntityName".to_string(), json!(self.ds_entity_name));
        if let Some(ref s) = self.entity_source {
            map.insert("entitySource".to_string(), json!(s));
        }
        map.insert("collectedOn".to_string(), json!(self.collected_on));
        map.insert("labelingAlgo".to_string(), json!(self.labeling_algo));
        map.insert("dsEntityInfo".to_string(), info);
        if let Some(id) = self.location_id {
            map.insert("locationId".to_string(), json!(id));
        }
        if let Some(ref fc) = self.files_category {
            let v = pyany_to_json_value(&fc.bind(py))?;
            map.insert("filesCategory".to_string(), v);
        }
        let v = Value::Object(map);
        serde_json::to_string(&v).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize NewDatasetEntity: {}",
                e
            ))
        })
    }
}

/// Normalize entity `split` (non-empty). Recommended: `train`, `validation`, `test`.
pub fn normalize_entity_split(split: &str) -> PyResult<String> {
    let s = split.trim().to_string();
    if s.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "split must be a non-empty string (e.g. 'train', 'validation', 'test')",
        ));
    }
    Ok(s)
}

/// Normalize entity file category to `input` or `output`.
pub fn normalize_entity_file_category(category: &str) -> PyResult<String> {
    let c = category.trim().to_ascii_lowercase();
    match c.as_str() {
        "input" | "output" => Ok(c),
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "file_category must be 'input' or 'output'",
        )),
    }
}

/// Mirrors `UpdateDatasetEntity` for entity updates. `remark` is required by the service.
///
/// Optional `split` is merged into `dsEntityInfo.split` on serialize.
#[pyclass]
pub struct UpdateDatasetEntity {
    #[pyo3(get, set)]
    pub ds_entity_name: Option<String>,
    #[pyo3(get, set)]
    pub entity_source: Option<String>,
    #[pyo3(get, set)]
    pub collected_on: Option<String>,
    #[pyo3(get, set)]
    pub labeling_algo: Option<String>,
    ds_entity_info: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    pub location_id: Option<i32>,
    #[pyo3(get, set)]
    pub ds_entity_status: Option<i32>,
    #[pyo3(get, set)]
    pub remark: String,
    files_category: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    pub version_id: i32,
    #[pyo3(get, set)]
    pub update_latest_entity: bool,
    #[pyo3(get, set)]
    pub split: Option<String>,
}

#[pymethods]
impl UpdateDatasetEntity {
    #[new]
    #[pyo3(signature = (remark, ds_entity_name=None, entity_source=None, collected_on=None, labeling_algo=None, ds_entity_info=None, location_id=None, ds_entity_status=None, files_category=None, version_id=0, update_latest_entity=false, split=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remark: String,
        ds_entity_name: Option<String>,
        entity_source: Option<String>,
        collected_on: Option<String>,
        labeling_algo: Option<String>,
        ds_entity_info: Option<Py<PyAny>>,
        location_id: Option<i32>,
        ds_entity_status: Option<i32>,
        files_category: Option<Py<PyAny>>,
        version_id: i32,
        update_latest_entity: bool,
        split: Option<String>,
    ) -> Self {
        UpdateDatasetEntity {
            ds_entity_name,
            entity_source,
            collected_on,
            labeling_algo,
            ds_entity_info,
            location_id,
            ds_entity_status,
            remark,
            files_category,
            version_id,
            update_latest_entity,
            split,
        }
    }

    #[getter]
    fn ds_entity_info<'py>(&self, py: Python<'py>) -> PyObject {
        match &self.ds_entity_info {
            Some(p) => p.clone_ref(py).into(),
            None => py.None().into(),
        }
    }

    #[setter]
    fn set_ds_entity_info(&mut self, value: Option<Py<PyAny>>) {
        self.ds_entity_info = value;
    }

    #[getter]
    fn files_category<'py>(&self, py: Python<'py>) -> PyObject {
        match &self.files_category {
            Some(p) => p.clone_ref(py).into(),
            None => py.None().into(),
        }
    }

    #[setter]
    fn set_files_category(&mut self, value: Option<Py<PyAny>>) {
        self.files_category = value;
    }

    pub fn to_api_json(&self, py: Python<'_>) -> PyResult<String> {
        use serde_json::{json, Map, Value};
        let mut map = Map::new();
        if let Some(ref n) = self.ds_entity_name {
            map.insert("dsEntityName".to_string(), json!(n));
        }
        if let Some(ref s) = self.entity_source {
            map.insert("entitySource".to_string(), json!(s));
        }
        if let Some(ref c) = self.collected_on {
            map.insert("collectedOn".to_string(), json!(c));
        }
        if let Some(ref a) = self.labeling_algo {
            map.insert("labelingAlgo".to_string(), json!(a));
        }
        let mut info_value = if let Some(ref p) = self.ds_entity_info {
            Some(pyany_to_json_value(&p.bind(py))?)
        } else {
            None
        };
        if let Some(ref split) = self.split {
            let normalized = normalize_entity_split(split)?;
            match info_value.as_mut() {
                Some(Value::Object(obj)) => {
                    obj.insert("split".to_string(), json!(normalized));
                }
                Some(_) => {
                    info_value = Some(json!({ "split": normalized }));
                }
                None => {
                    info_value = Some(json!({ "split": normalized }));
                }
            }
        }
        if let Some(v) = info_value {
            map.insert("dsEntityInfo".to_string(), v);
        }
        if let Some(id) = self.location_id {
            map.insert("locationId".to_string(), json!(id));
        }
        if let Some(st) = self.ds_entity_status {
            map.insert("dsEntityStatus".to_string(), json!(st));
        }
        map.insert("remark".to_string(), json!(self.remark));
        if let Some(ref fc) = self.files_category {
            let v = pyany_to_json_value(&fc.bind(py))?;
            map.insert("filesCategory".to_string(), v);
        }
        if self.version_id != 0 {
            map.insert("versionId".to_string(), json!(self.version_id));
        }
        if self.update_latest_entity {
            map.insert("updateLatestEntity".to_string(), json!(true));
        }
        let v = Value::Object(map);
        serde_json::to_string(&v).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize UpdateDatasetEntity: {}",
                e
            ))
        })
    }
}

// ---------------------------------------------------------------------------
// Label management
// ---------------------------------------------------------------------------

/// A single dataset label record returned by the labels endpoint.
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetLabel {
    #[serde(default)]
    pub label_id: i32,
    #[serde(default)]
    pub dataset_id: i32,
    pub label: String,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub modified_on: Option<String>,
}

#[pymethods]
impl DatasetLabel {
    #[getter] pub fn label_id(&self) -> i32 { self.label_id }
    #[getter] pub fn dataset_id(&self) -> i32 { self.dataset_id }
    #[getter] pub fn label(&self) -> &str { &self.label }
    #[getter] pub fn created_on(&self) -> Option<&str> { self.created_on.as_deref() }
    #[getter] pub fn modified_on(&self) -> Option<&str> { self.modified_on.as_deref() }
}

/// Request body for `PUT /datasets/labels/{dataset_id}` — rename a label.
#[pyclass]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatasetLabel {
    #[pyo3(get, set)]
    pub label_id: i32,
    #[pyo3(get, set)]
    pub label: String,
}

#[pymethods]
impl UpdateDatasetLabel {
    #[new]
    pub fn new(label_id: i32, label: String) -> Self {
        UpdateDatasetLabel { label_id, label }
    }

    pub fn to_api_json(&self) -> PyResult<String> {
        serde_json::to_string(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize UpdateDatasetLabel: {}", e
            ))
        })
    }
}

// ---------------------------------------------------------------------------
// Entity deletion
// ---------------------------------------------------------------------------

/// Request body for `DELETE /datasets/datasetEntities` — bulk soft-delete entities.
#[pyclass]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDatasetEntities {
    #[pyo3(get, set)]
    pub dataset_entity_ids: Vec<String>,
    #[pyo3(get, set)]
    pub remark: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub version_id: Option<i32>,
}

#[pymethods]
impl DeleteDatasetEntities {
    #[new]
    #[pyo3(signature = (dataset_entity_ids, remark, version_id=None))]
    pub fn new(
        dataset_entity_ids: Vec<String>,
        remark: String,
        version_id: Option<i32>,
    ) -> Self {
        DeleteDatasetEntities { dataset_entity_ids, remark, version_id }
    }

    pub fn to_api_json(&self) -> PyResult<String> {
        serde_json::to_string(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize DeleteDatasetEntities: {}", e
            ))
        })
    }
}

// ---------------------------------------------------------------------------
// Version management
// ---------------------------------------------------------------------------

/// Request body for `POST /datasets/versions/new/{dataset_id}`.
#[pyclass]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewDatasetVersion {
    #[pyo3(get, set)]
    pub version_availability: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub version_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[pyo3(get, set)]
    pub version_remark: Option<String>,
}

#[pymethods]
impl NewDatasetVersion {
    #[new]
    #[pyo3(signature = (version_availability=1, version_type=None, version_remark=None))]
    pub fn new(
        version_availability: i32,
        version_type: Option<String>,
        version_remark: Option<String>,
    ) -> Self {
        NewDatasetVersion { version_availability, version_type, version_remark }
    }

    pub fn to_api_json(&self) -> PyResult<String> {
        serde_json::to_string(self).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to serialize NewDatasetVersion: {}", e
            ))
        })
    }
}

// ---------------------------------------------------------------------------
// Bulk upload job status
// ---------------------------------------------------------------------------

/// Snapshot of a bulk-upload job (`GET …/bulk/jobs/{jobId}`).
///
/// Use [`BulkUploadJob::percent`], [`BulkUploadJob::is_terminal`], and
/// [`BulkUploadJob::status`] to drive progress UIs (same fields as the React panel).
#[pyclass]
#[derive(Clone, Debug)]
pub struct BulkUploadJob {
    raw: serde_json::Value,
}

impl BulkUploadJob {
    pub fn from_json_value(value: serde_json::Value) -> Self {
        BulkUploadJob { raw: value }
    }

    fn str_field(&self, camel: &str, snake: &str) -> Option<String> {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if v.is_null() {
                    None
                } else {
                    Some(v.to_string())
                }
            })
    }

    fn i64_field(&self, camel: &str, snake: &str) -> i64 {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
            .unwrap_or(0)
    }

    fn bool_field(&self, camel: &str, snake: &str) -> bool {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

#[pymethods]
impl BulkUploadJob {
    #[getter]
    fn job_id(&self) -> String {
        self.str_field("jobId", "job_id").unwrap_or_default()
    }

    #[getter]
    fn dataset_id(&self) -> i32 {
        self.i64_field("datasetId", "dataset_id") as i32
    }

    #[getter]
    fn upload_type(&self) -> String {
        self.str_field("uploadType", "upload_type").unwrap_or_default()
    }

    #[getter]
    fn status(&self) -> String {
        self.str_field("status", "status").unwrap_or_default()
    }

    #[getter]
    fn phase(&self) -> Option<String> {
        self.str_field("phase", "phase")
    }

    #[getter]
    fn total_rows(&self) -> i64 {
        self.i64_field("totalRows", "total_rows")
    }

    #[getter]
    fn processed_rows(&self) -> i64 {
        self.i64_field("processedRows", "processed_rows")
    }

    /// Job processing percent (`processedRows / totalRows`), or `None` if total is 0.
    #[getter]
    fn percent(&self) -> Option<i32> {
        let total = self.total_rows();
        if total <= 0 {
            return None;
        }
        let pct = ((self.processed_rows() as f64) * 100.0 / (total as f64)).round() as i32;
        Some(pct.clamp(0, 100))
    }

    #[getter]
    fn source(&self) -> Option<String> {
        self.str_field("source", "source")
    }

    #[getter]
    fn filename(&self) -> Option<String> {
        self.str_field("filename", "filename")
    }

    #[getter]
    fn failure_kind(&self) -> Option<String> {
        self.str_field("failureKind", "failure_kind")
    }

    #[getter]
    fn labeling_algo(&self) -> Option<String> {
        self.str_field("labelingAlgo", "labeling_algo")
    }

    #[getter]
    fn retryable(&self) -> bool {
        self.bool_field("retryable", "retryable")
    }

    #[getter]
    fn can_cancel(&self) -> bool {
        self.bool_field("canCancel", "can_cancel")
    }

    #[getter]
    fn can_retry(&self) -> bool {
        self.bool_field("canRetry", "can_retry")
    }

    #[getter]
    fn is_stale(&self) -> bool {
        self.bool_field("isStale", "is_stale")
    }

    #[getter]
    fn preflight_deferred(&self) -> bool {
        self.bool_field("preflightDeferred", "preflight_deferred")
    }

    /// True when status is a terminal job state (FE `isBulkJobTerminal` parity).
    ///
    /// Note: ``needs_correction`` is **not** terminal (FE still treats it as active for
    /// list filters) but :meth:`wait_for_bulk_upload_job` stops on it so callers can retry.
    fn is_terminal(&self) -> bool {
        matches!(
            self.status().to_ascii_lowercase().as_str(),
            "completed"
                | "complete"
                | "completed_with_errors"
                | "failed"
                | "error"
                | "cancelled"
                | "canceled"
        )
    }

    /// True when polling should stop (terminal **or** ``needs_correction``).
    fn is_wait_complete(&self) -> bool {
        self.is_terminal() || self.status().eq_ignore_ascii_case("needs_correction")
    }

    /// True when status is ``needs_correction`` (fix paths / retry with staging).
    fn needs_correction(&self) -> bool {
        self.status().eq_ignore_ascii_case("needs_correction")
    }

    /// Full API payload as a Python dict.
    fn as_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(&self.raw).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
        })?;
        let json_mod = py.import("json")?;
        Ok(json_mod.call_method1("loads", (json_str,))?.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "BulkUploadJob(job_id={:?}, status={:?}, percent={:?}, phase={:?})",
            self.job_id(),
            self.status(),
            self.percent(),
            self.phase()
        )
    }
}
/// Snapshot of a bulk-mutation job (`GET …/bulk-mutation/jobs/{jobId}`).
///
/// Covers self-verify, auto-verify, mark-labeled, delete, recover, delete-files (Kappa ≥ 2.11).
#[pyclass]
#[derive(Clone, Debug)]
pub struct BulkMutationJob {
    raw: serde_json::Value,
}

impl BulkMutationJob {
    pub fn from_json_value(value: serde_json::Value) -> Self {
        BulkMutationJob { raw: value }
    }

    fn str_field(&self, camel: &str, snake: &str) -> Option<String> {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if v.is_null() {
                    None
                } else {
                    Some(v.to_string())
                }
            })
    }

    fn i64_field(&self, camel: &str, snake: &str) -> i64 {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
            .unwrap_or(0)
    }

    fn bool_field(&self, camel: &str, snake: &str) -> bool {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

#[pymethods]
impl BulkMutationJob {
    #[getter]
    fn job_id(&self) -> String {
        self.str_field("jobId", "job_id").unwrap_or_default()
    }

    #[getter]
    fn dataset_id(&self) -> i32 {
        self.i64_field("datasetId", "dataset_id") as i32
    }

    #[getter]
    fn job_type(&self) -> String {
        self.str_field("jobType", "job_type").unwrap_or_default()
    }

    #[getter]
    fn status(&self) -> String {
        self.str_field("status", "status").unwrap_or_default()
    }

    #[getter]
    fn phase(&self) -> Option<String> {
        self.str_field("phase", "phase")
    }

    #[getter]
    fn total_count(&self) -> i64 {
        self.i64_field("totalCount", "total_count")
    }

    #[getter]
    fn processed_count(&self) -> i64 {
        self.i64_field("processedCount", "processed_count")
    }

    #[getter]
    fn succeeded_count(&self) -> i64 {
        self.i64_field("succeededCount", "succeeded_count")
    }

    #[getter]
    fn skipped_count(&self) -> i64 {
        self.i64_field("skippedCount", "skipped_count")
    }

    #[getter]
    fn failed_count(&self) -> i64 {
        self.i64_field("failedCount", "failed_count")
    }

    /// Backend `overallPercent` when present; else derived from processed/total.
    #[getter]
    fn percent(&self) -> Option<i32> {
        if let Some(v) = self
            .raw
            .get("overallPercent")
            .or_else(|| self.raw.get("overall_percent"))
            .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
        {
            return Some((v as i32).clamp(0, 100));
        }
        let total = self.total_count();
        if total <= 0 {
            return None;
        }
        let pct = ((self.processed_count() as f64) * 100.0 / (total as f64)).round() as i32;
        Some(pct.clamp(0, 100))
    }

    #[getter]
    fn eta_human(&self) -> Option<String> {
        self.str_field("etaHuman", "eta_human")
    }

    #[getter]
    fn error_detail(&self) -> Option<String> {
        self.str_field("errorDetail", "error_detail")
    }

    #[getter]
    fn can_cancel(&self) -> bool {
        self.bool_field("canCancel", "can_cancel")
    }

    /// True for `succeeded` / `failed` / `cancelled`.
    fn is_terminal(&self) -> bool {
        matches!(
            self.status().to_ascii_lowercase().as_str(),
            "succeeded" | "failed" | "error" | "cancelled" | "canceled"
        )
    }

    fn is_wait_complete(&self) -> bool {
        self.is_terminal()
    }

    fn as_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(&self.raw).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
        })?;
        let json_mod = py.import("json")?;
        Ok(json_mod.call_method1("loads", (json_str,))?.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "BulkMutationJob(job_id={:?}, job_type={:?}, status={:?}, percent={:?}, phase={:?})",
            self.job_id(),
            self.job_type(),
            self.status(),
            self.percent(),
            self.phase()
        )
    }
}

/// Snapshot of a dataset version archive build job (`GET …/versions/build-jobs/{jobId}`).
#[pyclass]
#[derive(Clone, Debug)]
pub struct VersionBuildJob {
    raw: serde_json::Value,
}

impl VersionBuildJob {
    pub fn from_json_value(value: serde_json::Value) -> Self {
        VersionBuildJob { raw: value }
    }

    fn str_field(&self, camel: &str, snake: &str) -> Option<String> {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if v.is_null() {
                    None
                } else {
                    Some(v.to_string())
                }
            })
    }

    fn i64_field(&self, camel: &str, snake: &str) -> i64 {
        self.raw
            .get(camel)
            .or_else(|| self.raw.get(snake))
            .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
            .unwrap_or(0)
    }
}

#[pymethods]
impl VersionBuildJob {
    #[getter]
    fn job_id(&self) -> String {
        self.str_field("jobId", "job_id").unwrap_or_default()
    }

    #[getter]
    fn dataset_id(&self) -> i32 {
        self.i64_field("datasetId", "dataset_id") as i32
    }

    #[getter]
    fn version_id(&self) -> i32 {
        self.i64_field("versionId", "version_id") as i32
    }

    #[getter]
    fn version_no(&self) -> String {
        self.str_field("versionNo", "version_no").unwrap_or_default()
    }

    #[getter]
    fn job_type(&self) -> String {
        self.str_field("jobType", "job_type").unwrap_or_default()
    }

    #[getter]
    fn status(&self) -> String {
        self.str_field("status", "status").unwrap_or_default()
    }

    #[getter]
    fn phase(&self) -> Option<String> {
        self.str_field("phase", "phase")
    }

    #[getter]
    fn total_entities(&self) -> i64 {
        self.i64_field("totalEntities", "total_entities")
    }

    #[getter]
    fn processed_entities(&self) -> i64 {
        self.i64_field("processedEntities", "processed_entities")
    }

    #[getter]
    fn shards_total(&self) -> i64 {
        self.i64_field("shardsTotal", "shards_total")
    }

    #[getter]
    fn shards_uploaded(&self) -> i64 {
        self.i64_field("shardsUploaded", "shards_uploaded")
    }

    #[getter]
    fn percent(&self) -> Option<i32> {
        if let Some(v) = self
            .raw
            .get("overallPercent")
            .or_else(|| self.raw.get("overall_percent"))
            .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
        {
            return Some((v as i32).clamp(0, 100));
        }
        let total = self.total_entities();
        if total <= 0 {
            return None;
        }
        let pct =
            ((self.processed_entities() as f64) * 100.0 / (total as f64)).round() as i32;
        Some(pct.clamp(0, 100))
    }

    #[getter]
    fn eta_human(&self) -> Option<String> {
        self.str_field("etaHuman", "eta_human")
    }

    #[getter]
    fn error_json(&self) -> Option<String> {
        self.str_field("errorJson", "error_json")
    }

    /// True for `completed` / `failed` / `cancelled`.
    fn is_terminal(&self) -> bool {
        matches!(
            self.status().to_ascii_lowercase().as_str(),
            "completed" | "complete" | "failed" | "error" | "cancelled" | "canceled"
        )
    }

    fn is_wait_complete(&self) -> bool {
        self.is_terminal()
    }

    /// True when the build finished successfully (archive ready to publish/download).
    fn is_ready(&self) -> bool {
        matches!(
            self.status().to_ascii_lowercase().as_str(),
            "completed" | "complete"
        ) || self
            .phase()
            .map(|p| p.eq_ignore_ascii_case("ready"))
            .unwrap_or(false)
    }

    fn as_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(&self.raw).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string())
        })?;
        let json_mod = py.import("json")?;
        Ok(json_mod.call_method1("loads", (json_str,))?.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "VersionBuildJob(job_id={:?}, version_no={:?}, status={:?}, percent={:?}, phase={:?})",
            self.job_id(),
            self.version_no(),
            self.status(),
            self.percent(),
            self.phase()
        )
    }
}

#[cfg(test)]
mod bulk_upload_job_tests {
    use super::{BulkMutationJob, BulkUploadJob, VersionBuildJob};
    use serde_json::json;

    #[test]
    fn is_terminal_includes_completed_with_errors() {
        let job = BulkUploadJob::from_json_value(json!({
            "jobId": "j1",
            "status": "completed_with_errors"
        }));
        assert!(job.is_terminal());
    }

    #[test]
    fn is_terminal_false_while_running() {
        let job = BulkUploadJob::from_json_value(json!({
            "jobId": "j1",
            "status": "processing"
        }));
        assert!(!job.is_terminal());
        assert!(!job.is_wait_complete());
    }

    #[test]
    fn needs_correction_stops_wait_but_not_terminal() {
        let job = BulkUploadJob::from_json_value(json!({
            "jobId": "j1",
            "status": "needs_correction",
            "retryable": true
        }));
        assert!(!job.is_terminal());
        assert!(job.needs_correction());
        assert!(job.is_wait_complete());
    }

    #[test]
    fn mutation_job_terminal_statuses() {
        let ok = BulkMutationJob::from_json_value(json!({
            "jobId": "m1",
            "status": "succeeded",
            "overallPercent": 100
        }));
        assert!(ok.is_terminal());
        assert_eq!(ok.percent(), Some(100));

        let run = BulkMutationJob::from_json_value(json!({
            "jobId": "m1",
            "status": "running",
            "totalCount": 10,
            "processedCount": 5
        }));
        assert!(!run.is_wait_complete());
        assert_eq!(run.percent(), Some(50));
    }

    #[test]
    fn version_build_job_ready() {
        let job = VersionBuildJob::from_json_value(json!({
            "jobId": "b1",
            "status": "completed",
            "phase": "ready",
            "versionNo": "1.0.0"
        }));
        assert!(job.is_terminal());
        assert!(job.is_ready());
    }
}
