use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

/// 创建Java虚拟环境
pub fn create(name: Option<String>, java_version: String, maven_version: String, gradle_version: String) -> Result<()> {
    let venv_name = name.unwrap_or_else(|| "default".to_string());
    let venv_dir = get_venv_directory(&venv_name)?;
    
    println!("🌱 创建Java虚拟环境...");
    println!("名称: {}", venv_name);
    println!("Java版本: {}", java_version);
    println!("Maven版本: {}", maven_version);
    println!("Gradle版本: {}", gradle_version);
    
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
    
    // 创建虚拟环境配置文件
    create_venv_config(&venv_dir, &java_version, &maven_version, &gradle_version)?;
    
    // 下载并安装Java
    install_java(&venv_dir, &java_version)?;
    
    // 下载并安装Maven
    install_maven(&venv_dir, &maven_version)?;
    
    // 下载并安装Gradle
    install_gradle(&venv_dir, &gradle_version)?;
    
    // 创建激活脚本
    create_activation_scripts(&venv_dir, &venv_name)?;
    
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
    let lib_path = venv_dir.join("lib");
    
    // 获取当前PATH
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_path.display(), current_path);
    
    // 设置JAVA_HOME
    let java_home = venv_dir.join("lib").join("java");
    if java_home.exists() {
        env::set_var("JAVA_HOME", java_home.clone());
        println!("设置 JAVA_HOME: {}", java_home.display());
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
    println!("要永久激活，请运行: source {}/bin/activate", venv_dir.display());
    
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
                    let mut maven_version = "未知".to_string();
                    let mut gradle_version = "未知".to_string();
                    
                    for line in config_content.lines() {
                        if line.starts_with("java_version = \"") {
                            java_version = line.trim_start_matches("java_version = \"").trim_end_matches("\"").to_string();
                        } else if line.starts_with("maven_version = \"") {
                            maven_version = line.trim_start_matches("maven_version = \"").trim_end_matches("\"").to_string();
                        } else if line.starts_with("gradle_version = \"") {
                            gradle_version = line.trim_start_matches("gradle_version = \"").trim_end_matches("\"").to_string();
                        }
                    }
                    
                    venvs.push((name.to_string(), java_version, maven_version, gradle_version));
                }
            }
        }
    }
    
    if venvs.is_empty() {
        println!("  没有找到虚拟环境");
    } else {
        // 检查当前激活的虚拟环境
        let active_venv = get_active_venv()?;
        
        for (name, java, maven, gradle) in venvs {
            let status = if active_venv.as_ref().map(|s| s == &name).unwrap_or(false) {
                "🔌 激活"
            } else {
                "   "
            };
            println!("{} {} (Java: {}, Maven: {}, Gradle: {})", status, name, java, maven, gradle);
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
        return Err(anyhow::anyhow!("无法删除正在使用的虚拟环境 '{}'，请先停用", name));
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
        get_active_venv().unwrap_or(None).unwrap_or_else(|| "default".to_string())
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
    let is_active = active_venv.as_ref().map(|s| s == &venv_name).unwrap_or(false);
    println!("");
    println!("状态: {}", if is_active { "🔌 激活" } else { "  未激活" });
    
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

fn create_venv_config(venv_dir: &Path, java_version: &str, maven_version: &str, gradle_version: &str) -> Result<()> {
    let config_content = format!(
        r#"# jx虚拟环境配置文件
# 创建时间: {}
java_version = "{}"
maven_version = "{}"
gradle_version = "{}"

[paths]
bin = "bin"
lib = "lib"
conf = "conf"
cache = "cache"
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        java_version,
        maven_version,
        gradle_version
    );
    
    let config_file = venv_dir.join("conf").join("venv.toml");
    fs::write(config_file, config_content)?;
    
    Ok(())
}

fn install_java(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Java {}...", version);
    
    let java_dir = venv_dir.join("lib").join("java");
    fs::create_dir_all(&java_dir)?;
    
    // 检查是否已经安装了指定版本的Java
    let java_bin = java_dir.join("jdk").join("bin").join("java");
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
    let filename = get_java_filename(major_version, &arch, &os)?;
    
    println!("🌐 从 {} 下载Java...", download_url);
    
    // 下载Java
    let temp_dir = std::env::temp_dir().join("jx_java_download");
    fs::create_dir_all(&temp_dir)?;
    let download_path = temp_dir.join(&filename);
    
    download_file(&download_url, &download_path)?;
    
    // 解压Java
    println!("📦 解压Java...");
    extract_java_archive(&download_path, &java_dir, &filename)?;
    
    // 设置执行权限
    set_java_permissions(&java_dir)?;
    
    // 创建符号链接到bin目录
    let bin_dir = venv_dir.join("bin");
    create_java_symlinks(&java_dir, &bin_dir)?;
    
    // 验证安装
    if let Ok(output) = Command::new(&java_bin).arg("-version").output() {
        let version_output = String::from_utf8_lossy(&output.stderr);
        println!("✅ Java安装成功!");
        println!("版本信息: {}", version_output.lines().next().unwrap_or(""));
    } else {
        return Err(anyhow::anyhow!("Java安装验证失败"));
    }
    
    // 清理临时文件
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir)?;
    }
    
    Ok(())
}

