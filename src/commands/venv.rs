use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::io::AsyncWriteExt;

// Adoptium API 数据结构
#[derive(Debug, Deserialize)]
struct AdoptiumBinary {
    architecture: String,
    os: String,
    image_type: String,
    package: AdoptiumPackage,
}

#[derive(Debug, Deserialize)]
struct AdoptiumPackage {
    name: String,
    link: String,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    download_count: u64,
    #[allow(dead_code)]
    checksum: Option<String>,
    #[allow(dead_code)]
    signature_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdoptiumRelease {
    binary: AdoptiumBinary,
    #[allow(dead_code)]
    release_name: String,
    #[allow(dead_code)]
    release_link: String,
    #[allow(dead_code)]
    vendor: String,
    #[allow(dead_code)]
    version: AdoptiumVersion,
}

#[derive(Debug, Deserialize)]
struct AdoptiumVersion {
    #[allow(dead_code)]
    major: u8,
    #[allow(dead_code)]
    minor: u8,
    #[allow(dead_code)]
    security: u8,
    #[allow(dead_code)]
    build: u8,
    #[allow(dead_code)]
    openjdk_version: String,
    #[allow(dead_code)]
    semver: String,
}

/// venv.toml 配置结构
#[derive(Debug, Clone, serde::Deserialize)]
struct VenvConfig {
    java_version: String,
    maven_version: String,
    gradle_version: String,
    #[serde(default)]
    cache_paths: CachePaths,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct CachePaths {
    java: String,
    maven: String,
    gradle: String,
}

fn load_venv_config(path: &Path) -> Result<VenvConfig> {
    let content = fs::read_to_string(path)?;
    let config: VenvConfig = toml::from_str(&content).context("解析 venv.toml 失败")?;
    Ok(config)
}

fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)?;
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(source, target).or_else(|_| {
            // Windows 上如果符号链接失败，复制文件
            fs::copy(source, target)?;
            Ok(())
        })?;
    }

    Ok(())
}

fn check_symlink_validity(bin_dir: &Path) -> Result<Vec<String>> {
    let mut broken = Vec::new();

    for entry in fs::read_dir(bin_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_symlink() {
            let target = fs::read_link(&path)?;
            if !target.exists() {
                broken.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
    }

    Ok(broken)
}

fn extract_to_cache(archive_path: &Path, target_dir: &Path, prefix: &str) -> Result<()> {
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
    }

    let filename = archive_path.file_name().unwrap().to_string_lossy();
    let parent = target_dir.parent().unwrap();

    if filename.ends_with(".tar.gz") {
        let output = Command::new("tar")
            .args(&[
                "-xzf",
                archive_path.to_str().unwrap(),
                "-C",
                parent.to_str().unwrap(),
            ])
            .output()
            .context("解压失败")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "解压失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    } else if filename.ends_with(".zip") {
        let output = Command::new("unzip")
            .args(&[
                "-q",
                archive_path.to_str().unwrap(),
                "-d",
                parent.to_str().unwrap(),
            ])
            .output()
            .context("解压失败")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "解压失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    } else {
        return Err(anyhow::anyhow!("不支持的压缩格式"));
    }

    // 查找并重命名解压后的目录
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(prefix)
        {
            fs::rename(path, target_dir)?;
            break;
        }
    }

    Ok(())
}

fn set_java_bin_permissions(java_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = find_java_bin_dir(java_dir);
        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let mut perms = fs::metadata(&path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&path, perms)?;
                }
            }
        }
    }
    Ok(())
}

fn find_java_bin_dir(java_dir: &Path) -> PathBuf {
    if java_dir.join("Contents").join("Home").join("bin").exists() {
        java_dir.join("Contents").join("Home").join("bin")
    } else {
        java_dir.join("bin")
    }
}

fn find_java_bin(java_dir: &Path) -> PathBuf {
    find_java_bin_dir(java_dir).join("java")
}

fn create_java_bin_symlinks(java_dir: &Path, bin_dir: &Path) -> Result<()> {
    let jdk_bin = find_java_bin_dir(java_dir);

    let commands = ["java", "javac", "jar", "javadoc", "keytool"];

    for cmd in &commands {
        let source = jdk_bin.join(cmd);
        let target = bin_dir.join(cmd);

        if source.exists() {
            create_symlink(&source, &target)?;
        }
    }

    Ok(())
}

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

    #[clap(about = "删除虚拟环境")]
    Remove(VenvRemoveArgs),

    #[clap(about = "显示虚拟环境信息")]
    Info(VenvInfoArgs),
}

