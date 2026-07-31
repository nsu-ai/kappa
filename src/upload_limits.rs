// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Client-side upload size / layout preflight (FE-aligned, Kappa ≥ 2.10.0).

use pyo3::prelude::*;
use std::path::Path;

/// Single entity file and CSV bulk maximum (bytes).
pub const ENTITY_OR_CSV_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

/// Bulk archive (`.zip` for `input_output` / `classes`) maximum (bytes).
pub const BULK_ARCHIVE_MAX_BYTES: u64 = 50 * 1024 * 1024 * 1024; // 50 GiB

pub fn format_bytes(n: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    if n as f64 >= GIB {
        format!("{:.0} GB", n as f64 / GIB)
    } else if n >= 1024 * 1024 {
        format!("{:.0} MB", n as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} bytes", n)
    }
}

pub fn normalize_archive_layout(layout: &str) -> PyResult<String> {
    let n = layout.trim().to_ascii_lowercase();
    match n.as_str() {
        "input_output" | "classes" => Ok(n),
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "archive_layout must be 'input_output' or 'classes'",
        )),
    }
}

pub fn normalize_bulk_split(split: &str) -> PyResult<String> {
    let n = split.trim().to_string();
    if n.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "bulk_split must be a non-empty string (train, validation, or test)",
        ));
    }
    Ok(n)
}

pub fn validate_source(source: &str) -> PyResult<()> {
    if source.trim().len() < 3 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "source must be at least 3 characters",
        ));
    }
    Ok(())
}

/// Ensure `path` is a non-empty file within `max_bytes`.
pub fn validate_upload_file(path: &Path, max_bytes: u64, kind: &str) -> PyResult<u64> {
    let meta = std::fs::metadata(path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{} file not found or unreadable ({}): {}",
            kind,
            path.display(),
            e
        ))
    })?;
    if !meta.is_file() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{} path is not a file: {}",
            kind,
            path.display()
        )));
    }
    let size = meta.len();
    if size == 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{} file is empty: {}",
            kind,
            path.display()
        )));
    }
    if size > max_bytes {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{} exceeds maximum upload size ({} > {}). Max for {} is {}.",
            path.display(),
            format_bytes(size),
            format_bytes(max_bytes),
            kind,
            format_bytes(max_bytes),
        )));
    }
    Ok(size)
}
