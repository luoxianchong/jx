# jx - 快速的Java包管理器

jx是一个用Rust编写的快速Java包管理器，类似于Python的uv工具。它提供了现代化的依赖管理、项目构建和包管理功能。

## 特性

- **快速**: 用Rust编写，性能优异
- **虚拟环境管理**: 自动下载和管理Java/Maven/Gradle版本，无需系统预装
- **智能缓存**: 高效的依赖缓存系统，避免重复下载
- **依赖解析**: 自动解析传递依赖并可视化展示
- **项目模板**: 快速创建Maven和Gradle项目
- **环境自愈**: 自动检测和修复损坏的符号链接

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/luoxianchong/jx.git
cd jx

# 编译
cargo build --release

# 安装到系统
cargo install --path .
```

### 系统要求

- Rust 1.70+
- curl（用于下载Java/Maven/Gradle）
- tar 或 unzip（用于解压）

**注意**: 使用虚拟环境功能时，无需系统预装Java/Maven/Gradle，jx会自动下载和管理。

## 快速开始

### 创建虚拟环境

```bash
# 在当前项目目录创建虚拟环境（默认Java 17 + Maven）
jx venv create

# 指定Java版本
jx venv create --java-version 21

# 使用Gradle代替Maven
jx venv create --java-version 17 --gradle-version 8.5

# 查看虚拟环境信息
jx venv info

# 删除虚拟环境
jx venv remove
```

虚拟环境创建后会在项目目录生成 `.jx/` 目录，包含:
- `bin/` - Java/Maven/Gradle的符号链接
- `venv.toml` - 环境配置文件
- `.active` - 激活标记

### 创建新项目

```bash
# 创建Maven项目
jx init my-project --template maven

# 创建Gradle项目
jx init my-project --template gradle

# 在当前目录创建项目
jx init --template maven
```

### 安装依赖

```bash
# 安装所有依赖
jx install

# 仅安装生产依赖
jx install --production

# 强制重新安装
jx install --force
```

### 添加依赖

```bash
# 添加编译依赖
jx add org.springframework:spring-core:5.3.0

# 添加测试依赖
jx add junit:junit:4.13.2 --scope test

# 添加运行时依赖
jx add org.apache.commons:commons-lang3:3.12.0 --scope runtime
```

### 构建和运行

```bash
# 构建项目
jx build

# 运行项目（自动使用.jx环境）
jx run

# 运行指定主类
jx run com.example.Main

# 运行测试
jx test

# 清理构建文件
jx clean
```

### 查看依赖树

```bash
# 显示直接依赖
jx tree

# 显示传递依赖
jx tree --transitive
```

## 命令参考

### 虚拟环境管理

- `jx venv create [--java-version <8|11|17|21|25>] [--maven-version <VERSION>] [--gradle-version <VERSION>]` - 创建虚拟环境
- `jx venv remove` - 删除虚拟环境
- `jx venv info` - 显示虚拟环境信息

### 项目管理

- `jx init [NAME] --template <maven|gradle>` - 初始化新项目
- `jx info` - 显示项目信息
- `jx clean` - 清理构建文件

### 依赖管理

- `jx install [--file FILE] [--production] [--force]` - 安装依赖
- `jx add <DEPENDENCY> [--scope <compile|runtime|test|provided>]` - 添加依赖
- `jx remove <DEPENDENCY>` - 移除依赖
- `jx update [DEPENDENCY] [--latest]` - 更新依赖
- `jx tree [--transitive]` - 显示依赖树

### 构建和运行

- `jx build [--mode <debug|release>] [--no-test]` - 构建项目
- `jx run [MAIN_CLASS] [ARGS...]` - 运行项目
- `jx test [TEST_CLASS] [--method METHOD]` - 运行测试

### 搜索和发布

- `jx search <QUERY> [--limit N]` - 搜索依赖
- `jx publish [--repository URL] [--no-sign]` - 发布包

### 通用选项

- `--verbose` - 启用详细输出
- `--quiet` - 静默模式
- `--help` - 显示帮助信息
- `--version` - 显示版本信息

## 虚拟环境特性

jx的虚拟环境功能类似于Python的venv，提供以下优势:

### 自动下载和安装

- 从Adoptium API获取Java JDK下载链接
- 自动下载并解压到 `~/.jx/cache/` 目录
- 支持Java 8、11、17、21、25版本
- 支持x64和aarch64架构

### 智能缓存

所有下载的工具都会缓存在 `~/.jx/cache/`:
- `java/` - Java JDK缓存
- `maven/` - Maven缓存
- `gradle/` - Gradle缓存
- `archives/` - 原始压缩包缓存

不同项目可以共享同一缓存，避免重复下载。

### 环境自愈

当符号链接损坏时（例如缓存被手动删除），jx会:
- 自动检测损坏的符号链接
- 重新下载缺失的工具
- 重建符号链接

在运行 `jx run` 等命令时会自动触发环境检查和修复。

## 配置文件

jx使用`jx.toml`配置文件来管理项目设置：

```toml
[project]
name = "my-java-project"
version = "1.0.0"
description = "A Java project"
java_version = "11"

