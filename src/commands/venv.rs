use crate::utils::{calculate_directory_size, format_file_size};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::io::AsyncWriteExt;

// Adoptium API 数据结构
#[derive(Debug, Deserialize)]
struct AdoptiumBinary {
    architecture: String,
    os: String,
    image_type: String,
    package: AdoptiumPackage,
}

#[derive(Debug, Deserialize)]
struct AdoptiumPackage {
    name: String,
    link: String,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    download_count: u64,
    #[allow(dead_code)]
    checksum: Option<String>,
    #[allow(dead_code)]
    signature_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdoptiumRelease {
    binary: AdoptiumBinary,
    #[allow(dead_code)]
    release_name: String,
    #[allow(dead_code)]
    release_link: String,
    #[allow(dead_code)]
    vendor: String,
    #[allow(dead_code)]
    version: AdoptiumVersion,
}

#[derive(Debug, Deserialize)]
struct AdoptiumVersion {
    #[allow(dead_code)]
    major: u8,
    #[allow(dead_code)]
    minor: u8,
    #[allow(dead_code)]
    security: u8,
    #[allow(dead_code)]
    build: u8,
    #[allow(dead_code)]
    openjdk_version: String,
    #[allow(dead_code)]
    semver: String,
}

/// 构建工具类型
#[derive(Debug, Clone)]
pub enum BuildTool {
    Maven(String),
    Gradle(String),
}

/// 创建Java虚拟环境
pub async fn create(
    name: Option<String>,
    java_version: String,
    build_tool: BuildTool,
) -> Result<()> {
    let venv_name = name.unwrap_or_else(|| "default".to_string());
    let venv_dir = get_venv_directory(&venv_name)?;

    println!("🌱 创建Java虚拟环境...");
    println!("名称: {}", venv_name);
    println!("Java版本: {}", java_version);
    match &build_tool {
        BuildTool::Maven(version) => println!("Maven版本: {}", version),
        BuildTool::Gradle(version) => println!("Gradle版本: {}", version),
    }

    // 检查虚拟环境是否已存在
    if venv_dir.exists() {
        return Err(anyhow::anyhow!("虚拟环境 '{}' 已存在", venv_name));
    }

    // 创建虚拟环境目录结构
    fs::create_dir_all(&venv_dir)?;
    fs::create_dir_all(venv_dir.join("bin"))?;
    fs::create_dir_all(venv_dir.join("lib"))?;
    fs::create_dir_all(venv_dir.join("conf"))?;
    fs::create_dir_all(venv_dir.join("cache"))?;

    // 将后续安装流程包裹在可回滚的流程中
    let result: Result<()> = async {
        // 创建虚拟环境配置文件
        create_venv_config(&venv_dir, &java_version, &build_tool)?;

        // 下载并安装Java
        install_java(&venv_dir, &java_version).await?;

        // 根据构建工具类型安装相应的构建工具
        match &build_tool {
            BuildTool::Maven(version) => {
                install_maven(&venv_dir, version).await?;
            }
            BuildTool::Gradle(version) => {
                install_gradle(&venv_dir, version).await?;
            }
        }

        // 创建激活脚本
        create_activation_scripts(&venv_dir, &venv_name, &build_tool)?;

        Ok(())
    }
    .await;

    if let Err(err) = result {
        // 安装或下载失败时，尝试清理已创建的虚拟环境目录
        eprintln!("❌ 虚拟环境创建失败: {}", err);
        eprintln!("🧹 正在回滚并删除未完成的虚拟环境: {}", venv_dir.display());
        let _ = fs::remove_dir_all(&venv_dir);
        return Err(err);
    }

    println!("✅ 虚拟环境 '{}' 创建成功!", venv_name);
    println!("路径: {}", venv_dir.display());
    println!("");
    println!("激活虚拟环境:");
    println!("  jx venv activate {}", venv_name);
    println!("");
    println!("停用虚拟环境:");
    println!("  jx venv deactivate");

    Ok(())
}

/// 激活虚拟环境
pub fn activate(name: Option<String>) -> Result<()> {
    let venv_name = name.unwrap_or_else(|| "default".to_string());
    let venv_dir = get_venv_directory(&venv_name)?;

    if !venv_dir.exists() {
        return Err(anyhow::anyhow!("虚拟环境 '{}' 不存在", venv_name));
    }

    println!("🔌 激活虚拟环境 '{}'...", venv_name);

    // 设置环境变量
    let bin_path = venv_dir.join("bin");

    // 获取当前PATH
    let current_path = env::var("PATH").unwrap_or_default();
    let _new_path = format!("{}:{}", bin_path.display(), current_path);

    // 设置JAVA_HOME
    let java_home = venv_dir.join("lib").join("java");
    if java_home.exists() {
        // 检查是否是macOS结构
        let java_home_path = if java_home.join("jdk").join("Contents").join("Home").exists() {
            java_home.join("jdk").join("Contents").join("Home")
        } else {
            java_home.join("jdk")
        };
        env::set_var("JAVA_HOME", java_home_path.clone());
        println!("设置 JAVA_HOME: {}", java_home_path.display());
    }

    // 设置MAVEN_HOME
    let maven_home = venv_dir.join("lib").join("maven");
    if maven_home.exists() {
        env::set_var("MAVEN_HOME", maven_home.clone());
        println!("设置 MAVEN_HOME: {}", maven_home.display());
    }

    // 设置GRADLE_HOME
    let gradle_home = venv_dir.join("lib").join("gradle");
    if gradle_home.exists() {
        env::set_var("GRADLE_HOME", gradle_home.clone());
        println!("设置 GRADLE_HOME: {}", gradle_home.display());
    }

    // 创建激活状态文件
    let activation_file = get_jx_home()?.join(".active_venv");
    fs::write(activation_file, &venv_name)?;

    println!("✅ 虚拟环境 '{}' 已激活", venv_name);
    println!("");
    println!("注意: 环境变量已设置，但仅对当前shell会话有效");
    println!(
        "要永久激活，请运行: source {}/bin/activate",
        venv_dir.display()
    );

    Ok(())
}

/// 停用虚拟环境
pub fn deactivate() -> Result<()> {
    let activation_file = get_jx_home()?.join(".active_venv");

    if !activation_file.exists() {
        println!("⚠️ 当前没有激活的虚拟环境");
        return Ok(());
    }

    let active_venv = fs::read_to_string(&activation_file)?;
    println!("🔌 停用虚拟环境 '{}'...", active_venv.trim());

    // 删除激活状态文件
    fs::remove_file(activation_file)?;

    // 清除环境变量
    env::remove_var("JAVA_HOME");
    env::remove_var("MAVEN_HOME");
    env::remove_var("GRADLE_HOME");

    println!("✅ 虚拟环境已停用");
    println!("");
    println!("注意: 环境变量已清除，但仅对当前shell会话有效");
    println!("要完全清除，请重新启动shell或手动清除环境变量");

    Ok(())
}

/// 列出所有虚拟环境
pub fn list() -> Result<()> {
    let venv_base = get_venv_base_directory()?;

    if !venv_base.exists() {
        println!("📁 没有找到虚拟环境目录");
        return Ok(());
    }

    println!("📋 可用的虚拟环境:");
    println!("");

    let mut venvs = Vec::new();
    for entry in fs::read_dir(&venv_base)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap().to_string_lossy();
            let config_file = path.join("conf").join("venv.toml");

            if config_file.exists() {
                if let Ok(config_content) = fs::read_to_string(&config_file) {
                    let mut java_version = "未知".to_string();

                    let mut build_tool_info = "未知".to_string();

                    for line in config_content.lines() {
                        if line.starts_with("java_version = \"") {
                            java_version = line
                                .trim_start_matches("java_version = \"")
                                .trim_end_matches("\"")
                                .to_string();
                        } else if line.starts_with("build_tool = \"") {
                            let tool_type = line
                                .trim_start_matches("build_tool = \"")
                                .trim_end_matches("\"")
                                .to_string();
                            if tool_type == "maven" {
                                build_tool_info = format!(
                                    "Maven: {}",
                                    config_content
                                        .lines()
                                        .find(|l| l.starts_with("build_tool_version = \""))
                                        .map(|l| l
                                            .trim_start_matches("build_tool_version = \"")
                                            .trim_end_matches("\""))
                                        .unwrap_or("未知")
                                );
                            } else if tool_type == "gradle" {
                                build_tool_info = format!(
                                    "Gradle: {}",
                                    config_content
                                        .lines()
                                        .find(|l| l.starts_with("build_tool_version = \""))
                                        .map(|l| l
                                            .trim_start_matches("build_tool_version = \"")
                                            .trim_end_matches("\""))
                                        .unwrap_or("未知")
                                );
                            }
                        }
                    }

                    venvs.push((name.to_string(), java_version, build_tool_info));
                }
            }
        }
    }

    if venvs.is_empty() {
        println!("  没有找到虚拟环境");
    } else {
        // 检查当前激活的虚拟环境
        let active_venv = get_active_venv()?;

        for (name, java, build_tool) in venvs {
            let status = if active_venv.as_ref().map(|s| s == &name).unwrap_or(false) {
                "🔌 激活"
            } else {
                "   "
            };
            println!("{} {} (Java: {}, {})", status, name, java, build_tool);
        }
    }

    Ok(())
}

