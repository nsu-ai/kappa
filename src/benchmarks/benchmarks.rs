// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use std::fs;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use crate::traits::ApiClient;
use crate::client::KappaApkClient;
use crate::utils::zip_utils::{self, cache_is_complete};
use crate::models::benchmarks_model::{
    Benchmark,
    BenchmarkResult,
    FileInformation,
    Prediction,
    Results,
    MetricValue,
};
use crate::models::datasets_model::{DatasetItem, ItemFile, AnnotationValue};
use crate::utils::file_utils::FileUtils;
use crate::utils::git_utils;

/// Benchmarks client with optional default benchmark_id
#[pyclass]
pub struct Benchmarks {
    benchmark_id: String,
    client: Py<KappaApkClient>,
    benchmark_details: Option<Benchmark>,
    data: Option<Vec<DatasetItem>>,
    result: Option<BenchmarkResult>,
    model_information: Option<Vec<FileInformation>>,
    file_information: Option<Vec<FileInformation>>,
    model_id_override: Option<String>,
}

impl Benchmarks {

    /// Create a new Benchmarks instance
    /// 
    /// # Parameters
    /// 
    /// * `benchmark_id` - The ID of the benchmark to use
    /// 
    /// # Returns
    /// 
    /// A new `Benchmarks` instance
    pub fn new(benchmark_id: String, client: Py<KappaApkClient>) -> Self {
        Self {
            benchmark_id,
            client,
            benchmark_details: None,
            data: None,
            result: None,
            model_information: None,
            file_information: None,
            model_id_override: None,
        }
    }

    pub fn set_benchmark_id(&mut self, benchmark_id: String) { self.benchmark_id = benchmark_id; }

    pub fn data(&self) -> Option<Vec<DatasetItem>> { self.data.clone() }

    /// Setup file markers for AI/ML application benchmarks.
    fn internal_setup_file_markers(&mut self) -> PyResult<()> {
        self.internal_update_project_file_information(Some(10))?;
        Ok(())
    }

    /// Find the nearest `src` directory from current working directory by walking up.
    fn internal_find_project_src_dir() -> Option<PathBuf> {
        if let Ok(mut dir) = env::current_dir() {
            loop {
                let candidate = dir.join("src");
                if candidate.is_dir() {
                    return Some(candidate);
                }
                if !dir.pop() { break; }
            }
        }
        None
    }

