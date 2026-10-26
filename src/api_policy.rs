// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Shared HTTP policy for Kappa-framework 2.15.
//!
//! Retired v1 mounts are rejected before a socket is opened. Multipart upload
//! names are reduced to one path segment. Error bodies for lockout, the device
//! cap, and anonymous sessions are mapped to their own text so they are not
//! reported as a missing dataset permission.

use pyo3::exceptions::{PyPermissionError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;

/// Inclusive range accepted by dataset publish on Kappa ≥ 2.15.
pub const PUBLISH_TYPE_MIN: i32 = 0;
pub const PUBLISH_TYPE_MAX: i32 = 3;

/// Legend for the publish values this SDK still sends.
pub const PUBLISH_TYPE_LEGEND: &str =
    "0 Not Published, 1 Private, 2 Open Source, 3 Public on Demand";

const FALLBACK_UPLOAD_NAME: &str = "upload.bin";

/// `publish_dataset_version` accepts only `0..=3`. Open visibility is 2 and 3.
pub fn validate_publish_type(publish_type: i32) -> PyResult<()> {
    if (PUBLISH_TYPE_MIN..=PUBLISH_TYPE_MAX).contains(&publish_type) {
        Ok(())
    } else {
        Err(PyErr::new::<PyValueError, _>(format!(
            "INVALID_PUBLISH_TYPE: publish_type must be {PUBLISH_TYPE_LEGEND} (got {publish_type}). Open visibility is 2 or 3."
        )))
    }
}

/// True when `endpoint` still addresses a v1 mount (`micro-services/v1` or `/api/v1`).
pub fn targets_retired_v1(endpoint: &str) -> bool {
    let path = endpoint.split(['?', '#']).next().unwrap_or(endpoint);
    let decoded = percent_decode_ascii(path);
    let normalized = collapse_slashes(&decoded).to_ascii_lowercase();
    bounded_contains(&normalized, "micro-services/v1") || api_v1_mount(&normalized)
}

/// Reject a retired v1 path before any socket is opened.
pub fn reject_retired_v1_path(endpoint: &str) -> PyResult<()> {
    if targets_retired_v1(endpoint) {
        Err(PyErr::new::<PyValueError, _>(
            "Retired API path: Kappa-framework 2.15 removed v1 mounts (micro-services/v1 and /api/v1). Use the v2 route.",
        ))
    } else {
        Ok(())
    }
}

/// Reduce a multipart filename to a single path segment.
///
/// `..`, an absolute path, and a trailing slash never leave the process.
/// An empty result becomes `upload.bin`.
pub fn safe_upload_filename(name: &str) -> String {
    let decoded = percent_decode_ascii(name);
    let normalized = decoded.replace('\\', "/");
    let segment = normalized
        .rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or("");
    let cleaned: String = segment.chars().filter(|c| !c.is_control()).collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        FALLBACK_UPLOAD_NAME.to_string()
    } else {
        cleaned.to_string()
    }
}

/// Login responses that must wipe the in-memory password so `connect()` cannot retry.
pub fn login_failure_drops_password(body: &str) -> bool {
    let upper = body.to_ascii_uppercase();
    has_error_code(&upper, "LOGIN_TEMPORARILY_BLOCKED")
        || has_error_code(&upper, "TOO_MANY_ACTIVE_DEVICES")
}

/// Map a failed `connect()` body. Lockout and the device cap use [`http_error_to_pyerr`].
/// A wrong password stays a `ValueError` and does not claim the password was wiped.
pub fn login_error_to_pyerr(status: reqwest::StatusCode, text: String) -> PyErr {
    if login_failure_drops_password(&text) {
        return http_error_to_pyerr(status, text);
    }
    let detail = extract_http_detail(&text);
    let shown = if detail.is_empty() { text } else { detail };
    PyErr::new::<PyValueError, _>(format!("Login failed (HTTP {status}): {shown}"))
}