fn parse_java_version(version: &str) -> Result<(u8, String)> {
    let major_version = version.parse::<u8>()
        .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?;
    
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
        return Err(anyhow::anyhow!("不支持的操作系统: {}", std::env::consts::OS));
    };
    
    Ok(os)
}

fn build_java_download_url(major_version: u8, arch: &str, os: &str) -> Result<String> {
    // 使用Adoptium (Eclipse Temurin) 作为Java发行版
    let base_url = "https://github.com/adoptium/temurin8-binaries/releases/download";
    
    let version_tag = match major_version {
        8 => "jdk8u392-b08",
        11 => "jdk-11.0.21+9",
        17 => "jdk-17.0.9+9",
        21 => "jdk-21.0.1+12",
        _ => return Err(anyhow::anyhow!("不支持的Java版本: {}", major_version)),
    };
    
    let os_arch = match (os, arch) {
        ("linux", "x64") => "linux-x64",
        ("linux", "aarch64") => "linux-aarch64",
        ("linux", "arm") => "linux-arm",
        ("mac", "x64") => "macosx-x64",
        ("mac", "aarch64") => "macosx-aarch64",
        ("windows", "x64") => "windows-x64",
        ("windows", "aarch64") => "windows-aarch64",
        _ => return Err(anyhow::anyhow!("不支持的OS-架构组合: {}-{}", os, arch)),
    };
    
    let extension = if os == "windows" { "zip" } else { "tar.gz" };
    
    // 修复URL格式：移除重复的版本号
    let url = format!(
        "{}/{}/OpenJDK{}U-{}.{}",
        base_url, version_tag, major_version, os_arch, extension
    );
    
    Ok(url)
}

fn get_java_filename(major_version: u8, arch: &str, os: &str) -> Result<String> {
    let os_arch = match (os, arch) {
        ("linux", "x64") => "linux-x64",
        ("linux", "aarch64") => "linux-aarch64",
        ("linux", "arm") => "linux-arm",
        ("mac", "x64") => "macosx-x64",
        ("mac", "aarch64") => "macosx-aarch64",
        ("windows", "x64") => "windows-x64",
        ("windows", "aarch64") => "windows-aarch64",
        _ => return Err(anyhow::anyhow!("不支持的OS-架构组合: {}-{}", os, arch)),
    };
    
    let extension = if os == "windows" { "zip" } else { "tar.gz" };
    let filename = format!("OpenJDK{}U-{}.{}", major_version, os_arch, extension);
    
    Ok(filename)
}

fn download_file(url: &str, path: &Path) -> Result<()> {
    // 使用curl下载文件
    let output = Command::new("curl")
        .args(&["-L", "-o", path.to_str().unwrap(), url])
        .output()
        .context("执行curl命令失败")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("下载失败: {}", error));
    }
    
    Ok(())
}