#[derive(clap::Args)]
pub struct VenvCreateArgs {
    #[clap(index = 1, help = "虚拟环境名称")]
    pub name: Option<String>,

    #[clap(
        long,
        alias = "jv",
        default_value = "17",
        help = "Java版本 (8, 11, 17, 21, 25, 26)"
    )]
    pub java_version: String,

    #[clap(long, alias = "mv", help = "Maven版本 (默认使用Maven作为构建工具)")]
    pub maven_version: Option<String>,

    #[clap(long, alias = "gv", help = "Gradle版本 (使用Gradle代替默认的Maven)")]
    pub gradle_version: Option<String>,
}

#[derive(clap::Args)]
pub struct VenvRemoveArgs {}

#[derive(clap::Args)]
pub struct VenvInfoArgs {
    #[clap(index = 1, help = "虚拟环境名称 (已忽略，项目环境不需要名称)")]
    pub name: Option<String>,
}

impl VenvCommand {
    pub async fn execute(&self, _verbose: bool) -> Result<()> {
        match &self.command {
            VenvSubcommands::Create(args) => args.execute().await,
            VenvSubcommands::Remove(args) => args.execute(),
            VenvSubcommands::Info(args) => args.execute(),
        }
    }
}

impl VenvCreateArgs {
    pub async fn execute(&self) -> Result<()> {
        let build_tool = if let Some(gradle_version) = &self.gradle_version {
            BuildTool::Gradle(gradle_version.clone())
        } else {
            let maven_version = self
                .maven_version
                .clone()
                .unwrap_or_else(|| "3.9.9".to_string());
            BuildTool::Maven(maven_version)
        };
        create(self.name.clone(), self.java_version.clone(), build_tool).await
    }
}

impl VenvRemoveArgs {
    pub fn execute(&self) -> Result<()> {
        remove()
    }
}

impl VenvInfoArgs {
    pub fn execute(&self) -> Result<()> {
        info(self.name.clone())
    }
}

/// 创建Java虚拟环境
pub async fn create(
    _name: Option<String>,
    java_version: String,
    build_tool: BuildTool,
) -> Result<()> {
    let venv_dir = std::env::current_dir()?.join(".jx");

    println!("🌱 创建Java虚拟环境...");
    println!("Java版本: {}", java_version);
    match &build_tool {
        BuildTool::Maven(version) => println!("Maven版本: {}", version),
        BuildTool::Gradle(version) => println!("Gradle版本: {}", version),
    }

    // 检查虚拟环境是否已存在
    if venv_dir.exists() {
        return Err(anyhow::anyhow!("虚拟环境已存在于: {}", venv_dir.display()));
    }

    // 创建虚拟环境目录结构（仅 bin/）
    fs::create_dir_all(&venv_dir)?;
    fs::create_dir_all(venv_dir.join("bin"))?;

    // 将后续安装流程包裹在可回滚的流程中
    let result: Result<()> = async {
        // 创建虚拟环境配置文件
        create_venv_config(&venv_dir, &java_version, &build_tool)?;

        // 下载并安装Java
        install_java(&venv_dir, &java_version).await?;

        // 根据构建工具类型安装相应的构建工具
        match &build_tool {
            BuildTool::Maven(version) => {
                install_maven(&venv_dir, version).await?;
            }
            BuildTool::Gradle(version) => {
                install_gradle(&venv_dir, version).await?;
            }
        }

        Ok(())
    }
    .await;

    if let Err(err) = result {
        // 安装或下载失败时，尝试清理已创建的虚拟环境目录
        eprintln!("❌ 虚拟环境创建失败: {}", err);
        eprintln!("🧹 正在回滚并删除未完成的虚拟环境: {}", venv_dir.display());
        let _ = fs::remove_dir_all(&venv_dir);
        return Err(err);
    }

    println!("✅ 虚拟环境创建成功!");
    println!("路径: {}", venv_dir.display());
    println!("");
    println!("虚拟环境已就绪。");

    Ok(())
}

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

