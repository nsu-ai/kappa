// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! One-call publishing of a model inference plus its artifacts.
//!
//! Kappa validates every inference result against the model's *effective* inference schema
//! (a model override, its model-type template, or the generic template). This module reads
//! that schema, shapes the result document to match it, validates it server-side, creates the
//! inference and then uploads the artifact files — the flow a training script would otherwise
//! have to stitch together from five separate calls.

use pyo3::prelude::*;
use serde_json::{Map, Value};
use std::path::Path;

use crate::model_artifacts::ModelArtifactsApi;
use crate::models_api::ModelsApi;
use crate::traits::ApiClient;

pub struct InferenceWriter;

/// What a caller wants written; everything except `model_id` is optional.
#[derive(Default)]
pub struct InferenceWrite {
    pub predictions: Option<Value>,
    pub metrics: Option<Value>,
    /// A complete `inferenceResult` document; skips the builder when given.
    pub inference_result: Option<Value>,
    pub benchmark_id: Option<String>,
    pub artifacts: Vec<String>,
    pub file_category: Option<i32>,
    pub validate: bool,
    pub use_session: Option<bool>,
    pub skip_existing: bool,
    pub on_progress: Option<PyObject>,
}

impl InferenceWriter {
    /// Build → validate → create → upload, returning a summary of each step.
    ///
    /// The returned document has `modelId`, `inferenceId`, the `schema` that was applied,
    /// `validation`, `artifacts` and the `inferenceResult` that was sent.
    pub fn write<T: ApiClient>(
        client: &T,
        model_id: &str,
        mut request: InferenceWrite,
    ) -> PyResult<Value> {
        let schema = Self::effective_schema(client, model_id);

        let local_files = artifact_file_info(&request.artifacts)?;
        let document = match request.inference_result.clone() {
            Some(document) => normalize_document(
                document,
                model_id,
                request.benchmark_id.as_deref(),
                &local_files,
            ),
            None => build_document(
                model_id,
                request.benchmark_id.as_deref(),
                request.predictions.clone(),
                request.metrics.clone(),
                &local_files,
                schema.as_ref(),
            )?,
        };

        let mut validation = Value::Null;
        if request.validate && schema.is_some() {
            validation = Self::validate(client, model_id, &document)?;
        }

        let created = ModelsApi::create_model_inference(
            client,
            model_id,
            serde_json::json!({ "inferenceResult": document }).to_string(),
        )
        .map_err(|e| annotate_create_error(e, schema.as_ref()))?;
        let created =
            crate::utils::python_json::pyobject_to_rust_value(&created, "create inference")?;
        let inference_id = created
            .get("inferenceId")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Inference was created but the server returned no inferenceId",
                )
            })? as i32;

        let artifacts = if request.artifacts.is_empty() {
            Value::Array(Vec::new())
        } else {
            ModelArtifactsApi::upload_model_artifacts(
                client,
                model_id,
                inference_id,
                request.artifacts.clone(),
                request.file_category,
                request.use_session,
                request.skip_existing,
                None,
                request.on_progress.take(),
            )
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Inference {} was created but uploading its artifacts failed: {}",
                    inference_id, e
                ))
            })?
        };

        let schema_summary = schema
            .as_ref()
            .map(|schema| {
                serde_json::json!({
                    "source": schema.get("source").cloned().unwrap_or(Value::Null),
                    "kind": schema.get("kind").cloned().unwrap_or(Value::Null),
                    "schemaVersion": schema.get("schemaVersion").cloned().unwrap_or(Value::Null),
                    "isCustomized": schema.get("isCustomized").cloned().unwrap_or(Value::Null),
                })
            })
            .unwrap_or(Value::Null);

        Ok(serde_json::json!({
            "modelId": model_id,
            "inferenceId": inference_id,
            "schema": schema_summary,
            "validation": validation,
            "artifacts": artifacts,
            "inferenceResult": document,
        }))
    }

    /// The model's effective schema, or `None` on backends without the schema routes.
    fn effective_schema<T: ApiClient>(client: &T, model_id: &str) -> Option<Value> {
        let response = ModelsApi::get_model_inference_schema(client, model_id).ok()?;
        crate::utils::python_json::pyobject_to_rust_value(&response, "inference schema").ok()
    }

    fn validate<T: ApiClient>(client: &T, model_id: &str, document: &Value) -> PyResult<Value> {
        let response =
            match ModelsApi::validate_inference_result(client, model_id, document.to_string()) {
                Ok(response) => response,
                // Older backends have no validate route; the create call still validates.
                Err(e) if e.to_string().contains("404") => return Ok(Value::Null),
                Err(e) => return Err(e),
            };
        let value =
            crate::utils::python_json::pyobject_to_rust_value(&response, "schema validation")?;
        let valid = value.get("valid").and_then(|v| v.as_bool()).unwrap_or(true);
        if !valid {
            let errors = value
                .get("errors")
                .and_then(|v| v.as_array())
                .map(|errors| {
                    errors
                        .iter()
                        .filter_map(|e| e.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                })
                .unwrap_or_else(|| "schema validation failed".to_string());
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Inference result does not match the model's inference schema: {}",
                errors
            )));
        }
        Ok(value)
    }
}

