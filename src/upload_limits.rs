// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Client-side upload size / layout preflight (FE-aligned, Kappa ≥ 2.10.0).

use pyo3::prelude::*;
use std::path::Path;

/// Bulk CSV client preflight maximum (bytes).
/// Backend default is `BULK_UPLOAD_MAX_CSV_BYTES` = 50 MiB; raise that env for larger CSVs.
/// Client allows up to 2 GiB so oversized files fail early only when clearly beyond practical limits.
pub const BULK_CSV_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

/// Backend default CSV cap (document / warn threshold).
pub const BULK_CSV_BE_DEFAULT_BYTES: u64 = 50 * 1024 * 1024; // 50 MiB

/// Single entity file maximum (bytes).
pub const ENTITY_FILE_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

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

/// Validate archive `dataset_schema` for the chosen layout (FE / BE parity).
pub fn validate_archive_dataset_schema(
    layout: &str,
    schema: Option<&serde_json::Value>,
) -> PyResult<()> {
    let Some(schema) = schema else {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "dataset_schema is required for archive_layout='{}' \
             (input_output needs inputDataPath; classes needs a non-empty classes list)",
            layout
        )));
    };
    let obj = schema.as_object().ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "dataset_schema must be a JSON object",
        )
    })?;
    match layout {
        "input_output" => {
            let path = obj
                .get("inputDataPath")
                .or_else(|| obj.get("input_data_path"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .unwrap_or("");
            if path.is_empty() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "archive_layout='input_output' requires dataset_schema.inputDataPath \
                     (optional outputDataPath)",
                ));
            }
        }
        "classes" => {
            let classes = obj
                .get("classes")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "archive_layout='classes' requires dataset_schema.classes (non-empty list)",
                    )
                })?;
            if classes.is_empty() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "archive_layout='classes' requires a non-empty dataset_schema.classes list",
                ));
            }
        }
        _ => {}
    }
    Ok(())
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

#[cfg(test)]
mod upload_limits_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn archive_schema_requires_input_data_path() {
        assert!(validate_archive_dataset_schema("input_output", None).is_err());
        assert!(validate_archive_dataset_schema("input_output", Some(&json!({}))).is_err());
        assert!(validate_archive_dataset_schema(
            "input_output",
            Some(&json!({"inputDataPath": "input"}))
        )
        .is_ok());
    }

    #[test]
    fn archive_schema_requires_non_empty_classes() {
        assert!(validate_archive_dataset_schema("classes", Some(&json!({}))).is_err());
        assert!(validate_archive_dataset_schema(
            "classes",
            Some(&json!({"classes": []}))
        )
        .is_err());
        assert!(validate_archive_dataset_schema(
            "classes",
            Some(&json!({"classes": [{"className": "cat", "path": "cat"}]}))
        )
        .is_ok());
    }
}

