# 移除 .active 激活标志文件设计

## 背景

当前 `.active` 文件作为激活标志存在，位于 `.jx/.active`。但实际 `jx run` 执行时不检查此文件，而是通过 `prepare_project_environment()` 检查 `.jx/bin/` 目录和符号链接有效性来判断环境是否可用。`.active` 文件是冗余的概念。

## 设计决策

完全移除 `.active` 文件，环境状态由实际可用的符号链接决定。

## 修改点

### 1. venv create 命令

**文件**: `src/commands/venv.rs`

**当前行为**: 创建 `.jx/.active` 空文件（第378行）

```rust
fs::write(venv_dir.join(".active"), "")?;
```

**新行为**: 移除此行，不创建 `.active` 文件

**成功提示调整**: 移除"虚拟环境已自动激活"提示，改为"虚拟环境已就绪"

### 2. venv info 命令

**文件**: `src/commands/venv.rs`

**当前行为**: 显示状态"激活/未激活"（第483-488行）

```rust
println!("状态: {}", if venv_dir.join(".active").exists() {
    "激活"
} else {
    "未激活"
});
```

**新行为**: 移除状态显示，只显示环境详情（路径、工具版本等）

### 3. venv remove 命令

**文件**: `src/commands/venv.rs`

**当前行为**（第409-411行）:
- 检查 `.active` 文件是否存在，存在时显示"停用虚拟环境..."

```rust
let active_file = venv_dir.join(".active");
if active_file.exists() {
    println!("停用虚拟环境...");
}
```

**新行为**:
- 移除 `.active` 文件检查
- 添加删除确认提示："此操作将删除 .jx/ 目录及其所有工具链接，是否继续？[y/N]"
- 用户确认后显示"已删除虚拟环境"

### 4. jx run 命令

**文件**: `src/commands/run.rs`, `src/environment.rs`

**当前行为**: 不检查 `.active` 文件，只检查 `.jx/` 和 `.jx/bin/` 目录及符号链接有效性

**新行为**: 保持不变（已符合设计）

## 不涉及的内容

- 符号链接机制保持不变
- `prepare_project_environment()` 函数保持不变
- `heal_venv()` 自愈机制保持不变

## 影响范围

- `src/commands/venv.rs`: 约 3 处修改
- 无其他文件改动