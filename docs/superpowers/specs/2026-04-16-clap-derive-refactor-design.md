# Clap Builder 模式改造为 Derive 模式设计文档

## 概述

将 jx 项目的命令行参数解析从 clap builder 链式模式完全改造为 derive 宏模式，提高代码可读性和可维护性。

## 目标

- 将 main.rs 中约 400 行 builder 链式调用改为 derive 结构体定义
- 保留所有现有命令的语义（名称、参数、默认值、别名）
- 将命令参数定义与处理逻辑合并到同一文件
- 使用嵌套枚举处理 venv 子命令

## 文件结构

```
src/
├── main.rs          # 简化为解析和调用入口
├── cli.rs           # 新增：命令结构体定义
└── commands/
    ├── mod.rs       # 模块导出
    ├── init.rs      # InitCommand + execute
    ├── install.rs   # InstallCommand + execute
    ├── add.rs       # AddCommand + execute
    ├── remove.rs    # RemoveCommand + execute
    ├── update.rs    # UpdateCommand + execute
    ├── build.rs     # BuildCommand + execute
    ├── run.rs       # RunCommand + execute
    ├── test.rs      # TestCommand + execute
    ├── clean.rs     # CleanCommand（无参数）
    ├── info.rs      # InfoCommand（无参数）
    ├── tree.rs      # TreeCommand + execute
    ├── search.rs    # SearchCommand + execute
    └── venv.rs      # VenvCommand + 嵌套枚举 + execute
```

## 命令结构体设计

### 主入口 (cli.rs)

```rust
use clap::{Parser, Subcommand};

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

### 各命令结构体

#### InitCommand

```rust
#[derive(clap::Args)]
pub struct InitCommand {
    #[clap(index = 1, help = "项目名称")]
    pub name: Option<String>,

    #[clap(short, long, default_value = "maven", 
           possible_values = ["maven", "gradle"], help = "项目类型")]
    pub template: String,
}
```

#### InstallCommand

```rust
#[derive(clap::Args)]
pub struct InstallCommand {
    #[clap(short, long, help = "指定依赖文件")]
    pub file: Option<String>,

    #[clap(long, help = "仅安装生产依赖")]
    pub production: bool,

    #[clap(long, help = "强制重新安装")]
    pub force: bool,
}
```

#### AddCommand

```rust
#[derive(clap::Args)]
pub struct AddCommand {
    #[clap(index = 1, required = true, 
           help = "依赖坐标 (groupId:artifactId:version)")]
    pub dependency: String,

    #[clap(short, long, default_value = "compile", 
           possible_values = ["compile", "runtime", "test", "provided"],
           help = "依赖类型")]
    pub scope: String,
}
```

#### RemoveCommand

```rust
#[derive(clap::Args)]
pub struct RemoveCommand {
    #[clap(index = 1, required = true, 
           help = "依赖坐标 (groupId:artifactId)")]
    pub dependency: String,
}
```

#### UpdateCommand

```rust
#[derive(clap::Args)]
pub struct UpdateCommand {
    #[clap(index = 1, help = "依赖坐标 (groupId:artifactId)")]
    pub dependency: Option<String>,

    #[clap(long, help = "更新到最新版本")]
    pub latest: bool,
}
```

#### BuildCommand

```rust
#[derive(clap::Args)]
pub struct BuildCommand {
    #[clap(short, long, default_value = "debug", 
           possible_values = ["debug", "release"], help = "构建模式")]
    pub mode: String,

    #[clap(long, help = "跳过测试")]
    pub no_test: bool,
}
```

#### RunCommand

```rust
#[derive(clap::Args)]
pub struct RunCommand {
    #[clap(index = 1, help = "主类名")]
    pub main_class: Option<String>,

    #[clap(index = 2, help = "程序参数")]
    pub args: Vec<String>,
}
```

#### TestCommand

```rust
#[derive(clap::Args)]
pub struct TestCommand {
    #[clap(index = 1, help = "测试类名")]
    pub test_class: Option<String>,

    #[clap(long, help = "测试方法名")]
    pub method: Option<String>,
}
```

#### TreeCommand

```rust
#[derive(clap::Args)]
pub struct TreeCommand {
    #[clap(long, help = "显示传递依赖")]
    pub transitive: bool,
}
```

#### SearchCommand

```rust
#[derive(clap::Args)]
pub struct SearchCommand {
    #[clap(index = 1, required = true, help = "搜索关键词")]
    pub query: String,

