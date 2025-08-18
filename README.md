# jx - 快速的Java包管理器

jx是一个用Rust编写的快速Java包管理器，类似于Python的uv工具。它提供了现代化的依赖管理、项目构建和包管理功能。

## 🚀 特性

- **快速**: 用Rust编写，性能优异
- **现代化**: 支持Maven和Gradle项目
- **智能缓存**: 高效的依赖缓存系统
- **依赖解析**: 自动解析传递依赖
- **项目模板**: 快速创建Maven和Gradle项目
- **统一接口**: 统一的命令行接口管理不同类型的项目

## 📦 安装

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
- Java 8+
- Maven 3.6+ (可选)
- Gradle 7+ (可选)

## 🎯 快速开始

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

# 运行项目
jx run

# 运行测试
jx test

# 清理构建文件
jx clean
```

## 📚 命令参考

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

## 🔧 配置文件

jx使用`jx.toml`配置文件来管理项目设置：

```toml
[project]
name = "my-java-project"
type = "maven"
version = "1.0.0"
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
# Maven仓库
maven_central = "https://repo1.maven.org/maven2/"
jcenter = "https://jcenter.bintray.com/"
```

## 🏗️ 项目结构

jx支持标准的Maven和Gradle项目结构：

```
my-project/
├── jx.toml              # jx配置文件
├── pom.xml              # Maven配置 (可选)
├── build.gradle         # Gradle配置 (可选)
├── src/
│   ├── main/
│   │   ├── java/        # Java源码
│   │   └── resources/   # 资源文件
│   └── test/
│       ├── java/        # 测试源码
│       └── resources/   # 测试资源
├── target/              # 构建输出
└── lib/                 # 依赖库
```

## 🔍 依赖搜索

jx集成了Maven Central搜索功能：

```bash
# 搜索Spring相关依赖
jx search spring

# 限制搜索结果数量
jx search junit --limit 10
```

## 📊 性能特性

- **并行下载**: 使用异步I/O并行下载依赖
- **智能缓存**: 避免重复下载相同的依赖
- **增量构建**: 只重新构建修改的文件
- **内存优化**: 高效的内存使用和垃圾回收

## 🛠️ 开发

### 项目结构

```
src/
├── main.rs              # 主入口
├── commands/            # 命令实现
│   ├── mod.rs
│   ├── init.rs
│   ├── install.rs
│   ├── add.rs
│   └── ...
├── config.rs            # 配置管理
├── dependency.rs        # 依赖模型
├── download.rs          # 下载管理
├── install.rs           # 安装管理
├── lock.rs              # 锁定文件
├── project.rs           # 项目管理
├── registry.rs          # 仓库管理
├── resolve.rs           # 依赖解析
└── utils.rs             # 工具函数
```

### 添加新命令

1. 在`src/commands/`目录下创建新的命令文件
2. 实现`execute`函数
3. 在`src/commands/mod.rs`中导出新模块
4. 在`src/main.rs`中添加命令处理

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行集成测试
cargo test --test integration_tests
```

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建Pull Request

## 📄 许可证

本项目采用MIT许可证 - 查看[LICENSE](LICENSE)文件了解详情。

## 🙏 致谢

- 感谢[Maven](https://maven.apache.org/)和[Gradle](https://gradle.org/)项目提供的灵感
- 感谢[Rust](https://rust-lang.org/)社区提供的优秀工具链
- 感谢所有贡献者的辛勤工作

## 📞 支持

如果您遇到问题或有建议，请：

- 查看[Issues](https://github.com/your-username/jx/issues)
- 创建新的Issue
- 联系维护团队

---

**jx** - 让Java开发更快速、更简单！ 🚀
