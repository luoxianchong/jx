# 项目级虚拟环境设计文档

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 jx venv 从全局虚拟环境改造为项目级虚拟环境，类似 uv venv 的设计模式

**Architecture:** 项目根目录下创建 `.jx/` 隐藏目录，使用符号链接指向全局缓存的 JDK/Maven/Gradle，通过 `venv.toml` 记录版本配置实现自愈机制

**Tech Stack:** Rust, clap derive, 符号链接 (Unix symlink / Windows junction), TOML 配置

---

## 核心变更

### 1. 虚拟环境位置变更

**当前行为：**
- 虚拟环境存储在全局目录 `~/.jx/venvs/<name>/`
- 每个环境独立复制 JDK/Maven/Gradle（占用大量磁盘空间）

**新行为：**
- 虚拟环境存储在项目根目录 `<project>/.jx/`
- 使用符号链接指向全局缓存 `~/.jx/cache/`
- 支持用户指定已有 JDK 路径（混合模式）

### 2. 创建即激活

**当前行为：**
- `jx venv create <name>` 创建环境
- 用户需手动执行 `jx venv activate <name>` 或 `source ~/.jx/venvs/<name>/bin/activate`

**新行为：**
- `jx venv create` 在当前项目目录创建 `.jx/`
- 创建完成后自动激活（设置环境变量指向 `.jx/bin/`）
- `jx run` 自动检测并使用项目 `.jx/` 环境

### 3. 删除即停用

**当前行为：**
- `jx venv remove <name>` 删除环境
- 如果环境正在激活状态，拒绝删除并提示用户先停用

**新行为：**
- `jx venv remove` 删除项目 `.jx/` 目录
- 删除时自动停用（清理激活状态）

### 4. 符号链接而非复制

**当前行为：**
- 下载 JDK/Maven/Gradle 到缓存
- 复制整个解压目录到虚拟环境的 `lib/` 目录
- 创建 bin 命令的符号链接

**新行为：**
- 下载 JDK/Maven/Gradle 到全局缓存 `~/.jx/cache/`
- `.jx/bin/` 直接符号链接到全局缓存的 bin 目录
- 不复制任何文件，节省磁盘空间

### 5. 自愈机制

**新增功能：**
- `venv.toml` 记录版本配置（java_version, maven_version, gradle_version, jdk_path 等）
- 当全局缓存被删除或符号链接失效时，`jx run` 自动检测并根据配置重新下载
- 下载完成后自动重建符号链接

---

## 目录结构

### 项目级 `.jx/` 目录

```
<project-root>/.jx/
├── bin/
│   ├── java      -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/java
│   ├── javac     -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/javac
│   ├── jar       -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/jar
│   ├── javadoc   -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/javadoc
│   ├── keytool   -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/keytool
│   ├── mvn       -> ~/.jx/cache/maven/apache-maven-3.9.9/bin/mvn
│   └── gradle    -> ~/.jx/cache/gradle/gradle-8.4/bin/gradle
└── venv.toml     # 版本配置文件
```

### venv.toml 配置文件格式

```toml
# jx 项目虚拟环境配置
# 创建时间: 2026-04-16 10:30:00

java_version = "17"
java_vendor = "adoptium"        # adoptium 或 custom
java_path = ""                  # 空表示使用缓存，非空表示用户指定路径

maven_version = "3.9.9"
gradle_version = ""             # 空表示不使用 Gradle

[cache_paths]
java = "jdk-17-mac-aarch64"     # 缓存目录名（用于重建链接）
maven = "apache-maven-3.9.9"
gradle = ""
```

### 全局缓存目录 `~/.jx/cache/`

```
~/.jx/cache/
├── java/
│   ├── jdk-8-mac-aarch64/
│   ├── jdk-11-mac-aarch64/
│   ├── jdk-17-mac-aarch64/     # 解压后的 JDK 目录
│   ├── jdk-21-mac-aarch64/
│   └── jdk-25-mac-aarch64/
├── maven/
│   ├── apache-maven-3.8.8/
│   ├── apache-maven-3.9.9/     # 解压后的 Maven 目录
│   └── ...
├── gradle/
│   ├── gradle-7.6/
│   ├── gradle-8.4/             # 解压后的 Gradle 目录
│   └── ...
└── archives/                   # 压缩包缓存（可选保留）
    ├── java/
    ├── maven/
    └── gradle/
```

---

## 命令变更

### jx venv create

**新参数设计：**

```
jx venv create [OPTIONS]

OPTIONS:
    --java-version, -j <VERSION>   Java版本 (8, 11, 17, 21, 25) [default: 17]
    --java-path <PATH>             使用已有的 JDK 路径（不下载）
    --maven-version, -m <VERSION>  Maven版本 [default: 3.9.9]
    --gradle-version, -g <VERSION> Gradle版本（使用 Gradle 代替 Maven）
```

**行为变更：**

1. 在当前目录创建 `.jx/` 目录结构
2. 检查全局缓存是否存在指定版本
3. 如果不存在且用户未指定 `--java-path`，下载到全局缓存
4. 创建 `bin/` 目录下的符号链接
5. 写入 `venv.toml` 配置文件
6. 设置激活状态（写入 `.jx/.active` 标记文件）
7. 输出成功信息

**输出示例：**

```
🌱 创建项目虚拟环境...
Java版本: 17
Maven版本: 3.9.9

✅ 虚拟环境创建成功!
路径: ./jx

工具链:
  java  -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/java
  mvn   -> ~/.jx/cache/maven/apache-maven-3.9.9/bin/mvn

jx run 将自动使用此环境
```