    #[clap(short, long, default_value = "20", help = "最大结果数")]
    pub limit: u32,
}
```

### Venv 嵌套子命令

```rust
#[derive(clap::Args)]
pub struct VenvCommand {
    #[clap(subcommand)]
    pub command: VenvSubcommands,
}

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

    #[clap(long, alias = "jv", default_value = "17", 
           help = "Java版本 (8, 11, 17, 21, 25)")]
    pub java_version: String,

    #[clap(long, alias = "mv", default_value = "3.9.9", 
           conflicts_with = "gradle_version",
           help = "Maven版本 (默认使用Maven作为构建工具)")]
    pub maven_version: String,

    #[clap(long, alias = "gv", 
           conflicts_with = "maven_version",
           help = "Gradle版本 (使用Gradle代替默认的Maven)")]
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

## main.rs 简化设计

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

## 命令文件改造示例

### add.rs 改造后

```rust
use anyhow::Result;

#[derive(clap::Args)]
#[clap(about = "添加新的依赖")]
pub struct AddCommand {
    #[clap(index = 1, required = true, 
           help = "依赖坐标 (groupId:artifactId:version)")]
    pub dependency: String,

    #[clap(short, long, default_value = "compile", 
           possible_values = ["compile", "runtime", "test", "provided"],
           help = "依赖类型")]
    pub scope: String,
}

impl AddCommand {
    pub fn execute(&self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        // ... 原有逻辑，使用 self.dependency 和 self.scope
    }
}
```

### venv.rs 改造要点

- 定义 `VenvCommand` 结构体和 `VenvSubcommands` 枚举
- 各子命令参数结构体定义
- `VenvCommand::execute` 方法匹配子命令并调用对应处理函数
- 原有的辅助函数保持不变

## 保留语义对照表

| 原参数 | derive 映射 |
|--------|-------------|
| `.short('v')` | `#[clap(short)]` |
| `.long("verbose")` | `#[clap(long)]` |
| `.default_value("maven")` | `#[clap(default_value = "maven")]` |
| `.possible_values(&["maven", "gradle"])` | `#[clap(possible_values = ["maven", "gradle"])` |
| `.index(1)` | `#[clap(index = 1)]` |
| `.required(true)` | `#[clap(required = true)]` |
| `.takes_value(true)` | 默认行为（Option 或 String 类型） |
| `.multiple(true)` | 使用 `Vec<String>` 类型 |
| `.conflicts_with("gradle-version")` | `#[clap(conflicts_with = "gradle_version")]` |
| `.alias("jv")` | `#[clap(alias = "jv")]` |

## 注意事项

1. **clap 版本**: 当前使用 clap 3.2，derive 模式语法与 clap 4.x 有差异，需注意兼容性
2. **async 命令**: venv 的 create 是 async，execute 方法返回 `anyhow::Result<()>` 需在 main.rs 调用时 await
3. **参数重命名**: clap derive 中字段名会作为 long 参数名，snake_case 会自动转换为 kebab-case（如 `java_version` -> `java-version`）
4. **Clean/Info 命令**: 无参数命令，在 Commands 枚举中不带结构体变体

## 改动量估算

| 文件 | 原行数 | 改造后行数 | 变化 |
|------|--------|------------|------|
| main.rs | ~410 | ~50 | 减少 85% |
| cli.rs | - | ~80 | 新增 |
| commands/init.rs | ~50 | ~60 | +10 |
| commands/install.rs | ~40 | ~50 | +10 |
| commands/add.rs | ~340 | ~350 | +10 |
| commands/remove.rs | ~30 | ~40 | +10 |
| commands/update.rs | ~40 | ~50 | +10 |
| commands/build.rs | ~150 | ~160 | +10 |
| commands/run.rs | ~60 | ~70 | +10 |
| commands/test.rs | ~50 | ~60 | +10 |
| commands/clean.rs | ~30 | ~30 | 不变 |
| commands/info.rs | ~40 | ~40 | 不变 |
| commands/tree.rs | ~50 | ~60 | +10 |
| commands/search.rs | ~50 | ~60 | +10 |
| commands/venv.rs | ~1670 | ~1680 | +10 |

**总计**: 原约 3100 行 → 改造后约 2600 行（减少约 15%，主要是 main.rs 简化）