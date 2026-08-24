// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Shared handling of Kappa ≥ 2.11 version package manifests.
//!
//! Dataset versions and benchmark evaluation sets expose the same manifest shape, so both
//! download paths resolve it the same way the web client does: legacy single zip when the
//! manifest says so, sharded fetch otherwise.

use pyo3::prelude::*;
use std::fs;
use std::path::Path;

use crate::utils::cache_paths;
use crate::utils::zip_utils;

/// How a manifest wants its payload fetched.
pub enum PackagePlan {
    /// Manifest reports `legacySingleZip`, or holds at most one shard.
    LegacyZip,
    /// Shard names, in manifest order (e.g. `shards/shard-00001.zip`).
    Shards(Vec<String>),
}

/// Read `buildStatus`, `legacySingleZip` and `shards[]` out of a package manifest.
///
/// Errors when the archive build has not reached `ready`, mirroring the backend's
/// `VERSION_ARCHIVE_NOT_READY` (409) gate.
pub fn plan_from_manifest(manifest: &serde_json::Value) -> PyResult<PackagePlan> {
    // A real manifest always lists shards. Anything else is a 2xx envelope such as the
    // 208 "download request still pending" reply.
    if manifest.get("shards").is_none() {
        let detail = manifest
            .get("detail")
            .or_else(|| manifest.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if is_pending_approval_error(detail) {
            return Err(PyErr::new::<pyo3::exceptions::PyPermissionError, _>(format!(
                "Download approval is still pending: {}",
                detail
            )));
        }
    }

    let build_status = manifest
        .get("buildStatus")
        .or_else(|| manifest.get("build_status"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !build_status.is_empty() && !build_status.eq_ignore_ascii_case("ready") {
        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Version archive not ready (buildStatus={}). Wait for the version build job to complete before downloading.",
            build_status
        )));
    }

    let legacy_single_zip = manifest
        .get("legacySingleZip")
        .or_else(|| manifest.get("legacy_single_zip"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let shards: Vec<String> = manifest
        .get("shards")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|shard| {
                    shard
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    if legacy_single_zip || shards.len() <= 1 {
        return Ok(PackagePlan::LegacyZip);
    }
    Ok(PackagePlan::Shards(shards))
}

/// Percent-encode a shard name while keeping its `/` separators intact.
pub fn encode_shard_name(name: &str) -> String {
    name.split('/')
        .map(|segment| urlencoding::encode(segment).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Fetch and extract every shard into `data_dir`, then mark the cache complete.
///
/// `shard_url` maps an (unencoded) shard name to its absolute download URL.
pub async fn download_shards<F>(
    http: &reqwest::Client,
    token: &str,
    data_dir: &Path,
    shards: &[String],
    shard_url: F,
) -> PyResult<()>
where
    F: Fn(&str) -> String,
{
    zip_utils::prepare_cache_dir(data_dir)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let shards_tmp = data_dir.join(cache_paths::SHARDS_TMP_DIR);
    zip_utils::create_dir_all_retry(&shards_tmp).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to create shard temp dir {}: {}",
            shards_tmp.display(),
            e
        ))
    })?;

    for (idx, name) in shards.iter().enumerate() {
        let url = shard_url(name);
        let local_name = Path::new(name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("shard.zip");
        let dest = shards_tmp.join(format!("{:05}_{}", idx, local_name));
        zip_utils::download_url_to_file(http, &url, Some(token), &dest)
            .await
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to download shard '{}': {}. If status is 409, wait until buildStatus=ready.",
                    name, e
                ))
            })?;
        zip_utils::extract_zip_file(&dest, data_dir).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to extract shard '{}': {}",
                name, e
            ))
        })?;
        let _ = fs::remove_file(&dest);
    }

    let _ = fs::remove_dir_all(&shards_tmp);
    zip_utils::mark_cache_complete(data_dir).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to mark cache complete: {}",
            e
        ))
    })?;
    Ok(())
}

/// True when an error from a package/manifest call means "try the fallback path".
///
/// The web client treats missing manifests and permission failures on the direct dataset
/// route as a signal to retry through the benchmark proxy.
pub fn is_fallback_error(message: &str) -> bool {
    // Private org-only versions are a hard deny — do not retry via the benchmark proxy.
    if message.contains("PRIVATE_VERSION_ORG_ONLY") {
        return false;
    }
    let lower = message.to_ascii_lowercase();
    message.contains("404")
        || message.contains("403")
        || message.contains("401")
        || lower.contains("not found")
        || lower.contains("forbidden")
        || lower.contains("not authenticated")
        || lower.contains("permission")
}

/// True when the caller still has a pending download-approval request (HTTP 208).
pub fn is_pending_approval_error(message: &str) -> bool {
    message.contains("208")
        || message
            .to_ascii_uppercase()
            .contains("YOUR_REQUEST_IS_PENDING")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_manifest_plans_single_zip() {
        let manifest = serde_json::json!({
            "buildStatus": "ready",
            "legacySingleZip": true,
            "shards": [{ "name": "shards/shard-00001.zip" }, { "name": "shards/shard-00002.zip" }],
        });
        assert!(matches!(
            plan_from_manifest(&manifest).unwrap(),
            PackagePlan::LegacyZip
        ));
    }

    #[test]
    fn single_shard_plans_single_zip() {
        let manifest = serde_json::json!({
            "shards": [{ "name": "shards/shard-00001.zip" }],
        });
        assert!(matches!(
            plan_from_manifest(&manifest).unwrap(),
            PackagePlan::LegacyZip
        ));
    }

    #[test]
    fn multi_shard_plans_shards_in_order() {
        let manifest = serde_json::json!({
            "buildStatus": "ready",
            "shards": [
                { "name": "shards/shard-00001.zip" },
                { "name": "shards/shard-00002.zip" },
            ],
        });
        match plan_from_manifest(&manifest).unwrap() {
            PackagePlan::Shards(names) => assert_eq!(
                names,
                vec![
                    "shards/shard-00001.zip".to_string(),
                    "shards/shard-00002.zip".to_string()
                ]
            ),
            PackagePlan::LegacyZip => panic!("expected sharded plan"),
        }
    }

    #[test]
    fn pending_approval_envelope_is_rejected() {
        // Comparing the message would need a Python interpreter, so just assert the reject.
        let payload = serde_json::json!({ "detail": "YOUR_REQUEST_IS_PENDING" });
        assert!(plan_from_manifest(&payload).is_err());
        // A shard-less manifest without that marker still falls back to the legacy zip.
        assert!(plan_from_manifest(&serde_json::json!({})).is_ok());
    }

    #[test]
    fn unready_build_is_rejected() {
        let manifest = serde_json::json!({ "buildStatus": "building", "shards": [] });
        assert!(plan_from_manifest(&manifest).is_err());
    }

    #[test]
    fn shard_name_keeps_separators() {
        assert_eq!(
            encode_shard_name("shards/shard 00001.zip"),
            "shards/shard%2000001.zip"
        );
    }

    #[test]
    fn fallback_and_pending_detection() {
        assert!(is_fallback_error("HTTP 403 Forbidden"));
        assert!(is_fallback_error("Request failed: not found"));
        assert!(!is_fallback_error("HTTP 500 server error"));
        assert!(!is_fallback_error(
            "HTTP 403: PRIVATE_VERSION_ORG_ONLY"
        ));
        assert!(is_pending_approval_error("HTTP 208 YOUR_REQUEST_IS_PENDING"));
    }
}
