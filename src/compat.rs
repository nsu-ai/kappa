// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! SDK ↔ Kappa-framework version compatibility helpers.

use crate::upload_limits::{
    BULK_ARCHIVE_MAX_BYTES, BULK_CSV_BE_DEFAULT_BYTES, BULK_CSV_MAX_BYTES, ENTITY_FILE_MAX_BYTES,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Minimum Kappa-framework product version required by this SDK release.
pub const MIN_BACKEND_VERSION: &str = "2.11.0";

/// Notes shown to users about why 2.11.0+ is required.
pub const COMPAT_NOTES: &[&str] = &[
    "Bulk mutations (self-verify, auto-verify, mark-labeled, delete/recover/files): async 202 + job poll",
    "Version create/refresh: archive build jobs; wait until buildStatus=ready before publish/download",
    "Version package: sharded manifest + shard download (legacy single-zip archive still available)",
    "Bulk upload: staging/retry, archiveLayout, admission (429), large zip (up to 50 GB)",
    "Entity files: file_category input|output; single-file max 2 GB",
    "CSV bulk: client allows up to 2 GB; backend default BULK_UPLOAD_MAX_CSV_BYTES is 50 MB",
    "Archive dataset_schema: input_output needs inputDataPath; classes needs classes[]",
    "Tabular custom-schema + entity split; Dataset Admin / dataset.delete",
    "Model inference artifacts + file_category 1–5",
    "ML tags: first tag must be predefined for dataset_type/model_type (dataset_tags_{id})",
];

/// Return the minimum supported Kappa-framework version (`"2.11.0"`).
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
    d.set_item("bulk_csv_max_bytes", BULK_CSV_MAX_BYTES)?;
    d.set_item("bulk_csv_be_default_bytes", BULK_CSV_BE_DEFAULT_BYTES)?;
    d.set_item("entity_file_max_bytes", ENTITY_FILE_MAX_BYTES)?;
    d.set_item("bulk_archive_max_bytes", BULK_ARCHIVE_MAX_BYTES)?;
    let notes: Vec<&str> = COMPAT_NOTES.to_vec();
    d.set_item("notes", notes)?;
    Ok(d.into())
}
