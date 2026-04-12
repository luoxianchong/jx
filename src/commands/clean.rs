use anyhow::Result;
use glob;
use std::fs;
use std::path::Path;

pub fn execute() -> Result<()> {
    println!("🧹 清理构建文件...");

    let current_dir = std::env::current_dir()?;

    // 检查项目类型
    let project_type = detect_project_type(&current_dir)?;
    println!("检测到项目类型: {}", project_type);

    let mut cleaned_items = Vec::new();

    // 清理Maven项目
    if project_type == "maven" || project_type == "both" {
        let maven_target = current_dir.join("target");
        if maven_target.exists() {
            fs::remove_dir_all(&maven_target)?;
            cleaned_items.push("Maven target目录".to_string());
        }
    }

    // 清理Gradle项目
    if project_type == "gradle" || project_type == "both" {
        let gradle_build = current_dir.join("build");
        if gradle_build.exists() {
            fs::remove_dir_all(&gradle_build)?;
            cleaned_items.push("Gradle build目录".to_string());
        }

        let gradle_gradle = current_dir.join(".gradle");
        if gradle_gradle.exists() {
            fs::remove_dir_all(&gradle_gradle)?;
            cleaned_items.push("Gradle缓存目录".to_string());
        }
    }

    // 清理通用构建目录
    let lib_dir = current_dir.join("lib");
    if lib_dir.exists() {
        fs::remove_dir_all(&lib_dir)?;
        cleaned_items.push("lib依赖目录".to_string());
    }

    let out_dir = current_dir.join("out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
        cleaned_items.push("out输出目录".to_string());
    }

    // 清理临时文件
    clean_temp_files(&current_dir, &mut cleaned_items)?;

    // 清理IDE相关文件
    clean_ide_files(&current_dir, &mut cleaned_items)?;

    if cleaned_items.is_empty() {
        println!("✅ 项目已经是干净状态，无需清理");
    } else {
        println!("✅ 清理完成！已清理以下内容:");
        for item in &cleaned_items {
            println!("  - {}", item);
        }
    }

    Ok(())
}

fn detect_project_type(project_dir: &Path) -> Result<String> {
    let has_pom = project_dir.join("pom.xml").exists();
    let has_gradle = project_dir.join("build.gradle").exists();
    let has_settings_gradle = project_dir.join("settings.gradle").exists();

    if has_pom && (has_gradle || has_settings_gradle) {
        Ok("both".to_string())
    } else if has_pom {
        Ok("maven".to_string())
    } else if has_gradle || has_settings_gradle {
        Ok("gradle".to_string())
    } else {
        Ok("unknown".to_string())
    }
}

fn clean_temp_files(project_dir: &Path, cleaned_items: &mut Vec<String>) -> Result<()> {
    // 清理常见的临时文件
    let temp_patterns = [
        "*.tmp", "*.temp", "*.log", "*.cache", "*.bak", "*.swp", "*.swo",
    ];

    for pattern in &temp_patterns {
        let entries = glob::glob(&format!("{}/**/{}", project_dir.display(), pattern))
            .unwrap_or_else(|_| glob::glob("").unwrap());

        for entry in entries {
            if let Ok(path) = entry {
                if path.is_file() {
                    fs::remove_file(&path)?;
                    if let Some(file_name) = path.file_name() {
                        cleaned_items.push(format!("临时文件: {}", file_name.to_string_lossy()));
                    }
                }
            }
        }
    }

    Ok(())
}

fn clean_ide_files(project_dir: &Path, cleaned_items: &mut Vec<String>) -> Result<()> {
    // 清理IDE相关目录和文件
    let ide_dirs = [
        ".idea",
        ".vscode",
        ".eclipse",
        ".metadata",
        "bin",
        "out",
        "target",
        "build",
    ];

    for dir_name in &ide_dirs {
        let ide_path = project_dir.join(dir_name);
        if ide_path.exists() && ide_path.is_dir() {
            // 只清理IDE生成的目录，不清理项目构建目录
            if dir_name == &"target" || dir_name == &"build" {
                continue; // 这些已经在前面处理过了
            }

            fs::remove_dir_all(&ide_path)?;
            cleaned_items.push(format!("IDE目录: {}", dir_name));
        }
    }

    // 清理IDE配置文件
    let ide_files = [
        "*.iml",
        "*.ipr",
        "*.iws",
        ".project",
        ".classpath",
        ".settings",
        ".factorypath",
    ];

    for pattern in &ide_files {
        let entries = glob::glob(&format!("{}/**/{}", project_dir.display(), pattern))
            .unwrap_or_else(|_| glob::glob("").unwrap());

        for entry in entries {
            if let Ok(path) = entry {
                if path.is_file() {
                    fs::remove_file(&path)?;
                    if let Some(file_name) = path.file_name() {
                        cleaned_items.push(format!("IDE文件: {}", file_name.to_string_lossy()));
                    }
                }
            }
        }
    }

    Ok(())
}
