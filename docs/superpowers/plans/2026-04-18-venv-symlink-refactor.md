# Venv 符号链接改造实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 jx venv 从复制模式改造成符号链接模式

**Architecture:** 重写 install_java/install_maven/install_gradle 函数，仅解压到全局缓存并创建符号链接，移除项目 lib/ 目录创建，新增自愈机制

**Tech Stack:** Rust, clap derive, 符号链接 (Unix symlink / Windows junction), TOML

---

## 文件变更概览

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/commands/venv.rs` | 重写 | 重写安装函数，新增自愈机制 |

---

### Task 1: 添加 VenvConfig 结构体和 load_venv_config 函数

**Files:**
- Modify: `src/commands/venv.rs`

- [ ] **Step 1: 添加 VenvConfig 结构体**

在 AdoptiumVersion 结构体之后添加：

```rust
/// venv.toml 配置结构
#[derive(Debug, Clone, serde::Deserialize)]
struct VenvConfig {
    java_version: String,
    maven_version: String,
    gradle_version: String,
    #[serde(default)]
    cache_paths: CachePaths,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct CachePaths {
    java: String,
    maven: String,
    gradle: String,
}

fn load_venv_config(path: &Path) -> Result<VenvConfig> {
    let content = fs::read_to_string(path)?;
    let config: VenvConfig = toml::from_str(&content)
        .context("解析 venv.toml 失败")?;
    Ok(config)
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 2: 添加 create_symlink 函数（跨平台）

**Files:**
- Modify: `src/commands/venv.rs`

- [ ] **Step 1: 添加 create_symlink 函数**

在辅助函数区域添加：

```rust
fn create_symlink(source: &Path, target: &Path) -> Result<()> {
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
                // Windows 上如果符号链接失败，复制文件
                fs::copy(source, target)?;
                Ok(())
            })?;
    }
    
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 3: 添加 check_symlink_validity 函数

**Files:**
- Modify: `src/commands/venv.rs`

- [ ] **Step 1: 添加 check_symlink_validity 函数**

```rust
fn check_symlink_validity(bin_dir: &Path) -> Result<Vec<String>> {
    let mut broken = Vec::new();
    
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
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 4: 重写 create_venv_config 函数

**Files:**
- Modify: `src/commands/venv.rs:423-452`

- [ ] **Step 1: 重写 create_venv_config 函数**

将现有 `create_venv_config` 函数替换为：

```rust
fn create_venv_config(venv_dir: &Path, java_version: &str, build_tool: &BuildTool) -> Result<()> {
    let (maven_version, gradle_version) = match build_tool {
        BuildTool::Maven(version) => (version.clone(), String::new()),
        BuildTool::Gradle(version) => (String::new(), version.clone()),
    };
    
    // 获取缓存路径名称
    let (_, arch) = parse_java_version(java_version)?;
    let os = get_os_type()?;
    let java_cache_name = format!("jdk-{}-{}-{}", java_version.split('.').next().unwrap_or(java_version), os, arch);
    
    let maven_cache_name = if !maven_version.is_empty() {
        format!("apache-maven-{}", maven_version)
    } else {
        String::new()
    };
    
    let gradle_cache_name = if !gradle_version.is_empty() {
        format!("gradle-{}", gradle_version)
    } else {
        String::new()
    };
    
    let config_content = format!(
        r#"# jx 项目虚拟环境配置
# 创建时间: {}

java_version = "{}"
maven_version = "{}"
gradle_version = ""

[cache_paths]
java = "{}"
maven = "{}"
gradle = "{}"
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        java_version,
        maven_version,
        gradle_version,
        java_cache_name,
        maven_cache_name,
        gradle_cache_name
    );
    
    let config_file = venv_dir.join("venv.toml");
    fs::write(config_file, config_content)?;
    
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 5: 重写 create 函数（移除 lib/ 目录创建）

**Files:**
- Modify: `src/commands/venv.rs:150-219`

- [ ] **Step 1: 修改 create 函数的目录创建部分**

将 `create` 函数中的目录创建部分：

```rust
    // 创建虚拟环境目录结构
    fs::create_dir_all(&venv_dir)?;
    fs::create_dir_all(venv_dir.join("bin"))?;
    fs::create_dir_all(venv_dir.join("lib"))?;
    fs::create_dir_all(venv_dir.join("conf"))?;
    fs::create_dir_all(venv_dir.join("cache"))?;
```

改为：

```rust
    // 创建虚拟环境目录结构（仅 bin/）
    fs::create_dir_all(&venv_dir)?;
    fs::create_dir_all(venv_dir.join("bin"))?;
```

- [ ] **Step 2: 移除 create_activation_scripts 调用**

删除行：

```rust
        // 创建激活脚本
        create_activation_scripts(&venv_dir, "", &build_tool)?;
```

- [ ] **Step 3: 验证编译**

Run: `cargo build`
Expected: 编译成功（可能有 create_activation_scripts 未使用警告）

---

### Task 6: 重写 install_java 函数

**Files:**
- Modify: `src/commands/venv.rs:454-536`

- [ ] **Step 1: 完全重写 install_java 函数**

将整个 `install_java` 函数替换为：

```rust
async fn install_java(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Java {}...", version);
    
    // 解析版本获取缓存目录名
    let (major_version, arch) = parse_java_version(version)?;
    let os = get_os_type()?;
    let cache_name = format!("jdk-{}-{}-{}", major_version, os, arch);
    
    let cache_dir = get_cache_directory()?;
    let java_cache_dir = cache_dir.join("java");
    fs::create_dir_all(&java_cache_dir)?;
    
    let cached_java = java_cache_dir.join(&cache_name);
    
    // 如果缓存不存在，下载并解压
    if !cached_java.exists() {
        println!("🌐 下载Java {}...", major_version);
        
        let download_url = build_java_download_url(major_version, &arch, &os)?;
        let filename = get_java_filename_from_url(&download_url)?;
        
        let archives_dir = cache_dir.join("archives").join("java");
        fs::create_dir_all(&archives_dir)?;
        let archive_path = archives_dir.join(&filename);
        
        // 下载压缩包
        download_file(&download_url, &archive_path).await?;
        
        // 解压到缓存目录
        println!("📦 解压Java到缓存...");
        extract_to_cache(&archive_path, &cached_java, "jdk")?;
    } else {
        println!("📋 使用缓存中的Java {}", major_version);
    }
    
    // 设置执行权限
    set_java_bin_permissions(&cached_java)?;
    
    // 创建符号链接到项目 bin/
    let bin_dir = venv_dir.join("bin");
    create_java_bin_symlinks(&cached_java, &bin_dir)?;
    
    // 验证
    let java_bin = find_java_bin(&cached_java);
    if let Ok(output) = Command::new(&java_bin).arg("-version").output() {
        let version_output = String::from_utf8_lossy(&output.stderr);
        println!("✅ Java安装成功!");
        println!("版本: {}", version_output.lines().next().unwrap_or(""));
    }
    
    Ok(())
}
```

- [ ] **Step 2: 添加辅助函数**

添加以下辅助函数：

```rust
fn extract_to_cache(archive_path: &Path, target_dir: &Path, prefix: &str) -> Result<()> {
    fs::create_dir_all(target_dir)?;
    
    let filename = archive_path.file_name().unwrap().to_string_lossy();
    
    if filename.ends_with(".tar.gz") {
        let output = Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap(), "-C", target_dir.parent().unwrap().to_str().unwrap()])
            .output()
            .context("解压失败")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("解压失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // 查找并重命名解压后的目录
        for entry in fs::read_dir(target_dir.parent().unwrap())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with(prefix) {
                if target_dir.exists() {
                    fs::remove_dir_all(target_dir)?;
                }
                fs::rename(path, target_dir)?;
                break;
            }
        }
    } else if filename.ends_with(".zip") {
        let output = Command::new("unzip")
            .args(&["-q", archive_path.to_str().unwrap(), "-d", target_dir.parent().unwrap().to_str().unwrap()])
            .output()
            .context("解压失败")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("解压失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        for entry in fs::read_dir(target_dir.parent().unwrap())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with(prefix) {
                if target_dir.exists() {
                    fs::remove_dir_all(target_dir)?;
                }
                fs::rename(path, target_dir)?;
                break;
            }
        }
    }
    
