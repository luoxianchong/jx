# jx run 自愈功能设计文档

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

## 目标

为 `jx run` 实现自动检测 `.jx/` 环境、检查符号链接有效性、自动触发自愈的功能。

## 背景

符号链接改造已完成（commit `19fd846`），但 `run.rs` 未实现设计文档中要求的自愈触发逻辑：
- 用户删除全局缓存后，`jx run` 应自动检测并重新下载
- 当前 `run.rs` 直接调用系统 `mvn/gradle`，未利用项目 `.jx/` 环境

## 核心设计

### 新模块: `src/environment.rs`

统一处理项目环境检测、自愈、环境变量设置。

#### 公开接口

```rust
/// 检测并准备项目环境
/// 返回: 环境准备成功时返回 Some(EnvConfig)，无 .jx/ 时返回 None
pub fn prepare_project_environment() -> Result<Option<EnvConfig>>;

/// 环境配置
pub struct EnvConfig {
    pub bin_dir: PathBuf,           // .jx/bin 路径
    pub java_home: Option<PathBuf>, // JAVA_HOME (从符号链接解析)
    pub maven_home: Option<PathBuf>,
    pub gradle_home: Option<PathBuf>,
}
```

#### 内部函数

| 函数 | 说明 |
|------|------|
| `detect_jx_env()` | 检测 `.jx/` 是否存在 |
| `check_and_heal()` | 检查链接有效性 + 自动调用自愈 |
| `resolve_home_from_symlink()` | 从符号链接目标路径解析工具 HOME 目录 |

#### 共享函数（从 venv.rs 复用）

| 函数 | 说明 |
|------|------|
| `check_symlink_validity()` | 检查符号链接有效性，返回损坏列表 |
| `heal_venv()` | 自愈流程：重新下载缓存 + 重建链接 |
| `load_venv_config()` | 解析 venv.toml |

### 调用方改动

#### run.rs

在 `execute()` 开头添加环境检测：

```rust
impl RunCommand {
    pub fn execute(&self) -> Result<()> {
        // 新增: 检测并准备项目环境
        let env_config = environment::prepare_project_environment()?;
        
        if let Some(config) = env_config {
            // 使用 .jx/ 环境执行
            run_with_jx_env(&config, &self.main_class, &self.args)?;
        } else {
            // 原有逻辑（无 .jx/ 环境）
            run_without_env(...)?;
        }
        Ok(())
    }
}

fn run_with_jx_env(config: &EnvConfig, main_class: &Option<String>, args: &[String]) -> Result<()> {
    // 设置环境变量
    let mut cmd = Command::new("mvn"); // 或 gradle
    cmd.env("PATH", prepend_to_path(&config.bin_dir));
    
    if let Some(java_home) = &config.java_home {
        cmd.env("JAVA_HOME", java_home);
    }
    // ...
}
```

#### venv.rs

- `check_symlink_validity()` 和 `heal_venv()` 移至 `environment.rs`
- `venv info` 通过调用 `environment.rs` 的函数复用逻辑

## 执行流程

```
jx run 执行流程:
┌─────────────────────────────────────────────────────┐
│ 1. prepare_project_environment()                     │
│    ├─ 检查当前目录是否存在 .jx/                       │
│    │   ├─ 不存在 → 返回 None，使用原有逻辑            │
│    │   └─ 存在 → 继续检测                             │
│    │                                                  │
│    ├─ check_symlink_validity(.jx/bin)                │
│    │   ├─ 无损坏 → 返回 EnvConfig                     │
│    │   └─ 有损坏 → heal_venv()                        │
│    │       ├─ 缓存存在 → 重建链接                     │
│    │       └─ 缓存不存在 → 下载 + 解压 + 重建链接     │
│    │                                                  │
│    └─ 返回 EnvConfig (bin_dir, java_home, ...)       │
├─────────────────────────────────────────────────────┤
│ 2. 设置环境变量                                       │
│    PATH = .jx/bin:$PATH                             │
│    JAVA_HOME = 符号链接目标的 Contents/Home 或 bin   │
│                                                      │
│ 3. 执行构建命令                                       │
│    Command::new("mvn").env(...).status()             │
└─────────────────────────────────────────────────────┘
```

## PATH 处理

`.jx/bin` 优先置于 PATH 最前面：

```rust
fn prepend_to_path(bin_dir: &Path) -> String {
    let current_path = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", bin_dir.display(), current_path)
}
```

## JAVA_HOME 解析

从 `.jx/bin/java` 符号链接目标路径解析：

```rust
fn resolve_java_home(bin_dir: &Path) -> Option<PathBuf> {
    let java_symlink = bin_dir.join("java");
    if java_symlink.is_symlink() {
        let target = fs::read_link(&java_symlink)?;
        // macOS: target = .../Contents/Home/bin/java
        // Linux: target = .../bin/java
        // 返回 Contents/Home 或 bin 的父目录
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
```

## 错误处理

### 自愈失败

```
⚠️ 检测到损坏的符号链接: java, mvn
正在自动修复...
❌ 修复失败: 网络连接错误
请检查网络后重试，或手动执行: jx venv info
```

### venv.toml 解析失败

```
❌ 无法读取 .jx/venv.toml: 文件不存在
请重新创建虚拟环境: jx venv create
```

## 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/environment.rs` | 新增 | 环境检测、自愈、环境变量模块 |
| `src/commands/run.rs` | 修改 | 调用 environment 模块，使用项目环境执行 |
| `src/commands/venv.rs` | 修改 | 移动共享函数至 environment.rs |
| `src/lib.rs` 或 `src/main.rs` | 修改 | 注册 environment 模块 |

## 测试计划

1. **无 .jx/ 环境** — 验证原有逻辑不受影响
2. **环境有效** — 验证 PATH 设置正确，使用项目工具
3. **链接损坏** — 验证自动自愈触发并成功重建
4. **缓存不存在** — 验证自动下载并重建链接
5. **自愈失败** — 验证错误提示清晰

## 依赖说明

此设计基于符号链接改造（commit `19fd846`），需在 `venv.rs` 现有 `heal_venv()` 实现基础上重构。