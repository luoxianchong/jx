# 移除 .active 激活标志文件实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 移除冗余的 `.active` 文件，简化虚拟环境状态判断逻辑

**Architecture:** 修改 `venv.rs` 中三处代码：create、remove、info 函数，移除 `.active` 文件相关逻辑

**Tech Stack:** Rust, anyhow, clap

---

## 文件结构

- **Modify**: `src/commands/venv.rs` - 移除 `.active` 文件创建、检查和状态显示逻辑

---

### Task 1: 移除 venv create 中的 .active 文件创建

**Files:**
- Modify: `src/commands/venv.rs:378`
- Modify: `src/commands/venv.rs:395`

- [ ] **Step 1: 移除 .active 文件创建代码**

找到第 377-378 行：
```rust
        // 创建激活标记文件
        fs::write(venv_dir.join(".active"), "")?;
```

删除这两行。

- [ ] **Step 2: 修改成功提示消息**

找到第 395 行：
```rust
    println!("虚拟环境已自动激活。");
```

改为：
```rust
    println!("虚拟环境已就绪。");
```

- [ ] **Step 3: 编译验证**

Run: `cargo build`
Expected: 编译成功，无错误

- [ ] **Step 4: 提交**

```bash
git add src/commands/venv.rs
git commit -m "feat(venv): remove .active file creation from venv create"
```

---

### Task 2: 移除 venv info 中的状态显示

**Files:**
- Modify: `src/commands/venv.rs:480-488`

- [ ] **Step 1: 移除状态显示代码**

找到第 480-488 行：
```rust
    println!("");
    println!(
        "状态: {}",
        if venv_dir.join(".active").exists() {
            "🔌 激活"
        } else {
            "未激活"
        }
    );
```

删除这整段代码（9 行）。

- [ ] **Step 2: 编译验证**

Run: `cargo build`
Expected: 编译成功，无错误

- [ ] **Step 3: 提交**

```bash
git add src/commands/venv.rs
git commit -m "feat(venv): remove status display from venv info"
```

---

### Task 3: 添加 venv remove 删除确认提示

**Files:**
- Modify: `src/commands/venv.rs:400-422`

- [ ] **Step 1: 移除 .active 文件检查，添加确认提示**

找到 `remove()` 函数（第 400-422 行），替换为：

```rust
/// 删除虚拟环境
pub fn remove() -> Result<()> {
    let venv_dir = std::env::current_dir()?.join(".jx");

    if !venv_dir.exists() {
        return Err(anyhow::anyhow!("当前目录下不存在虚拟环境 (.jx/)"));
    }

    // 确认删除
    println!("此操作将删除 .jx/ 目录及其所有工具链接，是否继续？ [y/N]");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let answer = input.trim().to_lowercase();
    if answer != "y" && answer != "yes" {
        println!("已取消删除");
        return Ok(());
    }

    println!("🗑️ 删除虚拟环境...");
    println!("路径: {}", venv_dir.display());

    // 递归删除目录
    fs::remove_dir_all(&venv_dir)?;

    println!("✅ 虚拟环境已删除");

    Ok(())
}
```

- [ ] **Step 2: 编译验证**

Run: `cargo build`
Expected: 编译成功，无错误

- [ ] **Step 3: 提交**

```bash
git add src/commands/venv.rs
git commit -m "feat(venv): add deletion confirmation prompt to venv remove"
```

---

### Task 4: 最终验证和测试

**Files:**
- 无文件修改，仅测试验证

- [ ] **Step 1: 功能测试 - venv create**

创建测试目录并运行 venv create：
```bash
mkdir -p /tmp/jx-test && cd /tmp/jx-test
jx venv create --java-version 17
ls -la .jx/
```

Expected: `.jx/` 目录存在，但没有 `.active` 文件；成功提示显示"虚拟环境已就绪"

- [ ] **Step 2: 功能测试 - venv info**

```bash
cd /tmp/jx-test
jx venv info
```

Expected: 输出不包含"状态"行，只显示路径、版本和符号链接状态

- [ ] **Step 3: 功能测试 - venv remove**

```bash
cd /tmp/jx-test
jx venv remove
# 输入 n 取消
jx venv remove
# 输入 y 确认删除
ls -la
```

Expected: 第一次输入 n 后取消删除，`.jx/` 仍然存在；第二次输入 y 后成功删除

- [ ] **Step 4: 清理测试目录**

```bash
rm -rf /tmp/jx-test
```

- [ ] **Step 5: 最终提交（如有遗漏修改）**

```bash
git status
# 如果有未提交的修改，补充提交
```

---

## 自检清单

1. **Spec coverage**: 设计文档中的 3 个修改点均已覆盖（create、info、remove）
2. **Placeholder scan**: 无 TBD/TODO/模糊描述
3. **Type consistency**: 所有修改在同一文件，函数签名未改变