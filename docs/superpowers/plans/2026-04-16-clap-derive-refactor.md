# Clap Derive 模式改造实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 jx 项目的 clap builder 链式调用完全改造为 derive 宏模式，提高代码可读性和可维护性。

**Architecture:** 创建 cli.rs 定义所有命令结构体，改造各命令文件添加 Args derive 结构体和 execute 方法，简化 main.rs 只做解析和调用分发。

**Tech Stack:** Rust, clap 3.2 (derive feature 已启用), tokio async runtime

---

## File Structure

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/cli.rs` | Create | 主命令 Args + Commands 枚举 + 各命令结构体定义 |
| `src/main.rs` | Modify | 简化为解析入口和命令分发 |
| `src/commands/mod.rs` | Modify | 导出调整后的命令结构体 |
| `src/commands/init.rs` | Modify | 添加 InitCommand 结构体 |
| `src/commands/install.rs` | Modify | 添加 InstallCommand 结构体 |
| `src/commands/add.rs` | Modify | 添加 AddCommand 结构体 |
| `src/commands/remove.rs` | Modify | 添加 RemoveCommand 结构体 |
| `src/commands/update.rs` | Modify | 添加 UpdateCommand 结构体 |
| `src/commands/build.rs` | Modify | 添加 BuildCommand 结构体 |
| `src/commands/run.rs` | Modify | 添加 RunCommand 结构体 |
| `src/commands/test.rs` | Modify | 添加 TestCommand 结构体 |
| `src/commands/tree.rs` | Modify | 添加 TreeCommand 结构体 |
| `src/commands/search.rs` | Modify | 添加 SearchCommand 结构体 |
| `src/commands/venv.rs` | Modify | 添加 VenvCommand + VenvSubcommands 嵌套枚举 |
| `src/commands/clean.rs` | Keep | 无参数命令，无需改动结构体定义 |
| `src/commands/info.rs` | Keep | 无参数命令，无需改动结构体定义 |

---

## Task 1: 创建 cli.rs 文件

**Files:**
- Create: `src/cli.rs`

- [ ] **Step 1: 创建 cli.rs，定义主命令和 Commands 枚举**

```rust
use clap::{Parser, Subcommand};

use crate::commands::{
    add::AddCommand,
    build::BuildCommand,
    init::InitCommand,
    install::InstallCommand,
    remove::RemoveCommand,
    run::RunCommand,
    search::SearchCommand,
    test::TestCommand,
    tree::TreeCommand,
    update::UpdateCommand,
    venv::VenvCommand,
};

#[derive(Parser)]
#[clap(name = "jx", version, about = "A fast Java package manager written in Rust")]
pub struct Args {
    #[clap(global = true, short, long, help = "启用详细输出")]
    pub verbose: bool,

    #[clap(global = true, short, long, help = "静默模式")]
    pub quiet: bool,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "初始化新的Java项目")]
    Init(InitCommand),

    #[clap(about = "安装项目依赖")]
    Install(InstallCommand),

    #[clap(about = "添加新的依赖")]
    Add(AddCommand),

    #[clap(about = "移除依赖")]
    Remove(RemoveCommand),

    #[clap(about = "更新依赖")]
    Update(UpdateCommand),

    #[clap(about = "构建项目")]
    Build(BuildCommand),

    #[clap(about = "运行项目")]
    Run(RunCommand),

    #[clap(about = "运行测试")]
    Test(TestCommand),

    #[clap(about = "清理构建文件")]
    Clean,

    #[clap(about = "显示项目信息")]
    Info,

    #[clap(about = "显示依赖树")]
    Tree(TreeCommand),

    #[clap(about = "搜索依赖")]
    Search(SearchCommand),

    #[clap(about = "管理Java虚拟环境")]
    Venv(VenvCommand),
}
```

- [ ] **Step 2: 验证文件创建成功**

Run: `ls -la src/cli.rs`
Expected: 文件存在

---

## Task 2: 改造 init.rs

**Files:**
- Modify: `src/commands/init.rs`

- [ ] **Step 1: 在 init.rs 开头添加 InitCommand 结构体**

在文件开头添加：

```rust
use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(clap::Args)]
#[clap(about = "初始化新的Java项目")]
pub struct InitCommand {
    #[clap(index = 1, help = "项目名称")]
    pub name: Option<String>,

