// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Local detection of an inference pipeline draft (Kappa ≥ 2.14).
//!
//! Reads the **running user program** (`sys.modules`, `__main__.__file__`, optional model
//! object) then the artifact files they asked us to upload. Never executes the model.
//! Mixed frameworks → no guess; the caller can pass an explicit body.

use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

use crate::models_api::ModelsApi;
use crate::traits::ApiClient;

pub const PIPELINE_TYPE_ONLINE: i32 = 0;
pub const PIPELINE_TYPE_BATCH: i32 = 1;
pub const PIPELINE_TYPE_BENCHMARK: i32 = 2;

const PREFERRED_ENTRYPOINTS: &[&str] = &[
    "eval.py",
    "evaluate.py",
    "predict.py",
    "inference.py",
    "infer.py",
    "__main__.py",
];

/// Outcome of local detection. `draft` is set only when we are confident enough to PUT.
#[derive(Debug, Clone)]
pub struct PipelineDetect {
    pub draft: Option<Value>,
    pub attached: bool,
    pub reason: String,
    pub candidates: Vec<String>,
}

impl PipelineDetect {
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self {
            draft: None,
            attached: false,
            reason: reason.into(),
            candidates: Vec::new(),
        }
    }

    pub fn to_status_json(&self) -> Value {
        json!({
            "attached": self.attached,
            "reason": self.reason,
            "candidates": self.candidates,
            "draft": self.draft,
        })
    }
}

/// Build a PUT body from explicit caller JSON (marks `source: manual` unless already set).
pub fn manual_draft(mut body: Value, pipeline_type: i32) -> Value {
    if let Some(obj) = body.as_object_mut() {
        obj.entry("pipelineType".to_string())
            .or_insert(json!(pipeline_type));
        let config = obj
            .entry("pipelineConfig".to_string())
            .or_insert_with(|| json!({}));
        if let Some(cfg) = config.as_object_mut() {
            cfg.entry("source".to_string())
                .or_insert_with(|| json!("manual"));
        }
    }
    body
}

/// Detect a pipeline draft from the current interpreter, optional model object, and files.
pub fn detect(
    py: Python<'_>,
    artifact_paths: &[String],
    model: Option<&Bound<'_, PyAny>>,
    entrypoint_override: Option<&str>,
    project_file_names: &[String],
    pipeline_type: i32,
) -> PipelineDetect {
    let files = detect_from_files(artifact_paths);
    let (module_framework, modules) = detect_from_modules(py);
    let object_framework = model.and_then(detect_from_model_object);
    let model_class = model.and_then(model_class_name);

    let mut candidates: BTreeSet<String> = BTreeSet::new();
    candidates.extend(files.frameworks.iter().cloned());
    if let Some(f) = &module_framework {
        candidates.insert(f.clone());
    }
    if let Some(f) = &object_framework {
        candidates.insert(f.clone());
    }

    let framework = files
        .framework
        .clone()
        .or(object_framework)
        .or(module_framework);

    if files.frameworks.len() > 1 {
        let mut out = PipelineDetect::skipped(
            "mixed artifact frameworks; pass pipeline=… to attach explicitly",
        );
        out.candidates = candidates.into_iter().collect();
        return out;
    }

    let Some(framework) = framework else {
        let mut out = PipelineDetect::skipped("no framework detected from process or artifacts");
        out.candidates = candidates.into_iter().collect();
        return out;
    };

    let entrypoint = entrypoint_override
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| detect_entrypoint(py))
        .or_else(|| entrypoint_from_project_files(project_file_names));

    let python = python_version(py);
    let mut config = Map::new();
    config.insert("framework".to_string(), json!(framework));
    config.insert("source".to_string(), json!("apk_autodetect"));
    if let Some(ep) = &entrypoint {
        config.insert("entrypoint".to_string(), json!(ep));
    }
    if let Some(ver) = python {
        config.insert("python".to_string(), json!(ver));
    }
    if let Some(cls) = &model_class {
        config.insert("modelClass".to_string(), json!(cls));
    }
    if let Some(name) = &files.artifact_ref {
        config.insert("artifactRef".to_string(), json!(name));
    }
    config.insert(
        "signals".to_string(),
        json!({
            "modules": modules,
            "files": files.files,
        }),
    );

    let mut draft = Map::new();
    draft.insert(
        "pipelineName".to_string(),
        json!(format!("auto: {}", framework)),
    );
    draft.insert("pipelineType".to_string(), json!(pipeline_type));
    draft.insert("pipelineConfig".to_string(), Value::Object(config));
    draft.insert("endpointUrl".to_string(), Value::Null);
    if let Some(name) = files.artifact_ref {
        draft.insert("artifactRef".to_string(), json!(name));
    }

    PipelineDetect {
        draft: Some(Value::Object(draft)),
        attached: true,
        reason: "apk_autodetect".to_string(),
        candidates: candidates.into_iter().collect(),
    }
}

pub fn is_missing_pipeline_route(message: &str) -> bool {
    message.contains("404") || message.to_ascii_lowercase().contains("not found")
}

