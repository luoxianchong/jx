# jx run 自愈功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `jx run` 实现自动检测 `.jx/` 环境、检查符号链接有效性、自动触发自愈的功能。

**Architecture:** 创建新模块 `src/environment.rs` 统一处理环境检测和自愈，修改 `run.rs` 调用此模块，从 `venv.rs` 提取共享函数。

**Tech Stack:** Rust, anyhow, std::fs, std::process::Command

---

## 文件结构

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/environment.rs` | 新增 | 环境检测、自愈、PATH/JAVA_HOME 设置 |
| `src/main.rs` | 修改 | 注册 environment 模块 |
| `src/commands/run.rs` | 修改 | 调用 environment 模块，使用项目环境执行 |
| `src/commands/venv.rs` | 修改 | 引用 environment 模块的公开函数 |

---

### Task 1: 创建 environment.rs 模块基础结构

**Files:**
- Create: `src/environment.rs`

- [ ] **Step 1: 创建模块文件并定义公开接口**

```rust
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
            target.parent().unwrap().parent().unwrap().into()
        } else if target.ends_with("bin/java") {
            target.parent().unwrap().parent().unwrap().into()
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
            target.parent().unwrap().parent().unwrap().into()
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
            target.parent().unwrap().parent().unwrap().into()
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
```

- [ ] **Step 2: 编译验证模块结构**

Run: `cargo build --release 2>&1 | head -50`
Expected: 编译错误（模块未注册）

- [ ] **Step 3: 在 main.rs 注册模块**

在 `src/main.rs` 第 11 行后添加：

```rust
mod environment;
```

Run: `cargo build --release 2>&1 | tail -20`
Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
git add src/environment.rs src/main.rs
git commit -m "feat: add environment module for jx env detection

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 2: 从 venv.rs 提取共享函数到 environment.rs

**Files:**
- Modify: `src/environment.rs`
- Modify: `src/commands/venv.rs`

- [ ] **Step 1: 在 environment.rs 添加自愈相关函数**

在 `src/environment.rs` 末尾添加以下函数（从 venv.rs 复用）：

```rust
/// venv.toml 配置结构
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VenvConfig {
    pub java_version: String,
    pub maven_version: String,
    pub gradle_version: String,
    #[serde(default)]
    pub cache_paths: CachePaths,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CachePaths {
    pub java: String,
    pub maven: String,
    pub gradle: String,
}

/// 加载 venv.toml 配置
pub fn load_venv_config(path: &Path) -> Result<VenvConfig> {
    let content = fs::read_to_string(path)?;
    let config: VenvConfig = toml::from_str(&content)
        .context("解析 venv.toml 失败")?;
    Ok(config)
}

/// 获取缓存目录
fn get_cache_directory() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    let jx_home = home.join(".jx");
    let cache_dir = jx_home.join("cache");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// 获取操作系统类型
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

/// 解析 Java 版本
fn parse_java_version(version: &str) -> Result<(u8, String)> {
    let major_version = if version.starts_with("1.") {
        let minor = version.strip_prefix("1.").unwrap();
        minor.parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    } else if version.contains('.') {
        let major = version.split('.').next().unwrap();
        major.parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    } else {
        version.parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    };

    if major_version < 8 || major_version > 25 {
        return Err(anyhow::anyhow!("不支持的Java版本: {} (支持范围: 8-25)", major_version));
    }

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

/// 创建符号链接
pub fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)?;
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(source, target)
            .or_else(|_| {
                fs::copy(source, target)?;
                Ok(())
            })?;
    }

    Ok(())
}

/// 查找 JDK bin 目录（处理 macOS 特殊结构）
fn find_java_bin_dir(java_dir: &Path) -> PathBuf {
    if java_dir.join("Contents").join("Home").join("bin").exists() {
        java_dir.join("Contents").join("Home").join("bin")
    } else {
        java_dir.join("bin")
    }
}

/// 创建 Java bin 符号链接
fn create_java_bin_symlinks(java_dir: &Path, bin_dir: &Path) -> Result<()> {
    let jdk_bin = find_java_bin_dir(java_dir);
    let commands = ["java", "javac", "jar", "javadoc", "keytool"];
    
    for cmd in &commands {
        let source = jdk_bin.join(cmd);
        let target = bin_dir.join(cmd);
        if source.exists() {
            create_symlink(&source, &target)?;
        }
    }
    Ok(())
}