/// Map an HTTP error body to a Python exception.
pub fn http_error_to_pyerr(status: reqwest::StatusCode, text: String) -> PyErr {
    let detail = extract_http_detail(&text);
    let blob = format!("{text}\n{detail}").to_ascii_uppercase();
    let msg = if detail.is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {detail}")
    };

    if has_error_code(&blob, "LOGIN_TEMPORARILY_BLOCKED") {
        return PyErr::new::<PyPermissionError, _>(
            "HTTP 429 LOGIN_TEMPORARILY_BLOCKED: sign-in is temporarily blocked after repeated failures. Wait for the lockout to end, then create a new KappaApkClient.",
        );
    }
    if has_error_code(&blob, "TOO_MANY_ACTIVE_DEVICES") {
        return PyErr::new::<PyPermissionError, _>(
            "HTTP 403 TOO_MANY_ACTIVE_DEVICES: this account already has the maximum number of active devices. Close another session, then create a new KappaApkClient.",
        );
    }
    if has_error_code(&blob, "AUTHENTICATED_SESSION_REQUIRED") {
        return PyErr::new::<PyPermissionError, _>(
            "HTTP 403 AUTHENTICATED_SESSION_REQUIRED: this route needs a signed-in user. An anonymous session is not a missing dataset permission.",
        );
    }
    if has_error_code(&blob, "OTP_RESEND_LIMIT") {
        return PyErr::new::<PyRuntimeError, _>(
            "HTTP 429 OTP_RESEND_LIMIT: the password-reset code was requested too many times. Wait before requesting another code.",
        );
    }
    if has_error_code(&blob, "PASSWORD_RESET_TEMPORARILY_BLOCKED") {
        return PyErr::new::<PyRuntimeError, _>(
            "HTTP 429 PASSWORD_RESET_TEMPORARILY_BLOCKED: password reset is temporarily blocked after wrong codes.",
        );
    }
    if has_error_code(&blob, "INVALID_PUBLISH_TYPE") {
        return PyErr::new::<PyValueError, _>(format!(
            "HTTP 400 INVALID_PUBLISH_TYPE: publish_type must be {PUBLISH_TYPE_LEGEND}. Open visibility is 2 or 3."
        ));
    }

    if status.as_u16() == 409 {
        let extra = if has_error_code(&blob, "DATASET_PERMANENTLY_DELETED") {
            " Dataset was permanently deleted after the recover window (entity files purged)."
        } else if has_error_code(&blob, "DATASET_DELETED") {
            " Dataset is soft-deleted (status 0); recover first or omit short-info edits."
        } else if has_error_code(&blob, "PRIMARY_TAG_IMMUTABLE") {
            " Cannot remove or replace the primary ML tag via PATCH tags."
        } else {
            ""
        };
        PyErr::new::<PyRuntimeError, _>(format!("{msg}{extra}"))
    } else if status.as_u16() == 403 {
        if has_error_code(&blob, "PRIVATE_VERSION_ORG_ONLY") {
            PyErr::new::<PyPermissionError, _>(
                "HTTP 403: This version is private to the owning organization.",
            )
        } else {
            PyErr::new::<PyPermissionError, _>(format!(
                "{msg}. Hint: call client.get_my_permissions(dataset_id=...) or has_permission('dataset.write', dataset_id=...)."
            ))
        }
    } else if status.as_u16() == 401 {
        let detail_bit = if detail.is_empty() {
            String::new()
        } else {
            format!(" {detail}.")
        };
        PyErr::new::<PyPermissionError, _>(format!(
            "HTTP 401:{detail_bit} The cached session is no longer valid. Create a new KappaApkClient and call connect(). A password change revokes every session, and connect() already cleared the saved password."
        ))
    } else if status.as_u16() == 429 {
        PyErr::new::<PyRuntimeError, _>(format!(
            "{msg} (server busy — retry after Retry-After / retryAfterSeconds)"
        ))
    } else {
        PyErr::new::<PyValueError, _>(msg)
    }
}