/// PUT the draft; 404 / older BE → `attached=false` without raising.
pub fn put_or_skip<T: ApiClient>(
    client: &T,
    model_id: &str,
    inference_id: i32,
    mut detect: PipelineDetect,
) -> PipelineDetect {
    let Some(draft) = detect.draft.clone() else {
        return detect;
    };
    match ModelsApi::put_inference_pipeline(client, model_id, inference_id, draft.to_string()) {
        Ok(_) => detect,
        Err(e) if is_missing_pipeline_route(&e.to_string()) => {
            detect.attached = false;
            detect.reason = "backend has no inference pipeline route".to_string();
            detect
        }
        Err(e) => {
            detect.attached = false;
            detect.reason = e.to_string();
            detect
        }
    }
}

/// Weight/layout signals from local paths (files and directories).
#[derive(Debug, Clone, Default)]
pub struct FileDetect {
    pub framework: Option<String>,
    pub artifact_ref: Option<String>,
    pub frameworks: Vec<String>,
    pub files: Vec<String>,
}

/// Framework + primary artifact name from local paths (files and directories).
pub fn detect_from_files(paths: &[String]) -> FileDetect {
    let mut hits: Vec<(String, String, u64)> = Vec::new();
    let mut saw_hf_config = false;
    let mut saw_hf_weight = false;
    let mut hf_weight_name: Option<(String, u64)> = None;

    for raw in paths {
        let path = Path::new(raw);
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    inspect_file(&entry.path(), &mut hits, &mut saw_hf_config, &mut saw_hf_weight, &mut hf_weight_name);
                }
            }
        } else {
            inspect_file(path, &mut hits, &mut saw_hf_config, &mut saw_hf_weight, &mut hf_weight_name);
        }
    }

    let files: Vec<String> = hits
        .iter()
        .map(|h| h.1.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    if saw_hf_config && saw_hf_weight {
        let artifact = hf_weight_name
            .as_ref()
            .map(|(n, _)| n.clone())
            .or_else(|| hits.first().map(|h| h.1.clone()));
        let mut names = files;
        if let Some(name) = &artifact
            && !names.iter().any(|n| n == name)
        {
            names.push(name.clone());
        }
        return FileDetect {
            framework: Some("transformers".to_string()),
            artifact_ref: artifact,
            frameworks: vec!["transformers".to_string()],
            files: names,
        };
    }

    let mut frameworks: BTreeSet<String> = hits.iter().map(|h| h.0.clone()).collect();
    if frameworks.len() > 1 {
        return FileDetect {
            framework: None,
            artifact_ref: None,
            frameworks: frameworks.into_iter().collect(),
            files,
        };
    }
    let framework = frameworks.pop_first();
    let artifact = framework.as_ref().and_then(|fw| {
        hits.iter()
            .filter(|h| &h.0 == fw)
            .max_by_key(|h| h.2)
            .map(|h| h.1.clone())
    });
    FileDetect {
        framework: framework.clone(),
        artifact_ref: artifact,
        frameworks: framework.into_iter().collect(),
        files,
    }
}

fn inspect_file(
    path: &Path,
    hits: &mut Vec<(String, String, u64)>,
    saw_hf_config: &mut bool,
    saw_hf_weight: &mut bool,
    hf_weight_name: &mut Option<(String, u64)>,
) {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return;
    }
    let lower = name.to_ascii_lowercase();
    let size = path.metadata().map(|m| m.len()).unwrap_or(0);

    if lower == "config.json" {
        *saw_hf_config = true;
        return;
    }
    if lower.ends_with(".safetensors") || lower == "pytorch_model.bin" || lower.starts_with("model-") && lower.contains(".safetensors")
    {
        *saw_hf_weight = true;
        let take = hf_weight_name
            .as_ref()
            .map(|(_, s)| size > *s)
            .unwrap_or(true);
        if take {
            *hf_weight_name = Some((name.clone(), size));
        }
    }

    if let Some(fw) = framework_from_filename(&lower) {
        hits.push((fw.to_string(), name, size));
    }
}

fn framework_from_filename(lower: &str) -> Option<&'static str> {
    if lower.ends_with(".onnx") {
        return Some("onnx");
    }
    if lower.ends_with(".pt")
        || lower.ends_with(".pth")
        || lower.ends_with(".ckpt")
        || lower == "pytorch_model.bin"
    {
        return Some("pytorch");
    }
    if lower.ends_with(".safetensors") {
        return Some("pytorch");
    }
    if lower.ends_with(".pb")
        || lower.ends_with(".keras")
        || lower.ends_with(".h5")
        || lower == "saved_model.pb"
    {
        return Some("tensorflow");
    }
    if lower.ends_with(".pkl") || lower.ends_with(".joblib") {
        return Some("sklearn");
    }
    if lower.ends_with(".pdparams") {
        return Some("paddle");
    }
    None
}