/// 显示虚拟环境信息
pub fn info(_name: Option<String>) -> Result<()> {
    let venv_dir = std::env::current_dir()?.join(".jx");

    if !venv_dir.exists() {
        return Err(anyhow::anyhow!("当前目录没有虚拟环境 (.jx/ 不存在)"));
    }

    println!("ℹ️ 项目虚拟环境信息");
    println!("");
    println!("路径: {}", venv_dir.display());

    // 读取配置
    let config_file = venv_dir.join("venv.toml");
    if config_file.exists() {
        let config = load_venv_config(&config_file)?;
        println!("Java版本: {}", config.java_version);
        if !config.maven_version.is_empty() {
            println!("Maven版本: {}", config.maven_version);
        }
        if !config.gradle_version.is_empty() {
            println!("Gradle版本: {}", config.gradle_version);
        }
    }

    // 符号链接状态
    let bin_dir = venv_dir.join("bin");
    if bin_dir.exists() {
        println!("");
        println!("符号链接状态:");

        let broken = check_symlink_validity(&bin_dir)?;

        for entry in fs::read_dir(&bin_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();

            if path.is_symlink() {
                let target = fs::read_link(&path)?;
                let status = if broken.contains(&name.to_string()) {
                    "❌ 损坏"
                } else {
                    "✓"
                };
                println!("  {} {} -> {}", name, status, target.display());
            }
        }

        if !broken.is_empty() {
            println!("");
            println!("⚠️ 检测到损坏的符号链接，运行修复命令可自动重建");
        }
    }

    Ok(())
}

// 辅助函数

fn get_jx_home() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    let jx_home = home.join(".jx");
    fs::create_dir_all(&jx_home)?;
    Ok(jx_home)
}

fn get_cache_directory() -> Result<PathBuf> {
    let jx_home = get_jx_home()?;
    let cache_dir = jx_home.join("cache");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

fn create_venv_config(venv_dir: &Path, java_version: &str, build_tool: &BuildTool) -> Result<()> {
    let (maven_version, gradle_version) = match build_tool {
        BuildTool::Maven(version) => (version.clone(), String::new()),
        BuildTool::Gradle(version) => (String::new(), version.clone()),
    };

    // 获取缓存路径名称
    let (_, arch) = parse_java_version(java_version)?;
    let os = get_os_type()?;
    let java_cache_name = format!(
        "jdk-{}-{}-{}",
        java_version.split('.').next().unwrap_or(java_version),
        os,
        arch
    );

    let maven_cache_name = if !maven_version.is_empty() {
        format!("apache-maven-{}", maven_version)
    } else {
        String::new()
    };

    let gradle_cache_name = if !gradle_version.is_empty() {
        format!("gradle-{}", gradle_version)
    } else {
        String::new()
    };

    let config_content = format!(
        r#"# jx 项目虚拟环境配置
# 创建时间: {}

java_version = "{}"
maven_version = "{}"
gradle_version = "{}"

[cache_paths]
java = "{}"
maven = "{}"
gradle = "{}"
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        java_version,
        maven_version,
        gradle_version,
        java_cache_name,
        maven_cache_name,
        gradle_cache_name
    );

    let config_file = venv_dir.join("venv.toml");
    fs::write(config_file, config_content)?;

    Ok(())
}

async fn install_java(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Java {}...", version);

    // 解析版本获取缓存目录名
    let (major_version, arch) = parse_java_version(version)?;
    let os = get_os_type()?;
    let cache_name = format!("jdk-{}-{}-{}", major_version, os, arch);

    let cache_dir = get_cache_directory()?;
    let java_cache_dir = cache_dir.join("java");
    fs::create_dir_all(&java_cache_dir)?;

    let cached_java = java_cache_dir.join(&cache_name);

    // 如果缓存不存在，下载并解压
    if !cached_java.exists() {
        println!("🌐 下载Java {}...", major_version);

        let download_url = build_java_download_url(major_version, &arch, &os)?;
        let filename = get_java_filename_from_url(&download_url)?;

        let archives_dir = cache_dir.join("archives").join("java");
        fs::create_dir_all(&archives_dir)?;
        let archive_path = archives_dir.join(&filename);

        // 下载压缩包
        download_file(&download_url, &archive_path).await?;

        // 解压到缓存目录
        println!("📦 解压Java到缓存...");
        extract_to_cache(&archive_path, &cached_java, "jdk")?;
    } else {
        println!("📋 使用缓存中的Java {}", major_version);
    }

    // 设置执行权限
    set_java_bin_permissions(&cached_java)?;

    // 创建符号链接到项目 bin/
    let bin_dir = venv_dir.join("bin");
    create_java_bin_symlinks(&cached_java, &bin_dir)?;

    // 验证
    let java_bin = find_java_bin(&cached_java);
    if let Ok(output) = Command::new(&java_bin).arg("-version").output() {
        let version_output = String::from_utf8_lossy(&output.stderr);
        println!("✅ Java安装成功!");
        println!("版本: {}", version_output.lines().next().unwrap_or(""));
    }

    Ok(())
}

fn parse_java_version(version: &str) -> Result<(u8, String)> {
    // 支持多种版本格式：8, 11, 17, 21, 1.8, 11.0, 17.0, 21.0等
    let major_version = if version.starts_with("1.") {
        // 处理1.x格式（如1.8）
        let minor = version.strip_prefix("1.").unwrap();
        minor
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    } else if version.contains('.') {
        // 处理x.y格式（如11.0, 17.0）
        let major = version.split('.').next().unwrap();
        major
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    } else {
        // 处理单个数字格式（如8, 11, 17, 21）
        version
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("无效的Java版本: {}", version))?
    };

    // 验证版本是否支持
    if major_version < 8 || major_version > 25 {
        return Err(anyhow::anyhow!(
            "不支持的Java版本: {} (支持范围: 8-25)",
            major_version
        ));
    }

    // 获取系统架构
    let arch = if cfg!(target_arch = "x86_64") {
        "x64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "aarch64".to_string()
    } else if cfg!(target_arch = "arm") {
        "arm".to_string()
    } else {
        return Err(anyhow::anyhow!("不支持的架构: {}", std::env::consts::ARCH));
    };

    Ok((major_version, arch))
}