/// `fileName` / `fileType` / `fileSize` / `fileHash` for each local artifact.
///
/// The server writes the same shape into `modelInformation` when a session completes, so
/// sending it up front keeps predictions-only and artifact-bearing inferences consistent.
fn artifact_file_info(paths: &[String]) -> PyResult<Vec<Value>> {
    let mut infos = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in paths {
        let path = Path::new(raw);
        let files: Vec<std::path::PathBuf> = if path.is_dir() {
            walkdir::WalkDir::new(path)
                .sort_by_file_name()
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .map(|entry| entry.into_path())
                .collect()
        } else if path.is_file() {
            vec![path.to_path_buf()]
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Artifact path does not exist: {}",
                raw
            )));
        };

        for file in files {
            let name = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if name.is_empty() || !seen.insert(name.clone()) {
                continue;
            }
            let size = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
            let file_type = file
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            // Hashing re-reads the file; skip it for weight shards.
            let hash = if ModelArtifactsApi::should_checksum(size) {
                crate::utils::git_utils::git_hash_object(&file).unwrap_or_default()
            } else {
                String::new()
            };
            infos.push(serde_json::json!({
                "fileName": name,
                "fileType": file_type,
                "fileSize": size.to_string(),
                "fileHash": hash,
            }));
        }
    }
    Ok(infos)
}

/// Assemble the L0 document (`results.predictions[]` + optional metrics) for this schema.
fn build_document(
    model_id: &str,
    benchmark_id: Option<&str>,
    predictions: Option<Value>,
    metrics: Option<Value>,
    model_information: &[Value],
    schema: Option<&Value>,
) -> PyResult<Value> {
    let predictions = predictions.unwrap_or_else(|| Value::Array(Vec::new()));
    let predictions = predictions.as_array().cloned().ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>("predictions must be a list of objects")
    })?;
    if predictions.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "predictions must not be empty — the inference schema requires results.predictions[]",
        ));
    }

    let predicted_key = schema.and_then(required_predicted_key);
    let normalized: Vec<Value> = predictions
        .into_iter()
        .enumerate()
        .map(|(index, prediction)| {
            normalize_prediction(prediction, index, predicted_key.as_deref())
        })
        .collect::<PyResult<Vec<Value>>>()?;

    let metrics = match metrics {
        Some(Value::Object(map)) => Value::Object(map),
        Some(Value::Null) | None => Value::Object(Map::new()),
        Some(_) => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "metrics must be a dict of metric name -> value",
            ));
        }
    };

    let mut results = Map::new();
    results.insert("metrics".to_string(), metrics);
    results.insert("predictions".to_string(), Value::Array(normalized));

    let mut document = Map::new();
    document.insert(
        "benchmarkId".to_string(),
        benchmark_id
            .map(|id| Value::String(id.to_string()))
            .unwrap_or(Value::Null),
    );
    document.insert("modelId".to_string(), Value::String(model_id.to_string()));
    document.insert(
        "modelInformation".to_string(),
        Value::Array(model_information.to_vec()),
    );
    document.insert("fileInformation".to_string(), Value::Null);
    document.insert("results".to_string(), Value::Object(results));
    Ok(Value::Object(document))
}

/// Fill in the platform keys a caller-supplied document usually omits.
fn normalize_document(
    document: Value,
    model_id: &str,
    benchmark_id: Option<&str>,
    model_information: &[Value],
) -> Value {
    let mut map = match document {
        Value::Object(map) => map,
        other => {
            let mut map = Map::new();
            map.insert("results".to_string(), other);
            map
        }
    };
    map.entry("modelId".to_string())
        .or_insert_with(|| Value::String(model_id.to_string()));
    if let Some(id) = benchmark_id {
        map.entry("benchmarkId".to_string())
            .or_insert_with(|| Value::String(id.to_string()));
    }
    if !model_information.is_empty() {
        map.entry("modelInformation".to_string())
            .or_insert_with(|| Value::Array(model_information.to_vec()));
    }
    Value::Object(map)
}

/// Accept snake_case keys and bare labels, emitting what the schema requires.
fn normalize_prediction(
    prediction: Value,
    index: usize,
    predicted_key: Option<&str>,
) -> PyResult<Value> {
    let mut map = prediction.as_object().cloned().ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "prediction #{} must be an object",
            index
        ))
    })?;

    if let Some(entity_id) = map.remove("entity_id")
        && !map.contains_key("entityId")
    {
        map.insert("entityId".to_string(), entity_id);
    }
    let entity_id = map
        .get("entityId")
        .and_then(|v| v.as_str().map(String::from));
    match entity_id {
        Some(id) if !id.is_empty() => {}
        Some(_) | None => {
            // Non-string ids are common when entity ids come from a dataframe.
            match map.get("entityId") {
                Some(Value::Number(n)) => {
                    map.insert("entityId".to_string(), Value::String(n.to_string()));
                }
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "prediction #{} is missing a non-empty entityId",
                        index
                    )));
                }
            }
        }
    }

    if let Some(predicted) = map.remove("prediction")
        && !map.contains_key("predicted")
    {
        map.insert("predicted".to_string(), predicted);
    }
    let predicted = map.remove("predicted").unwrap_or(Value::Null);
    let predicted = match (predicted, predicted_key) {
        // A bare label / text answer is wrapped into the shape the template requires.
        (Value::String(label), Some(key)) => {
            serde_json::json!({ key: label })
        }
        (Value::Null, _) => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "prediction #{} is missing 'predicted'",
                index
            )));
        }
        (other, _) => other,
    };
    map.insert("predicted".to_string(), predicted);

    Ok(Value::Object(map))
}