### jx venv remove

**新参数设计：**

```
jx venv remove [OPTIONS]

OPTIONS:
    --force, -f    强制删除（即使有未完成的构建）
```

**行为变更：**

1. 检查当前目录是否存在 `.jx/`
2. 如果 `.jx/.active` 存在，清理激活状态
3. 删除整个 `.jx/` 目录
4. 输出成功信息

### 移除的命令

- **jx venv activate** - 不再需要，创建时自动激活
- **jx venv deactivate** - 不再需要，删除时自动停用
- **jx venv list** - 不再需要，虚拟环境是项目级的，无全局列表

**注意：** 可保留 `jx venv info` 用于显示当前项目环境信息

### jx venv info

**行为变更：**

显示当前项目 `.jx/` 目录的环境信息

**输出示例：**

```
ℹ️ 项目虚拟环境信息

路径: ./jx
Java版本: 17 (Adoptium)
Maven版本: 3.9.9

符号链接状态:
  java  ✓ ~/.jx/cache/java/jdk-17-mac-aarch64/bin/java
  mvn   ✓ ~/.jx/cache/maven/apache-maven-3.9.9/bin/mvn

磁盘占用:
  项目环境: 4KB (仅配置和符号链接)
  全局缓存: 350MB (共享)
```

### jx run

**新增行为：**

1. 检查当前目录是否存在 `.jx/`
2. 如果存在，检查符号链接是否有效
3. 如果符号链接失效（缓存被删除），根据 `venv.toml` 自动重新下载
4. 重建符号链接
5. 设置环境变量：
   - `PATH=./jx/bin:$PATH`
   - `JAVA_HOME` 指向符号链接目标的父目录
   - `MAVEN_HOME` 或 `GRADLE_HOME` 同理
6. 执行项目的构建/运行命令

---

## 符号链接实现

### Unix (macOS/Linux)

```rust
use std::os::unix::fs::symlink;

fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)?;
    }
    symlink(source, target)?;
    Ok(())
}
```

### Windows

Windows 需要特殊处理，因为普通符号链接需要管理员权限：

```rust
#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)?;
    }
    // Windows 10+ 开发者模式允许普通用户创建符号链接
    // 否则使用 junction (目录链接) 或复制文件
    std::os::windows::fs::symlink_file(source, target)
        .or_else(|_| fs::copy(source, target))?;
    Ok(())
}
```

---

## 自愈机制实现

### 符号链接有效性检查

```rust
fn check_symlink_validity(bin_dir: &Path) -> Result<Vec<String>> {
    let mut broken_links = Vec::new();
    
    for entry in fs::read_dir(bin_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_symlink() {
            let target = fs::read_link(&path)?;
            if !target.exists() {
                broken_links.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }
    
    Ok(broken_links)
}
```

### 自愈流程

```rust
fn heal_venv(venv_dir: &Path) -> Result<()> {
    let config = load_venv_config(venv_dir)?;
    let cache_dir = get_cache_directory()?;
    
    // 检查并恢复 Java
    if !config.java_path.is_empty() {
        // 用户指定路径，检查是否存在
        let java_path = PathBuf::from(&config.java_path);
        if !java_path.exists() {
            return Err(anyhow!("用户指定的 JDK 路径不存在: {}", config.java_path));
        }
        recreate_java_symlinks(&java_path, &venv_dir.join("bin"))?;
    } else {
        // 使用缓存
        let java_cache = cache_dir.join("java").join(&config.cache_paths.java);
        if !java_cache.exists() {
            println!("🔗 Java 缓存不存在，正在重新下载...");
            download_java(&config.java_version, &java_cache)?;
        }
        recreate_java_symlinks(&java_cache, &venv_dir.join("bin"))?;
    }
    
    // 类似处理 Maven/Gradle
    // ...
    
    Ok(())
}
```

---

## 错误处理

### 用户指定 JDK 路径不存在

```
❌ 错误: 用户指定的 JDK 路径不存在: /custom/jdk-17

请检查路径是否正确，或移除 --java-path 参数让 jx 自动下载 JDK
```

### 全局缓存目录被删除且下载失败

```
⚠️ Java 缓存不存在，正在重新下载...

❌ 下载失败: 网络连接错误

请检查网络连接后重试，或手动下载 JDK 到:
  ~/.jx/cache/java/jdk-17-mac-aarch64/
```

### 项目目录已存在 .jx/

```
❌ 错误: 当前目录已存在 .jx/ 虚拟环境

若要重新创建，请先删除: jx venv remove
```

---

## 测试计划

1. **创建测试**
   - 创建新环境，验证 `.jx/` 目录结构
   - 验证符号链接正确指向缓存
   - 验证 `venv.toml` 配置正确

2. **自愈测试**
   - 删除全局缓存，执行 `jx run`，验证自动重新下载
   - 验证符号链接重建成功

3. **用户指定路径测试**
   - 使用 `--java-path` 创建环境
   - 验证符号链接指向用户路径
   - 删除用户路径后验证错误提示

4. **删除测试**
   - 删除环境，验证 `.jx/` 目录完全移除
   - 验证激活状态清理

5. **跨平台测试**
   - macOS 符号链接测试
   - Linux 符号链接测试
   - Windows junction/复制测试

---

## 迁移说明

此变更**不兼容**现有全局虚拟环境。用户需要：

1. 在项目目录重新执行 `jx venv create`
2. 删除旧的全局环境（可选）：`rm -rf ~/.jx/venvs/`

全局缓存 `~/.jx/cache/` 可保留，新设计会复用现有缓存。