fn get_os_type() -> Result<String> {
    let os = if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "mac".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        return Err(anyhow::anyhow!(
            "不支持的操作系统: {}",
            std::env::consts::OS
        ));
    };

    Ok(os)
}

fn get_adoptium_releases(version: u8) -> Result<Vec<AdoptiumRelease>> {
    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/{}/hotspot",
        version
    );

    // 使用curl命令获取API响应
    let output = Command::new("curl")
        .args(&["-s", "-H", "User-Agent: jx/0.1.0", &url])
        .output()
        .context("执行curl命令失败")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Adoptium API请求失败: {}", error));
    }

    let response_text = String::from_utf8_lossy(&output.stdout);
    let adoptium_releases: Vec<AdoptiumRelease> =
        serde_json::from_str(&response_text).context("解析Adoptium API响应失败")?;

    Ok(adoptium_releases)
}

fn build_java_download_url(major_version: u8, arch: &str, os: &str) -> Result<String> {
    // 获取Adoptium API数据
    let releases = get_adoptium_releases(major_version)?;

    if releases.is_empty() {
        return Err(anyhow::anyhow!("未找到Java {}的可用版本", major_version));
    }

    // 查找匹配的发布版本
    for release in &releases {
        let binary = &release.binary;

        // 检查操作系统和架构是否匹配
        let os_match = match (os, binary.os.as_str()) {
            ("linux", "linux") => true,
            ("mac", "mac") => true,
            ("windows", "windows") => true,
            _ => false,
        };

        let arch_match = match (arch, binary.architecture.as_str()) {
            ("x64", "x64") => true,
            ("aarch64", "aarch64") => true,
            ("arm", "arm") => true,
            _ => false,
        };

        // 检查是否是JDK包（不是JRE）
        let is_jdk = binary.image_type == "jdk";

        if os_match && arch_match && is_jdk {
            // 根据操作系统选择正确的文件扩展名
            let expected_extension = if os == "windows" { "zip" } else { "tar.gz" };
            if binary.package.name.ends_with(expected_extension) {
                return Ok(binary.package.link.clone());
            }
        }
    }

    Err(anyhow::anyhow!(
        "未找到适合 {}-{} 的Java {}下载链接",
        os,
        arch,
        major_version
    ))
}

fn get_java_filename_from_url(url: &str) -> Result<String> {
    // 从URL中提取文件名
    if let Some(last_slash) = url.rfind('/') {
        let filename = &url[last_slash + 1..];
        // 解码URL编码的字符
        let decoded =
            urlencoding::decode(filename).map_err(|e| anyhow::anyhow!("URL解码失败: {}", e))?;
        Ok(decoded.to_string())
    } else {
        Err(anyhow::anyhow!("无法从URL中提取文件名: {}", url))
    }
}

