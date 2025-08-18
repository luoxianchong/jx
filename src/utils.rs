use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else {
        format!("{:.1} GB", size as f64 / GB as f64)
    }
}

pub fn calculate_directory_size(dir_path: &Path) -> Result<u64> {
    let mut total_size = 0;
    
    for entry in WalkDir::new(dir_path) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                total_size += metadata.len();
            }
        }
    }
    
    Ok(total_size)
}

pub fn find_files_by_extension(dir_path: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(dir_path) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    files.push(path.to_path_buf());
                }
            }
        }
    }
    
    Ok(files)
}

pub fn ensure_directory_exists(dir_path: &Path) -> Result<()> {
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }
    Ok(())
}

pub fn is_java_file(file_path: &Path) -> bool {
    file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "java")
        .unwrap_or(false)
}

pub fn is_jar_file(file_path: &Path) -> bool {
    file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "jar")
        .unwrap_or(false)
}

pub fn get_home_directory() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))
}

pub fn get_jx_config_dir() -> Result<PathBuf> {
    let home = get_home_directory()?;
    Ok(home.join(".jx"))
}

pub fn get_jx_cache_dir() -> Result<PathBuf> {
    let config_dir = get_jx_config_dir()?;
    Ok(config_dir.join("cache"))
}

pub fn ensure_jx_directories() -> Result<()> {
    let config_dir = get_jx_config_dir()?;
    let cache_dir = get_jx_cache_dir()?;
    
    ensure_directory_exists(&config_dir)?;
    ensure_directory_exists(&cache_dir)?;
    
    Ok(())
}
