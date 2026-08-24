// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::benchmarks::benchmarks_api::BenchmarksApi;
use crate::client::KappaApkClient;
use crate::models::benchmarks_model::{
    Benchmark, BenchmarkResult, FileInformation, MetricValue, Prediction, Results,
};
use crate::models::datasets_model::{AnnotationValue, DatasetItem, ItemFile};
use crate::traits::ApiClient;
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
    /// Directory whose files describe the model; also the default artifact upload source.
    model_path: Option<String>,
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
            model_path: None,
        }
    }

    pub fn set_benchmark_id(&mut self, benchmark_id: String) {
        self.benchmark_id = benchmark_id;
    }

    pub fn data(&self) -> Option<Vec<DatasetItem>> {
        self.data.clone()
    }

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
                if !dir.pop() {
                    break;
                }
            }
        }
        None
    }

    /// Select random files from project `src` using FileUtils and set `file_information`.
    fn internal_update_project_file_information(
        &mut self,
        count: Option<usize>,
    ) -> PyResult<Vec<FileInformation>> {
        let src_dir = Self::internal_find_project_src_dir().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Could not locate project src directory",
            )
        })?;

        let file_utils = FileUtils::new(Some(src_dir.to_string_lossy().to_string()));
        let selected = file_utils
            .randomly_select_files(Some(src_dir.clone()), count.unwrap_or(10))
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "File selection failed: {}",
                    e
                ))
            })?;

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
            let endpoint = format!("/model-micro-services/v2/benchmarks/{}", self.benchmark_id);
            client.make_request(
                "GET".to_string(),
                endpoint,
                None,
                Some(client.require_token()?),
            )
        })?;
        let benchmark = Python::with_gil(|py| -> PyResult<Benchmark> {
            let json_module = py.import("json")?;
            let json_str: String = json_module
                .getattr("dumps")?
                .call1((json_obj.clone_ref(py),))?
                .extract()?;
            let value: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to parse benchmarks JSON: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to decode benchmarks: {}",
                    e
                ))
            })
        })?;
        self.benchmark_details = Some(benchmark.clone());
        Ok(benchmark)
    }

    /// Download the benchmark evaluation set and load its items.
    ///
    /// Follows the same order as the web client: the dataset-services package for the
    /// benchmark's `datasetId` / `datasetVersionNo` first, then the benchmark proxy when
    /// only `benchmark.read` is held. Each path uses the legacy single zip when the
    /// manifest reports one. Downloads are cached under the OS cache dir
    /// `kappa-framework/benchmarks/{benchmark_id}/` (legacy `~/cache/…` reused if complete),
    /// so a second call is a no-op.
    pub fn internal_dataset(&mut self, dataset_path: Option<String>) -> PyResult<Vec<DatasetItem>> {
        let benchmark_id = self.benchmark_id.clone();
        // Reuse cached details when we have them so we skip one benchmark GET.
        let coordinates = self
            .benchmark_details
            .as_ref()
            .map(|bd| (bd.dataset_id, bd.dataset_version_no.clone()));

        let data_path = Python::with_gil(|py| -> PyResult<String> {
            let client = self.client.borrow(py);
            let (dataset_id, version_no) = match coordinates {
                Some((id, Some(no))) if id > 0 && !no.is_empty() => (Some(id), Some(no)),
                _ => (None, None),
            };
            BenchmarksApi::download_benchmark_dataset_package(
                &*client,
                &benchmark_id,
                dataset_id,
                version_no,
                dataset_path,
            )
        })?;
        let data_dir = PathBuf::from(data_path);

        // Parse DatasetItems from the extracted directory
        let mut items: Vec<DatasetItem> = Vec::new();
        let entries: Vec<fs::DirEntry> = fs::read_dir(&data_dir)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to read dataset directory: {}",
                    e
                ))
            })?
            .filter_map(|res| res.ok())
            .collect();

        for entry in &entries {
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if !file_type.is_file() {
                continue;
            }
            let file_name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            if !file_name.ends_with("_info.json") {
                continue;
            }

            let entity_id = file_name.trim_end_matches("_info.json").to_string();
            let info_path = entry.path();

            let (annotations, files) = match fs::read_to_string(&info_path) {
                Ok(content) => {
                    let value: serde_json::Value = match serde_json::from_str(&content) {
                        Ok(v) => v,
                        Err(_) => serde_json::Value::Null,
                    };
                    let annotations_opt = value.get("annotations").cloned().and_then(|v| {
                        serde_json::from_value::<
                            Vec<std::collections::HashMap<String, AnnotationValue>>,
                        >(v)
                        .ok()
                    });
                    let files_opt: Option<Vec<ItemFile>> = value
                        .get("files")
                        .cloned()
                        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
                        .map(|names| {
                            names
                                .into_iter()
                                .map(|name| ItemFile {
                                    file_id: entity_id.clone(),
                                    file_name: name.clone(),
                                    file: data_dir.join(&name),
                                })
                                .collect()
                        });
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
                "No dataset items found in directory.",
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
        let read_dir = fs::read_dir(&model_path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to read model path '{}': {}",
                model_path, e
            ))
        })?;
        for entry_res in read_dir {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }
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

            model_files.push(FileInformation {
                file_name,
                file_type,
                file_size,
                file_hash,
            });
        }
        self.model_information = Some(model_files.clone());
        self.model_path = Some(model_path);
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
        if !model_path.is_empty() {
            // Rescan so the result carries this run's model files.
            self.set_model_path(model_path)?;
        }
        // model_information describes the model files, file_information the project
        // sources sampled by setup_project(); either may be absent.
        result.model_information = self.model_information.clone();
        result.file_information = self.file_information.clone();
        self.result = Some(result.clone());
        Ok(result)
    }

    /// The id of the model version built from `inference_id`, if one exists.
    fn version_id_for_inference(&self, model_id: &str, inference_id: i32) -> PyResult<Option<i32>> {
        let versions = Python::with_gil(|py| -> PyResult<Option<PyObject>> {
            let client = self.client.borrow(py);
            match crate::models_api::ModelsApi::list_model_versions(&*client, model_id) {
                Ok(list) => Ok(Some(list)),
                // A model with no versions yet answers 404 NO_RECORD_FOUND.
                Err(e) if e.to_string().contains("404") => Ok(None),
                Err(e) => Err(e),
            }
        })?;
        let Some(versions) = versions else {
            return Ok(None);
        };
        let value = crate::utils::python_json::pyobject_to_rust_value(&versions, "model versions")?;
        Ok(value
            .as_array()
            .and_then(|items| {
                items.iter().find(|item| {
                    item.get("inferenceId").and_then(|v| v.as_i64()) == Some(inference_id as i64)
                })
            })
            .and_then(|item| item.get("id"))
            .and_then(|v| v.as_i64())
            .map(|v| v as i32))
    }

    /// Version the freshly saved inference so the benchmark has something to link to.
    fn create_version_for_inference(&self, model_id: &str, inference_id: i32) -> PyResult<i32> {
        let body = serde_json::json!({
            "inferenceId": inference_id,
            "versionType": "minor",
            "versionAvailability": 1,
            "versionRemark": format!("Benchmark {}", self.benchmark_id),
        })
        .to_string();
        Python::with_gil(|py| -> PyResult<PyObject> {
            let client = self.client.borrow(py);
            crate::models_api::ModelsApi::create_model_version(&*client, model_id, body)
        })
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Inference {} was saved but versioning it failed: {}. Pass model_version_id=… to link an existing version instead.",
                inference_id, e
            ))
        })?;
        // The create route answers with a message, so read the id back from the list.
        self.version_id_for_inference(model_id, inference_id)?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Created a model version for inference {} but the server did not list it.",
                    inference_id
                ))
            })
    }

    pub fn internal_submit_benchmark(
        &mut self,
        strict: bool,
        model_version_id: Option<i32>,
        complete_inference: bool,
        create_version: bool,
        upload_artifacts: bool,
        artifact_paths: Option<Vec<String>>,
        on_progress: Option<PyObject>,
        attach_pipeline: bool,
        pipeline: Option<PyObject>,
        pipeline_type: Option<i32>,
        model: Option<PyObject>,
        entrypoint: Option<String>,
    ) -> PyResult<PyObject> {
        let result = self.result.clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No benchmark result found. Call save_benchmark() first.",
            )
        })?;
        // Benchmarks start life with mlmodelVersionId = 0; only an explicit assignment
        // (web UI or a previous link call) makes it usable as the link target.
        let assigned_version_id = if complete_inference && model_version_id.is_none() {
            self.details()?.model_version_id.filter(|id| *id > 0)
        } else {
            None
        };
        if complete_inference
            && model_version_id.is_none()
            && assigned_version_id.is_none()
            && !create_version
        {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No model version is assigned to this benchmark. Pass model_version_id=…, keep create_version=True to version the submitted inference, or pass complete_inference=False.",
            ));
        }
        let artifact_paths = if upload_artifacts {
            let paths = artifact_paths
                .or_else(|| self.model_path.clone().map(|path| vec![path]))
                .unwrap_or_default();
            if paths.is_empty() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "upload_artifacts=True needs artifact_paths, or a model_path passed to save_benchmark()/set_model_path().",
                ));
            }
            paths
        } else {
            Vec::new()
        };

        let model_id = match self.model_id_override.clone().or_else(|| {
            self.benchmark_details
                .as_ref()
                .and_then(|bd| bd.model_id.clone())
        }) {
            Some(id) => id,
            None => self
                .details()
                .ok()
                .and_then(|bd| bd.model_id)
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>("Model ID not found.")
                })?,
        };

        // The document carries the model it belongs to; only the URL had it before.
        let mut result = result;
        result.model_id = model_id.clone();

        let json_obj = Python::with_gil(|py| -> PyResult<PyObject> {
            let client = self.client.borrow(py);

            let payload = serde_json::json!({ "inferenceResult": result });
            let body = serde_json::to_string(&payload).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize payload: {}",
                    e
                ))
            })?;

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
            let endpoint = format!("/model-micro-services/v2/models/inferences/{}", model_id);
            client.make_request("POST".to_string(), endpoint, Some(body), Some(token))
        })?;

        let created = crate::utils::python_json::pyobject_to_rust_value(
            &json_obj,
            "create inference response",
        )?;
        let created_inference_id = created
            .get("inferenceId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        let detect_paths = if artifact_paths.is_empty() {
            self.model_path.clone().map(|p| vec![p]).unwrap_or_default()
        } else {
            artifact_paths.clone()
        };

        if !artifact_paths.is_empty() {
            let inference_id = created_inference_id.ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Inference was saved but the server returned no inferenceId, so artifacts were not uploaded.",
                )
            })?;
            Python::with_gil(|py| -> PyResult<()> {
                let client = self.client.borrow(py);
                crate::model_artifacts::ModelArtifactsApi::upload_model_artifacts(
                    &*client,
                    &model_id,
                    inference_id,
                    artifact_paths,
                    Some(3),
                    None,
                    true,
                    None,
                    on_progress,
                )
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Inference {} was saved but uploading its artifacts failed: {}",
                        inference_id, e
                    ))
                })?;
                Ok(())
            })?;
        }

        let pipeline_status = if attach_pipeline {
            let inference_id = created_inference_id.unwrap_or(0);
            let project_names: Vec<String> = self
                .file_information
                .as_ref()
                .map(|files| files.iter().map(|f| f.file_name.clone()).collect())
                .unwrap_or_default();
            Python::with_gil(|py| {
                let client = self.client.borrow(py);
                let pipeline_owned = pipeline.as_ref().map(|obj| obj.bind(py));
                let model_owned = model.as_ref().map(|obj| obj.bind(py));
                #[allow(clippy::option_as_ref_deref)]
                {
                    crate::client::attach_inference_pipeline(
                        py,
                        &*client,
                        &model_id,
                        inference_id,
                        &detect_paths,
                        pipeline_owned.as_ref().map(|b| &**b),
                        pipeline_type.unwrap_or(crate::pipeline_detect::PIPELINE_TYPE_BENCHMARK),
                        model_owned.as_ref().map(|b| &**b),
                        entrypoint.as_deref(),
                        &project_names,
                        None,
                    )
                }
            })
        } else {
            crate::pipeline_detect::PipelineDetect::skipped("skipped").to_status_json()
        };

        if complete_inference {
            let version_id = match model_version_id.or(assigned_version_id) {
                Some(id) => id,
                None => {
                    let inference_id = created_inference_id.ok_or_else(|| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Inference was saved but the server returned no inferenceId, so it could not be versioned and linked to the benchmark.",
                        )
                    })?;
                    match self.version_id_for_inference(&model_id, inference_id)? {
                        Some(id) => id,
                        None => self.create_version_for_inference(&model_id, inference_id)?,
                    }
                }
            };
            let benchmark_id = self.benchmark_id.clone();
            Python::with_gil(|py| -> PyResult<()> {
                let client = self.client.borrow(py);
                BenchmarksApi::complete_benchmark_inference(&*client, &benchmark_id, version_id)
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                            "Inference was saved but linking it to benchmark {} failed: {}",
                            benchmark_id, e
                        ))
                    })?;
                Ok(())
            })?;
            // The link moved the benchmark to status 5 and stamped the version on it.
            self.benchmark_details = None;
        }
        let mut created = created;
        if let Some(obj) = created.as_object_mut() {
            obj.insert("pipeline".to_string(), pipeline_status);
        }
        Python::with_gil(|py| crate::utils::python_json::json_value_to_pyobject(py, &created))
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
    pub fn benchmark_id(&self) -> String {
        self.benchmark_id.clone()
    }

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
    pub fn setup_project(&mut self) -> PyResult<()> {
        self.internal_setup_file_markers()
    }

    /// Optionally set/override the model ID used for submission.
    pub fn set_model_id(&mut self, model_id: String) {
        self.model_id_override = Some(model_id);
    }

    /// Get the current model ID (from override or benchmark details)
    pub fn get_model_id(&self) -> Option<String> {
        self.model_id_override.clone().or_else(|| {
            self.benchmark_details
                .as_ref()
                .and_then(|bd| bd.model_id.clone())
        })
    }

    /// Debug method to print benchmark details
    pub fn debug_benchmark_details(&self) -> PyResult<String> {
        if let Some(bd) = &self.benchmark_details {
            Ok(format!(
                "Benchmark ID: {}, Model ID: {:?}, Dataset ID: {}, Dataset Version ID: {}",
                bd.benchmark_id, bd.model_id, bd.dataset_id, bd.dataset_version_id
            ))
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
    pub fn dataset(&mut self, dataset_path: Option<String>) -> PyResult<Vec<DatasetItem>> {
        self.internal_dataset(dataset_path)
    }

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
        model_path: Option<String>,
    ) -> PyResult<BenchmarkResult> {
        // Convert metrics (dict) to HashMap<String, MetricValue>
        let metrics_map: Option<HashMap<String, MetricValue>> = if let Some(metrics_obj) = metrics {
            Some(Python::with_gil(
                |py| -> PyResult<HashMap<String, MetricValue>> {
                    let json_module = py.import("json")?;
                    let dumps = json_module.getattr("dumps")?;
                    let json_str_obj = dumps.call1((metrics_obj.clone_ref(py),))?;
                    let json_str: String = json_str_obj.extract()?;
                    let value: serde_json::Value =
                        serde_json::from_str(&json_str).map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                "Failed to parse metrics JSON: {}",
                                e
                            ))
                        })?;
                    let map: HashMap<String, MetricValue> =
                        serde_json::from_value(value).map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                "Failed to decode metrics: {}",
                                e
                            ))
                        })?;
                    Ok(map)
                },
            )?)
        } else {
            None
        };

        // Convert predictions (list of dict) to Vec<Prediction>
        let predictions_vec: Vec<Prediction> =
            Python::with_gil(|py| -> PyResult<Vec<Prediction>> {
                let json_module = py.import("json")?;
                let dumps = json_module.getattr("dumps")?;
                let json_str_obj = dumps.call1((predictions.clone_ref(py),))?;
                let json_str: String = json_str_obj.extract()?;
                serde_json::from_str::<Vec<Prediction>>(&json_str).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to decode predictions: {}",
                        e
                    ))
                })
            })?;

        self.internal_save_benchmark(predictions_vec, metrics_map, model_path)
    }

    /// The result built by the last :meth:`save_benchmark` call, if any.
    #[getter]
    pub fn saved_result(&self) -> Option<BenchmarkResult> {
        self.result.clone()
    }

    /// Submit the saved benchmark result as a model inference.
    ///
    /// When ``strict=True`` (default), validates against the model inference schema first.
    ///
    /// With ``complete_inference=True`` (default) the saved inference is then linked to the
    /// benchmark, which moves it from *Pending Inference* to *Inference Completed*. The
    /// model version comes from ``model_version_id``, else the benchmark's
    /// ``mlmodelVersionId`` when one was assigned, else the version holding the inference
    /// just saved — which ``create_version=True`` (default) creates when it is missing.
    ///
    /// ``upload_artifacts=True`` also uploads the model files themselves to the new
    /// inference — ``artifact_paths`` when given, otherwise the ``model_path`` from
    /// :meth:`save_benchmark`. Weight files past the server's sync cap take a resumable
    /// multipart session, so multi-GB checkpoints work here. ``on_progress`` receives
    /// ``(file_name, bytes_sent, total_bytes, percent)``.
    ///
    /// After artifacts are uploaded, a pipeline draft is auto-detected from the running
    /// program and PUT on the inference **before** the version is created (Kappa ≥ 2.14;
    /// missing route is skipped). Pass ``attach_pipeline=False`` to skip, or ``pipeline=``
    /// with an explicit body.
    ///
    /// # Python Example
    /// ```python
    /// benchmark.save_benchmark(predictions, metrics, model_path="./model")
    /// benchmark.submit_benchmark(upload_artifacts=True)
    /// ```
    #[pyo3(signature = (
        strict=true,
        model_version_id=None,
        complete_inference=true,
        create_version=true,
        upload_artifacts=false,
        artifact_paths=None,
        on_progress=None,
        attach_pipeline=true,
        pipeline=None,
        pipeline_type=None,
        model=None,
        entrypoint=None
    ))]
    pub fn submit_benchmark(
        &mut self,
        strict: bool,
        model_version_id: Option<i32>,
        complete_inference: bool,
        create_version: bool,
        upload_artifacts: bool,
        artifact_paths: Option<Vec<String>>,
        on_progress: Option<PyObject>,
        attach_pipeline: bool,
        pipeline: Option<PyObject>,
        pipeline_type: Option<i32>,
        model: Option<PyObject>,
        entrypoint: Option<String>,
    ) -> PyResult<PyObject> {
        self.internal_submit_benchmark(
            strict,
            model_version_id,
            complete_inference,
            create_version,
            upload_artifacts,
            artifact_paths,
            on_progress,
            attach_pipeline,
            pipeline,
            pipeline_type,
            model,
            entrypoint,
        )
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
        assert!(validate_save_payload(&[sample_prediction()], None, false, false,).is_ok());
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