async fn download_file(url: &str, path: &Path) -> Result<()> {
    println!("下载: {}", url);

    // 创建HTTP客户端
    let client = reqwest::Client::new();

    // 发送GET请求
    let response = client.get(url).send().await.context("发送HTTP请求失败")?;

    // 检查响应状态
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP请求失败，状态码: {}",
            response.status()
        ));
    }

    // 获取文件大小
    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("无法获取文件大小"))?;

    // 创建进度条
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );
    pb.set_message(format!("下载文件"));

    // 创建文件
    let mut file = tokio::fs::File::create(path)
        .await
        .context("创建文件失败")?;

    // 下载并写入文件
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.context("下载数据失败")?;
        file.write_all(&chunk).await.context("写入文件失败")?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    // 关闭文件
    file.flush().await.context("刷新文件缓冲区失败")?;

    // 完成进度条
    pb.finish_with_message(format!("下载完成"));

    println!("下载完成");

    Ok(())
}

fn set_maven_bin_permissions(maven_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = maven_dir.join("bin");
        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let mut perms = fs::metadata(&path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&path, perms)?;
                }
            }
        }
    }
    Ok(())
}

fn create_maven_bin_symlinks(maven_dir: &Path, bin_dir: &Path) -> Result<()> {
    let mvn_bin = maven_dir.join("bin").join("mvn");
    if mvn_bin.exists() {
        create_symlink(&mvn_bin, &bin_dir.join("mvn"))?;
    }
    Ok(())
}

fn set_gradle_bin_permissions(gradle_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = gradle_dir.join("bin");
        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let mut perms = fs::metadata(&path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&path, perms)?;
                }
            }
        }
    }
    Ok(())
}

fn create_gradle_bin_symlinks(gradle_dir: &Path, bin_dir: &Path) -> Result<()> {
    let gradle_bin = gradle_dir.join("bin").join("gradle");
    if gradle_bin.exists() {
        create_symlink(&gradle_bin, &bin_dir.join("gradle"))?;
    }
    Ok(())
}

pub fn heal_venv(venv_dir: &Path) -> Result<()> {
    println!("🔧 修复虚拟环境...");

    let config = load_venv_config(&venv_dir.join("venv.toml"))?;
    let cache_dir = get_cache_directory()?;
    let bin_dir = venv_dir.join("bin");

    fs::create_dir_all(&bin_dir)?;

    // 修复 Java
    if !config.cache_paths.java.is_empty() {
        let java_cache = cache_dir.join("java").join(&config.cache_paths.java);

        if !java_cache.exists() {
            println!("🔗 Java 缓存不存在，正在重新下载...");
            let major: u8 = config
                .java_version
                .split('.')
                .next()
                .unwrap_or(&config.java_version)
                .parse()
                .map_err(|_| anyhow::anyhow!("无效的Java版本"))?;

            download_java_to_cache_sync(major, &java_cache)?;
        }

        create_java_bin_symlinks(&java_cache, &bin_dir)?;
        println!("✅ Java 符号链接已重建");
    }

    // 修复 Maven
    if !config.cache_paths.maven.is_empty() {
        let maven_cache = cache_dir.join("maven").join(&config.cache_paths.maven);

        if !maven_cache.exists() {
            println!("🔗 Maven 缓存不存在，正在重新下载...");
            download_maven_to_cache_sync(&config.maven_version, &maven_cache)?;
        }

        create_maven_bin_symlinks(&maven_cache, &bin_dir)?;
        println!("✅ Maven 符号链接已重建");
    }

    // 修复 Gradle
    if !config.cache_paths.gradle.is_empty() {
        let gradle_cache = cache_dir.join("gradle").join(&config.cache_paths.gradle);

        if !gradle_cache.exists() {
            println!("🔗 Gradle 缓存不存在，正在重新下载...");
            download_gradle_to_cache_sync(&config.gradle_version, &gradle_cache)?;
        }

        create_gradle_bin_symlinks(&gradle_cache, &bin_dir)?;
        println!("✅ Gradle 符号链接已重建");
    }

    println!("✅ 虚拟环境修复完成");
    Ok(())
}