/// 删除虚拟环境
pub fn remove(name: String) -> Result<()> {
    let venv_dir = get_venv_directory(&name)?;

    if !venv_dir.exists() {
        return Err(anyhow::anyhow!("虚拟环境 '{}' 不存在", name));
    }

    // 检查是否正在使用
    let active_venv = get_active_venv()?;
    if active_venv.as_ref().map(|s| s == &name).unwrap_or(false) {
        return Err(anyhow::anyhow!(
            "无法删除正在使用的虚拟环境 '{}'，请先停用",
            name
        ));
    }

    println!("🗑️ 删除虚拟环境 '{}'...", name);
    println!("路径: {}", venv_dir.display());

    // 递归删除目录
    fs::remove_dir_all(&venv_dir)?;

    println!("✅ 虚拟环境 '{}' 已删除", name);

    Ok(())
}

/// 显示虚拟环境信息
pub fn info(name: Option<String>) -> Result<()> {
    let venv_name = name.unwrap_or_else(|| {
        get_active_venv()
            .unwrap_or(None)
            .unwrap_or_else(|| "default".to_string())
    });

    let venv_dir = get_venv_directory(&venv_name)?;

    if !venv_dir.exists() {
        return Err(anyhow::anyhow!("虚拟环境 '{}' 不存在", venv_name));
    }

    println!("ℹ️ 虚拟环境信息: {}", venv_name);
    println!("");

    // 基本信息
    println!("路径: {}", venv_dir.display());

    let metadata = fs::metadata(&venv_dir)?;
    if let Ok(created) = metadata.created() {
        let datetime = chrono::DateTime::<chrono::Utc>::from(created);
        println!("创建时间: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
    }

    // 配置信息
    let config_file = venv_dir.join("conf").join("venv.toml");
    if config_file.exists() {
        let config_content = fs::read_to_string(&config_file)?;
        println!("");
        println!("配置:");
        for line in config_content.lines() {
            if line.trim().starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if line.contains('=') {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim().trim_matches('"');
                    println!("  {}: {}", key, value);
                }
            }
        }
    }

    // 状态信息
    let active_venv = get_active_venv()?;
    let is_active = active_venv
        .as_ref()
        .map(|s| s == &venv_name)
        .unwrap_or(false);
    println!("");
    println!(
        "状态: {}",
        if is_active {
            "🔌 激活"
        } else {
            "  未激活"
        }
    );

    // 磁盘使用情况
    if let Ok(size) = calculate_directory_size(&venv_dir) {
        println!("大小: {}", format_file_size(size));
    }

    Ok(())
}