/// 创建 Maven 符号链接
fn create_maven_bin_symlinks(maven_dir: &Path, bin_dir: &Path) -> Result<()> {
    let mvn_bin = maven_dir.join("bin").join("mvn");
    if mvn_bin.exists() {
        create_symlink(&mvn_bin, &bin_dir.join("mvn"))?;
    }
    Ok(())
}

/// 创建 Gradle 符号链接
fn create_gradle_bin_symlinks(gradle_dir: &Path, bin_dir: &Path) -> Result<()> {
    let gradle_bin = gradle_dir.join("bin").join("gradle");
    if gradle_bin.exists() {
        create_symlink(&gradle_bin, &bin_dir.join("gradle"))?;
    }
    Ok(())
}

/// 解压到缓存目录
fn extract_to_cache(archive_path: &Path, target_dir: &Path, prefix: &str) -> Result<()> {
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
    }
    
    let filename = archive_path.file_name().unwrap().to_string_lossy();
    let parent = target_dir.parent().unwrap();
    
    if filename.ends_with(".tar.gz") {
        let output = std::process::Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap(), "-C", parent.to_str().unwrap()])
            .output()
            .context("解压失败")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("解压失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else if filename.ends_with(".zip") {
        let output = std::process::Command::new("unzip")
            .args(&["-q", archive_path.to_str().unwrap(), "-d", parent.to_str().unwrap()])
            .output()
            .context("解压失败")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("解压失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else {
        return Err(anyhow::anyhow!("不支持的压缩格式"));
    }
    
    // 查找并重命名解压后的目录
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with(prefix) {
            fs::rename(path, target_dir)?;
            break;
        }
    }
    
    Ok(())
}