fn extract_http_detail(text: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
        json.get("detail")
            .map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else {
                    v.to_string()
                }
            })
            .unwrap_or_else(|| text.to_string())
    } else {
        text.to_string()
    }
}

fn has_error_code(upper: &str, code: &str) -> bool {
    let bytes = upper.as_bytes();
    let needle = code.as_bytes();
    if needle.is_empty() || needle.len() > bytes.len() {
        return false;
    }
    let mut start = 0;
    while start + needle.len() <= bytes.len() {
        if let Some(rel) = upper[start..].find(code) {
            let idx = start + rel;
            let before_ok = idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric();
            let after = idx + needle.len();
            let after_ok = after == bytes.len() || !bytes[after].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return true;
            }
            start = idx + 1;
        } else {
            break;
        }
    }
    false
}

fn api_v1_mount(normalized: &str) -> bool {
    bounded_contains(normalized, "/api/v1")
        || normalized == "api/v1"
        || normalized.starts_with("api/v1/")
}

fn bounded_contains(hay: &str, needle: &str) -> bool {
    let bytes = hay.as_bytes();
    let needle_bytes = needle.as_bytes();
    if needle_bytes.is_empty() || needle_bytes.len() > bytes.len() {
        return false;
    }
    let mut start = 0;
    while start + needle_bytes.len() <= bytes.len() {
        if let Some(rel) = hay[start..].find(needle) {
            let idx = start + rel;
            let after = idx + needle_bytes.len();
            let boundary = after == bytes.len() || bytes[after] == b'/';
            if boundary {
                return true;
            }
            start = idx + 1;
        } else {
            break;
        }
    }
    false
}

fn collapse_slashes(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut prev_slash = false;
    for c in path.chars() {
        if c == '/' {
            if prev_slash {
                continue;
            }
            prev_slash = true;
        } else {
            prev_slash = false;
        }
        out.push(c);
    }
    out
}