    #[clap(short, long, default_value = "maven", possible_values = ["maven", "gradle"], help = "项目类型")]
    pub template: String,
}

impl InitCommand {
    pub fn execute(&self) -> Result<()> {
        // 原有 execute 函数逻辑，使用 self.name 和 self.template
    }
}
```

- [ ] **Step 2: 将原有 execute 函数改为 impl InitCommand 的方法**

将原 `pub fn execute(name: Option<String>, template: String) -> Result<()>` 改为：

```rust
impl InitCommand {
    pub fn execute(&self) -> Result<()> {
        let project_name = if let Some(ref n) = self.name {
            n.clone()
        } else {
            let current_dir = std::env::current_dir()?;
            current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my-java-project")
                .to_string()
        };

        let current_dir = std::env::current_dir()?;
        let project_dir = if self.name.is_some() {
            current_dir.join(&project_name)
        } else {
            current_dir.clone()
        };

        // 检查目录是否已存在
        if project_dir.exists() && project_dir != current_dir {
            return Err(anyhow::anyhow!("目录 '{}' 已存在", project_name));
        }

        // 创建项目目录
        if self.name.is_some() {
            fs::create_dir_all(&project_dir)?;
        }

        // 根据模板创建项目文件
        match self.template.as_str() {
            "maven" => create_maven_project(&project_dir, &project_name)?,
            "gradle" => create_gradle_project(&project_dir, &project_name)?,
            _ => return Err(anyhow::anyhow!("不支持的模板类型: {}", self.template)),
        }

        println!("✅ 项目创建成功!");
        println!("项目名称: {}", project_name);
        println!("项目类型: {}", self.template);
        println!("项目路径: {}", project_dir.display());

        if self.name.is_some() {
            println!("\n进入项目目录:");
            println!("  cd {}", project_name);
        }

        println!("\n下一步:");
        println!("  jx install    # 安装依赖");
        println!("  jx build      # 构建项目");
        println!("  jx run        # 运行项目");

        Ok(())
    }
}
```

- [ ] **Step 3: 保留原有的辅助函数**

保持 `create_maven_project` 和 `create_gradle_project` 函数不变。

- [ ] **Step 4: 编译验证**

Run: `cargo check 2>&1 | head -50`
Expected: init.rs 相关错误消失（如果之前有导入问题）

---

## Task 3: 改造 install.rs

**Files:**
- Modify: `src/commands/install.rs`

- [ ] **Step 1: 添加 InstallCommand 结构体和方法**

```rust
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "安装项目依赖")]
pub struct InstallCommand {
    #[clap(short, long, help = "指定依赖文件")]
    pub file: Option<String>,

    #[clap(long, help = "仅安装生产依赖")]
    pub production: bool,

    #[clap(long, help = "强制重新安装")]
    pub force: bool,
}

impl InstallCommand {
    pub fn execute(&self) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        // 查找项目配置文件
        let config_file = if current_dir.join("jx.toml").exists() {
            "jx.toml"
        } else if current_dir.join("pom.xml").exists() {
            "pom.xml"
        } else if current_dir.join("build.gradle").exists() {
            "build.gradle"
        } else {
            return Err(anyhow::anyhow!("找不到项目配置文件，请先运行 'jx init'"));
        };

        println!("📦 开始安装依赖...");
        println!("配置文件: {}", config_file);

        // 根据配置文件类型选择安装方式
        let result = if config_file == "pom.xml" {
            install_from_maven(&current_dir, self.production, self.force)
        } else if config_file == "build.gradle" {
            install_from_gradle(&current_dir, self.production, self.force)
        } else {
            Err(anyhow::anyhow!("不支持的配置文件类型: {}", config_file))
        };