fn download_java_to_cache_sync(major_version: u8, target_dir: &Path) -> Result<()> {
    let (_, arch) = parse_java_version(&major_version.to_string())?;
    let os = get_os_type()?;

    let download_url = build_java_download_url(major_version, &arch, &os)?;
    let filename = get_java_filename_from_url(&download_url)?;

    let archives_dir = get_cache_directory()?.join("archives").join("java");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(&filename);

    let output = Command::new("curl")
        .args(&["-L", "-o", archive_path.to_str().unwrap(), &download_url])
        .output()
        .context("下载 Java 失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "下载失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    extract_to_cache(&archive_path, target_dir, "jdk")?;
    set_java_bin_permissions(target_dir)?;

    Ok(())
}

fn download_maven_to_cache_sync(version: &str, target_dir: &Path) -> Result<()> {
    let download_url = format!(
        "https://archive.apache.org/dist/maven/maven-3/{}/binaries/apache-maven-{}-bin.tar.gz",
        version, version
    );

    let archives_dir = get_cache_directory()?.join("archives").join("maven");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(format!("apache-maven-{}-bin.tar.gz", version));

    let output = Command::new("curl")
        .args(&["-L", "-o", archive_path.to_str().unwrap(), &download_url])
        .output()
        .context("下载 Maven 失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "下载失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    extract_to_cache(&archive_path, target_dir, "apache-maven")?;
    set_maven_bin_permissions(target_dir)?;

    Ok(())
}

fn download_gradle_to_cache_sync(version: &str, target_dir: &Path) -> Result<()> {
    let download_url = format!(
        "https://services.gradle.org/distributions/gradle-{}-bin.zip",
        version
    );

    let archives_dir = get_cache_directory()?.join("archives").join("gradle");
    fs::create_dir_all(&archives_dir)?;
    let archive_path = archives_dir.join(format!("gradle-{}-bin.zip", version));

    let output = Command::new("curl")
        .args(&["-L", "-o", archive_path.to_str().unwrap(), &download_url])
        .output()
        .context("下载 Gradle 失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "下载失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    extract_to_cache(&archive_path, target_dir, "gradle-")?;
    set_gradle_bin_permissions(target_dir)?;

    Ok(())
}

async fn install_maven(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Maven {}...", version);

    let cache_name = format!("apache-maven-{}", version);

    let cache_dir = get_cache_directory()?;
    let maven_cache_dir = cache_dir.join("maven");
    fs::create_dir_all(&maven_cache_dir)?;

    let cached_maven = maven_cache_dir.join(&cache_name);

    if !cached_maven.exists() {
        println!("🌐 下载Maven {}...", version);

        let download_url = format!(
            "https://archive.apache.org/dist/maven/maven-3/{}/binaries/apache-maven-{}-bin.tar.gz",
            version, version
        );

        let archives_dir = cache_dir.join("archives").join("maven");
        fs::create_dir_all(&archives_dir)?;
        let archive_path = archives_dir.join(format!("apache-maven-{}-bin.tar.gz", version));

        download_file(&download_url, &archive_path).await?;

        println!("📦 解压Maven到缓存...");
        extract_to_cache(&archive_path, &cached_maven, "apache-maven")?;
    } else {
        println!("📋 使用缓存中的Maven {}", version);
    }

    // 设置权限
    set_maven_bin_permissions(&cached_maven)?;

    // 创建符号链接
    let bin_dir = venv_dir.join("bin");
    create_maven_bin_symlinks(&cached_maven, &bin_dir)?;

    let mvn_bin = cached_maven.join("bin").join("mvn");
    if let Ok(output) = Command::new(&mvn_bin).arg("--version").output() {
        println!("✅ Maven安装成功!");
        println!(
            "版本: {}",
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
        );
    }

    Ok(())
}

async fn install_gradle(venv_dir: &Path, version: &str) -> Result<()> {
    println!("📥 安装Gradle {}...", version);

    let cache_name = format!("gradle-{}", version);

    let cache_dir = get_cache_directory()?;
    let gradle_cache_dir = cache_dir.join("gradle");
    fs::create_dir_all(&gradle_cache_dir)?;

    let cached_gradle = gradle_cache_dir.join(&cache_name);

    if !cached_gradle.exists() {
        println!("🌐 下载Gradle {}...", version);

        let download_url = format!(
            "https://services.gradle.org/distributions/gradle-{}-bin.zip",
            version
        );

        let archives_dir = cache_dir.join("archives").join("gradle");
        fs::create_dir_all(&archives_dir)?;
        let archive_path = archives_dir.join(format!("gradle-{}-bin.zip", version));

        download_file(&download_url, &archive_path).await?;

        println!("📦 解压Gradle到缓存...");
        extract_to_cache(&archive_path, &cached_gradle, "gradle-")?;
    } else {
        println!("📋 使用缓存中的Gradle {}", version);
    }

    set_gradle_bin_permissions(&cached_gradle)?;

    let bin_dir = venv_dir.join("bin");
    create_gradle_bin_symlinks(&cached_gradle, &bin_dir)?;

    let gradle_bin = cached_gradle.join("bin").join("gradle");
    if let Ok(output) = Command::new(&gradle_bin).arg("--version").output() {
        println!("✅ Gradle安装成功!");
        println!(
            "版本: {}",
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
        );
    }

    Ok(())
}
