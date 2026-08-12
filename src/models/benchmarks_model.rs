// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dataset model exposed to Python.
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Benchmark {
    pub benchmark_id: String,
    #[serde(rename = "mlModelId", default)]
    pub model_id: Option<String>,
    pub dataset_id: i32,
    pub dataset_version_id: i32,
    /// Version number string (e.g. `"1.0.0"`); the package download routes key off this.
    #[serde(default)]
    pub dataset_version_no: Option<String>,
    #[serde(rename = "mlmodelVersionId", default)]
    pub model_version_id: Option<i32>,
    #[serde(default)]
    pub benchmark_description: Option<String>,
    pub benchmark_status: i32,
    pub user_id: i32,
    #[serde(default)]
    pub report_id: Option<i32>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub modified_on: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Str(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    VecString(Vec<String>),
    VecInt(Vec<i32>),
    VecFloat(Vec<f64>),
    VecBool(Vec<bool>),
    Map(HashMap<String, MetricValue>),
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prediction {
    pub entity_id: String,
    #[serde(default)]
    pub original: Option<serde_json::Value>,
    #[serde(default)]
    pub predicted: Option<serde_json::Value>,
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Results {
    pub metrics: Option<HashMap<String, MetricValue>>,
    pub predictions: Vec<Prediction>,
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInformation {
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub file_hash: String,
}

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResult {
    pub benchmark_id: String,
    #[serde(default)]
    pub model_id: String,
    pub model_information: Option<Vec<FileInformation>>,
    pub file_information: Option<Vec<FileInformation>>,
    pub results: Results,
}

#[pymethods]
impl BenchmarkResult {
    #[getter]
    fn benchmark_id(&self) -> String {
        self.benchmark_id.clone()
    }

    /// The result exactly as it is sent to the model service (`inferenceResult`).
    fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        crate::utils::python_json::rust_value_to_pyobject(py, self)
    }

    fn __repr__(&self) -> String {
        let metrics = self
            .results
            .metrics
            .as_ref()
            .map(|m| m.len())
            .unwrap_or_default();
        let files = self
            .model_information
            .as_ref()
            .or(self.file_information.as_ref())
            .map(|f| f.len())
            .unwrap_or_default();
        format!(
            "BenchmarkResult(benchmark_id='{}', predictions={}, metrics={}, files={})",
            self.benchmark_id,
            self.results.predictions.len(),
            metrics,
            files
        )
    }
}