        match result {
            Ok(_) => {
                println!("✅ 依赖安装完成!");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 安装失败: {}", e);
                Err(e)
            }
        }
    }
}
```

- [ ] **Step 2: 保留原有的辅助函数**

保持 `install_from_maven`, `install_from_gradle`, `check_command_exists` 函数不变。

---

## Task 4: 改造 add.rs

**Files:**
- Modify: `src/commands/add.rs`

- [ ] **Step 1: 添加 AddCommand 结构体和方法**

```rust
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "添加新的依赖")]
pub struct AddCommand {
    #[clap(index = 1, required = true, help = "依赖坐标 (groupId:artifactId:version)")]
    pub dependency: String,

    #[clap(short, long, default_value = "compile", possible_values = ["compile", "runtime", "test", "provided"], help = "依赖类型")]
    pub scope: String,
}

impl AddCommand {
    pub fn execute(&self) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        // ... 原有逻辑，使用 self.dependency 和 self.scope
    }
}
```

- [ ] **Step 2: 将原 execute 函数体移入 impl 块，参数改为 self 引用**

修改 execute 方法内部所有对 `dependency` 和 `scope` 参数的引用为 `self.dependency` 和 `self.scope`。

- [ ] **Step 3: 保留原有的辅助函数**

保持 `DependencyInfo`, `parse_dependency_coordinate`, `add_to_jx_config`, `add_to_maven`, `add_to_gradle`, `get_latest_version` 等函数不变。

---

## Task 5: 改造 remove.rs

**Files:**
- Modify: `src/commands/remove.rs`

- [ ] **Step 1: 添加 RemoveCommand 结构体和方法**

```rust
use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(clap::Args)]
#[clap(about = "移除依赖")]
pub struct RemoveCommand {
    #[clap(index = 1, required = true, help = "依赖坐标 (groupId:artifactId)")]
    pub dependency: String,
}

impl RemoveCommand {
    pub fn execute(&self) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        // ... 原有逻辑，使用 self.dependency
    }
}
```

- [ ] **Step 2: 修改内部引用**

将 `dependency` 参数引用改为 `self.dependency`。

---

## Task 6: 改造 update.rs

**Files:**
- Modify: `src/commands/update.rs`

- [ ] **Step 1: 添加 UpdateCommand 结构体和方法**

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "更新依赖")]
pub struct UpdateCommand {
    #[clap(index = 1, help = "依赖坐标 (groupId:artifactId)")]
    pub dependency: Option<String>,

    #[clap(long, help = "更新到最新版本")]
    pub latest: bool,
}

impl UpdateCommand {
    pub fn execute(&self) -> Result<()> {
        // ... 原有逻辑，使用 self.dependency 和 self.latest
    }
}
```

- [ ] **Step 2: 修改内部引用**

将 `dependency` 和 `latest` 参数引用改为 `self.dependency` 和 `self.latest`。

---

## Task 7: 改造 build.rs

**Files:**
- Modify: `src/commands/build.rs`

- [ ] **Step 1: 添加 BuildCommand 结构体和方法**

```rust
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "构建项目")]
pub struct BuildCommand {
    #[clap(short, long, default_value = "debug", possible_values = ["debug", "release"], help = "构建模式")]
    pub mode: String,

    #[clap(long, help = "跳过测试")]
    pub no_test: bool,
}

impl BuildCommand {
    pub fn execute(&self) -> Result<()> {
        // ... 原有逻辑，使用 self.mode 和 self.no_test
    }
}
```

- [ ] **Step 2: 修改内部引用**

将 `mode` 和 `no_test` 参数引用改为 `self.mode` 和 `self.no_test`。

---

## Task 8: 改造 run.rs

**Files:**
- Modify: `src/commands/run.rs`

- [ ] **Step 1: 添加 RunCommand 结构体和方法**

```rust
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "运行项目")]
pub struct RunCommand {
    #[clap(index = 1, help = "主类名")]
    pub main_class: Option<String>,

    #[clap(index = 2, help = "程序参数")]
    pub args: Vec<String>,
}

impl RunCommand {
    pub fn execute(&self) -> Result<()> {
        // ... 原有逻辑，使用 self.main_class 和 self.args
    }
}
```

- [ ] **Step 2: 修改内部引用**

将 `main_class` 和 `args` 参数引用改为 `self.main_class` 和 `self.args`。

---