// 辅助函数

fn get_jx_home() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    let jx_home = home.join(".jx");
    fs::create_dir_all(&jx_home)?;
    Ok(jx_home)
}

fn get_cache_directory() -> Result<PathBuf> {
    let jx_home = get_jx_home()?;
    let cache_dir = jx_home.join("cache");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst.parent().unwrap())?;

    // 使用cp命令递归复制目录
    let output = Command::new("cp")
        .args(&["-R", src.to_str().unwrap(), dst.to_str().unwrap()])
        .output()
        .context("复制目录失败")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("复制目录失败: {}", error));
    }

    Ok(())
}

fn rename_extracted_java(extract_dir: &Path, target_dir: &Path) -> Result<()> {
    // 查找解压后的JDK目录
    for entry in fs::read_dir(extract_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("jdk")
        {
            if target_dir.exists() {
                fs::remove_dir_all(target_dir)?;
            }
            fs::rename(path, target_dir)?;
            return Ok(());
        }
    }

    Err(anyhow::anyhow!("未找到解压后的JDK目录"))
}

fn rename_extracted_maven(extract_dir: &Path, target_dir: &Path) -> Result<()> {
    // 查找解压后的Maven目录
    for entry in fs::read_dir(extract_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("apache-maven")
        {
            if target_dir.exists() {
                fs::remove_dir_all(target_dir)?;
            }
            fs::rename(path, target_dir)?;
            return Ok(());
        }
    }

    Err(anyhow::anyhow!("未找到解压后的Maven目录"))
}

fn rename_extracted_gradle(extract_dir: &Path, target_dir: &Path) -> Result<()> {
    // 查找解压后的Gradle目录
    for entry in fs::read_dir(extract_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("gradle-")
        {
            if target_dir.exists() {
                fs::remove_dir_all(target_dir)?;
            }
            fs::rename(path, target_dir)?;
            return Ok(());
        }
    }

    Err(anyhow::anyhow!("未找到解压后的Gradle目录"))
}

fn get_venv_base_directory() -> Result<PathBuf> {
    let jx_home = get_jx_home()?;
    let venv_base = jx_home.join("venvs");
    fs::create_dir_all(&venv_base)?;
    Ok(venv_base)
}

fn get_venv_directory(name: &str) -> Result<PathBuf> {
    let venv_base = get_venv_base_directory()?;
    Ok(venv_base.join(name))
}

fn get_active_venv() -> Result<Option<String>> {
    let activation_file = get_jx_home()?.join(".active_venv");
    if activation_file.exists() {
        let content = fs::read_to_string(&activation_file)?;
        Ok(Some(content.trim().to_string()))
    } else {
        Ok(None)
    }
}

