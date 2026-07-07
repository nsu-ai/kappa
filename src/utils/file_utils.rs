// Copyright 2025 Kappa-Apk @nsu.ru
// SPDX-License-Identifier: Apache-2.0

// File utility functions

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Supported programming file extensions
const PROGRAMMING_EXTENSIONS: &[&str] = &[
    "py", "java", "js", "ts", "rs", "cpp", "c", "h", "hpp", "cs", "php", "rb", "go", 
    "swift", "kt", "scala", "r"
    ];

/// Information about a programming file
#[derive(Debug, Clone)]
pub struct ProgrammingFile {
    pub name: String,
    pub path: PathBuf,
    pub file_type: String,
    #[allow(dead_code)]
    pub content: Option<Vec<u8>>,
}

/// Result type for file operations
pub type FileResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct FileUtils {
    source_dir: PathBuf,
}

#[allow(dead_code)]
impl FileUtils {
    pub fn new(source_path: Option<String>) -> Self {
        Self {
            source_dir: PathBuf::from(source_path.unwrap_or(".".to_string())),
        }
    }

    pub fn source_dir(&self) -> PathBuf {
        self.source_dir.clone()
    }

    pub fn set_source_dir(&mut self, source_path: String) {
        self.source_dir = PathBuf::from(source_path);
    }

    pub fn scan_programming_files(&self, source_dir: Option<PathBuf>) -> FileResult<Vec<ProgrammingFile>> {
        let mut files = Vec::new();
        let source_path_buf: PathBuf = match source_dir {
            Some(p) => p,
            None => self.source_dir.clone(),
        };
        let source_path = source_path_buf.as_path();

        // Empty path check not needed; use current dir if empty

        if !source_path.exists() {
            return Err(format!("Directory does not exist: {}", source_path.display()).into());
        }
        
        if !source_path.is_dir() {
            return Err(format!("Path is not a directory: {}", source_path.display()).into());
        }
        
        self.scan_directory_recursive(source_path, &mut files)?;
        Ok(files)
    }

    /// Recursively scans a directory for programming files
    fn scan_directory_recursive(&self, dir: &Path, files: &mut Vec<ProgrammingFile>) -> FileResult<()> {
        let entries = fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Skip hidden directories and common non-source directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                    && !dir_name.starts_with('.')
                    && !["node_modules", "target", "build", "dist", "__pycache__", ".git"]
                        .contains(&dir_name)
                {
                    self.scan_directory_recursive(&path, files)?;
                }
            } else if path.is_file()
                && let Some(extension) = path.extension().and_then(|ext| ext.to_str())
                && PROGRAMMING_EXTENSIONS.contains(&extension.to_lowercase().as_str())
            {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let file_type = self.get_file_type(&extension);

                files.push(ProgrammingFile {
                    name: file_name,
                    path: path.clone(),
                    file_type,
                    content: None,
                });
            }
        }
        
        Ok(())
    }

    /// Determines the programming language type from file extension
    fn get_file_type(&self, extension: &str) -> String {
        let ext_lower = extension.to_lowercase();
        match ext_lower.as_str() {
            "py" => "Python".to_string(),
            "java" => "Java".to_string(),
            "js" => "JavaScript".to_string(),
            "ts" => "TypeScript".to_string(),
            "rs" => "Rust".to_string(),
            "cpp" | "cxx" | "cc" => "C++".to_string(),
            "c" => "C".to_string(),
            "h" | "hpp" => "Header".to_string(),
            "cs" => "C#".to_string(),
            "php" => "PHP".to_string(),
            "rb" => "Ruby".to_string(),
            "go" => "Go".to_string(),
            "swift" => "Swift".to_string(),
            "kt" => "Kotlin".to_string(),
            "scala" => "Scala".to_string(),
            "r" => "R".to_string(),
            "m" | "mm" => "Objective-C".to_string(),
            "pl" => "Perl".to_string(),
            "lua" => "Lua".to_string(),
            _ => format!("Unknown ({})", extension),
        }
    }

    /// Randomly selects programming files from a directory
    pub fn randomly_select_files(&self,
        source_dir: Option<PathBuf>, 
        count: usize
    ) -> FileResult<Vec<ProgrammingFile>> {
        let all_files = self.scan_programming_files(source_dir)?;
        
        if all_files.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut rng = thread_rng();
        let mut selected = Vec::new();
        let indices: Vec<usize> = (0..all_files.len()).collect();
        let chosen_indices = indices.choose_multiple(&mut rng, count.min(all_files.len()));
        
        for &idx in chosen_indices {
            selected.push(all_files[idx].clone());
        }
        
        Ok(selected)
    }

    /// Randomly selects a single programming file from a directory
    pub fn randomly_select_file(&self, source_dir: Option<PathBuf>) -> FileResult<Option<ProgrammingFile>> {
        let files = self.randomly_select_files(source_dir, 1)?;
        Ok(files.into_iter().next())
    }

    /// Reads the content of a programming file
    pub fn read_file_content(&self, file: &mut ProgrammingFile) -> FileResult<()> {
        let content = fs::read(&file.path)?;
        file.content = Some(content);
        Ok(())
    }

    /// Reads content for multiple programming files
    pub fn read_files_content(&self, files: &mut [ProgrammingFile]) -> FileResult<()> {
        for file in files.iter_mut() {
            if let Err(e) = self.read_file_content(file) {
                eprintln!("Warning: Failed to read file {}: {}", file.path.display(), e);
            }
        }
        Ok(())
    }

    /// Gets file statistics by type
    pub fn get_file_statistics(&self, source_dir: Option<PathBuf>) -> FileResult<HashMap<String, usize>> {
        let files = self.scan_programming_files(source_dir)?;
        let mut stats = HashMap::new();
        
        for file in files {
            *stats.entry(file.file_type).or_insert(0) += 1;
        }
        
        Ok(stats)
    }

    /// Finds programming files by type
    pub fn find_files_by_type(&self,
        source_dir: Option<PathBuf>, 
        file_type: &str
    ) -> FileResult<Vec<ProgrammingFile>> {
        let all_files = self.scan_programming_files(source_dir)?;
        let filtered: Vec<ProgrammingFile> = all_files
            .into_iter()
            .filter(|file| file.file_type.to_lowercase() == file_type.to_lowercase())
            .collect();
        
        Ok(filtered)
    }

    /// Randomly selects a file of a specific type
    pub fn randomly_select_file_by_type(&self,
        source_dir: Option<PathBuf>, 
        file_type: &str
    ) -> FileResult<Option<ProgrammingFile>> {
        let files = self.find_files_by_type(source_dir, file_type)?;
        
        if files.is_empty() {
            return Ok(None);
        }
        
        let mut rng = thread_rng();
        Ok(files.choose(&mut rng).cloned())
    }
}