# Venv 子命令简化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 移除 `jx venv` 的 `activate`、`deactivate`、`list` 子命令

**Architecture:** 直接从 `VenvSubcommands` 枚举和相关代码中删除这三个命令的定义、参数结构体、函数实现及辅助函数

**Tech Stack:** Rust, clap derive

---

## 文件变更概览

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/commands/venv.rs` | 修改 | 移除子命令枚举成员、Args 结构体、函数 |

---

### Task 1: 移除 VenvActivateArgs 结构体

**Files:**
- Modify: `src/commands/venv.rs:117-124`

- [ ] **Step 1: 删除 VenvActivateArgs 结构体定义**

删除以下代码块（第117-124行）：

```rust
#[derive(clap::Args)]
pub struct VenvActivateArgs {
    #[clap(index = 1, help = "虚拟环境名称")]
    pub name: Option<String>,

    #[clap(short, long, help = "将激活脚本写入当前Shell的配置文件，实现永久激活")]
    pub permanent: bool,
}
```

- [ ] **Step 2: 删除 VenvActivateArgs 的 execute 实现**

删除以下代码块（第163-167行）：

```rust
impl VenvActivateArgs {
    pub fn execute(&self) -> Result<()> {
        activate(self.name.clone(), self.permanent)
    }
}
```

---

### Task 2: 移除 VenvSubcommands 枚举成员

**Files:**
- Modify: `src/commands/venv.rs:80-100`

- [ ] **Step 1: 从枚举中移除 Activate、Deactivate、List**

将 `VenvSubcommands` 枚举从：

```rust
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
```

改为：

```rust
#[derive(clap::Subcommand)]
pub enum VenvSubcommands {
    #[clap(about = "创建虚拟环境")]
    Create(VenvCreateArgs),

    #[clap(about = "删除虚拟环境")]
    Remove(VenvRemoveArgs),

    #[clap(about = "显示虚拟环境信息")]
    Info(VenvInfoArgs),
}
```

---

### Task 3: 移除 VenvCommand::execute 分发逻辑

**Files:**
- Modify: `src/commands/venv.rs:138-149`

- [ ] **Step 1: 修改 execute match 分支**

将 `VenvCommand::execute` 的 match 从：

```rust
impl VenvCommand {
    pub async fn execute(&self, _verbose: bool) -> Result<()> {
        match &self.command {
            VenvSubcommands::Create(args) => args.execute().await,
            VenvSubcommands::Activate(args) => args.execute(),
            VenvSubcommands::Deactivate => deactivate(),
            VenvSubcommands::List => list(),
            VenvSubcommands::Remove(args) => args.execute(),
            VenvSubcommands::Info(args) => args.execute(),
        }
    }
}
```

改为：

```rust
impl VenvCommand {
    pub async fn execute(&self, _verbose: bool) -> Result<()> {
        match &self.command {
            VenvSubcommands::Create(args) => args.execute().await,
            VenvSubcommands::Remove(args) => args.execute(),
            VenvSubcommands::Info(args) => args.execute(),
        }
    }
}
```

---

### Task 4: 删除 activate、deactivate、list 函数

**Files:**
- Modify: `src/commands/venv.rs:256-387, 389-476`

- [ ] **Step 1: 删除 activate 函数**

删除第256-340行的 `activate()` 函数完整实现。

- [ ] **Step 2: 删除 deactivate 函数**

删除第342-387行的 `deactivate()` 函数完整实现。

- [ ] **Step 3: 删除 list 函数**

删除第389-476行的 `list()` 函数完整实现。

---

### Task 5: 删除全局 venv 目录辅助函数

**Files:**
- Modify: `src/commands/venv.rs:685-695`

- [ ] **Step 1: 删除 get_venv_base_directory 函数**

删除第685-690行：

```rust
fn get_venv_base_directory() -> Result<PathBuf> {
    let jx_home = get_jx_home()?;
    let venv_base = jx_home.join("venvs");
    fs::create_dir_all(&venv_base)?;
    Ok(venv_base)
}
```

- [ ] **Step 2: 删除 get_venv_directory 函数**

删除第692-695行：

```rust
fn get_venv_directory(name: &str) -> Result<PathBuf> {
    let venv_base = get_venv_base_directory()?;
    Ok(venv_base.join(name))
}
```

---

### Task 6: 删除 shell profile 相关辅助函数

**Files:**
- Modify: `src/commands/venv.rs:707-814`

- [ ] **Step 1: 删除 get_shell_profile_path 函数**

删除第707-726行：

```rust
fn get_shell_profile_path() -> Option<PathBuf> {
    let shell_path = env::var("SHELL").ok();
    let shell_name = shell_path
        .as_deref()
        .and_then(|p| Path::new(p).file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("sh");

    let relative = match shell_name {
        "zsh" => PathBuf::from(".zshrc"),
        "bash" => PathBuf::from(".bashrc"),
        "fish" => PathBuf::from(".config/fish/config.fish"),
        "tcsh" => PathBuf::from(".tcshrc"),
        "csh" => PathBuf::from(".cshrc"),
        _ => PathBuf::from(".profile"),
    };

    let home = dirs::home_dir()?;
    Some(home.join(relative))
}
```

- [ ] **Step 2: 删除 ensure_shell_profile_activation 函数**

删除第728-764行。

- [ ] **Step 3: 删除 clear_shell_profile_activation 函数**

删除第766-779行。

- [ ] **Step 4: 删除 strip_shell_profile_block 函数**

删除第781-814行。

---

### Task 7: 验证编译

- [ ] **Step 1: 运行 cargo build**

Run: `cargo build`
Expected: 编译成功，无错误

- [ ] **Step 2: 运行 cargo clippy（可选）**

Run: `cargo clippy`
Expected: 无警告或仅与未改动代码相关的警告

---

### Task 8: 验证命令变更

- [ ] **Step 1: 检查 venv help 输出**

Run: `cargo run -- venv --help`
Expected: 仅显示 `create`、`remove`、`info` 子命令

- [ ] **Step 2: 验证旧命令报错**

Run: `cargo run -- venv activate`
Expected: 显示 "unrecognized subcommand" 错误

- [ ] **Step 3: 提交变更**

```bash
git add src/commands/venv.rs
git commit -m "refactor: remove venv activate, deactivate, list subcommands"
```