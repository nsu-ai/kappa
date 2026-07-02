// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use pyo3::PyErr;
use pyo3::PyResult;
// use pyo3::Python;
use reqwest::Client;
use std::path::Path;
use walkdir::WalkDir;

use crate::models::benchmarks_model::FileInformation;
use crate::utils::git_utils;

#[pyclass]
pub struct BenchmarkVerification {
    server_url: String,
    benchmark_id: String,
    path: String,
}

impl BenchmarkVerification {
    /// Create a new Verifications object
    /// 
    /// # Parameters
    /// 
    /// * `benchmark_id` - The ID of the benchmark
    /// * `path` - The local directory path to scan for files
    pub fn new(server_url: String, benchmark_id: String, path: String) -> Self {
        Self { server_url, path, benchmark_id }
    }

    fn get_files_url(&self) -> String {
        format!(
            "{}/model-micro-services/v2/benchmarks/app/verifications/files/{}",
            self.server_url, self.benchmark_id
        )
    }

    fn validate_url(&self) -> String {
        format!(
            "{}/model-micro-services/v2/benchmarks/app/verifications/{}",
            self.server_url, self.benchmark_id
        )
    }

    // fn to_pyobject_from_value(value: &serde_json::Value) -> PyResult<PyObject> {
    //     let s = serde_json::to_string(value)
    //         .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize JSON: {}", e)))?;
    //     Python::with_gil(|py| {
    //         let json = py.import("json")?;
    //         let loads = json.getattr("loads")?;
    //         let py_obj = loads.call1((s,))?;
    //         Ok(py_obj.into())
    //     })
    // }

    fn fetch_required_files(&self) -> PyResult<Vec<String>> {
        let client = Client::new();
        let url = self.get_files_url();

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;

        let files = rt.block_on(async move {
            let resp = client
                .get(url)
                .header("accept", "application/json")
                .send()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Request failed: {}", e)))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_else(|_| "".to_string());
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("HTTP {}: {}", status, text)));
            }
            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))?;
            
            // Convert JSON to Vec<String> (list of file names)
            serde_json::from_value(json)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse benchmark files: {}", e)))
        })?;

        Ok(files)
    }

    fn walk_and_hash_files(root: &str, benchmark_files: &[String]) -> PyResult<Vec<FileInformation>> {
        let root = Path::new(root);
        let mut files: Vec<FileInformation> = Vec::new();
        
        // Create a set of required file names for quick lookup
        let required_files: std::collections::HashSet<String> = benchmark_files
            .iter()
            .map(|f| f.clone())
            .collect();
        
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                
                // Only process files that are in the benchmark requirements
                if required_files.contains(&file_name) {
                    let file_type = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
                    let file_hash = git_utils::git_hash_object(path).unwrap_or_default();
                    files.push(FileInformation { file_name, file_type, file_size, file_hash });
                }
            }
        }
        Ok(files)
    }
}

#[pymethods]
impl BenchmarkVerification {
    /// Create a new BenchmarkVerification object
    /// 
    /// # Parameters
    /// 
    /// * `server_url` - The URL of the kappa-framework server
    /// * `benchmark_id` - The ID of the benchmark
    /// * `path` - The local directory path to scan for files
    /// 
    /// # Python Example
    /// 
    /// ```python
    /// import kappa_apk
    /// benchmark_verification = kappa_apk.BenchmarkVerification(
    ///     "http://172.16.71.146:8060", "8c97da09-375c-47cd-814c-d1798dbd48f4", "path/to/model_or_application"
    /// )
    /// ```
    #[new]
    pub fn py_new(server_url: String, benchmark_id: String, path: String) -> Self { Self::new(server_url, benchmark_id, path) }

    /// Get verification result for the current benchmark
    /// 
    /// # Parameters
    /// 
    /// None
    /// 
    /// # Returns
    /// 
    /// True if server validates benchmark successfully, otherwise False.
    /// 
    /// # Python Example
    /// 
    /// ```python
    /// import kappa_apk
    /// benchmark_verification = kappa_apk.BenchmarkVerification(
    ///     "http://172.16.71.146:8060", "8c97da09-375c-47cd-814c-d1798dbd48f4", "path/to/model_or_application"
    /// )
    /// result = benchmark_verification.result()
    /// print(result)
    /// ```
    pub fn result(&self) -> PyResult<bool> {
        let benchmark_files = self.fetch_required_files()?;
        if benchmark_files.is_empty() {
            return Ok::<bool, PyErr>(false);
        }
        let file_infos = Self::walk_and_hash_files(&self.path , &benchmark_files)?;
        if file_infos.is_empty() {
            return Ok::<bool, PyErr>(false);
        }
        let client = Client::new();
        let url = self.validate_url();
        let body = serde_json::to_string(&file_infos)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize files: {}", e)))?;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;

        let ok = rt.block_on(async move {
            let resp = client
                .post(url)
                .header("accept", "application/json")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Request failed: {}", e)))?;
            if !resp.status().is_success() {
                return Ok::<bool, PyErr>(false);
            }
            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e)))?;
            Ok::<bool, PyErr>(parse_verification_verdict(&json))
        })?;

        Ok(ok)
    }
}

fn parse_verification_verdict(json: &serde_json::Value) -> bool {
    json.get("valid")
        .and_then(|v| v.as_bool())
        .or_else(|| json.get("success").and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

#[cfg(test)]
mod verification_tests {
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
    fn parse_success_verdict_from_fixture() {
        let json = load_fixture("benchmark_verification_response.json");
        assert!(parse_verification_verdict(&json));
    }

    #[test]
    fn parse_verdict_defaults_to_false_when_missing() {
        assert!(!parse_verification_verdict(&serde_json::json!({})));
    }

    #[test]
    fn load_required_files_fixture_is_array() {
        let json = load_fixture("benchmark_verification_files.json");
        let files: Vec<String> = serde_json::from_value(json).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"main.py".to_string()));
    }
}