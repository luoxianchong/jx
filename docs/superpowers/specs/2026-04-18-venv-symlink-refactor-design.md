# Venv 符号链接改造设计文档

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

## 目标

将 `jx venv` 从复制模式改造成符号链接模式，实现项目级轻量虚拟环境。

## 背景

子命令简化已完成（commit `82ee989`），`create/remove/info` 已改为项目级路径（`.jx/`）。但当前仍是复制模式，需改造为符号链接模式。

## 核心变更

- `.jx/bin/` 下的工具改为符号链接，指向 `~/.jx/cache/`
- 移除项目的 `lib/` 目录（不再复制 JDK/Maven/Gradle）
- 新增自愈机制：链接失效时自动重建

## 目录结构

### 项目级 `.jx/` 目录

```
项目/.jx/
├── bin/
│   ├── java      -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/java
│   ├── javac     -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/javac
│   ├── jar       -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/jar
│   ├── javadoc   -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/javadoc
│   ├── keytool   -> ~/.jx/cache/java/jdk-17-mac-aarch64/bin/keytool
│   ├── mvn       -> ~/.jx/cache/maven/apache-maven-3.9.9/bin/mvn
│   └── gradle    -> ~/.jx/cache/gradle/gradle-8.4/bin/gradle
├── venv.toml     # 版本配置文件
└── .active       # 激活标记文件
```

**注意：** 不再创建 `lib/`、`conf/`、`cache/` 子目录。

### 全局缓存目录 `~/.jx/cache/`

```
~/.jx/cache/
├── java/
│   ├── jdk-8-mac-aarch64/
│   ├── jdk-11-mac-aarch64/
│   ├── jdk-17-mac-aarch64/     # 解压后的 JDK
│   ├── jdk-21-mac-aarch64/
│   └── jdk-25-mac-aarch64/
├── maven/
│   ├── apache-maven-3.8.8/
│   ├── apache-maven-3.9.9/     # 解压后的 Maven
│   └── ...
├── gradle/
│   ├── gradle-7.6/
│   ├── gradle-8.4/             # 解压后的 Gradle
│   └── ...
└── archives/                   # 压缩包缓存（可选保留）
    ├── java/
    ├── maven/
    └── gradle/
```

## 缓存命名规则

- **Java:** `jdk-{major}-{os}-{arch}`
  - 如 `jdk-17-mac-aarch64`, `jdk-21-linux-x64`
- **Maven:** `apache-maven-{version}`
  - 如 `apache-maven-3.9.9`
- **Gradle:** `gradle-{version}`
  - 如 `gradle-8.4`

## venv.toml 格式

```toml
# jx 项目虚拟环境配置
# 创建时间: 2026-04-18 10:30:00

java_version = "17"
maven_version = "3.9.9"
gradle_version = ""

[cache_paths]
java = "jdk-17-mac-aarch64"
maven = "apache-maven-3.9.9"
gradle = ""
```

**位置:** `.jx/venv.toml`（根目录，不在 `conf/`）

**字段说明:**
- `java_version` - Java 大版本号
- `maven_version` - Maven 版本，空表示不使用
- `gradle_version` - Gradle 版本，空表示不使用
- `[cache_paths]` - 缓存目录名，用于自愈时定位

## 符号链接创建逻辑

### install_java

1. 解析版本获取 `(major_version, arch, os)`
2. 缓存目录: `~/.jx/cache/java/jdk-{major}-{os}-{arch}/`
3. 如果缓存存在 → 直接使用
4. 如果缓存不存在 → 下载压缩包 → 解压到缓存目录
5. 在 `.jx/bin/` 创建符号链接: `java`, `javac`, `jar`, `javadoc`, `keytool`

### install_maven

1. 缓存目录: `~/.jx/cache/maven/apache-maven-{version}/`
2. 缓存不存在时下载并解压
3. 创建 `mvn` 符号链接

### install_gradle

1. 缓存目录: `~/.jx/cache/gradle/gradle-{version}/`
2. 缓存不存在时下载并解压
3. 创建 `gradle` 符号链接

### 符号链接实现

**Unix (macOS/Linux):**
```rust
fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)?;
    }
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}
```

**Windows:**
```rust
#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)?;
    }
    // Windows 10+ 开发者模式允许普通用户创建符号链接
    std::os::windows::fs::symlink_file(source, target)
        .or_else(|_| fs::copy(source, target))?;
    Ok(())
}
```

