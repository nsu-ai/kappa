// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0
//
// Reserved upload/archive helpers — not yet wired into the public API.

#![allow(dead_code)]

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use zip::ZipArchive;

/// Result type for zip utilities
pub type ZipResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Zip files from a directory with good compression
/// 
/// # Parameters
/// 
/// * `source_dir` - Path to the directory to zip
/// * `output_path` - Optional path for the output zip file. If None, creates a temporary file
/// 
/// # Returns
/// 
/// Path to the created zip file
/// 
/// # Errors
/// 
/// Returns an error if the directory doesn't exist, can't be read, or zip creation fails
pub fn zip_directory(source_dir: &Path, output_path: Option<PathBuf>) -> ZipResult<PathBuf> {
    if !source_dir.is_dir() {
        return Err(format!("Source path is not a directory: {}", source_dir.display()).into());
    }

    // Determine output path
    let zip_path = match output_path {
        Some(path) => path,
        None => {
            // Create temporary zip file with timestamp-based name
            let temp_dir = std::env::temp_dir();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            temp_dir.join(format!("archive_{}.zip", timestamp))
        }
    };

    let file = fs::File::create(&zip_path)
        .map_err(|e| format!("Failed to create zip file: {}", e))?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9)); // Maximum compression

    // Walk through the directory recursively and add all files
    for entry in WalkDir::new(source_dir) {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        
        let path = entry.path();
        if path.is_file() {
            // Get relative path from source_dir for proper zip structure
            let relative_path = path.strip_prefix(source_dir)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;

            let file_name = relative_path.to_string_lossy().to_string();

            zip.start_file(&file_name, options)
                .map_err(|e| format!("Failed to add file to zip: {}", e))?;

            let mut file_content = fs::File::open(path)
                .map_err(|e| format!("Failed to open file for zipping: {}", e))?;

            let mut buffer = Vec::new();
            file_content.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read file content: {}", e))?;

            zip.write_all(&buffer)
                .map_err(|e| format!("Failed to write file to zip: {}", e))?;
        }
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip file: {}", e))?;

    Ok(zip_path)
}

/// Unzip files from a zip archive to a destination directory
/// 
/// # Parameters
/// 
/// * `zip_path` - Path to the zip file to extract
/// * `dest_dir` - Directory where files should be extracted
/// 
/// # Returns
/// 
/// Path to the destination directory
/// 
/// # Errors
/// 
/// Returns an error if the zip file doesn't exist, can't be read, or extraction fails
pub fn unzip_file(zip_path: &Path, dest_dir: &Path) -> ZipResult<PathBuf> {
    if !zip_path.is_file() {
        return Err(format!("Zip path is not a file: {}", zip_path.display()).into());
    }

    // Create destination directory if it doesn't exist
    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    let zip_file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;

    let mut archive = ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to access file in zip: {}", e))?;

        let outpath = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            // Directory entry
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            // File entry
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }
    }

    Ok(dest_dir.to_path_buf())
}

/// Zip a single file
/// 
/// # Parameters
/// 
/// * `file_path` - Path to the file to zip
/// * `output_path` - Optional path for the output zip file. If None, creates a temporary file
/// 
/// # Returns
/// 
/// Path to the created zip file
/// 
/// # Errors
/// 
/// Returns an error if the file doesn't exist, can't be read, or zip creation fails
pub fn zip_file(file_path: &Path, output_path: Option<PathBuf>) -> ZipResult<PathBuf> {
    if !file_path.is_file() {
        return Err(format!("Path is not a file: {}", file_path.display()).into());
    }

    // Determine output path
    let zip_path = match output_path {
        Some(path) => path,
        None => {
            let temp_dir = std::env::temp_dir();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let file_name = file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            temp_dir.join(format!("{}_{}.zip", file_name, timestamp))
        }
    };

    let file = fs::File::create(&zip_path)
        .map_err(|e| format!("Failed to create zip file: {}", e))?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    zip.start_file(&file_name, options)
        .map_err(|e| format!("Failed to add file to zip: {}", e))?;

    let mut file_content = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file for zipping: {}", e))?;

    let mut buffer = Vec::new();
    file_content.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file content: {}", e))?;

    zip.write_all(&buffer)
        .map_err(|e| format!("Failed to write file to zip: {}", e))?;

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip file: {}", e))?;

    Ok(zip_path)
}