/// 设置 Java bin 权限
fn set_java_bin_permissions(java_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bin_dir = find_java_bin_dir(java_dir);
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

/// 设置 Maven bin 权限
fn set_maven_bin_permissions(maven_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bin_dir = maven_dir.join("bin");
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

/// 设置 Gradle bin 权限
fn set_gradle_bin_permissions(gradle_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bin_dir = gradle_dir.join("bin");
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

/// 获取 Adoptium API 发布信息
fn get_adoptium_releases(version: u8) -> Result<Vec<serde_json::Value>> {
    let url = format!("https://api.adoptium.net/v3/assets/latest/{}/hotspot", version);
    
    let output = std::process::Command::new("curl")
        .args(&["-s", "-H", "User-Agent: jx/0.1.0", &url])
        .output()
        .context("执行curl命令失败")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Adoptium API请求失败: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let releases: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .context("解析Adoptium API响应失败")?;
    
    Ok(releases)
}

/// 构建 Java 下载 URL
fn build_java_download_url(major_version: u8, arch: &str, os: &str) -> Result<String> {
    let releases = get_adoptium_releases(major_version)?;
    
    if releases.is_empty() {
        return Err(anyhow::anyhow!("未找到Java {}的可用版本", major_version));
    }
    
    for release in &releases {
        let binary = release.get("binary").unwrap();
        let binary_os = binary.get("os").unwrap().as_str().unwrap();
        let binary_arch = binary.get("architecture").unwrap().as_str().unwrap();
        let image_type = binary.get("image_type").unwrap().as_str().unwrap();
        let package = binary.get("package").unwrap();
        let name = package.get("name").unwrap().as_str().unwrap();
        let link = package.get("link").unwrap().as_str().unwrap();
        
        let os_match = match (os, binary_os) {
            ("linux", "linux") => true,
            ("mac", "mac") => true,
            ("windows", "windows") => true,
            _ => false,
        };
        
        let arch_match = match (arch, binary_arch) {
            ("x64", "x64") => true,
            ("aarch64", "aarch64") => true,
            ("arm", "arm") => true,
            _ => false,
        };
        
        let is_jdk = image_type == "jdk";
        let expected_extension = if os == "windows" { "zip" } else { "tar.gz" };
        
        if os_match && arch_match && is_jdk && name.ends_with(expected_extension) {
            return Ok(link.to_string());
        }
    }
    
    Err(anyhow::anyhow!("未找到适合 {}-{} 的Java {}下载链接", os, arch, major_version))
}

/// 从 URL 提取文件名
fn get_java_filename_from_url(url: &str) -> Result<String> {
    if let Some(last_slash) = url.rfind('/') {
        let filename = &url[last_slash + 1..];
        let decoded = urlencoding::decode(filename)
            .map_err(|e| anyhow::anyhow!("URL解码失败: {}", e))?;
        Ok(decoded.to_string())
    } else {
        Err(anyhow::anyhow!("无法从URL中提取文件名: {}", url))
    }
}

/// 下载 Java 到缓存（同步版本）
fn download_java_to_cache_sync(major_version: u8, target_dir: &Path) -> Result<()> {
    let (_, arch) = parse_java_version(&major_version.to_string())?;
    let os = get_os_type()?;
    
    let download_url = build_java_download_url(major_version, &arch, &os)?;
    let filename = get_java_filename_from_url(&download_url)?;
    
    let archives_dir = get_cache_directory()?.join("archives").join("java");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(&filename);
    
    let output = std::process::Command::new("curl")
        .args(&["-L", "-o", archive_path.to_str().unwrap(), &download_url])
        .output()
        .context("下载 Java 失败")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("下载失败: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    extract_to_cache(&archive_path, target_dir, "jdk")?;
    set_java_bin_permissions(target_dir)?;
    
    Ok(())
}

/// 下载 Maven 到缓存（同步版本）
fn download_maven_to_cache_sync(version: &str, target_dir: &Path) -> Result<()> {
    let download_url = format!(
        "https://archive.apache.org/dist/maven/maven-3/{}/binaries/apache-maven-{}-bin.tar.gz",
        version, version
    );
    
    let archives_dir = get_cache_directory()?.join("archives").join("maven");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(format!("apache-maven-{}-bin.tar.gz", version));
    
    let output = std::process::Command::new("curl")
        .args(&["-L", "-o", archive_path.to_str().unwrap(), &download_url])
        .output()
        .context("下载 Maven 失败")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("下载失败: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    extract_to_cache(&archive_path, target_dir, "apache-maven")?;
    set_maven_bin_permissions(target_dir)?;
    
    Ok(())
}

/// 下载 Gradle 到缓存（同步版本）
fn download_gradle_to_cache_sync(version: &str, target_dir: &Path) -> Result<()> {
    let download_url = format!(
        "https://services.gradle.org/distributions/gradle-{}-bin.zip",
        version
    );
    
    let archives_dir = get_cache_directory()?.join("archives").join("gradle");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(format!("gradle-{}-bin.zip", version));
    
    let output = std::process::Command::new("curl")
        .args(&["-L", "-o", archive_path.to_str().unwrap(), &download_url])
        .output()
        .context("下载 Gradle 失败")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("下载失败: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    extract_to_cache(&archive_path, target_dir, "gradle-")?;
    set_gradle_bin_permissions(target_dir)?;
    
    Ok(())
}

/// 自愈虚拟环境
pub fn heal_venv(venv_dir: &Path) -> Result<()> {
    println!("🔧 修复虚拟环境...");
    
    let config = load_venv_config(&venv_dir.join("venv.toml"))?;
    let cache_dir = get_cache_directory()?;
    let bin_dir = venv_dir.join("bin");
    
    fs::create_dir_all(&bin_dir)?;
    
    // 修复 Java
    if !config.cache_paths.java.is_empty() {
        let java_cache = cache_dir.join("java").join(&config.cache_paths.java);
        
        if !java_cache.exists() {
            println!("🔗 Java 缓存不存在，正在重新下载...");
            let major: u8 = config.java_version.split('.')
                .next()
                .unwrap_or(&config.java_version)
                .parse()
                .map_err(|_| anyhow::anyhow!("无效的Java版本"))?;
            
            download_java_to_cache_sync(major, &java_cache)?;
        }
        
        create_java_bin_symlinks(&java_cache, &bin_dir)?;
        println!("✅ Java 符号链接已重建");
    }
    
    // 修复 Maven
    if !config.cache_paths.maven.is_empty() {
        let maven_cache = cache_dir.join("maven").join(&config.cache_paths.maven);
        
        if !maven_cache.exists() {
            println!("🔗 Maven 缓存不存在，正在重新下载...");
            download_maven_to_cache_sync(&config.maven_version, &maven_cache)?;
        }
        
        create_maven_bin_symlinks(&maven_cache, &bin_dir)?;
        println!("✅ Maven 符号链接已重建");
    }
    
    // 修复 Gradle
    if !config.cache_paths.gradle.is_empty() {
        let gradle_cache = cache_dir.join("gradle").join(&config.cache_paths.gradle);
        
        if !gradle_cache.exists() {
            println!("🔗 Gradle 缓存不存在，正在重新下载...");
            download_gradle_to_cache_sync(&config.gradle_version, &gradle_cache)?;
        }
        
        create_gradle_bin_symlinks(&gradle_cache, &bin_dir)?;
        println!("✅ Gradle 符号链接已重建");
    }
    
    println!("✅ 虚拟环境修复完成");
    Ok(())
}
```

- [ ] **Step 2: 编译验证**

Run: `cargo build --release 2>&1 | tail -20`
Expected: 编译成功（可能有警告）

- [ ] **Step 3: 修改 venv.rs 引用 environment 模块**

在 `src/commands/venv.rs` 文件开头修改：

将第 1-10 行改为：

```rust
use crate::environment::{check_symlink_validity, create_symlink, heal_venv, load_venv_config, VenvConfig, CachePaths};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicativ::{ProgressBar, ProgressStyle};
use reqwest;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::io::AsyncWriteExt;
```

删除 venv.rs 中重复定义的函数（第 63-126 行）：
- `VenvConfig` 结构体（第 63-71 行）
- `CachePaths` 结构体（第 73-78 行）
- `load_venv_config` 函数（第 80-85 行）
- `create_symlink` 函数（第 87-108 行）
- `check_symlink_validity` 函数（第 110-126 行）
- `heal_venv` 函数（第 850-906 行）

删除 venv.rs 中的以下辅助函数（已在 environment.rs 中定义）：
- `find_java_bin_dir`（第 192-198 行）
- `create_java_bin_symlinks`（第 204-219 行）
- `create_maven_bin_symlinks`（第 813-819 行）
- `create_gradle_bin_symlinks`（第 842-848 行）

- [ ] **Step 4: 编译验证**

Run: `cargo build --release 2>&1 | tail -30`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src/environment.rs src/commands/venv.rs
git commit -m "refactor: extract venv shared functions to environment module

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 3: 修改 run.rs 使用环境模块

**Files:**
- Modify: `src/commands/run.rs`

- [ ] **Step 1: 添加 environment 模块引用**

在 `src/commands/run.rs` 文件开头添加：

```rust
use crate::environment::{prepare_project_environment, EnvConfig, prepend_to_path};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
```

- [ ] **Step 2: 重构 execute() 方法**

修改 `RunCommand::execute()` 方法，将原有逻辑改为：

```rust
impl RunCommand {
    pub fn execute(&self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        
        // 检测并准备项目环境
        let env_config = prepare_project_environment()?;
        
        if let Some(config) = env_config {
            // 使用 .jx/ 环境执行
            self.run_with_env(&current_dir, &config)?;
        } else {
            // 原有逻辑（无 .jx/ 环境）
            self.run_without_env(&current_dir)?;
        }
        Ok(())
    }
    
    fn run_with_env(&self, project_dir: &Path, config: &EnvConfig) -> Result<()> {
        println!("🚀 运行项目（使用 .jx 环境）...");
        
        // 查找项目配置文件
        let config_file = self.find_config_file(project_dir)?;
        
        let class_to_run = if let Some(ref class) = self.main_class {
            class.clone()
        } else {
            get_main_class_from_config(project_dir, &config_file)?
        };
        
        println!("主类: {}", class_to_run);
        if !self.args.is_empty() {
            println!("参数: {}", self.args.join(" "));
        }
        
        // 根据配置文件类型运行项目（带环境变量）
        let result = match config_file {
            "jx.toml" => run_jx_project_with_env(project_dir, &class_to_run, &self.args, config),
            "pom.xml" => run_maven_project_with_env(project_dir, &class_to_run, &self.args, config),
            "build.gradle" => run_gradle_project_with_env(project_dir, &class_to_run, &self.args, config),
            _ => Err(anyhow::anyhow!("不支持的配置文件类型")),
        };
        
        match result {
            Ok(_) => {
                println!("✅ 项目运行完成!");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 运行失败: {}", e);
                Err(e)
            }
        }
    }
    
    fn run_without_env(&self, project_dir: &Path) -> Result<()> {
        println!("🚀 运行项目...");
        
        // 查找项目配置文件
        let config_file = self.find_config_file(project_dir)?;
        
        let class_to_run = if let Some(ref class) = self.main_class {
            class.clone()
        } else {
            get_main_class_from_config(project_dir, &config_file)?
        };
        
        println!("主类: {}", class_to_run);
        if !self.args.is_empty() {
            println!("参数: {}", self.args.join(" "));
        }
        
        // 根据配置文件类型运行项目（原有逻辑）
        let result = match config_file {
            "jx.toml" => run_jx_project(project_dir, &class_to_run, &self.args),
            "pom.xml" => run_maven_project(project_dir, &class_to_run, &self.args),
            "build.gradle" => run_gradle_project(project_dir, &class_to_run, &self.args),
            _ => Err(anyhow::anyhow!("不支持的配置文件类型")),
        };
        
        match result {
            Ok(_) => {
                println!("✅ 项目运行完成!");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 运行失败: {}", e);
                Err(e)
            }
        }
    }
    
    fn find_config_file(&self, project_dir: &Path) -> Result<&str> {
        if project_dir.join("jx.toml").exists() {
            Ok("jx.toml")
        } else if project_dir.join("pom.xml").exists() {
            Ok("pom.xml")
        } else if project_dir.join("build.gradle").exists() {
            Ok("build.gradle")
        } else {
            Err(anyhow::anyhow!("找不到项目配置文件，请先运行 'jx init'"))
        }
    }
}
```

- [ ] **Step 3: 添加带环境变量的运行函数**

在文件末尾添加：

```rust
fn run_jx_project_with_env(project_dir: &Path, main_class: &str, args: &[String], config: &EnvConfig) -> Result<()> {
    let config_path = project_dir.join("jx.toml");
    let config_content = std::fs::read_to_string(&config_path)?;
    
    if config_content.contains("type = \"maven\"") {
        run_maven_project_with_env(project_dir, main_class, args, config)
    } else if config_content.contains("type = \"gradle\"") {
        run_gradle_project_with_env(project_dir, main_class, args, config)
    } else {
        Err(anyhow::anyhow!("在jx.toml中找不到有效的项目类型"))
    }
}

fn run_maven_project_with_env(project_dir: &Path, main_class: &str, args: &[String], config: &EnvConfig) -> Result<()> {
    println!("使用Maven运行项目...");
    
    // 先编译项目
    println!("编译项目...");
    let compile_output = Command::new("mvn")
        .arg("compile")
        .env("PATH", prepend_to_path(&config.bin_dir))
        .env("JAVA_HOME", config.java_home.clone().unwrap_or_default())
        .current_dir(project_dir)
        .output()
        .context("Maven编译失败")?;
    
    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Maven编译失败: {}", error));
    }
    
    // 运行项目
    println!("运行项目...");
    println!("──────────────────────────────────────────");
    
    let mut mvn_cmd = Command::new("mvn");
    mvn_cmd.arg("exec:java");
    mvn_cmd.arg(format!("-Dexec.mainClass={}", main_class));
    mvn_cmd.env("PATH", prepend_to_path(&config.bin_dir));
    
    if let Some(java_home) = &config.java_home {
        mvn_cmd.env("JAVA_HOME", java_home);
    }
    
    if !args.is_empty() {
        let args_str = args.join(" ");
        mvn_cmd.arg(format!("-Dexec.args={}", args_str));
    }
    
    mvn_cmd.current_dir(project_dir);
    
    let status = mvn_cmd.status().context("Maven运行失败")?;
    
    println!("──────────────────────────────────────────");
    
    if !status.success() {
        return Err(anyhow::anyhow!("程序执行失败"));
    }
    
    Ok(())
}

fn run_gradle_project_with_env(project_dir: &Path, _main_class: &str, args: &[String], config: &EnvConfig) -> Result<()> {
    println!("使用Gradle运行项目...");
    
    // 先编译项目
    println!("编译项目...");
    let compile_output = Command::new("gradle")
        .arg("compileJava")
        .env("PATH", prepend_to_path(&config.bin_dir))
        .env("JAVA_HOME", config.java_home.clone().unwrap_or_default())
        .current_dir(project_dir)
        .output()
        .context("Gradle编译失败")?;
    
    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Gradle编译失败: {}", error));
    }
    
    // 运行项目
    println!("运行项目...");
    println!("──────────────────────────────────────────");
    
    let mut gradle_cmd = Command::new("gradle");
    gradle_cmd.arg("run");
    gradle_cmd.env("PATH", prepend_to_path(&config.bin_dir));
    
    if let Some(java_home) = &config.java_home {
        gradle_cmd.env("JAVA_HOME", java_home);
    }
    
    if !args.is_empty() {
        let args_str = args.join(" ");
        gradle_cmd.arg("--args");
        gradle_cmd.arg(&args_str);
    }
    
    gradle_cmd.current_dir(project_dir);
    
    let status = gradle_cmd.status().context("Gradle运行失败")?;
    
    println!("──────────────────────────────────────────");
    
    if !status.success() {
        return Err(anyhow::anyhow!("程序执行失败"));
    }
    
    Ok(())
}
```

- [ ] **Step 4: 编译验证**

Run: `cargo build --release 2>&1 | tail -30`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src/commands/run.rs
git commit -m "feat: integrate environment module into jx run

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 4: 功能测试验证

**Files:**
- 无文件修改，仅测试

- [ ] **Step 1: 测试无 .jx/ 环境时运行**

创建临时测试目录并验证：

```bash
mkdir -p /tmp/jx-test-noenv && cd /tmp/jx-test-noenv
# 创建一个简单的 pom.xml
echo '<project><modelVersion>4.0.0</modelVersion><groupId>test</groupId><artifactId>test</artifactId><version>1.0</version></project>' > pom.xml
/Users/ing/Documents/wkspace/rust/jx/target/release/jx run
```

Expected: 输出 "找不到项目配置文件" 或正常执行原有逻辑（无环境检测）

- [ ] **Step 2: 测试有 .jx/ 环境时运行**

```bash
mkdir -p /tmp/jx-test-env && cd /tmp/jx-test-env
/Users/ing/Documents/wkspace/rust/jx/target/release/jx venv create --java-version 17 --maven-version 3.9.9
# 创建简单 pom.xml
echo '<project><modelVersion>4.0.0</modelVersion><groupId>test</groupId><artifactId>test</artifactId><version>1.0</version></project>' > pom.xml
/Users/ing/Documents/wkspace/rust/jx/target/release/jx run
```

Expected: 输出 "运行项目（使用 .jx 环境）..."，使用项目工具执行

- [ ] **Step 3: 测试符号链接损坏时自动自愈**

```bash
cd /tmp/jx-test-env
# 删除缓存中的 Java（模拟损坏）
rm -rf ~/.jx/cache/java/jdk-17-mac-*
/Users/ing/Documents/wkspace/rust/jx/target/release/jx run
```

Expected: 输出 "检测到损坏的符号链接"，触发自动修复，重新下载并重建链接

- [ ] **Step 4: 清理测试环境**

```bash
rm -rf /tmp/jx-test-noenv /tmp/jx-test-env ~/.jx/cache/java/jdk-17-mac-*
```

- [ ] **Step 5: Commit 测试验证**

```bash
git add -A
git commit -m "test: verify jx run environment integration

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

### Task 5: 最终集成测试

- [ ] **Step 1: 运行完整编译和测试**

```bash
cargo build --release
```

Expected: 编译成功，无错误

- [ ] **Step 2: 验证 venv info 功能正常**

创建测试环境并验证 `venv info` 是否正常调用 environment 模块：

```bash
mkdir -p /tmp/jx-final-test && cd /tmp/jx-final-test
/Users/ing/Documents/wkspace/rust/jx/target/release/jx venv create --java-version 17
/Users/ing/Documents/wkspace/rust/jx/target/release/jx venv info
```

Expected: 正常显示环境信息和符号链接状态

- [ ] **Step 3: 清理并最终提交**

```bash
rm -rf /tmp/jx-final-test
git log --oneline -5
```

Expected: 显示最近 5 个提交，包含所有本次实现的提交

- [ ] **Step 4: 完成**

实现完成。所有功能已验证：
- environment.rs 模块创建
- run.rs 集成环境检测和自愈
- venv.rs 复用 environment 模块函数
- 功能测试验证通过