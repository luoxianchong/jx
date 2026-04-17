# Venv 子命令简化设计文档

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

## 目标

移除 `jx venv` 的 `activate`、`deactivate`、`list` 子命令，实现"创建即激活、删除即停用"的简化体验。

## 背景

基于 `2026-04-16-venv-project-level-design.md` 的项目级虚拟环境设计，虚拟环境存储在项目根目录 `.jx/`。由于环境是项目级的，不再需要全局激活/停用/列表功能。

## 命令结构变更

### 当前结构

```
jx venv
  ├── create    创建环境
  ├── activate  激活环境
  ├── deactivate 停用环境
  ├── list      列出环境
  ├── remove    删除环境
  └── info      显示信息
```

### 新结构

```
jx venv
  ├── create    创建并自动激活
  ├── remove    删除并自动停用
  └── info      显示当前项目环境信息
```

## 代码变更清单

### 1. VenvSubcommands 枚举

文件：`src/commands/venv.rs`

移除以下枚举成员：
- `Activate(VenvActivateArgs)`
- `Deactivate`
- `List`

保留：
- `Create(VenvCreateArgs)`
- `Remove(VenvRemoveArgs)`
- `Info(VenvInfoArgs)`

### 2. Args 结构体

移除：
- `VenvActivateArgs` 结构体及其 `execute()` 实现

保留：
- `VenvCreateArgs`
- `VenvRemoveArgs`
- `VenvInfoArgs`

### 3. 函数

移除：
- `activate(name: Option<String>, permanent: bool) -> Result<()>`
- `deactivate() -> Result<()>`
- `list() -> Result<()>`

移除的辅助函数：
- `get_venv_base_directory()` - 全局 venv 基目录（项目级不再需要）
- `get_venv_directory(name: &str)` - 全局 venv 目录（项目级不再需要）
- `get_shell_profile_path()` - shell 配置文件路径（永久激活不再需要）
- `ensure_shell_profile_activation()` - 写入永久激活配置
- `clear_shell_profile_activation()` - 清除永久激活配置
- `strip_shell_profile_block()` - 处理 shell profile 块

保留：
- `create()` - 创建环境（需调整为项目级）
- `remove()` - 删除环境（需调整为项目级）
- `info()` - 显示信息（需调整为项目级）
- `get_active_venv()` - 获取当前激活状态（用于 remove 检查）
- 其他安装/下载相关函数

### 4. VenvCommand::execute() 分发逻辑

修改 match 分支，移除 `Activate`、`Deactivate`、`List` 的处理。

## 实现步骤

1. 删除 `VenvActivateArgs` 结构体定义
2. 从 `VenvSubcommands` 枚举移除 `Activate`、`Deactivate`、`List`
3. 从 `VenvCommand::execute()` 移除对应的 match 分支
4. 删除 `activate()`、`deactivate()`、`list()` 函数
5. 删除 `get_venv_base_directory()`、`get_venv_directory()` 函数
6. 删除 shell profile 相关的辅助函数（`get_shell_profile_path()`、`ensure_shell_profile_activation()`、`clear_shell_profile_activation()`、`strip_shell_profile_block()`）
7. 清理任何遗留的未使用代码

## 用户影响

- 使用旧命令 `jx venv activate`、`jx venv deactivate`、`jx venv list` 将收到 clap 的 "unrecognized subcommand" 错误
- 这是设计文档预期的行为，符合项目级虚拟环境的简化理念

## 测试

- 运行 `cargo build` 确保编译成功
- 运行 `jx venv --help` 确认仅显示 `create`、`remove`、`info` 子命令
- 运行 `jx venv activate` 确认显示未知命令错误

## 依赖说明

此设计依赖于 `2026-04-16-venv-project-level-design.md` 的完整实现。如果项目级虚拟环境尚未实现，`create`、`remove`、`info` 函数需要先完成项目级改造，再进行此简化。

当前实现顺序：
1. 先完成项目级虚拟环境改造（`2026-04-16-venv-project-level-design.md`）
2. 再进行此子命令简化（当前文档）