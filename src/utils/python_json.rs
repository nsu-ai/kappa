// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;

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