    Ok(())
}

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

fn find_java_bin_dir(java_dir: &Path) -> PathBuf {
    // macOS 结构
    if java_dir.join("Contents").join("Home").join("bin").exists() {
        java_dir.join("Contents").join("Home").join("bin")
    } else {
        java_dir.join("bin")
    }
}

fn find_java_bin(java_dir: &Path) -> PathBuf {
    find_java_bin_dir(java_dir).join("java")
}

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
```

- [ ] **Step 3: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 7: 重写 install_maven 函数

**Files:**
- Modify: `src/commands/venv.rs` (install_maven 部分，约第900-1100行)

- [ ] **Step 1: 完全重写 install_maven 函数**

```rust
async fn install_maven(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Maven {}...", version);
    
    let cache_name = format!("apache-maven-{}", version);
    
    let cache_dir = get_cache_directory()?;
    let maven_cache_dir = cache_dir.join("maven");
    fs::create_dir_all(&maven_cache_dir)?;
    
    let cached_maven = maven_cache_dir.join(&cache_name);
    
    if !cached_maven.exists() {
        println!("🌐 下载Maven {}...", version);
        
        let download_url = format!(
            "https://archive.apache.org/dist/maven/maven-3/{}/binaries/apache-maven-{}-bin.tar.gz",
            version, version
        );
        
        let archives_dir = cache_dir.join("archives").join("maven");
        fs::create_dir_all(&archives_dir)?;
        let archive_path = archives_dir.join(format!("apache-maven-{}-bin.tar.gz", version));
        
        download_file(&download_url, &archive_path).await?;
        
        println!("📦 解压Maven到缓存...");
        extract_to_cache(&archive_path, &cached_maven, "apache-maven")?;
    } else {
        println!("📋 使用缓存中的Maven {}", version);
    }
    
    // 设置权限
    set_maven_bin_permissions(&cached_maven)?;
    
    // 创建符号链接
    let bin_dir = venv_dir.join("bin");
    create_maven_bin_symlinks(&cached_maven, &bin_dir)?;
    
    let mvn_bin = cached_maven.join("bin").join("mvn");
    if let Ok(output) = Command::new(&mvn_bin).arg("--version").output() {
        println!("✅ Maven安装成功!");
        println!("版本: {}", String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or(""));
    }
    
    Ok(())
}

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