fn create_venv_config(venv_dir: &Path, java_version: &str, build_tool: &BuildTool) -> Result<()> {
    let (tool_type, tool_version) = match build_tool {
        BuildTool::Maven(version) => ("maven", version),
        BuildTool::Gradle(version) => ("gradle", version),
    };

    let config_content = format!(
        r#"# jx虚拟环境配置文件
# 创建时间: {}
java_version = "{}"
build_tool = "{}"
build_tool_version = "{}"

[paths]
bin = "bin"
lib = "lib"
conf = "conf"
cache = "cache"
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        java_version,
        tool_type,
        tool_version
    );

    let config_file = venv_dir.join("conf").join("venv.toml");
    fs::write(config_file, config_content)?;

    Ok(())
}

async fn install_java(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Java {}...", version);

    let java_dir = venv_dir.join("lib").join("java");
    fs::create_dir_all(&java_dir)?;

    // 检查是否已经安装了指定版本的Java
    let java_bin = get_java_executable_path(&java_dir);
    if java_bin.exists() {
        // 检查版本
        if let Ok(output) = Command::new(&java_bin).arg("-version").output() {
            let version_output = String::from_utf8_lossy(&output.stderr);
            if version_output.contains(&format!("version \"{}", version)) {
                println!("✅ Java {} 已安装", version);
                return Ok(());
            }
        }
    }

    // 确定Java版本和架构
    let (major_version, arch) = parse_java_version(version)?;
    let os = get_os_type()?;

    // 构建下载URL
    let download_url = build_java_download_url(major_version, &arch, &os)?;
    let filename = get_java_filename_from_url(&download_url)?;

    // 检查缓存目录
    let cache_dir = get_cache_directory()?;
    let java_cache_dir = cache_dir.join("java");
    fs::create_dir_all(&java_cache_dir)?;
    let cached_archive = java_cache_dir.join(&filename);
    let cached_extracted = java_cache_dir.join(format!("jdk-{}-{}-{}", major_version, os, arch));

    // 如果缓存中已存在解压后的目录，直接复制
    if cached_extracted.exists() {
        println!("📋 从缓存复制Java {}...", major_version);
        copy_directory(&cached_extracted, &java_dir.join("jdk"))?;
    } else {
        // 检查是否有缓存的压缩包
        if cached_archive.exists() {
            println!("📋 从缓存解压Java {}...", major_version);
            extract_java_archive(&cached_archive, &java_cache_dir, &filename)?;
            // 重命名解压后的目录
            rename_extracted_java(&java_cache_dir, &cached_extracted)?;
            // 复制到目标目录
            copy_directory(&cached_extracted, &java_dir.join("jdk"))?;
        } else {
            // 下载Java
            println!("🌐 从 {} 下载Java...", download_url);
            download_file(&download_url, &cached_archive).await?;

            // 解压到缓存目录
            println!("📦 解压Java到缓存...");
            extract_java_archive(&cached_archive, &java_cache_dir, &filename)?;
            // 重命名解压后的目录
            rename_extracted_java(&java_cache_dir, &cached_extracted)?;
            // 复制到目标目录
            copy_directory(&cached_extracted, &java_dir.join("jdk"))?;
        }
    }

    // 设置执行权限
    set_java_permissions(&java_dir)?;

    // 创建符号链接到bin目录
    let bin_dir = venv_dir.join("bin");
    create_java_symlinks(&java_dir, &bin_dir)?;

    // 验证安装
    let java_bin = get_java_executable_path(&java_dir);
    if let Ok(output) = Command::new(&java_bin).arg("-version").output() {
        let version_output = String::from_utf8_lossy(&output.stderr);
        println!("✅ Java安装成功!");
        println!("版本信息: {}", version_output.lines().next().unwrap_or(""));
    } else {
        return Err(anyhow::anyhow!("Java安装验证失败"));
    }

    // 缓存文件保留，无需清理

    Ok(())
}

fn parse_java_version(version: &str) -> Result<(u8, String)> {
    // 支持多种版本格式：8, 11, 17, 21, 1.8, 11.0, 17.0, 21.0等
    let major_version = if version.starts_with("1.") {
        // 处理1.x格式（如1.8）
        let minor = version.strip_prefix("1.").unwrap();
        minor
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    } else if version.contains('.') {
        // 处理x.y格式（如11.0, 17.0）
        let major = version.split('.').next().unwrap();
        major
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    } else {
        // 处理单个数字格式（如8, 11, 17, 21）
        version
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    };

    // 验证版本是否支持
    if major_version < 8 || major_version > 25 {
        return Err(anyhow::anyhow!(
            "不支持的Java版本: {} (支持范围: 8-25)",
            major_version
        ));
    }

    // 获取系统架构
    let arch = if cfg!(target_arch = "x86_64") {
        "x64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "aarch64".to_string()
    } else if cfg!(target_arch = "arm") {
        "arm".to_string()
    } else {
        return Err(anyhow::anyhow!("不支持的架构: {}", std::env::consts::ARCH));
    };

    Ok((major_version, arch))
}