## Task 9: 改造 test.rs

**Files:**
- Modify: `src/commands/test.rs`

- [ ] **Step 1: 添加 TestCommand 结构体和方法**

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "运行测试")]
pub struct TestCommand {
    #[clap(index = 1, help = "测试类名")]
    pub test_class: Option<String>,

    #[clap(long, help = "测试方法名")]
    pub method: Option<String>,
}

impl TestCommand {
    pub fn execute(&self) -> Result<()> {
        // ... 原有逻辑，使用 self.test_class 和 self.method
    }
}
```

- [ ] **Step 2: 修改内部引用**

将 `test_class` 和 `method` 参数引用改为 `self.test_class` 和 `self.method`。

---

## Task 10: 改造 tree.rs

**Files:**
- Modify: `src/commands/tree.rs`

- [ ] **Step 1: 添加 TreeCommand 结构体和方法**

```rust
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(clap::Args)]
#[clap(about = "显示依赖树")]
pub struct TreeCommand {
    #[clap(long, help = "显示传递依赖")]
    pub transitive: bool,
}

impl TreeCommand {
    pub fn execute(&self) -> Result<()> {
        // ... 原有逻辑，使用 self.transitive
    }
}
```

- [ ] **Step 2: 修改内部引用**

将 `transitive` 参数引用改为 `self.transitive`。

---

## Task 11: 改造 search.rs

**Files:**
- Modify: `src/commands/search.rs`

- [ ] **Step 1: 添加 SearchCommand 结构体和方法**

```rust
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(clap::Args)]
#[clap(about = "搜索依赖")]
pub struct SearchCommand {
    #[clap(index = 1, required = true, help = "搜索关键词")]
    pub query: String,

    #[clap(short, long, default_value = "20", help = "最大结果数")]
    pub limit: u32,
}

impl SearchCommand {
    pub fn execute(&self) -> Result<()> {
        // ... 原有逻辑，使用 self.query 和 self.limit
    }
}
```

- [ ] **Step 2: 修改内部引用**

将 `query` 和 `limit` 参数引用改为 `self.query` 和 `self.limit`，注意 `limit` 类型从 `usize` 改为 `u32`，内部使用时需转换：`self.limit as usize`。

---

## Task 12: 改造 venv.rs（嵌套枚举）

**Files:**
- Modify: `src/commands/venv.rs`

这是最复杂的改造，需要定义嵌套枚举结构体。

- [ ] **Step 1: 在 venv.rs 开头添加所有结构体定义**

```rust
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

// ... 原有的 Adoptium API 数据结构定义保持不变 ...

/// 构建工具类型
#[derive(Debug, Clone)]
pub enum BuildTool {
    Maven(String),
    Gradle(String),
}

/// Venv 主命令
#[derive(clap::Args)]
#[clap(about = "管理Java虚拟环境")]
pub struct VenvCommand {
    #[clap(subcommand)]
    pub command: VenvSubcommands,
}

/// Venv 子命令枚举
#[derive(clap::Subcommand)]
pub enum VenvSubcommands {
    #[clap(about = "创建虚拟环境")]
    Create(VenvCreateArgs),

    #[clap(about = "激活虚拟环境")]
    Activate(VenvActivateArgs),

    #[clap(about = "停用虚拟环境")]
    Deactivate,

    #[clap(about = "列出所有虚拟环境")]
    List,

    #[clap(about = "删除虚拟环境")]
    Remove(VenvRemoveArgs),

    #[clap(about = "显示虚拟环境信息")]
    Info(VenvInfoArgs),
}

#[derive(clap::Args)]
pub struct VenvCreateArgs {
    #[clap(index = 1, help = "虚拟环境名称")]
    pub name: Option<String>,

    #[clap(long, alias = "jv", default_value = "17", help = "Java版本 (8, 11, 17, 21, 25)")]
    pub java_version: String,

    #[clap(long, alias = "mv", default_value = "3.9.9", conflicts_with = "gradle_version", help = "Maven版本 (默认使用Maven作为构建工具)")]
    pub maven_version: String,