    /// Select random files from project `src` using FileUtils and set `file_information`.
    fn internal_update_project_file_information(&mut self, count: Option<usize>) -> PyResult<Vec<FileInformation>> {
        let src_dir = Self::internal_find_project_src_dir()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Could not locate project src directory"))?;

        let file_utils = FileUtils::new(Some(src_dir.to_string_lossy().to_string()));
        let selected = file_utils
            .randomly_select_files(Some(src_dir.clone()), count.unwrap_or(10))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("File selection failed: {}", e)))?;

        let mut infos: Vec<FileInformation> = Vec::new();
        for pf in selected {
            let path = pf.path.clone();
            let file_name = pf.name.clone();
            let file_type = pf.file_type.clone();
            let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
            let file_hash = match git_utils::git_hash_object(&path) {
                Ok(h) => h,
                Err(_) => String::new(),
            };
            let info = FileInformation {
                file_name,
                file_type,
                file_size,
                file_hash,
            };
            infos.push(info);
        }

        self.file_information = Some(infos.clone());
        if let Some(result) = self.result.as_mut() {
            result.file_information = Some(infos.clone());
        }
        Ok(infos)
    }

    /// Get benchmark details
    /// 
    /// Returns a Python dict mirroring server JSON
    /// 
    /// # Python Example
    /// ```python
    /// benchmark = client.get_benchmark_details('eaa50325-5f3d-4e66-b7b5-b18e5a587563')
    /// print(benchmark.benchmark_id, benchmark.benchmark_description)
    /// ```
    pub fn details(&mut self) -> PyResult<Benchmark> {
        if let Some(bd) = &self.benchmark_details {
            return Ok(bd.clone());
        }
        let json_obj = Python::with_gil(|py| -> PyResult<PyObject> {
            let client = self.client.borrow(py);
            let endpoint = format!(
                "/model-micro-services/v2/benchmarks/{}",
                self.benchmark_id
            );
            client.make_request("GET".to_string(), endpoint, None, Some(client.require_token()?))
        })?;
        let benchmark = Python::with_gil(|py| -> PyResult<Benchmark> {
            let json_module = py.import("json")?;
            let json_str: String = json_module.getattr("dumps")?.call1((json_obj.clone_ref(py),))?.extract()?;
            let value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to parse benchmarks JSON: {}", e),
                ))?;
            serde_json::from_value(value).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to decode benchmarks: {}", e))
            })
        })?;
        self.benchmark_details = Some(benchmark.clone());
        Ok(benchmark)
    }

    /// Download benchmark dataset archive via the benchmark controller endpoint and load items.
    ///
    /// Uses `GET /model-micro-services/v2/benchmarks/datasets/download/{benchmark_id}`.
    /// Archives are cached under `~/cache/kappa-framework/benchmarks/{benchmark_id}/`;
    /// a second call with the same benchmark skips the download entirely.
    pub fn internal_dataset(
        &mut self,
        dataset_path: Option<String>,
    ) -> PyResult<Vec<DatasetItem>> {
        let dataset_path_str = dataset_path.unwrap_or_default();
        let benchmark_id = self.benchmark_id.clone();

        let data_dir: PathBuf = if dataset_path_str.is_empty() {
            let home_dir = dirs::home_dir().ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Could not determine home directory")
            })?;
            home_dir
                .join("cache")
                .join("kappa-framework")
                .join("benchmarks")
                .join(&benchmark_id)
        } else {
            PathBuf::from(&dataset_path_str).join(&benchmark_id)
        };

        if !cache_is_complete(&data_dir) {
            let (base_url, http_client, runtime, token) = Python::with_gil(|py| -> PyResult<_> {
                let client = self.client.borrow(py);
                let base_url = client.get_base_url();
                let http_client = client.get_http_client();
                let runtime = client.get_runtime();
                let token = client.require_token()?;
                Ok((base_url, http_client, runtime, token))
            })?;

            let url = format!(
                "{}/model-micro-services/v2/benchmarks/datasets/download/{}",
                base_url, benchmark_id
            );
            let data_dir_dl = data_dir.clone();

            runtime.block_on(async move {
                zip_utils::download_and_extract_zip(&http_client, &url, Some(&token), &data_dir_dl)
                    .await
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(e)
                    })
            })?;
        }

        // Parse DatasetItems from the extracted directory
        let mut items: Vec<DatasetItem> = Vec::new();
        let entries: Vec<fs::DirEntry> = fs::read_dir(&data_dir)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to read dataset directory: {}", e
            )))?
            .filter_map(|res| res.ok())
            .collect();

        for entry in &entries {
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if !file_type.is_file() { continue; }
            let file_name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            if !file_name.ends_with("_info.json") { continue; }

            let entity_id = file_name.trim_end_matches("_info.json").to_string();
            let info_path = entry.path();

            let (annotations, files) = match fs::read_to_string(&info_path) {
                Ok(content) => {
                    let value: serde_json::Value = match serde_json::from_str(&content) {
                        Ok(v) => v,
                        Err(_) => serde_json::Value::Null,
                    };
                    let annotations_opt = value
                        .get("annotations")
                        .cloned()
                        .and_then(|v| serde_json::from_value::<Vec<std::collections::HashMap<String, AnnotationValue>>>(v).ok());
                    let files_opt: Option<Vec<ItemFile>> = value
                        .get("files")
                        .cloned()
                        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
                        .map(|names| names.into_iter().map(|name| ItemFile {
                            file_id: entity_id.clone(),
                            file_name: name.clone(),
                            file: data_dir.join(&name),
                        }).collect());
                    (annotations_opt, files_opt)
                }
                Err(_) => (None, None),
            };

            items.push(DatasetItem {
                entity_id,
                files,
                annotations,
                entity_info: None,
            });
        }

        if items.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No dataset items found in directory."
            ));
        }

        self.data = Some(items.clone());
        Ok(items)
    }

    pub fn set_model_path(&mut self, model_path: String) -> PyResult<()> { 
        self.internal_update_model_files(model_path)?;
        Ok(())
    }

    fn internal_update_model_files(&mut self, model_path: String) -> PyResult<()> {
        let mut model_files: Vec<FileInformation> = Vec::new();
        let read_dir = fs::read_dir(&model_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to read model path '{}': {}",
                model_path, e
            )))?;
        for entry_res in read_dir {
            let entry = match entry_res { Ok(e) => e, Err(_) => continue };
            let file_path = entry.path();
            if !file_path.is_file() { continue; }
            let file_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let file_type = file_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let file_size = file_path.metadata().map(|m| m.len()).unwrap_or(0);
            let file_hash = git_utils::git_hash_object(&file_path).unwrap_or_default();

            model_files.push(FileInformation { file_name, file_type, file_size, file_hash });
        }
        self.model_information = Some(model_files.clone());
        if let Some(result) = self.result.as_mut() {
            result.model_information = Some(model_files);
        }
        Ok(())
    }

    pub fn internal_save_benchmark(
        &mut self,
        predictions: Vec<Prediction>,
        metrics: Option<HashMap<String, MetricValue>>,
        model_path: Option<String>,
    ) -> PyResult<BenchmarkResult> {
        validate_save_payload(
            &predictions,
            model_path.as_deref(),
            self.model_information.is_some(),
            self.file_information.is_some(),
        )?;

        let mut result: BenchmarkResult = BenchmarkResult {
            benchmark_id: self.benchmark_id.clone(),
            model_id: String::new(),
            model_information: None,
            file_information: None,
            results: Results {
                metrics: Some(metrics.unwrap_or(HashMap::new())),
                predictions,
            },
        };

        let model_path = model_path.unwrap_or_default();
        if model_path.is_empty() && self.model_information.is_some() {
            result.model_information = self.model_information.clone();
        } else if !model_path.is_empty() {
            self.set_model_path(model_path)?;
            result.file_information = self.file_information.clone();
        } else if self.file_information.is_some() {
            result.file_information = self.file_information.clone();
        }
        // else: predictions-only result (no model/application file metadata attached)
        self.result = Some(result.clone());
        Ok(result)
    }

    pub fn internal_submit_benchmark(
        &mut self,
        strict: bool,
    ) -> PyResult<PyObject> {
        let result = self.result.clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("No benchmark result found. Call save_benchmark() first.")
        })?;
        let json_obj = Python::with_gil(|py| -> PyResult<PyObject> {
            let client = self.client.borrow(py);

            let model_id = self.model_id_override.clone()
                .or_else(|| self.benchmark_details.as_ref().and_then(|bd| bd.model_id.clone()))
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>("Model ID not found.")
                })?;

            let payload = serde_json::json!({ "inferenceResult": result });
            let body = serde_json::to_string(&payload)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize payload: {}", e)))?;

            if strict {
                let result_only = serde_json::to_string(&result).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to serialize inference result: {}",
                        e
                    ))
                })?;
                let validation = crate::models_api::ModelsApi::validate_inference_result(
                    &*client,
                    &model_id,
                    result_only,
                )?;
                let valid = validation
                    .bind(py)
                    .call_method1("get", ("valid",))
                    .ok()
                    .and_then(|v| v.extract::<bool>().ok())
                    .unwrap_or(true);
                if !valid {
                    let errors = validation
                        .bind(py)
                        .call_method1("get", ("errors",))
                        .ok()
                        .map(|e| format!("{:?}", e))
                        .unwrap_or_else(|| "schema validation failed".to_string());
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Inference schema validation failed: {}",
                        errors
                    )));
                }
            }

            let token = client.require_token()?;
            let endpoint = format!(
                "/model-micro-services/v2/models/inferences/{}",
                model_id
            );
            client.make_request("POST".to_string(), endpoint, Some(body), Some(token))
        })?;
        Ok(json_obj)
    }
}

