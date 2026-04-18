// src/environment.rs
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// 环境配置，包含 .jx/bin 路径和解析出的 HOME 目录
#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub bin_dir: PathBuf,
    pub java_home: Option<PathBuf>,
    pub maven_home: Option<PathBuf>,
    pub gradle_home: Option<PathBuf>,
}

/// 检测并准备项目环境
/// 返回: 环境准备成功时返回 Some(EnvConfig)，无 .jx/ 时返回 None
pub fn prepare_project_environment() -> Result<Option<EnvConfig>> {
    let current_dir = std::env::current_dir()?;
    let venv_dir = current_dir.join(".jx");

    // 不存在 .jx/ 时返回 None
    if !venv_dir.exists() {
        return Ok(None);
    }

    let bin_dir = venv_dir.join("bin");
    if !bin_dir.exists() {
        return Ok(None);
    }

    // 检查符号链接有效性
    let broken = check_symlink_validity(&bin_dir)?;

    if !broken.is_empty() {
        // 有损坏链接，触发自愈
        println!("⚠️ 检测到损坏的符号链接: {}", broken.join(", "));
        println!("正在自动修复...");
        heal_venv(&venv_dir)?;
        println!("✅ 修复完成");
    }

    // 解析 HOME 目录
    let java_home = resolve_java_home(&bin_dir);
    let maven_home = resolve_maven_home(&bin_dir);
    let gradle_home = resolve_gradle_home(&bin_dir);

    Ok(Some(EnvConfig {
        bin_dir,
        java_home,
        maven_home,
        gradle_home,
    }))
}

/// 检查符号链接有效性，返回损坏的工具名称列表
pub fn check_symlink_validity(bin_dir: &Path) -> Result<Vec<String>> {
    let mut broken = Vec::new();

    if !bin_dir.exists() {
        return Ok(broken);
    }

    for entry in fs::read_dir(bin_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_symlink() {
            let target = fs::read_link(&path)?;
            if !target.exists() {
                broken.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }

    Ok(broken)
}

/// 从 java 符号链接解析 JAVA_HOME
fn resolve_java_home(bin_dir: &Path) -> Option<PathBuf> {
    let java_symlink = bin_dir.join("java");
    if java_symlink.is_symlink() {
        let target = fs::read_link(&java_symlink).ok()?;
        // macOS: .../Contents/Home/bin/java -> Contents/Home
        // Linux: .../bin/java -> 父目录
        if target.ends_with("Contents/Home/bin/java") {
            Some(target.parent().unwrap().parent().unwrap().to_path_buf())
        } else if target.ends_with("bin/java") {
            Some(target.parent().unwrap().parent().unwrap().to_path_buf())
        } else {
            None
        }
    } else {
        None
    }
}

/// 从 mvn 符号链接解析 MAVEN_HOME
fn resolve_maven_home(bin_dir: &Path) -> Option<PathBuf> {
    let mvn_symlink = bin_dir.join("mvn");
    if mvn_symlink.is_symlink() {
        let target = fs::read_link(&mvn_symlink).ok()?;
        // .../bin/mvn -> 父目录
        if target.ends_with("bin/mvn") {
            Some(target.parent().unwrap().parent().unwrap().to_path_buf())
        } else {
            None
        }
    } else {
        None
    }
}

/// 从 gradle 符号链接解析 GRADLE_HOME
fn resolve_gradle_home(bin_dir: &Path) -> Option<PathBuf> {
    let gradle_symlink = bin_dir.join("gradle");
    if gradle_symlink.is_symlink() {
        let target = fs::read_link(&gradle_symlink).ok()?;
        // .../bin/gradle -> 父目录
        if target.ends_with("bin/gradle") {
            Some(target.parent().unwrap().parent().unwrap().to_path_buf())
        } else {
            None
        }
    } else {
        None
    }
}

/// 将 bin_dir 添加到 PATH 最前面
pub fn prepend_to_path(bin_dir: &Path) -> String {
    let current_path = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", bin_dir.display(), current_path)
}

/// 自愈 venv 环境（存根，将在 Task 2 实现）
fn heal_venv(_venv_dir: &Path) -> Result<()> {
    // TODO: Task 2 将实现完整的自愈逻辑
    Ok(())
}