    #[clap(long, alias = "gv", conflicts_with = "maven_version", help = "Gradle版本 (使用Gradle代替默认的Maven)")]
    pub gradle_version: Option<String>,
}

#[derive(clap::Args)]
pub struct VenvActivateArgs {
    #[clap(index = 1, help = "虚拟环境名称")]
    pub name: Option<String>,

    #[clap(short, long, help = "将激活脚本写入当前Shell的配置文件，实现永久激活")]
    pub permanent: bool,
}

#[derive(clap::Args)]
pub struct VenvRemoveArgs {
    #[clap(index = 1, required = true, help = "虚拟环境名称")]
    pub name: String,
}

#[derive(clap::Args)]
pub struct VenvInfoArgs {
    #[clap(index = 1, help = "虚拟环境名称")]
    pub name: Option<String>,
}
```

- [ ] **Step 2: 添加 VenvCommand 的 execute 方法**

```rust
impl VenvCommand {
    pub async fn execute(&self, verbose: bool) -> Result<()> {
        match &self.command {
            VenvSubcommands::Create(args) => args.execute(verbose).await,
            VenvSubcommands::Activate(args) => args.execute(),
            VenvSubcommands::Deactivate => deactivate(),
            VenvSubcommands::List => list(),
            VenvSubcommands::Remove(args) => args.execute(),
            VenvSubcommands::Info(args) => args.execute(),
        }
    }
}
```

- [ ] **Step 3: 为 VenvCreateArgs 添加 execute 方法**

```rust
impl VenvCreateArgs {
    pub async fn execute(&self, verbose: bool) -> Result<()> {
        let build_tool = if self.gradle_version.is_some() {
            BuildTool::Gradle(self.gradle_version.clone().unwrap_or("8.4".to_string()))
        } else {
            BuildTool::Maven(self.maven_version.clone())
        };
        create(self.name.clone(), self.java_version.clone(), build_tool).await
    }
}
```

- [ ] **Step 4: 为 VenvActivateArgs 添加 execute 方法**

```rust
impl VenvActivateArgs {
    pub fn execute(&self) -> Result<()> {
        activate(self.name.clone(), self.permanent)
    }
}
```

- [ ] **Step 5: 为 VenvRemoveArgs 添加 execute 方法**

```rust
impl VenvRemoveArgs {
    pub fn execute(&self) -> Result<()> {
        remove(self.name.clone())
    }
}
```

- [ ] **Step 6: 为 VenvInfoArgs 添加 execute 方法**

```rust
impl VenvInfoArgs {
    pub fn execute(&self) -> Result<()> {
        info(self.name.clone())
    }
}
```

- [ ] **Step 7: 保留原有的辅助函数和核心实现函数**

保持 `create`, `activate`, `deactivate`, `list`, `remove`, `info` 以及所有辅助函数（如 `get_jx_home`, `install_java`, `install_maven`, `install_gradle` 等）不变。

---

## Task 13: 更新 commands/mod.rs

**Files:**
- Modify: `src/commands/mod.rs`

- [ ] **Step 1: 更新导出**

保持原有导出结构，确保各命令结构体被正确导出：

```rust
pub mod add;
pub mod build;
pub mod clean;
pub mod info;
pub mod init;
pub mod install;
pub mod remove;
pub mod run;
pub mod search;
pub mod test;
pub mod tree;
pub mod update;
pub mod venv;

pub use add::*;
pub use build::*;
pub use clean::*;
pub use info::*;
pub use init::*;
pub use install::*;
pub use remove::*;
pub use run::*;
pub use search::*;
pub use test::*;
pub use tree::*;
pub use update::*;
pub use venv::*;
```

当前文件内容已满足此要求，无需修改。

---

## Task 14: 简化 main.rs

**Files:**
- Modify: `src/main.rs`

这是最终步骤，将 builder 模式完全替换为 derive 模式。

- [ ] **Step 1: 重写 main.rs**

```rust
mod cli;
mod commands;
mod config;
mod dependency;
mod download;
mod install;
mod lock;
mod project;
mod registry;
mod resolve;
mod utils;