fn detect_from_modules(py: Python<'_>) -> (Option<String>, Vec<String>) {
    let Ok(sys) = py.import("sys") else {
        return (None, Vec::new());
    };
    let Ok(modules) = sys.getattr("modules") else {
        return (None, Vec::new());
    };
    let order = [
        ("transformers", "transformers"),
        ("torch", "pytorch"),
        ("onnxruntime", "onnx"),
        ("onnx", "onnx"),
        ("tensorflow", "tensorflow"),
        ("keras", "tensorflow"),
        ("sklearn", "sklearn"),
        ("paddle", "paddle"),
        ("jax", "jax"),
        ("flax", "jax"),
    ];
    let mut present = Vec::new();
    let mut framework = None;
    for (mod_name, fw) in order {
        if modules.call_method1("__contains__", (mod_name,)).ok().and_then(|v| v.extract::<bool>().ok())
            == Some(true)
        {
            present.push(mod_name.to_string());
            if framework.is_none() {
                framework = Some(fw.to_string());
            }
        }
    }
    (framework, present)
}

fn model_class_name(model: &Bound<'_, PyAny>) -> Option<String> {
    let ty = model.get_type();
    let module: String = ty.getattr("__module__").ok()?.extract().ok()?;
    let name: String = ty
        .getattr("__qualname__")
        .ok()
        .and_then(|n| n.extract().ok())
        .or_else(|| ty.getattr("__name__").ok().and_then(|n| n.extract().ok()))?;
    if module.is_empty() || name.is_empty() {
        return None;
    }
    Some(format!("{}.{}", module, name))
}

fn detect_from_model_object(model: &Bound<'_, PyAny>) -> Option<String> {
    let module: String = model
        .get_type()
        .getattr("__module__")
        .ok()
        .and_then(|m| m.extract().ok())?;
    let m = module.to_ascii_lowercase();
    if m.starts_with("transformers") {
        return Some("transformers".to_string());
    }
    if m.starts_with("torch") {
        return Some("pytorch".to_string());
    }
    if m.contains("onnx") {
        return Some("onnx".to_string());
    }
    if m.starts_with("tensorflow") || m.contains("keras") {
        return Some("tensorflow".to_string());
    }
    if m.starts_with("sklearn") {
        return Some("sklearn".to_string());
    }
    if m.starts_with("paddle") {
        return Some("paddle".to_string());
    }
    if m.starts_with("jax") || m.starts_with("flax") {
        return Some("jax".to_string());
    }
    None
}

fn detect_entrypoint(py: Python<'_>) -> Option<String> {
    let main = py.import("__main__").ok()?;
    let file: String = main.getattr("__file__").ok()?.extract().ok()?;
    if file.is_empty() || file == "-c" {
        return None;
    }
    Path::new(&file)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

fn entrypoint_from_project_files(names: &[String]) -> Option<String> {
    for wanted in PREFERRED_ENTRYPOINTS {
        if names.iter().any(|n| n.eq_ignore_ascii_case(wanted)) {
            return Some((*wanted).to_string());
        }
    }
    None
}

fn python_version(py: Python<'_>) -> Option<String> {
    let sys = py.import("sys").ok()?;
    let info = sys.getattr("version_info").ok()?;
    let major: i32 = info.getattr("major").ok()?.extract().ok()?;
    let minor: i32 = info.getattr("minor").ok()?.extract().ok()?;
    Some(format!("{}.{}", major, minor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kappa-pipe-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn pytorch_weight_is_detected() {
        let dir = temp_dir();
        let pt = dir.join("weights.pt");
        fs::write(&pt, b"x").unwrap();
        let found = detect_from_files(&[pt.to_string_lossy().to_string()]);
        assert_eq!(found.framework.as_deref(), Some("pytorch"));
        assert_eq!(found.artifact_ref.as_deref(), Some("weights.pt"));
        assert_eq!(found.files, vec!["weights.pt".to_string()]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn huggingface_layout_wins_over_safetensors_as_pytorch() {
        let dir = temp_dir();
        fs::write(dir.join("config.json"), b"{}").unwrap();
        fs::write(dir.join("model.safetensors"), b"w").unwrap();
        let found = detect_from_files(&[dir.to_string_lossy().to_string()]);
        assert_eq!(found.framework.as_deref(), Some("transformers"));
        assert_eq!(found.artifact_ref.as_deref(), Some("model.safetensors"));
        assert!(found.files.iter().any(|n| n == "model.safetensors"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn mixed_pt_and_onnx_is_ambiguous() {
        let dir = temp_dir();
        fs::write(dir.join("a.pt"), b"x").unwrap();
        fs::write(dir.join("b.onnx"), b"y").unwrap();
        let found = detect_from_files(&[dir.to_string_lossy().to_string()]);
        assert!(found.framework.is_none());
        assert!(found.artifact_ref.is_none());
        assert!(found.frameworks.contains(&"pytorch".to_string()));
        assert!(found.frameworks.contains(&"onnx".to_string()));
        assert!(found.files.contains(&"a.pt".to_string()));
        assert!(found.files.contains(&"b.onnx".to_string()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preferred_eval_script_from_project_files() {
        assert_eq!(
            entrypoint_from_project_files(&["utils.py".into(), "eval.py".into()]).as_deref(),
            Some("eval.py")
        );
        assert_eq!(entrypoint_from_project_files(&["utils.py".into()]), None);
    }
}