#[pymethods]
impl Benchmarks {
    #[getter]
    /// Get the benchmark ID
    /// 
    /// # Python Example
    /// ```python
    /// benchmark_id = benchmark.benchmark_id
    /// print(benchmark_id)
    /// ```
    pub fn benchmark_id(&self) -> String { self.benchmark_id.clone() }

    /// Setup file markers
    /// 
    /// # Parameters
    /// 
    /// * `client` - The client to use
    /// 
    /// # Returns
    /// 
    /// None
    /// ```python
    /// benchmark.setup_project()
    /// ```
    pub fn setup_project(&mut self) -> PyResult<()> { self.internal_setup_file_markers() }

    /// Optionally set/override the model ID used for submission.
    pub fn set_model_id(&mut self, model_id: String) { self.model_id_override = Some(model_id); }

    /// Get the current model ID (from override or benchmark details)
    pub fn get_model_id(&self) -> Option<String> {
        self.model_id_override.clone()
            .or_else(|| self.benchmark_details.as_ref().and_then(|bd| bd.model_id.clone()))
    }

    /// Debug method to print benchmark details
    pub fn debug_benchmark_details(&self) -> PyResult<String> {
        if let Some(bd) = &self.benchmark_details {
            Ok(format!("Benchmark ID: {}, Model ID: {:?}, Dataset ID: {}, Dataset Version ID: {}", 
                bd.benchmark_id, bd.model_id, bd.dataset_id, bd.dataset_version_id))
        } else {
            Ok("No benchmark details loaded".to_string())
        }
    }