fn get_os_type() -> Result<String> {
    let os = if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "mac".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        return Err(anyhow::anyhow!(
            "不支持的操作系统: {}",
            std::env::consts::OS
        ));
    };

    Ok(os)
}

fn get_java_executable_path(java_dir: &Path) -> PathBuf {
    let jdk_dir = java_dir.join("jdk");

    // 检查macOS结构 (Contents/Home/bin/java)
    let macos_java = jdk_dir
        .join("Contents")
        .join("Home")
        .join("bin")
        .join("java");
    if macos_java.exists() {
        return macos_java;
    }

    // 检查标准结构 (jdk/bin/java)
    let standard_java = jdk_dir.join("bin").join("java");
    if standard_java.exists() {
        return standard_java;
    }

    // 默认返回标准结构
    standard_java
}

fn get_adoptium_releases(version: u8) -> Result<Vec<AdoptiumRelease>> {
    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/{}/hotspot",
        version
    );

    // 使用curl命令获取API响应
    let output = Command::new("curl")
        .args(&["-s", "-H", "User-Agent: jx/0.1.0", &url])
        .output()
        .context("执行curl命令失败")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Adoptium API请求失败: {}", error));
    }

    let response_text = String::from_utf8_lossy(&output.stdout);
    let adoptium_releases: Vec<AdoptiumRelease> =
        serde_json::from_str(&response_text).context("解析Adoptium API响应失败")?;

    Ok(adoptium_releases)
}

fn build_java_download_url(major_version: u8, arch: &str, os: &str) -> Result<String> {
    // 获取Adoptium API数据
    let releases = get_adoptium_releases(major_version)?;

    if releases.is_empty() {
        return Err(anyhow::anyhow!("未找到Java {}的可用版本", major_version));
    }

    // 查找匹配的发布版本
    for release in &releases {
        let binary = &release.binary;

        // 检查操作系统和架构是否匹配
        let os_match = match (os, binary.os.as_str()) {
            ("linux", "linux") => true,
            ("mac", "mac") => true,
            ("windows", "windows") => true,
            _ => false,
        };

        let arch_match = match (arch, binary.architecture.as_str()) {
            ("x64", "x64") => true,
            ("aarch64", "aarch64") => true,
            ("arm", "arm") => true,
            _ => false,
        };

        // 检查是否是JDK包（不是JRE）
        let is_jdk = binary.image_type == "jdk";

        if os_match && arch_match && is_jdk {
            // 根据操作系统选择正确的文件扩展名
            let expected_extension = if os == "windows" { "zip" } else { "tar.gz" };
            if binary.package.name.ends_with(expected_extension) {
                return Ok(binary.package.link.clone());
            }
        }
    }

    Err(anyhow::anyhow!(
        "未找到适合 {}-{} 的Java {}下载链接",
        os,
        arch,
        major_version
    ))
}

fn get_java_filename_from_url(url: &str) -> Result<String> {
    // 从URL中提取文件名
    if let Some(last_slash) = url.rfind('/') {
        let filename = &url[last_slash + 1..];
        // 解码URL编码的字符
        let decoded =
            urlencoding::decode(filename).map_err(|e| anyhow::anyhow!("URL解码失败: {}", e))?;
        Ok(decoded.to_string())
    } else {
        Err(anyhow::anyhow!("无法从URL中提取文件名: {}", url))
    }
}

async fn download_file(url: &str, path: &Path) -> Result<()> {
    println!("下载: {}", url);

    // 创建HTTP客户端
    let client = reqwest::Client::new();

    // 发送GET请求
    let response = client.get(url).send().await.context("发送HTTP请求失败")?;

    // 检查响应状态
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP请求失败，状态码: {}",
            response.status()
        ));
    }

    // 获取文件大小
    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("无法获取文件大小"))?;

    // 创建进度条
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );
    pb.set_message(format!("下载文件"));

    // 创建文件
    let mut file = tokio::fs::File::create(path)
        .await
        .context("创建文件失败")?;

    // 下载并写入文件
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.context("下载数据失败")?;
        file.write_all(&chunk).await.context("写入文件失败")?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    // 关闭文件
    file.flush().await.context("刷新文件缓冲区失败")?;

    // 完成进度条
    pb.finish_with_message(format!("下载完成"));

    println!("下载完成");

    Ok(())
}