/// The single string field a template requires inside `predicted` (`class_name`, `text`, …).
fn required_predicted_key(schema: &Value) -> Option<String> {
    let predicted = schema
        .get("schemaJson")?
        .get("properties")?
        .get("results")?
        .get("properties")?
        .get("predictions")?
        .get("items")?
        .get("properties")?
        .get("predicted")?;
    let required = predicted.get("required")?.as_array()?;
    if required.len() != 1 {
        return None;
    }
    let key = required.first()?.as_str()?;
    let is_string = predicted
        .get("properties")
        .and_then(|props| props.get(key))
        .and_then(|prop| prop.get("type"))
        .and_then(|t| t.as_str())
        .map(|t| t == "string")
        .unwrap_or(false);
    is_string.then(|| key.to_string())
}

/// Turn the server's flat 400 into a pointer at the schema that rejected the payload.
fn annotate_create_error(err: PyErr, schema: Option<&Value>) -> PyErr {
    let message = err.to_string();
    if !message.contains("400") {
        return err;
    }
    let hint = schema
        .and_then(|schema| schema.get("kind").and_then(|v| v.as_str()))
        .map(|kind| format!(" The model's effective inference schema is '{}'; call get_model_inference_schema() to see its exampleJson.", kind))
        .unwrap_or_default();
    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}{}", message, hint))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classification_schema() -> Value {
        serde_json::json!({
            "source": "type",
            "kind": "classification",
            "schemaJson": {
                "properties": {
                    "results": {
                        "properties": {
                            "predictions": {
                                "items": {
                                    "properties": {
                                        "predicted": {
                                            "required": ["class_name"],
                                            "properties": {
                                                "class_name": {"type": "string"},
                                                "confidence": {"type": "number"}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn builds_l0_document_from_predictions() {
        let predictions = serde_json::json!([
            {"entity_id": "e1", "original": ["pizza"], "predicted": {"class_name": "pizza", "confidence": 0.9}}
        ]);
        let document = build_document(
            "m1",
            Some("b1"),
            Some(predictions),
            Some(serde_json::json!({"accuracy": 0.5})),
            &[],
            Some(&classification_schema()),
        )
        .unwrap();

        assert_eq!(document["modelId"], "m1");
        assert_eq!(document["benchmarkId"], "b1");
        let prediction = &document["results"]["predictions"][0];
        assert_eq!(prediction["entityId"], "e1");
        assert_eq!(prediction["predicted"]["class_name"], "pizza");
        assert_eq!(document["results"]["metrics"]["accuracy"], 0.5);
    }

    #[test]
    fn wraps_bare_label_using_schema_key() {
        let predictions = serde_json::json!([{"entityId": "e1", "predicted": "sushi"}]);
        let document = build_document(
            "m1",
            None,
            Some(predictions),
            None,
            &[],
            Some(&classification_schema()),
        )
        .unwrap();
        assert_eq!(
            document["results"]["predictions"][0]["predicted"]["class_name"],
            "sushi"
        );
    }

    #[test]
    fn rejects_predictions_without_entity_id() {
        let predictions = serde_json::json!([{"predicted": {"class_name": "pizza"}}]);
        // PyErr text needs a live interpreter, so only the failure itself is asserted.
        assert!(build_document("m1", None, Some(predictions), None, &[], None).is_err());
    }

    #[test]
    fn numeric_entity_ids_become_strings() {
        let predictions = serde_json::json!([{"entityId": 42, "predicted": {"class_name": "x"}}]);
        let document = build_document("m1", None, Some(predictions), None, &[], None).unwrap();
        assert_eq!(document["results"]["predictions"][0]["entityId"], "42");
    }

    #[test]
    fn caller_document_keeps_its_own_fields() {
        let document = normalize_document(
            serde_json::json!({"results": {"predictions": []}, "modelId": "given"}),
            "m1",
            Some("b1"),
            &[],
        );
        assert_eq!(document["modelId"], "given");
        assert_eq!(document["benchmarkId"], "b1");
    }

    #[test]
    fn predicted_key_only_when_single_string_field() {
        assert_eq!(
            required_predicted_key(&classification_schema()).as_deref(),
            Some("class_name")
        );
        let generic = serde_json::json!({"schemaJson": {"properties": {}}});
        assert_eq!(required_predicted_key(&generic), None);
    }
}