    /// Load dataset
    /// 
    /// # Parameters
    /// 
    /// * `dataset_path` - The path to the dataset
    /// 
    /// # Returns
    /// 
    /// A list of dataset items
    /// ```python
    /// data = benchmark.dataset()
    /// ```
    /// 
    /// ```python
    /// data = benchmark.dataset("/path/to/dataset")
    /// ```
    #[pyo3(signature = (dataset_path=None))]
    pub fn dataset(&mut self, dataset_path: Option<String>) -> PyResult<Vec<DatasetItem>> { self.internal_dataset(dataset_path) }

    /// Save benchmark
    /// 
    /// # Parameters
    /// 
    /// * `predictions` - The predictions
    /// * `metrics` - The metrics
    /// * `model_path` - The path to the model not required if model information is already set
    /// 
    /// # Returns
    /// 
    /// A benchmark result
    /// ```python
    /// benchmark.save_benchmark(predictions, metrics, model_path)
    /// ```
    #[pyo3(signature = (predictions, metrics=None, model_path=None))]
    pub fn save_benchmark(
        &mut self,
        predictions: PyObject,
        metrics: Option<PyObject>,
        model_path: Option<String>
    ) -> PyResult<BenchmarkResult> {
        // Convert metrics (dict) to HashMap<String, MetricValue>
        let metrics_map: Option<HashMap<String, MetricValue>> = if let Some(metrics_obj) = metrics {
            Some(Python::with_gil(|py| -> PyResult<HashMap<String, MetricValue>> {
                let json_module = py.import("json")?;
                let dumps = json_module.getattr("dumps")?;
                let json_str_obj = dumps.call1((metrics_obj.clone_ref(py),))?;
                let json_str: String = json_str_obj.extract()?;
                let value: serde_json::Value = serde_json::from_str(&json_str)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse metrics JSON: {}", e)))?;
                let map: HashMap<String, MetricValue> = serde_json::from_value(value)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to decode metrics: {}", e)))?;
                Ok(map)
            })?)
        } else { None };

        // Convert predictions (list of dict) to Vec<Prediction>
        let predictions_vec: Vec<Prediction> = Python::with_gil(|py| -> PyResult<Vec<Prediction>> {
            let json_module = py.import("json")?;
            let dumps = json_module.getattr("dumps")?;
            let json_str_obj = dumps.call1((predictions.clone_ref(py),))?;
            let json_str: String = json_str_obj.extract()?;
            serde_json::from_str::<Vec<Prediction>>(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to decode predictions: {}", e)))
        })?;

        self.internal_save_benchmark(predictions_vec, metrics_map, model_path)
    }

    /// Submit benchmark
    /// 
    /// # Parameters
    /// 
    /// # Returns
    /// 
    /// A benchmark result
    /// ```python
    /// Submit the saved benchmark result as a model inference.
    ///
    /// When ``strict=True`` (default), validates against the model inference schema first.
    #[pyo3(signature = (strict=true))]
    pub fn submit_benchmark(&mut self, strict: bool) -> PyResult<PyObject> {
        self.internal_submit_benchmark(strict)
    }
}

fn validate_save_payload(
    predictions: &[Prediction],
    model_path: Option<&str>,
    has_model_information: bool,
    has_file_information: bool,
) -> PyResult<()> {
    if predictions.is_empty()
        && model_path.is_none_or(|p| p.is_empty())
        && !has_model_information
        && !has_file_information
    {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "save_benchmark requires predictions and either setup_project(), model_path, or prior model metadata.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod save_tests {
    use super::*;

    fn sample_prediction() -> Prediction {
        Prediction {
            entity_id: "entity-1".to_string(),
            original: Some(serde_json::json!(0)),
            predicted: Some(serde_json::json!(1)),
        }
    }

    #[test]
    fn validate_accepts_predictions_only() {
        assert!(validate_save_payload(
            &[sample_prediction()],
            None,
            false,
            false,
        )
        .is_ok());
    }

    #[test]
    fn validate_rejects_empty_payload() {
        assert!(validate_save_payload(&[], None, false, false).is_err());
    }

    #[test]
    fn validate_accepts_model_path_without_predictions() {
        assert!(validate_save_payload(&[], Some("/models/run1"), false, false).is_ok());
    }
}