## 自愈机制

### 触发时机

- `jx run` 检测到 `.jx/` 存在时
- `jx venv info` 显示符号链接状态时

### 检查逻辑

```rust
fn check_symlink_validity(bin_dir: &Path) -> Vec<String> {
    let mut broken = Vec::new();
    for entry in fs::read_dir(bin_dir)? {
        let path = entry?.path();
        if path.is_symlink() {
            let target = fs::read_link(&path)?;
            if !target.exists() {
                broken.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }
    broken
}
```

### 自愈流程

```rust
fn heal_venv(venv_dir: &Path) -> Result<()> {
    let config = load_venv_config(&venv_dir.join("venv.toml"))?;
    let cache_dir = get_cache_directory()?;
    
    // 重建 Java 链接
    if !config.cache_paths.java.is_empty() {
        let java_cache = cache_dir.join("java").join(&config.cache_paths.java);
        if !java_cache.exists() {
            println!("🔗 Java 缓存不存在，正在重新下载...");
            download_and_extract_java(&config.java_version, &java_cache)?;
        }
        recreate_java_symlinks(&java_cache, &venv_dir.join("bin"))?;
    }
    
    // 类似处理 Maven/Gradle
    // ...
    
    Ok(())
}
```

## 代码变更清单

### 新增函数

| 函数 | 说明 |
|------|------|
| `create_symlink()` | 创建符号链接（跨平台） |
| `check_symlink_validity()` | 检查链接有效性，返回损坏列表 |
| `heal_venv()` | 自愈流程 |
| `load_venv_config()` | 解析 venv.toml |
| `recreate_java_symlinks()` | 重建 Java 符号链接 |
| `recreate_maven_symlinks()` | 重建 Maven 符号链接 |
| `recreate_gradle_symlinks()` | 重建 Gradle 符号链接 |
| `download_and_extract_java()` | 下载并解压 Java 到缓存 |
| `download_and_extract_maven()` | 下载并解压 Maven 到缓存 |
| `download_and_extract_gradle()` | 下载并解压 Gradle 到缓存 |

### 重写函数

| 函数 | 改动 |
|------|------|
| `install_java()` | 仅解压到缓存 + 创建链接，不复制 |
| `install_maven()` | 仅解压到缓存 + 创建链接 |
| `install_gradle()` | 仅解压到缓存 + 创建链接 |
| `create_venv_config()` | 更新 venv.toml 格式，添加 cache_paths |

### 删除函数

| 函数 | 原因 |
|------|------|
| `copy_directory()` | 不再需要复制 |
| `rename_extracted_java()` | 合并到安装函数 |
| `rename_extracted_maven()` | 合并到安装函数 |
| `rename_extracted_gradle()` | 合并到安装函数 |

### 修改函数

| 函数 | 改动 |
|------|------|
| `create()` | 移除 `lib/`、`conf/`、`cache/` 目录创建，仅创建 `bin/` |
| `info()` | 显示符号链接状态 |

## 错误处理

### 缓存不存在且下载失败

```
⚠️ Java 缓存不存在，正在重新下载...
❌ 下载失败: 网络连接错误
请检查网络后重试
```

### 项目已存在 .jx/

```
❌ 当前目录已存在 .jx/ 虚拟环境
若要重新创建，请先删除: jx venv remove
```

### 符号链接失效（自愈前）

```
⚠️ 检测到损坏的符号链接: java, mvn
正在自动修复...
✅ 修复完成
```

## 测试计划

1. **创建测试**
   - 创建新环境，验证 `.jx/bin/` 符号链接指向缓存
   - 验证 `venv.toml` 配置正确
   - 验证不创建 `lib/` 目录

2. **缓存复用测试**
   - 创建第二个项目，验证复用现有缓存（不重复下载）
   - 验证两个项目的符号链接指向同一缓存

3. **自愈测试**
   - 删除 `~/.jx/cache/java/jdk-17-mac-aarch64/`
   - 运行 `jx venv info`，验证检测到损坏链接
   - 验证自动重新下载并重建链接

4. **删除测试**
   - 删除环境，验证仅删除 `.jx/`（缓存保留）

5. **跨平台测试**
   - macOS 符号链接测试
   - Linux 符号链接测试
   - Windows junction/复制测试

## 依赖说明

此设计基于已完成的子命令简化（commit `82ee989`），需要在其基础上改造。