fn extract_java_archive(archive_path: &Path, target_dir: &Path, filename: &str) -> Result<()> {
    if filename.ends_with(".tar.gz") {
        // 解压tar.gz文件
        let output = Command::new("tar")
            .args(&[
                "-xzf",
                archive_path.to_str().unwrap(),
                "-C",
                target_dir.to_str().unwrap(),
            ])
            .output()
            .context("解压tar.gz文件失败")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("解压失败: {}", error));
        }

        // 重命名解压后的目录
        let entries: Vec<_> = fs::read_dir(target_dir)?.collect();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("jdk")
            {
                let new_path = target_dir.join("jdk");
                if new_path.exists() {
                    fs::remove_dir_all(&new_path)?;
                }
                fs::rename(path, new_path)?;
                break;
            }
        }
    } else if filename.ends_with(".zip") {
        // 解压zip文件
        let output = Command::new("unzip")
            .args(&[
                "-q",
                archive_path.to_str().unwrap(),
                "-d",
                target_dir.to_str().unwrap(),
            ])
            .output()
            .context("解压zip文件失败")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("解压zip文件失败: {}", error));
        }

        // 重命名解压后的目录
        let entries: Vec<_> = fs::read_dir(target_dir)?.collect();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("jdk")
            {
                let new_path = target_dir.join("jdk");
                if new_path.exists() {
                    fs::remove_dir_all(&new_path)?;
                }
                fs::rename(path, new_path)?;
                break;
            }
        }
    } else {
        return Err(anyhow::anyhow!("不支持的压缩格式: {}", filename));
    }

    Ok(())
}

fn set_java_permissions(java_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = java_dir.join("jdk").join("bin");
        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let mut perms = fs::metadata(&path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&path, perms)?;
                }
            }
        }
    }

    Ok(())
}

fn create_java_symlinks(java_dir: &Path, bin_dir: &Path) -> Result<()> {
    let jdk_dir = java_dir.join("jdk");

    // 查找Java bin目录
    let jdk_bin = if jdk_dir.join("Contents").join("Home").join("bin").exists() {
        // macOS结构
        jdk_dir.join("Contents").join("Home").join("bin")
    } else if jdk_dir.join("bin").exists() {
        // 标准结构
        jdk_dir.join("bin")
    } else {
        return Err(anyhow::anyhow!("Java bin目录不存在"));
    };

    // 创建常用Java命令的符号链接
    let java_commands = ["java", "javac", "javadoc", "jar", "keytool"];

    for command in &java_commands {
        let source = jdk_bin.join(command);
        let target = bin_dir.join(command);

        if source.exists() {
            if target.exists() {
                fs::remove_file(&target)?;
            }

            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&source, &target)?;
            }

            #[cfg(windows)]
            {
                // Windows上复制文件而不是创建符号链接
                fs::copy(&source, &target)?;
            }
        }
    }

    Ok(())
}

async fn install_maven(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Maven {}...", version);

    let maven_dir = venv_dir.join("lib").join("maven");
    fs::create_dir_all(&maven_dir)?;

    // 检查是否已经安装了指定版本的Maven
    let maven_version_dir = maven_dir.join(format!("apache-maven-{}", version));
    let mvn_bin = maven_version_dir.join("bin").join("mvn");
    if mvn_bin.exists() {
        if let Ok(output) = Command::new(&mvn_bin).arg("--version").output() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if version_output.contains(&format!("Apache Maven {}", version)) {
                println!("✅ Maven {} 已安装", version);
                return Ok(());
            }
        }
    }

    // 构建Maven下载URL
    let download_url = format!(
        "https://archive.apache.org/dist/maven/maven-3/{}/binaries/apache-maven-{}-bin.tar.gz",
        version, version
    );

    // 检查缓存目录
    let cache_dir = get_cache_directory()?;
    let maven_cache_dir = cache_dir.join("maven");
    fs::create_dir_all(&maven_cache_dir)?;
    let filename = format!("apache-maven-{}-bin.tar.gz", version);
    let cached_archive = maven_cache_dir.join(&filename);
    let cached_extracted = maven_cache_dir.join(format!("apache-maven-{}", version));

    // 如果缓存中已存在解压后的目录，直接复制
    if cached_extracted.exists() {
        println!("📋 从缓存复制Maven {}...", version);
        copy_directory(
            &cached_extracted,
            &maven_dir.join(format!("apache-maven-{}", version)),
        )?;
    } else {
        // 检查是否有缓存的压缩包
        if cached_archive.exists() {
            println!("📋 从缓存解压Maven {}...", version);
            let output = Command::new("tar")
                .args(&[
                    "-xzf",
                    cached_archive.to_str().unwrap(),
                    "-C",
                    maven_cache_dir.to_str().unwrap(),
                ])
                .output()
                .context("解压Maven失败")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("解压Maven失败: {}", error));
            }

            // 重命名解压后的目录
            rename_extracted_maven(&maven_cache_dir, &cached_extracted)?;
            // 复制到目标目录
            copy_directory(
                &cached_extracted,
                &maven_dir.join(format!("apache-maven-{}", version)),
            )?;
        } else {
            // 下载Maven
            println!("🌐 从 {} 下载Maven...", download_url);
            download_file(&download_url, &cached_archive).await?;

            // 解压到缓存目录
            println!("📦 解压Maven到缓存...");
            let output = Command::new("tar")
                .args(&[
                    "-xzf",
                    cached_archive.to_str().unwrap(),
                    "-C",
                    maven_cache_dir.to_str().unwrap(),
                ])
                .output()
                .context("解压Maven失败")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("解压Maven失败: {}", error));
            }

            // 重命名解压后的目录
            rename_extracted_maven(&maven_cache_dir, &cached_extracted)?;
            // 复制到目标目录
            copy_directory(
                &cached_extracted,
                &maven_dir.join(format!("apache-maven-{}", version)),
            )?;
        }
    }

    // 设置执行权限
    set_maven_permissions(&maven_dir)?;

    // 创建符号链接到bin目录
    let bin_dir = venv_dir.join("bin");
    create_maven_symlinks(&maven_dir, &bin_dir)?;

    // 验证安装
    let maven_version_dir = maven_dir.join(format!("apache-maven-{}", version));
    let mvn_bin = maven_version_dir.join("bin").join("mvn");
    if let Ok(output) = Command::new(&mvn_bin).arg("--version").output() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        println!("✅ Maven安装成功!");
        println!("版本信息: {}", version_output.lines().next().unwrap_or(""));
    } else {
        return Err(anyhow::anyhow!("Maven安装验证失败"));
    }

    // 缓存文件保留，无需清理

    Ok(())
}

