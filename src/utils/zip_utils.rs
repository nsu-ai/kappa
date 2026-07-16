// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use zip::ZipArchive;

/// Written after a successful archive download + extract into a cache directory.
pub const CACHE_MARKER_FILE: &str = ".kappa-cache-complete";

/// Result type for zip utilities (upload helpers).
#[allow(dead_code)]
pub type ZipResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn cache_is_complete(dir: &Path) -> bool {
    dir.is_dir() && dir.join(CACHE_MARKER_FILE).is_file()
}

pub fn mark_cache_complete(dir: &Path) -> std::io::Result<()> {
    fs::write(dir.join(CACHE_MARKER_FILE), b"ok")
}

/// Remove incomplete cache trees so a failed prior download can be retried cleanly.
pub fn prepare_cache_dir(dir: &Path) -> std::io::Result<()> {
    if dir.exists() && !cache_is_complete(dir) {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)
}

/// Resolve a zip entry path under `base`, rejecting traversal (`..`, absolute paths).
fn resolve_zip_entry_path(base: &Path, entry_name: &str) -> Result<PathBuf, String> {
    let entry_path = Path::new(entry_name);
    if entry_path.is_absolute() {
        return Err(format!("Zip entry '{}' is absolute", entry_name));
    }

    let mut out = base.to_path_buf();
    for component in entry_path.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(format!(
                    "Zip entry '{}' would escape the extraction directory",
                    entry_name
                ));
            }
        }
    }

    if let Ok(canonical_base) = base.canonicalize()
        && !out.starts_with(&canonical_base)
    {
        return Err(format!(
            "Zip entry '{}' would escape the extraction directory",
            entry_name
        ));
    }

    Ok(out)
}

/// Extract all entries from an open zip archive with zip-slip protection.
pub fn extract_zip_archive<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    dest_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create extraction directory: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to access zip entry {i}: {e}"))?;
        let outpath = resolve_zip_entry_path(dest_dir, entry.name())?;

        if entry.name().ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }
    }

    Ok(())
}

pub fn extract_zip_file(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;
    extract_zip_archive(&mut archive, dest_dir)
}

pub async fn download_url_to_file(
    http: &reqwest::Client,
    url: &str,
    auth_token: Option<&str>,
    dest: &Path,
) -> Result<(), String> {
    let mut request = http.get(url).header("accept", "*/*");
    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Failed to download: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("Failed to create download file: {e}"))?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {e}"))?;
    }
    file.flush()
        .await
        .map_err(|e| format!("Flush error: {e}"))?;
    Ok(())
}

/// Stream a zip from `url` into `dest_dir`, extract securely, and mark the cache complete.
pub async fn download_and_extract_zip(
    http: &reqwest::Client,
    url: &str,
    auth_token: Option<&str>,
    dest_dir: &Path,
) -> Result<(), String> {
    prepare_cache_dir(dest_dir).map_err(|e| format!("Failed to prepare cache dir: {e}"))?;
    let temp_zip = dest_dir.join("temp_archive.zip");
    download_url_to_file(http, url, auth_token, &temp_zip).await?;
    extract_zip_file(&temp_zip, dest_dir)?;
    fs::remove_file(&temp_zip).map_err(|e| format!("Failed to delete temp zip: {e}"))?;
    mark_cache_complete(dest_dir).map_err(|e| format!("Failed to mark cache complete: {e}"))?;
    Ok(())
}

/// Zip files from a directory with good compression (reserved for future upload helpers).
#[allow(dead_code)]
pub fn zip_directory(source_dir: &Path, output_path: Option<PathBuf>) -> ZipResult<PathBuf> {
    if !source_dir.is_dir() {
        return Err(format!("Source path is not a directory: {}", source_dir.display()).into());
    }

    let zip_path = match output_path {
        Some(path) => path,
        None => {
            let temp_dir = std::env::temp_dir();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            temp_dir.join(format!("archive_{timestamp}.zip"))
        }
    };

    let file = fs::File::create(&zip_path)
        .map_err(|e| format!("Failed to create zip file: {}", e))?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    for entry in WalkDir::new(source_dir) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;

        let path = entry.path();
        if path.is_file() {
            let relative_path = path
                .strip_prefix(source_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;

            let file_name = relative_path.to_string_lossy().to_string();

            zip.start_file(&file_name, options)
                .map_err(|e| format!("Failed to add file to zip: {}", e))?;

            let mut file_content = fs::File::open(path)
                .map_err(|e| format!("Failed to open file for zipping: {}", e))?;

            let mut buffer = Vec::new();
            file_content
                .read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read file content: {}", e))?;

            zip.write_all(&buffer)
                .map_err(|e| format!("Failed to write file to zip: {}", e))?;
        }
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip file: {}", e))?;

    Ok(zip_path)
}

/// Unzip files from a zip archive to a destination directory (zip-slip safe).
#[allow(dead_code)]
pub fn unzip_file(zip_path: &Path, dest_dir: &Path) -> ZipResult<PathBuf> {
    extract_zip_file(zip_path, dest_dir).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    Ok(dest_dir.to_path_buf())
}

/// Zip a single file (reserved for future upload helpers).
#[allow(dead_code)]
pub fn zip_file(file_path: &Path, output_path: Option<PathBuf>) -> ZipResult<PathBuf> {
    if !file_path.is_file() {
        return Err(format!("Path is not a file: {}", file_path.display()).into());
    }

    let zip_path = match output_path {
        Some(path) => path,
        None => {
            let temp_dir = std::env::temp_dir();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            temp_dir.join(format!("{file_name}_{timestamp}.zip"))
        }
    };

    let file = fs::File::create(&zip_path)
        .map_err(|e| format!("Failed to create zip file: {}", e))?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    zip.start_file(&file_name, options)
        .map_err(|e| format!("Failed to add file to zip: {}", e))?;

    let mut file_content = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file for zipping: {}", e))?;

    let mut buffer = Vec::new();
    file_content
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file content: {}", e))?;

    zip.write_all(&buffer)
        .map_err(|e| format!("Failed to write file to zip: {}", e))?;

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip file: {}", e))?;

    Ok(zip_path)
}

#[cfg(test)]
mod zip_tests {
    use super::*;
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    #[test]
    fn rejects_parent_dir_zip_entry() {
        let base = std::env::temp_dir().join(format!(
            "kappa_zip_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));
        fs::create_dir_all(&base).unwrap();
        let zip_path = base.join("evil.zip");
        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut zip = ZipWriter::new(file);
            zip.start_file("../escape.txt", FileOptions::default())
                .unwrap();
            zip.write_all(b"bad").unwrap();
            zip.finish().unwrap();
        }

        let err = extract_zip_file(&zip_path, &base.join("out")).unwrap_err();
        assert!(err.contains("escape"));

        let _ = fs::remove_dir_all(&base);
    }
}
