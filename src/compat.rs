// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! SDK ↔ Kappa-framework version compatibility helpers.

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Minimum Kappa-framework product version required by this SDK release.
pub const MIN_BACKEND_VERSION: &str = "2.10.0";

/// Notes shown to users about why 2.10.0+ is required.
pub const COMPAT_NOTES: &[&str] = &[
    "Bulk upload: staging/retry, archiveLayout, admission (429), large zip (up to 50 GB)",
    "Entity files: file_category input|output; single-file max 2 GB",
    "CSV bulk client limit: 2 GB",
    "Tabular custom-schema + entity split; Dataset Admin / dataset.delete",
    "Model inference artifacts + file_category 1–5",
];

/// Return the minimum supported Kappa-framework version (`"2.10.0"`).
#[pyfunction]
pub fn min_backend_version() -> &'static str {
    MIN_BACKEND_VERSION
}

/// Return `{ sdk_version, min_backend_version, notes }` for scripts and CI.
#[pyfunction]
pub fn compatibility_info(py: Python<'_>) -> PyResult<PyObject> {
    let d = PyDict::new(py);
    d.set_item("sdk_version", env!("CARGO_PKG_VERSION"))?;
    d.set_item("min_backend_version", MIN_BACKEND_VERSION)?;
    let notes: Vec<&str> = COMPAT_NOTES.to_vec();
    d.set_item("notes", notes)?;
    Ok(d.into())
}