fn create_maven_bin_symlinks(maven_dir: &Path, bin_dir: &Path) -> Result<()> {
    let mvn_bin = maven_dir.join("bin").join("mvn");
    if mvn_bin.exists() {
        create_symlink(&mvn_bin, &bin_dir.join("mvn"))?;
    }
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 8: 重写 install_gradle 函数

**Files:**
- Modify: `src/commands/venv.rs` (install_gradle 部分)

- [ ] **Step 1: 完全重写 install_gradle 函数**

```rust
async fn install_gradle(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Gradle {}...", version);
    
    let cache_name = format!("gradle-{}", version);
    
    let cache_dir = get_cache_directory()?;
    let gradle_cache_dir = cache_dir.join("gradle");
    fs::create_dir_all(&gradle_cache_dir)?;
    
    let cached_gradle = gradle_cache_dir.join(&cache_name);
    
    if !cached_gradle.exists() {
        println!("🌐 下载Gradle {}...", version);
        
        let download_url = format!(
            "https://services.gradle.org/distributions/gradle-{}-bin.zip",
            version
        );
        
        let archives_dir = cache_dir.join("archives").join("gradle");
        fs::create_dir_all(&archives_dir)?;
        let archive_path = archives_dir.join(format!("gradle-{}-bin.zip", version));
        
        download_file(&download_url, &archive_path).await?;
        
        println!("📦 解压Gradle到缓存...");
        extract_to_cache(&archive_path, &cached_gradle, "gradle-")?;
    } else {
        println!("📋 使用缓存中的Gradle {}", version);
    }
    
    set_gradle_bin_permissions(&cached_gradle)?;
    
    let bin_dir = venv_dir.join("bin");
    create_gradle_bin_symlinks(&cached_gradle, &bin_dir)?;
    
    let gradle_bin = cached_gradle.join("bin").join("gradle");
    if let Ok(output) = Command::new(&gradle_bin).arg("--version").output() {
        println!("✅ Gradle安装成功!");
        println!("版本: {}", String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or(""));
    }
    
    Ok(())
}

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

fn create_gradle_bin_symlinks(gradle_dir: &Path, bin_dir: &Path) -> Result<()> {
    let gradle_bin = gradle_dir.join("bin").join("gradle");
    if gradle_bin.exists() {
        create_symlink(&gradle_bin, &bin_dir.join("gradle"))?;
    }
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 9: 删除不再需要的函数

**Files:**
- Modify: `src/commands/venv.rs`

- [ ] **Step 1: 删除 copy_directory 函数**

删除整个 `copy_directory` 函数（约第321-339行）。

- [ ] **Step 2: 删除 rename_extracted_java/maven/gradle 函数**

删除这三个函数（约第341-411行）。

- [ ] **Step 3: 删除旧的 create_java_symlinks 函数**

删除旧的 `create_java_symlinks` 函数（约第869-920行），已被 `create_java_bin_symlinks` 替代。

- [ ] **Step 4: 删除旧的 create_maven_symlinks 和 create_gradle_symlinks 函数**

删除这两个旧函数。

- [ ] **Step 5: 删除 create_activation_scripts 函数**

删除整个 `create_activation_scripts` 函数。

- [ ] **Step 6: 删除 get_java_executable_path 函数**

删除此函数，已被 `find_java_bin` 替代。

- [ ] **Step 7: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 10: 更新 info 函数显示符号链接状态

**Files:**
- Modify: `src/commands/venv.rs:248-303`

- [ ] **Step 1: 修改 info 函数**

将 `info` 函数改为：

```rust
pub fn info(_name: Option<String>) -> Result<()> {
    let venv_dir = std::env::current_dir()?.join(".jx");
    
    if !venv_dir.exists() {
        return Err(anyhow::anyhow!("当前目录没有虚拟环境 (.jx/ 不存在)"));
    }
    
    println!("ℹ️ 项目虚拟环境信息");
    println!("");
    println!("路径: {}", venv_dir.display());
    
    // 读取配置
    let config_file = venv_dir.join("venv.toml");
    if config_file.exists() {
        let config = load_venv_config(&config_file)?;
        println!("Java版本: {}", config.java_version);
        if !config.maven_version.is_empty() {
            println!("Maven版本: {}", config.maven_version);
        }
        if !config.gradle_version.is_empty() {
            println!("Gradle版本: {}", config.gradle_version);
        }
    }
    
    // 符号链接状态
    let bin_dir = venv_dir.join("bin");
    if bin_dir.exists() {
        println!("");
        println!("符号链接状态:");
        
        let broken = check_symlink_validity(&bin_dir)?;
        
        for entry in fs::read_dir(&bin_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();
            
            if path.is_symlink() {
                let target = fs::read_link(&path)?;
                let status = if broken.contains(&name.to_string()) {
                    "❌ 损坏"
                } else {
                    "✓"
                };
                println!("  {} {} -> {}", name, status, target.display());
            }
        }
        
        if !broken.is_empty() {
            println!("");
            println!("⚠️ 检测到损坏的符号链接，运行修复命令可自动重建");
        }
    }
    
    println!("");
    println!("状态: {}", if venv_dir.join(".active").exists() { "🔌 激活" } else { "未激活" });
    
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功

---

### Task 11: 添加 heal_venv 自愈函数

**Files:**
- Modify: `src/commands/venv.rs`

- [ ] **Step 1: 添加 heal_venv 函数**

```rust
fn heal_venv(venv_dir: &Path) -> Result<()> {
    println!("🔧 修复虚拟环境...");
    
    let config = load_venv_config(&venv_dir.join("venv.toml"))?;
    let cache_dir = get_cache_directory()?;
    let bin_dir = venv_dir.join("bin");
    
    // 修复 Java
    if !config.cache_paths.java.is_empty() {
        let java_cache = cache_dir.join("java").join(&config.cache_paths.java);
        
        if !java_cache.exists() {
            println!("🔗 Java 缓存不存在，正在重新下载...");
            let major = config.java_version.split('.').next().unwrap_or(&config.java_version);
            let major_version: u8 = major.parse()
                .map_err(|_| anyhow::anyhow!("无效的Java版本"))?;
            
            download_java_to_cache(major_version, &java_cache)?;
        }
        
        create_java_bin_symlinks(&java_cache, &bin_dir)?;
        println!("✅ Java 符号链接已重建");
    }
    
    // 修复 Maven
    if !config.cache_paths.maven.is_empty() {
        let maven_cache = cache_dir.join("maven").join(&config.cache_paths.maven);
        
        if !maven_cache.exists() {
            println!("🔗 Maven 缓存不存在，正在重新下载...");
            download_maven_to_cache(&config.maven_version, &maven_cache)?;
        }
        
        create_maven_bin_symlinks(&maven_cache, &bin_dir)?;
        println!("✅ Maven 符号链接已重建");
    }
    
    // 修复 Gradle
    if !config.cache_paths.gradle.is_empty() {
        let gradle_cache = cache_dir.join("gradle").join(&config.cache_paths.gradle);
        
        if !gradle_cache.exists() {
            println!("🔗 Gradle 缓存不存在，正在重新下载...");
            download_gradle_to_cache(&config.gradle_version, &gradle_cache)?;
        }
        
        create_gradle_bin_symlinks(&gradle_cache, &bin_dir)?;
        println!("✅ Gradle 符号链接已重建");
    }
    
    println!("✅ 虚拟环境修复完成");
    Ok(())
}

async fn download_java_to_cache(major_version: u8, target_dir: &Path) -> Result<()> {
    let (_, arch) = parse_java_version(&major_version.to_string())?;
    let os = get_os_type()?;
    
    let download_url = build_java_download_url(major_version, &arch, &os)?;
    let filename = get_java_filename_from_url(&download_url)?;
    
    let archives_dir = get_cache_directory()?.join("archives").join("java");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(&filename);
    
    download_file(&download_url, &archive_path).await?;
    extract_to_cache(&archive_path, target_dir, "jdk")?;
    set_java_bin_permissions(target_dir)?;
    
    Ok(())
}

fn download_maven_to_cache(version: &str, target_dir: &Path) -> Result<()> {
    let download_url = format!(
        "https://archive.apache.org/dist/maven/maven-3/{}/binaries/apache-maven-{}-bin.tar.gz",
        version, version
    );
    
    let archives_dir = get_cache_directory()?.join("archives").join("maven");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(format!("apache-maven-{}-bin.tar.gz", version));
    
    // 同步下载（使用 curl）
    let output = Command::new("curl")
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

fn download_gradle_to_cache(version: &str, target_dir: &Path) -> Result<()> {
    let download_url = format!(
        "https://services.gradle.org/distributions/gradle-{}-bin.zip",
        version
    );
    
    let archives_dir = get_cache_directory()?.join("archives").join("gradle");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(format!("gradle-{}-bin.zip", version));
    
    let output = Command::new("curl")
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
```

- [ ] **Step 2: 验证编译**

Run: `cargo build`
Expected: 编译成功（可能有 async/heal_venv 签名问题）

---

### Task 12: 验证完整编译并测试

- [ ] **Step 1: 运行 cargo build**

Run: `cargo build`
Expected: 编译成功

- [ ] **Step 2: 运行 cargo clippy**

Run: `cargo clippy`
Expected: 无严重警告

- [ ] **Step 3: 测试 venv create**

Run: `cargo run -- venv create -j 17`
Expected: 创建 `.jx/` 目录，bin 下为符号链接

- [ ] **Step 4: 测试 venv info**

Run: `cargo run -- venv info`
Expected: 显示符号链接状态

- [ ] **Step 5: 提交**

```bash
git add src/commands/venv.rs
git commit -m "refactor(venv): convert to symlink-based project-level environment"
```