fn percent_decode_ascii(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_v1_paths_are_rejected() {
        assert!(targets_retired_v1("/user-micro-services/v1/session/new"));
        assert!(targets_retired_v1("/data-micro-services/v1/datasets"));
        assert!(targets_retired_v1("model-micro-services/v1/models"));
        assert!(targets_retired_v1("/api/v1/users"));
        assert!(targets_retired_v1("/API/V1/users"));
        assert!(targets_retired_v1("/user-micro-services//v1/session"));
        assert!(targets_retired_v1("/user-micro-services%2Fv1/session"));
        assert!(targets_retired_v1("https://kappa.example/api/v1/datasets"));
    }

    #[test]
    fn v2_paths_and_lookalikes_are_allowed() {
        assert!(!targets_retired_v1(
            "/user-micro-services/v2/session/new"
        ));
        assert!(!targets_retired_v1(
            "/data-micro-services/v2/datasets/filter"
        ));
        assert!(!targets_retired_v1("/api/v2/datasets"));
        assert!(!targets_retired_v1("/api/v10/datasets"));
        assert!(!targets_retired_v1(
            "/user-micro-services/v2/x?next=/api/v1"
        ));
        assert!(!targets_retired_v1("/model-micro-services/v15/models"));
        assert!(reject_retired_v1_path("/data-micro-services/v2/datasets").is_ok());
        assert!(reject_retired_v1_path("/user-micro-services/v1/session").is_err());
    }

    #[test]
    fn upload_names_are_one_segment() {
        assert_eq!(safe_upload_filename("../../etc/passwd"), "passwd");
        assert_eq!(safe_upload_filename("/etc/passwd"), "passwd");
        assert_eq!(safe_upload_filename(r"C:\foo\bar.txt"), "bar.txt");
        assert_eq!(safe_upload_filename(".."), "upload.bin");
        assert_eq!(safe_upload_filename(""), "upload.bin");
        assert_eq!(safe_upload_filename("a/../../b.txt"), "b.txt");
        assert_eq!(safe_upload_filename("..%2Fsecret.bin"), "secret.bin");
        assert_eq!(safe_upload_filename("report.csv"), "report.csv");
        assert_eq!(safe_upload_filename("dir/"), "dir");
        assert_eq!(safe_upload_filename("foo/.."), "upload.bin");
        assert_eq!(safe_upload_filename("notes.tar.gz"), "notes.tar.gz");
    }

    #[test]
    fn publish_type_is_zero_through_three() {
        assert!(validate_publish_type(0).is_ok());
        assert!(validate_publish_type(3).is_ok());
        assert!(validate_publish_type(4).is_err());
        assert!(validate_publish_type(-1).is_err());
    }

    #[test]
    fn login_lockout_codes_drop_the_password() {
        assert!(login_failure_drops_password(
            r#"{"detail":"LOGIN_TEMPORARILY_BLOCKED"}"#
        ));
        assert!(login_failure_drops_password(
            r#"{"detail":{"code":"TOO_MANY_ACTIVE_DEVICES"}}"#
        ));
        assert!(!login_failure_drops_password(
            r#"{"detail":"invalid credentials"}"#
        ));
    }

    #[test]
    fn error_text_keeps_platform_codes_distinct() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|_py| {
            let locked = http_error_to_pyerr(
                reqwest::StatusCode::TOO_MANY_REQUESTS,
                r#"{"detail":"LOGIN_TEMPORARILY_BLOCKED"}"#.to_string(),
            );
            let locked = locked.to_string();
            assert!(locked.contains("LOGIN_TEMPORARILY_BLOCKED"));
            assert!(!locked.contains("dataset.write"));

            let anon = http_error_to_pyerr(
                reqwest::StatusCode::FORBIDDEN,
                r#"{"detail":"AUTHENTICATED_SESSION_REQUIRED"}"#.to_string(),
            );
            let anon = anon.to_string();
            assert!(anon.contains("AUTHENTICATED_SESSION_REQUIRED"));
            assert!(!anon.contains("dataset.write"));

            let devices = http_error_to_pyerr(
                reqwest::StatusCode::FORBIDDEN,
                r#"{"detail":"TOO_MANY_ACTIVE_DEVICES"}"#.to_string(),
            );
            assert!(devices.to_string().contains("TOO_MANY_ACTIVE_DEVICES"));

            let expired = http_error_to_pyerr(
                reqwest::StatusCode::UNAUTHORIZED,
                r#"{"detail":"token expired"}"#.to_string(),
            );
            let expired = expired.to_string();
            assert!(expired.contains("KappaApkClient"));
            assert!(expired.contains("401"));

            let purchase = http_error_to_pyerr(
                reqwest::StatusCode::BAD_REQUEST,
                r#"{"detail":"INVALID_PUBLISH_TYPE"}"#.to_string(),
            );
            assert!(purchase.to_string().contains("INVALID_PUBLISH_TYPE"));

            let org = http_error_to_pyerr(
                reqwest::StatusCode::FORBIDDEN,
                r#"{"detail":"PRIVATE_VERSION_ORG_ONLY"}"#.to_string(),
            );
            assert!(org.to_string().contains("owning organization"));

            let busy = http_error_to_pyerr(
                reqwest::StatusCode::TOO_MANY_REQUESTS,
                r#"{"detail":"admission"}"#.to_string(),
            );
            assert!(busy.to_string().contains("Retry-After"));

            let login = login_error_to_pyerr(
                reqwest::StatusCode::UNAUTHORIZED,
                r#"{"detail":"invalid credentials"}"#.to_string(),
            );
            let login = login.to_string();
            assert!(login.contains("Login failed"));
            assert!(!login.contains("already cleared"));
        });
    }
}
