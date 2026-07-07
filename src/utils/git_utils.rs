// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

// Git utility functions

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use sha1::{Sha1, Digest};

/// Result type for git utilities
pub type GitResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Compute `git hash-object` (blob SHA-1) for a single file.
/// 
/// Uses `git` CLI when available; falls back to pure Rust implementation.
/// 
/// # Parameters
/// 
/// * `file_path` - Path to the file to hash
/// 
/// # Returns
/// 
/// The Git blob SHA-1 hash as a string
/// 
/// # Errors
/// 
/// Returns an error if the file doesn't exist or can't be read
pub fn git_hash_object<P: AsRef<Path>>(file_path: P) -> GitResult<String> {
    let path = file_path.as_ref();
    
    if !path.is_file() {
        return Err(format!("Not a file: {}", path.display()).into());
    }

    // Try via git CLI first for better performance
    if let Ok(hash) = try_git_cli_hash(path) {
        return Ok(hash);
    }

    // Fallback: compute Git blob SHA-1 manually
    compute_git_blob_sha1(path)
}

/// Try to compute hash using git CLI.
fn try_git_cli_hash(path: &Path) -> GitResult<String> {
    let output = Command::new("git")
        .args(["hash-object", path.to_string_lossy().as_ref()])
        .output()?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !hash.is_empty() {
            return Ok(hash);
        }
    }
    
    Err("Git CLI failed".into())
}

/// Compute `git hash-object` for multiple files, returning (path, hash) pairs.
#[allow(dead_code)]
pub fn git_hash_objects<P: AsRef<Path>>(files: &[P]) -> GitResult<Vec<(PathBuf, String)>> {
    let mut results = Vec::with_capacity(files.len());
    for file in files {
        let path = file.as_ref();
        let hash = git_hash_object(path)?;
        results.push((path.to_path_buf(), hash));
    }
    Ok(results)
}

/// Fallback pure-Rust computation of Git blob SHA-1.
/// 
/// Git computes SHA-1 over: b"blob <len>\0" + file_contents
/// This is a memory-efficient implementation that reads the file in chunks.
/// 
/// # Parameters
/// 
/// * `path` - Path to the file to hash
/// 
/// # Returns
/// 
/// The Git blob SHA-1 hash as a string
/// 
/// # Errors
/// 
/// Returns an error if the file can't be read or processed
fn compute_git_blob_sha1(path: &Path) -> GitResult<String> {
    let mut file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();

    // Build header: "blob <size>\0"
    let header = format!("blob {}\0", size);

    // Stream into SHA-1
    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());

    // Read file in chunks to avoid loading whole file into memory
    const BUFFER_SIZE: usize = 8192;
    let mut buffer = [0u8; BUFFER_SIZE];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let digest = hasher.finalize();
    Ok(format!("{:x}", digest))
}

