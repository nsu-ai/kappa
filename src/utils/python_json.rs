// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use serde::Serialize;

/// Convert a `serde_json::Value` to a Python object via `json.loads`.
pub fn json_value_to_pyobject(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    let json_str = serde_json::to_string(value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialization error: {}", e))
    })?;
    let parsed = py.import("json")?.call_method1("loads", (json_str,))?;
    Ok(parsed.into())
}

/// Serialize any serde value and convert it to a Python object.
pub fn rust_value_to_pyobject<T: Serialize>(py: Python<'_>, value: &T) -> PyResult<PyObject> {
    let json_value = serde_json::to_value(value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialization error: {}", e))
    })?;
    json_value_to_pyobject(py, &json_value)
}

/// Convert a PyObject (typically a Python dict/list) into serde_json::Value using json.dumps/loads.
pub fn pyobject_to_rust_value(obj: &PyObject, context: &str) -> PyResult<serde_json::Value> {
    Python::with_gil(|py| {
        let json_module = py.import("json")?;
        let dumps = json_module.getattr("dumps")?;
        let json_str_obj = dumps.call1((obj.clone_ref(py),))?;
        let json_str: String = json_str_obj.extract()?;
        serde_json::from_str(&json_str).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse {} JSON: {}",
                context, e
            ))
        })
    })
}