fn set_maven_permissions(maven_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = maven_dir.join("apache-maven").join("bin");
        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let mut perms = fs::metadata(&path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&path, perms)?;
                }
            }
        }
    }

    Ok(())
}

fn create_maven_symlinks(maven_dir: &Path, bin_dir: &Path) -> Result<()> {
    // 查找Maven bin目录
    let mut maven_bin = None;

    // 查找apache-maven-*目录
    for entry in fs::read_dir(maven_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("apache-maven")
        {
            let bin_path = path.join("bin");
            if bin_path.exists() {
                maven_bin = Some(bin_path);
                break;
            }
        }
    }

    let maven_bin = maven_bin.ok_or_else(|| anyhow::anyhow!("Maven bin目录不存在"))?;

    // 创建常用Maven命令的符号链接
    let maven_commands = ["mvn"];

    for command in &maven_commands {
        let source = maven_bin.join(command);
        let target = bin_dir.join(command);

        if source.exists() {
            if target.exists() {
                fs::remove_file(&target)?;
            }

            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&source, &target)?;
            }

            #[cfg(windows)]
            {
                // Windows上复制文件而不是创建符号链接
                fs::copy(&source, &target)?;
            }
        }
    }

    Ok(())
}

async fn install_gradle(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Gradle {}...", version);

    let gradle_dir = venv_dir.join("lib").join("gradle");
    fs::create_dir_all(&gradle_dir)?;

    // 检查是否已经安装了指定版本的Gradle
    let gradle_version_dir = gradle_dir.join(format!("gradle-{}", version));
    let gradle_bin = gradle_version_dir.join("bin").join("gradle");
    if gradle_bin.exists() {
        if let Ok(output) = Command::new(&gradle_bin).arg("--version").output() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if version_output.contains(&format!("Gradle {}", version)) {
                println!("✅ Gradle {} 已安装", version);
                return Ok(());
            }
        }
    }

    // 构建Gradle下载URL
    let download_url = format!(
        "https://services.gradle.org/distributions/gradle-{}-bin.zip",
        version
    );

    // 检查缓存目录
    let cache_dir = get_cache_directory()?;
    let gradle_cache_dir = cache_dir.join("gradle");
    fs::create_dir_all(&gradle_cache_dir)?;
    let filename = format!("gradle-{}-bin.zip", version);
    let cached_archive = gradle_cache_dir.join(&filename);
    let cached_extracted = gradle_cache_dir.join(format!("gradle-{}", version));

    // 如果缓存中已存在解压后的目录，直接复制
    if cached_extracted.exists() {
        println!("📋 从缓存复制Gradle {}...", version);
        copy_directory(
            &cached_extracted,
            &gradle_dir.join(format!("gradle-{}", version)),
        )?;
    } else {
        // 检查是否有缓存的压缩包
        if cached_archive.exists() {
            println!("📋 从缓存解压Gradle {}...", version);
            let output = Command::new("unzip")
                .args(&[
                    "-q",
                    cached_archive.to_str().unwrap(),
                    "-d",
                    gradle_cache_dir.to_str().unwrap(),
                ])
                .output()
                .context("解压Gradle失败")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("解压Gradle失败: {}", error));
            }

            // 重命名解压后的目录
            rename_extracted_gradle(&gradle_cache_dir, &cached_extracted)?;
            // 复制到目标目录
            copy_directory(
                &cached_extracted,
                &gradle_dir.join(format!("gradle-{}", version)),
            )?;
        } else {
            // 下载Gradle
            println!("🌐 从 {} 下载Gradle...", download_url);
            download_file(&download_url, &cached_archive).await?;

            // 解压到缓存目录
            println!("📦 解压Gradle到缓存...");
            let output = Command::new("tar")
                .args(&[
                    "-xzf",
                    cached_archive.to_str().unwrap(),
                    "-C",
                    gradle_cache_dir.to_str().unwrap(),
                ])
                .output()
                .context("解压Gradle失败")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("解压Gradle失败: {}", error));
            }

            // 重命名解压后的目录
            rename_extracted_gradle(&gradle_cache_dir, &cached_extracted)?;
            // 复制到目标目录
            copy_directory(
                &cached_extracted,
                &gradle_dir.join(format!("gradle-{}", version)),
            )?;
        }
    }

    // 设置执行权限
    set_gradle_permissions(&gradle_dir)?;

    // 创建符号链接到bin目录
    let bin_dir = venv_dir.join("bin");
    create_gradle_symlinks(&gradle_dir, &bin_dir)?;

    // 验证安装
    let gradle_version_dir = gradle_dir.join(format!("gradle-{}", version));
    let gradle_bin = gradle_version_dir.join("bin").join("gradle");
    if let Ok(output) = Command::new(&gradle_bin).arg("--version").output() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        println!("✅ Gradle安装成功!");
        println!("版本信息: {}", version_output.lines().next().unwrap_or(""));
    } else {
        return Err(anyhow::anyhow!("Gradle安装验证失败"));
    }

    // 缓存文件保留，无需清理

    Ok(())
}

