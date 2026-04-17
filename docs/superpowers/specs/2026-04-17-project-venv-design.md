---
name: project-level-venv
description: 项目级 Java 虚拟环境管理，简化命令结构为 venv/create/remove
type: project
---

# 项目级 Venv 设计文档

## 背景

当前 `jx venv` 命令采用全局级虚拟环境管理，存储在 `~/.jx/venvs/` 目录下，并包含多个子命令（create/activate/deactivate/list/remove/info）。用户期望简化操作流程：

- 创建时自动激活
- 删除时自动停用
- 移除独立的 activate/deactivate/list 命令

## 设计目标

1. **项目级隔离**：每个项目拥有独立的虚拟环境，存储在项目根目录
2. **简化命令**：仅保留 `venv`、`venv create`、`venv remove` 三个命令入口
3. **自动激活/停用**：创建即激活，删除即停用
4. **项目检测**：必须项目内运行，向上查找项目根目录

## 命令设计

### `jx venv`

显示当前项目的虚拟环境状态：

- **已存在 venv**：显示激活状态、Java 版本、构建工具、路径等信息
- **不存在 venv**：提示用户运行 `jx venv create` 创建

输出示例：

```
📁 项目虚拟环境: .jx
状态: 🔌 已激活
Java: 17 (Eclipse Temurin)
Maven: 3.9.9
路径: /path/to/project/.jx

停用命令: jx venv remove
```

### `jx venv create`

创建项目级虚拟环境并自动激活。

**参数**：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--java-version` / `-j` | Java 版本 (8, 11, 17, 21, 25) | 17 |
| `--maven-version` / `-m` | Maven 版本 | 3.9.9 |
| `--gradle-version` / `-g` | Gradle 版本（替代 Maven） | 无 |

**行为**：

1. 检测项目根目录（向上查找 `jx.toml`/`pom.xml`/`build.gradle`）
2. 在项目根目录创建 `.jx` 目录结构
3. 下载并安装指定版本的 Java 和构建工具（Maven 或 Gradle）
4. 创建激活脚本 `.jx/bin/activate`
5. 输出激活命令供用户执行：`source .jx/bin/activate`

**Why**: 由于 CLI 无法直接修改父 shell 的环境变量，需要用户手动执行 source 命令。

**How to apply**: 创建完成后，输出清晰提示引导用户执行激活脚本。

### `jx venv remove`

停用并删除项目的虚拟环境。

**行为**：

1. 检测项目根目录
2. 检查 `.jx` 目录是否存在
3. 清理 shell profile 中的永久激活配置（若有）
4. 删除 `.jx` 目录
5. 输出提示用户重新加载 shell 或执行 `source ~/.zshrc` 等

## 目录结构

项目级 venv 存储在项目根目录的 `.jx` 目录下：

```
project/
├── .jx/
│   ├── bin/
│   │   ├── activate          # 激活脚本
│   │   ├── java              # Java 命令符号链接
│   │   ├── javac             # javac 符号链接
│   │   ├── mvn               # Maven 符号链接（或 gradle）
│   │   └── ...
│   ├── lib/
│   │   ├── java/
│   │   │   └── jdk/          # JDK 安装目录
│   │   ├── maven/            # Maven 安装目录（或 gradle）
│   │   └── ...
│   ├── conf/
│   │   └── venv.toml         # venv 配置文件
│   ├── cache/                # 项目级缓存（可选）
│   └── .active               # 激活状态标记文件
├── jx.toml                   # 项目配置文件
├── pom.xml                   # Maven 配置（可选）
└── build.gradle              # Gradle 配置（可选）
```

## 项目检测逻辑

向上查找项目根目录的优先级：

1. `jx.toml` - jx 项目配置文件
2. `pom.xml` - Maven 项目
3. `build.gradle` 或 `build.gradle.kts` - Gradle 项目

查找逻辑：从当前工作目录开始，逐级向上查找，直到找到上述任一文件或到达文件系统根目录。

**Why**: 确保用户可以在子目录中运行 venv 命令，自动定位到项目根目录。

**How to apply**: 实现递归向上查找函数，返回项目根目录路径；未找到时报错退出。

## 激活机制

由于 CLI 无法直接修改父 shell 的环境变量，采用以下机制：

1. **创建激活脚本**：`.jx/bin/activate` 设置 JAVA_HOME、MAVEN_HOME/GRADLE_HOME、PATH
2. **输出激活命令**：创建完成后提示用户执行 `source .jx/bin/activate`
3. **状态记录**：`.jx/.active` 文件标记当前激活状态，用于 `jx venv` 命令显示状态

**永久激活**（可选）：用户可选择将激活命令写入 shell profile（`~/.zshrc` 等），实现每次打开终端自动激活。

**Why**: shell 子进程无法修改父进程环境，这是 shell 的基本限制。

**How to apply**: 创建时输出清晰提示；提供 `--permanent` 选项写入 shell profile。

## 配置文件格式

`.jx/conf/venv.toml`：

```toml
# jx虚拟环境配置文件
# 创建时间: 2024-01-15 10:30:00
java_version = "17"
build_tool = "maven"
build_tool_version = "3.9.9"

[paths]
bin = "bin"
lib = "lib"
conf = "conf"
```

## 缓存策略

全局缓存目录 `~/.jx/cache/` 保留，用于存储下载的 JDK/Maven/Gradle 压缩包和解压后的目录：

- `~/.jx/cache/java/jdk-{version}-{os}-{arch}/` - JDK 解压缓存
- `~/.jx/cache/maven/apache-maven-{version}/` - Maven 解压缓存
- `~/.jx/cache/gradle/gradle-{version}/` - Gradle 解压缓存

**Why**: 多个项目可能使用相同版本的 Java/Maven/Gradle，缓存可避免重复下载和解压。

**How to apply**: 创建项目级 venv 时，优先从全局缓存复制，若无则下载并缓存后复制。

## 错误处理

| 场景 | 处理方式 |
|------|----------|
| 不在项目目录内运行 | 报错退出，提示需要项目配置文件 |
| 项目已存在 `.jx` 目录 | 报错退出，提示先删除或指定不同路径 |
| 删除时 `.jx` 不存在 | 报错退出，提示 venv 未创建 |
| 下载失败 | 回滚删除已创建的 `.jx` 目录 |

## 迁移路径

1. 修改 `src/cli.rs`：移除 `VenvSubcommands` 中的 `Activate`/`Deactivate`/`List`/`Info`，保留 `Create`/`Remove`
2. 修改 `src/commands/venv.rs`：
   - 重构 `VenvCommand` 为仅有 `Create`/`Remove` 子命令
   - 默认行为（无子命令）显示 venv 状态
   - 实现项目根目录检测逻辑
   - 修改存储路径为项目级 `.jx` 目录
   - 创建时自动生成激活脚本并输出 source 命令
   - 删除时自动清理激活脚本和状态文件
3. 更新文档和帮助信息

## 测试计划

1. 项目检测：在项目根目录和子目录运行命令
2. 创建流程：验证下载、安装、激活脚本生成
3. 删除流程：验证目录清理、状态清除
4. 缓存复用：验证相同版本多项目共享缓存
5. 错误场景：非项目目录、已存在 venv、下载失败回滚