// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

//! Local cache / temp directories for dataset and benchmark downloads.
//!
//! Windows users hit create/extract failures when the SDK wrote `~/cache/...` (not a
//! real Windows cache), used a leading-dot temp folder, or compared zip destinations
//! against a `\\?\` canonical path. This helper uses the OS cache dir, sanitizes
//! path segments, and keeps the old `~/cache/kappa-framework` tree when it already
//! has a complete download.

use pyo3::prelude::*;
use std::path::{Component, Path, PathBuf};

const PRODUCT: &str = "kappa-framework";

/// Directory name for in-flight shard downloads (no leading dot — Windows-safe).
pub const SHARDS_TMP_DIR: &str = "shards_tmp";

/// OS cache root for Kappa (`%LOCALAPPDATA%` / `~/.cache` / `~/Library/Caches`).
///
/// Falls back to the process temp dir after creating it: `TMP`/`TEMP` can point at a
/// folder that does not exist yet, which is a reported Windows failure mode.
pub fn kappa_cache_root() -> PyResult<PathBuf> {
    let base = match dirs::cache_dir() {
        Some(dir) => dir,
        None => ensure_temp_dir().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Could not resolve a cache or temp directory: {}",
                e
            ))
        })?,
    };
    Ok(base.join(PRODUCT))
}

/// Historical default: `{home}/cache/kappa-framework`. Kept so existing Linux caches hit.
fn legacy_home_cache_root() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join("cache").join(PRODUCT))
}

/// Create `std::env::temp_dir()` if missing, then return it.
pub fn ensure_temp_dir() -> std::io::Result<PathBuf> {
    let dir = std::env::temp_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Cache folder for a dataset version archive / package.
pub fn dataset_cache_dir(
    dataset_path: Option<&str>,
    dataset_name: &str,
    version_no: &str,
) -> PyResult<PathBuf> {
    let leaf = format!(
        "{}_{}",
        sanitize_path_segment(dataset_name),
        sanitize_path_segment(version_no)
    );
    resolve_cache_dir(dataset_path, &["datasets"], &leaf)
}

/// Cache folder for a benchmark evaluation set.
pub fn benchmark_cache_dir(dataset_path: Option<&str>, benchmark_id: &str) -> PyResult<PathBuf> {
    resolve_cache_dir(
        dataset_path,
        &["benchmarks"],
        &sanitize_path_segment(benchmark_id),
    )
}

fn resolve_cache_dir(dataset_path: Option<&str>, mid: &[&str], leaf: &str) -> PyResult<PathBuf> {
    if let Some(root) = dataset_path.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(PathBuf::from(root).join(leaf));
    }

    let mut preferred = kappa_cache_root()?;
    for part in mid {
        preferred.push(part);
    }
    preferred.push(leaf);

    if let Some(mut legacy) = legacy_home_cache_root() {
        for part in mid {
            legacy.push(part);
        }
        legacy.push(leaf);
        if legacy_cache_complete(&legacy) {
            return Ok(legacy);
        }
    }
    Ok(preferred)
}

fn legacy_cache_complete(dir: &Path) -> bool {
    dir.is_dir() && dir.join(".kappa-cache-complete").is_file()
}

/// Replace characters Windows rejects in a single path segment.
pub fn sanitize_path_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0')
            || ch.is_control()
        {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    let out = out.trim_matches(|c: char| c == ' ' || c == '.').to_string();
    let stem = out.split('.').next().unwrap_or("");
    if stem.eq_ignore_ascii_case("con")
        || stem.eq_ignore_ascii_case("prn")
        || stem.eq_ignore_ascii_case("aux")
        || stem.eq_ignore_ascii_case("nul")
        || (stem.len() == 4
            && (stem[..3].eq_ignore_ascii_case("com") || stem[..3].eq_ignore_ascii_case("lpt"))
            && stem.as_bytes()[3].is_ascii_digit())
    {
        return format!("_{}", out);
    }
    if out.is_empty() {
        "download".to_string()
    } else {
        out
    }
}

/// True when `child` is `parent` or a subdirectory of it.
///
/// Windows `canonicalize` prefixes `\\?\`, which makes a naive `starts_with` fail and
/// reject every zip entry as a zip-slip escape.
pub fn is_same_or_subdir(child: &Path, parent: &Path) -> bool {
    let child_n = strip_verbatim(child);
    let parent_n = strip_verbatim(parent);
    let child_c: Vec<_> = child_n.components().collect();
    let parent_c: Vec<_> = parent_n.components().collect();
    if child_c.len() < parent_c.len() {
        return false;
    }
    child_c
        .iter()
        .zip(parent_c.iter())
        .all(|(a, b)| components_equal(a, b))
}

fn components_equal(a: &Component<'_>, b: &Component<'_>) -> bool {
    #[cfg(windows)]
    {
        a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

fn strip_verbatim(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{}", rest));
    }
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_windows_reserved_chars() {
        assert_eq!(sanitize_path_segment(r"a:b/c|d"), "a_b_c_d");
        assert_eq!(sanitize_path_segment("1.0.0"), "1.0.0");
        assert_eq!(sanitize_path_segment("con"), "_con");
        assert_eq!(sanitize_path_segment("..."), "download");
    }

    #[test]
    fn posix_subdir_is_accepted() {
        let parent = Path::new("/tmp/kappa-framework/datasets/x");
        let child = Path::new("/tmp/kappa-framework/datasets/x/entity.json");
        assert!(is_same_or_subdir(child, parent));
        assert!(!is_same_or_subdir(Path::new("/tmp/escaped.txt"), parent));
    }

    #[test]
    fn strips_windows_verbatim_prefix() {
        let stripped = strip_verbatim(Path::new(r"\\?\C:\Users\me\x"));
        assert_eq!(stripped.to_string_lossy(), r"C:\Users\me\x");
    }
}