fn set_gradle_permissions(gradle_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = gradle_dir.join("gradle").join("bin");
        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let mut perms = fs::metadata(&path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&path, perms)?;
                }
            }
        }
    }

    Ok(())
}

fn create_gradle_symlinks(gradle_dir: &Path, bin_dir: &Path) -> Result<()> {
    // 查找Gradle bin目录
    let mut gradle_bin = None;

    // 查找gradle-*目录
    for entry in fs::read_dir(gradle_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("gradle-")
        {
            let bin_path = path.join("bin");
            if bin_path.exists() {
                gradle_bin = Some(bin_path);
                break;
            }
        }
    }

    let gradle_bin = gradle_bin.ok_or_else(|| anyhow::anyhow!("Gradle bin目录不存在"))?;

    // 创建常用Gradle命令的符号链接
    let gradle_commands = ["gradle"];

    for command in &gradle_commands {
        let source = gradle_bin.join(command);
        let target = bin_dir.join(command);

        if source.exists() {
            if target.exists() {
                fs::remove_file(&target)?;
            }

            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&source, &target)?;
            }

            #[cfg(windows)]
            {
                // Windows上复制文件而不是创建符号链接
                fs::copy(&source, &target)?;
            }
        }
    }

    Ok(())
}

fn create_activation_scripts(venv_dir: &Path, name: &str, build_tool: &BuildTool) -> Result<()> {
    // 根据构建工具类型生成不同的激活脚本
    let (tool_home_var, tool_home_path, tool_display) = match build_tool {
        BuildTool::Maven(version) => (
            "MAVEN_HOME",
            format!("{}/lib/maven", venv_dir.display()),
            format!("Maven: {}", version),
        ),
        BuildTool::Gradle(version) => (
            "GRADLE_HOME",
            format!("{}/lib/gradle", venv_dir.display()),
            format!("Gradle: {}", version),
        ),
    };

    // 创建bash激活脚本
    let bash_script = format!(
        r#"#!/bin/bash
# jx虚拟环境激活脚本: {}
export JX_VENV_NAME="{}"
export JX_VENV_PATH="{}"

# 设置Java环境
if [ -d "{}/lib/java/jdk/Contents/Home" ]; then
    export JAVA_HOME="{}/lib/java/jdk/Contents/Home"
else
    export JAVA_HOME="{}/lib/java/jdk"
fi
export PATH="{}/bin:$PATH"

# 设置{}环境
export {}="{}"
export PATH="{}/bin:$PATH"

# 显示激活信息
echo "🔌 虚拟环境 '{}' 已激活"
echo "Java: $JAVA_HOME"
echo "{}: ${}"
echo ""
echo "停用虚拟环境: deactivate"

# 定义停用函数
deactivate() {{
    unset JX_VENV_NAME
    unset JX_VENV_PATH
    unset JAVA_HOME
    unset {}
    echo "🔌 虚拟环境 '{}' 已停用"
}}
"#,
        name,
        name,
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        tool_display,
        tool_home_var,
        tool_home_path,
        venv_dir.display(),
        name,
        tool_display,
        tool_home_var,
        tool_home_var,
        name
    );

    let bash_file = venv_dir.join("bin").join("activate");
    fs::write(&bash_file, bash_script)?;

    // 设置执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&bash_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bash_file, perms)?;
    }

    Ok(())
}
