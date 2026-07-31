// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! ML tag helpers aligned with Kappa-framework dataset/model create rules.
//!
//! Backend (`validate_dataset_tags`): at least one tag must resolve to
//! `ml_tags_config_fields_info` after normalizing display labels
//! (`"Image Classification"` → `image_classification`). Custom tags are allowed.
//!
//! Label Studio / CVAT pick the **first predefined** tag in user order
//! (`resolve_primary_ml_tag_display`). The React create dialog loads the catalog from
//! `GET /user-micro-services/v2/system/config/dataset_tags_{typeId}` (same list for models).
//! Put that predefined display value **first**, then any custom tags.

use pyo3::prelude::*;
use pyo3::types::PyTuple;

fn normalize_tag_key(tag: &str) -> String {
    let mut out = String::new();
    let mut prev_us = false;
    for c in tag.trim().chars() {
        let lower = c.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_us = false;
        } else if !out.is_empty() && !prev_us {
            out.push('_');
            prev_us = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

fn split_tags(tags: &str) -> Vec<String> {
    tags.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// Join a required primary (predefined) ML tag with optional custom tags.
///
/// The primary tag is always first. Duplicates (case/normalize-insensitive) are dropped.
#[pyfunction]
#[pyo3(signature = (primary_ml_tag, *extras))]
pub fn join_ml_tags(primary_ml_tag: String, extras: &Bound<'_, PyTuple>) -> PyResult<String> {
    let primary = primary_ml_tag.trim();
    if primary.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "primary_ml_tag must be a non-empty predefined display value \
             (e.g. 'Image Classification' for dataset_type=1)",
        ));
    }
    let mut out = vec![primary.to_string()];
    let primary_key = normalize_tag_key(primary);
    for i in 0..extras.len() {
        let raw: String = extras.get_item(i)?.extract()?;
        for part in split_tags(&raw) {
            if normalize_tag_key(&part) == primary_key {
                continue;
            }
            if out
                .iter()
                .any(|existing| normalize_tag_key(existing) == normalize_tag_key(&part))
            {
                continue;
            }
            out.push(part);
        }
    }
    Ok(out.join(","))
}

/// Move `primary_ml_tag` to the front of a comma-separated tag string (insert if missing).
#[pyfunction]
pub fn ensure_primary_ml_tag_first(tags: String, primary_ml_tag: String) -> PyResult<String> {
    let primary = primary_ml_tag.trim();
    if primary.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "primary_ml_tag must be non-empty",
        ));
    }
    let parts = split_tags(&tags);
    let primary_key = normalize_tag_key(primary);
    let rest: Vec<String> = parts
        .into_iter()
        .filter(|t| normalize_tag_key(t) != primary_key)
        .collect();
    if rest.is_empty() {
        return Ok(primary.to_string());
    }
    Ok(std::iter::once(primary.to_string())
        .chain(rest)
        .collect::<Vec<_>>()
        .join(","))
}

/// Validate tags against a predefined catalog (display values from `dataset_tags_{type}`).
///
/// Rules (FE + recommended SDK practice):
/// - at least one tag required
/// - at least one tag must be in `predefined_display_values`
/// - when `require_primary_first` (default true), the **first** tag must be predefined
#[pyfunction]
#[pyo3(signature = (tags, predefined_display_values, require_primary_first=true))]
pub fn validate_ml_tags(
    tags: String,
    predefined_display_values: Vec<String>,
    require_primary_first: bool,
) -> PyResult<()> {
    let parts = split_tags(&tags);
    if parts.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "At least one tag is required.",
        ));
    }
    let predefined_keys: Vec<String> = predefined_display_values
        .iter()
        .map(|t| normalize_tag_key(t))
        .filter(|k| !k.is_empty())
        .collect();
    if predefined_keys.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "predefined_display_values is empty — fetch via \
             client.list_predefined_ml_tags(type_id) first",
        ));
    }

    let has_predefined = parts
        .iter()
        .any(|t| predefined_keys.iter().any(|k| k == &normalize_tag_key(t)));
    if !has_predefined {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Tags must include at least one predefined ML tag for this type. \
             Got {:?}; catalog examples: {:?}",
            parts,
            predefined_display_values.iter().take(5).collect::<Vec<_>>()
        )));
    }

    if require_primary_first {
        let first_key = normalize_tag_key(&parts[0]);
        if !predefined_keys.iter().any(|k| k == &first_key) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "The first tag must be a predefined ML tag for this type \
                 (Label Studio / CVAT use the first predefined tag). \
                 Got first={:?}. Use join_ml_tags(primary, *custom) or \
                 ensure_primary_ml_tag_first(tags, primary).",
                parts[0]
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod ml_tags_tests {
    use super::*;

    #[test]
    fn normalize_image_classification() {
        assert_eq!(
            normalize_tag_key("Image Classification"),
            "image_classification"
        );
    }

    #[test]
    fn validate_requires_predefined_first() {
        let catalog = vec!["Image Classification".into(), "Object Detection".into()];
        assert!(validate_ml_tags(
            "Image Classification,apk,demo".into(),
            catalog.clone(),
            true
        )
        .is_ok());
        assert!(validate_ml_tags("apk,Image Classification".into(), catalog.clone(), true).is_err());
        assert!(validate_ml_tags("apk,demo".into(), catalog, true).is_err());
    }

    #[test]
    fn ensure_primary_moves_to_front() {
        let out =
            ensure_primary_ml_tag_first("apk,demo".into(), "Image Classification".into()).unwrap();
        assert_eq!(out, "Image Classification,apk,demo");
    }
}