fn extract_java_archive(archive_path: &Path, target_dir: &Path, filename: &str) -> Result<()> {
    if filename.ends_with(".tar.gz") {
        // 解压tar.gz文件
        let output = Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap(), "-C", target_dir.to_str().unwrap()])
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
            if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("jdk") {
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
            .args(&["-q", archive_path.to_str().unwrap(), "-d", target_dir.to_str().unwrap()])
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
            if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("jdk") {
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
    let jdk_bin = java_dir.join("jdk").join("bin");
    
    if !jdk_bin.exists() {
        return Err(anyhow::anyhow!("Java bin目录不存在"));
    }
    
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

fn install_maven(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Maven {}...", version);
    
    let maven_dir = venv_dir.join("lib").join("maven");
    fs::create_dir_all(&maven_dir)?;
    
    // 检查是否已经安装了指定版本的Maven
    let mvn_bin = maven_dir.join("apache-maven").join("bin").join("mvn");
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
    
    println!("🌐 从 {} 下载Maven...", download_url);
    
    // 下载Maven
    let temp_dir = std::env::temp_dir().join("jx_maven_download");
    fs::create_dir_all(&temp_dir)?;
    let filename = format!("apache-maven-{}-bin.tar.gz", version);
    let download_path = temp_dir.join(&filename);
    
    download_file(&download_url, &download_path)?;
    
    // 解压Maven
    println!("📦 解压Maven...");
    let output = Command::new("tar")
        .args(&["-xzf", download_path.to_str().unwrap(), "-C", maven_dir.to_str().unwrap()])
        .output()
        .context("解压Maven失败")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("解压Maven失败: {}", error));
    }
    
    // 设置执行权限
    set_maven_permissions(&maven_dir)?;
    
    // 创建符号链接到bin目录
    let bin_dir = venv_dir.join("bin");
    create_maven_symlinks(&maven_dir, &bin_dir)?;
    
    // 验证安装
    if let Ok(output) = Command::new(&mvn_bin).arg("--version").output() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        println!("✅ Maven安装成功!");
        println!("版本信息: {}", version_output.lines().next().unwrap_or(""));
    } else {
        return Err(anyhow::anyhow!("Maven安装验证失败"));
    }
    
    // 清理临时文件
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir)?;
    }
    
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
    let maven_bin = maven_dir.join("apache-maven").join("bin");
    
    if !maven_bin.exists() {
        return Err(anyhow::anyhow!("Maven bin目录不存在"));
    }
    
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

fn install_gradle(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Gradle {}...", version);
    
    let gradle_dir = venv_dir.join("lib").join("gradle");
    fs::create_dir_all(&gradle_dir)?;
    
    // 检查是否已经安装了指定版本的Gradle
    let gradle_bin = gradle_dir.join("gradle").join("bin").join("gradle");
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
    
    println!("🌐 从 {} 下载Gradle...", download_url);
    
    // 下载Gradle
    let temp_dir = std::env::temp_dir().join("jx_gradle_download");
    fs::create_dir_all(&temp_dir)?;
    let filename = format!("gradle-{}-bin.zip", version);
    let download_path = temp_dir.join(&filename);
    
    download_file(&download_url, &download_path)?;
    
    // 解压Gradle
    println!("📦 解压Gradle...");
    let output = Command::new("unzip")
        .args(&["-q", download_path.to_str().unwrap(), "-d", gradle_dir.to_str().unwrap()])
        .output()
        .context("解压Gradle失败")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("解压Gradle失败: {}", error));
    }
    
    // 设置执行权限
    set_gradle_permissions(&gradle_dir)?;
    
    // 创建符号链接到bin目录
    let bin_dir = venv_dir.join("bin");
    create_gradle_symlinks(&gradle_dir, &bin_dir)?;
    
    // 验证安装
    if let Ok(output) = Command::new(&gradle_bin).arg("--version").output() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        println!("✅ Gradle安装成功!");
        println!("版本信息: {}", version_output.lines().next().unwrap_or(""));
    } else {
        return Err(anyhow::anyhow!("Gradle安装验证失败"));
    }
    
    // 清理临时文件
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir)?;
    }
    
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
    let gradle_bin = gradle_dir.join("gradle").join("bin").join("gradle");
    
    if !gradle_bin.exists() {
        return Err(anyhow::anyhow!("Gradle bin目录不存在"));
    }
    
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