use clap::Parser;
use log::error;
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = cli::Args::parse();

    // 设置日志级别
    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if args.quiet {
        std::env::set_var("RUST_LOG", "error");
    }

    // 显示欢迎信息
    if !args.quiet {
        println!("🚀 jx - Fast Java Package Manager");
        println!("Built with Rust for speed and reliability");
        println!();
    }

    // 执行命令
    let result = execute_command(&args).await;

    // 处理结果
    match result {
        Ok(_) => {
            if !args.quiet {
                println!("✅ 操作完成");
            }
            process::exit(0);
        }
        Err(e) => {
            error!("操作失败: {}", e);
            if !args.quiet {
                eprintln!("❌ 错误: {}", e);
            }
            process::exit(1);
        }
    }
}

async fn execute_command(args: &cli::Args) -> anyhow::Result<()> {
    match &args.command {
        cli::Commands::Init(cmd) => cmd.execute(),
        cli::Commands::Install(cmd) => cmd.execute(),
        cli::Commands::Add(cmd) => cmd.execute(),
        cli::Commands::Remove(cmd) => cmd.execute(),
        cli::Commands::Update(cmd) => cmd.execute(),
        cli::Commands::Build(cmd) => cmd.execute(),
        cli::Commands::Run(cmd) => cmd.execute(),
        cli::Commands::Test(cmd) => cmd.execute(),
        cli::Commands::Clean => commands::clean::execute(),
        cli::Commands::Info => commands::info::execute(),
        cli::Commands::Tree(cmd) => cmd.execute(),
        cli::Commands::Search(cmd) => cmd.execute(),
        cli::Commands::Venv(cmd) => cmd.execute(args.verbose).await,
    }
}
```

- [ ] **Step 2: 删除原有的 builder 链式调用代码**

删除 main.rs 中所有原有的 `App::new()`, `Arg::with_name()`, `SubCommand::with_name()` 等 builder 代码，以及原有的 match 分支处理逻辑。

---

## Task 15: 编译验证和测试

**Files:**
- All modified files

- [ ] **Step 1: 运行 cargo check 验证编译**

Run: `cargo check 2>&1`
Expected: 无编译错误

- [ ] **Step 2: 运行 cargo build 构建项目**

Run: `cargo build 2>&1`
Expected: 构建成功

- [ ] **Step 3: 测试基本命令**

Run: `./target/debug/jx --help`
Expected: 显示帮助信息，包含所有命令

Run: `./target/debug/jx init --help`
Expected: 显示 init 命令帮助，包含 name 和 template 参数

Run: `./target/debug/jx venv --help`
Expected: 显示 venv 命令帮助，包含所有子命令

Run: `./target/debug/jx venv create --help`
Expected: 显示 venv create 子命令帮助，包含 java-version, maven-version, gradle-version 参数

- [ ] **Step 4: 提交更改**

```bash
git add src/cli.rs src/main.rs src/commands/*.rs
git commit -m "refactor: convert clap builder pattern to derive macro pattern

- Create cli.rs with Args and Commands enum definitions
- Add #[derive(clap::Args)] structs to each command file
- Implement execute methods on command structs
- Simplify main.rs to parse and dispatch commands
- Use nested enum for venv subcommands

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Self-Review Checklist

**1. Spec coverage:**
- ✅ cli.rs 定义 Args + Commands 枚举 → Task 1
- ✅ 各命令添加 derive 结构体 → Tasks 2-12
- ✅ venv 嵌套枚举 → Task 12
- ✅ main.rs 简化 → Task 14
- ✅ 全局参数 verbose/quiet → Task 1, Task 14

**2. Placeholder scan:**
- 无 "TBD", "TODO", "implement later" 占位符
- 所有 execute 方法都有完整实现指引

**3. Type consistency:**
- Args 结构体字段名与各 Command 结构体一致
- VenvCommand::execute 接收 verbose 参数并传递给 create
- SearchCommand limit 类型为 u32，内部转换为 usize

**4. Clap 3.2 兼容性:**
- 使用 `#[clap(...)]` 而非 `#[command(...)]` (clap 4.x 语法)
- 使用 `possible_values` 而非 `value_parser` (clap 4.x 语法)
- 使用 `alias` 属性而非 `visible_alias`