[build]
main_class = "com.example.Main"
test_class = "com.example.MainTest"
source_dir = "src/main/java"
target_dir = "target"

[dependencies]
# 编译依赖
org.springframework:spring-core = "5.3.0"
org.apache.commons:commons-lang3 = "3.12.0"

# 测试依赖
junit:junit = "4.13.2"

[repositories]
maven_central = "https://repo1.maven.org/maven2/"
```

虚拟环境使用 `.jx/venv.toml`:

```toml
java_version = "17"
maven_version = "3.9.9"
gradle_version = ""

[cache_paths]
java = "jdk-17-mac-x64"
maven = "apache-maven-3.9.9"
gradle = ""
```

## 项目结构

jx支持标准的Maven和Gradle项目结构：

```
my-project/
├── .jx/                  # 虚拟环境目录
│   ├── bin/              # 工具符号链接
│   ├── venv.toml         # 环境配置
│   └── .active           # 激活标记
├── jx.toml               # jx配置文件
├── pom.xml               # Maven配置 (可选)
├── build.gradle          # Gradle配置 (可选)
├── src/
│   ├── main/
│   │   ├── java/         # Java源码
│   │   └── resources/    # 资源文件
│   └── test/
│       ├── java/         # 测试源码
│       └── resources/    # 测试资源
├── target/               # 构建输出
└── lib/                  # 依赖库
```

## 开发

### 项目结构

```
src/
├── main.rs              # 主入口
├── cli.rs               # CLI定义
├── commands/            # 命令实现
│   ├── mod.rs
│   ├── init.rs          # 项目初始化
│   ├── install.rs       # 依赖安装
│   ├── add.rs           # 添加依赖
│   ├── remove.rs        # 移除依赖
│   ├── update.rs        # 更新依赖
│   ├── build.rs         # 构建项目
│   ├── run.rs           # 运行项目
│   ├── test.rs          # 运行测试
│   ├── clean.rs         # 清理
│   ├── info.rs          # 项目信息
│   ├── tree.rs          # 依赖树
│   ├── search.rs        # 搜索依赖
│   ├── venv.rs          # 虚拟环境管理
│   └── publish.rs       # 发布
├── config.rs            # 配置管理
├── dependency.rs        # 依赖模型
├── download.rs          # 下载管理
├── install.rs           # 安装管理
├── lock.rs              # 锁定文件
├── project.rs           # 项目管理
├── registry.rs          # 仓库管理
├── resolve.rs           # 依赖解析
├── environment.rs       # 环境检测与自愈
└── utils.rs             # 工具函数
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name
```

## 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建Pull Request

## 许可证

本项目采用MIT许可证 - 查看[LICENSE](LICENSE)文件了解详情。

## 致谢

- 感谢[Maven](https://maven.apache.org/)和[Gradle](https://gradle.org/)项目
- 感谢[Adoptium](https://adoptium.net/)提供的Java JDK
- 感谢[Rust](https://rust-lang.org/)社区

---

**jx** - 让Java开发更快速、更简单！