fn create_activation_scripts(venv_dir: &Path, name: &str) -> Result<()> {
    // 创建bash激活脚本
    let bash_script = format!(
        r#"#!/bin/bash
# jx虚拟环境激活脚本: {}
export JX_VENV_NAME="{}"
export JX_VENV_PATH="{}"

# 设置Java环境
export JAVA_HOME="{}/lib/java"
export PATH="{}/bin:$PATH"

# 设置Maven环境
export MAVEN_HOME="{}/lib/maven"
export PATH="{}/bin:$PATH"

# 设置Gradle环境
export GRADLE_HOME="{}/lib/gradle"
export PATH="{}/bin:$PATH"

# 显示激活信息
echo "🔌 虚拟环境 '{}' 已激活"
echo "Java: $JAVA_HOME"
echo "Maven: $MAVEN_HOME"
echo "Gradle: $GRADLE_HOME"
echo ""
echo "停用虚拟环境: deactivate"

# 定义停用函数
deactivate() {{
    unset JX_VENV_NAME
    unset JX_VENV_PATH
    unset JAVA_HOME
    unset MAVEN_HOME
    unset GRADLE_HOME
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
        venv_dir.display(),
        venv_dir.display(),
        name,
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


    // 创建zsh激活脚本
    let zsh_script = format!(
        r#"#!/bin/zsh
# jx虚拟环境激活脚本: {}
export JX_VENV="{}"
export JX_VENV_PATH="{}"

# 设置Java环境
if [ -d "$JX_VENV_PATH/lib/java" ]; then
    export JAVA_HOME="$JX_VENV_PATH/lib/java"
    echo "设置 JAVA_HOME: $JAVA_HOME"
fi

# 设置Maven环境
if [ -d "$JX_VENV_PATH/lib/maven" ]; then
    export MAVEN_HOME="$JX_VENV_PATH/lib/maven"
    export M2_HOME="$JX_VENV_PATH/lib/maven"
    export PATH="$MAVEN_HOME/bin:$PATH"
    echo "设置 MAVEN_HOME: $MAVEN_HOME"
fi

# 设置Gradle环境
if [ -d "$JX_VENV_PATH/lib/gradle" ]; then
    export GRADLE_HOME="$JX_VENV_PATH/lib/gradle"
    export PATH="$GRADLE_HOME/bin:$PATH"
    echo "设置 GRADLE_HOME: $GRADLE_HOME"
fi

echo "虚拟环境 '{}' 已激活"
echo "停用: jx venv deactivate"
"#,
        name, name, venv_dir.display(), name
    );

    let zsh_path = venv_dir.join("bin").join("activate.zsh");
    fs::write(&zsh_path, zsh_script)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&zsh_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&zsh_path, perms)?;
    }
    
    // 创建fish激活脚本
    let fish_script = format!(
        r#"# jx虚拟环境激活脚本 (fish): {}
set -gx JX_VENV_NAME "{}"
set -gx JX_VENV_PATH "{}"

# 设置Java环境
set -gx JAVA_HOME "{}/lib/java"
set -gx PATH "{}/bin" $PATH

# 设置Maven环境
set -gx MAVEN_HOME "{}/lib/maven"
set -gx PATH "{}/bin" $PATH

# 设置Gradle环境
set -gx GRADLE_HOME "{}/lib/gradle"
set -gx PATH "{}/bin" $PATH

# 显示激活信息
echo "🔌 虚拟环境 '{}' 已激活"
echo "Java: $JAVA_HOME"
echo "Maven: $MAVEN_HOME"
echo "Gradle: $GRADLE_HOME"
echo ""
echo "停用虚拟环境: deactivate"

# 定义停用函数
function deactivate
    set -e JX_VENV_NAME
    set -e JX_VENV_PATH
    set -e JAVA_HOME
    set -e MAVEN_HOME
    set -e GRADLE_HOME
    echo "🔌 虚拟环境 '{}' 已停用"
end
"#,
        name,
        name,
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        venv_dir.display(),
        name,
        name
    );
    
    let fish_file = venv_dir.join("bin").join("activate.fish");
    fs::write(&fish_file, fish_script)?;
    
    Ok(())
}

fn calculate_directory_size(dir_path: &Path) -> Result<u64> {
    let mut total_size = 0;
    
    for entry in walkdir::WalkDir::new(dir_path) {
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

fn format_file_size(size: u64) -> String {
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
