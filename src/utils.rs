use anyhow::Result;
use std::fs;
use std::path::Path;
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





