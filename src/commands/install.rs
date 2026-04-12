use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn execute(_file: Option<String>, _production: bool, force: bool) -> Result<()> {
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
        install_from_maven(&current_dir, _production, force)
    } else if config_file == "build.gradle" {
        install_from_gradle(&current_dir, _production, force)
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

fn install_from_maven(project_dir: &Path, production: bool, force: bool) -> Result<()> {
    println!("使用Maven安装依赖...");

    // 检查Maven是否安装
    if !check_command_exists("mvn") {
        return Err(anyhow::anyhow!("Maven未安装，请先安装Maven"));
    }

    // 构建Maven命令
    let mut mvn_args = vec!["dependency:resolve"];

    if production {
        mvn_args.push("-Dscope=compile");
    }

    if force {
        mvn_args.push("-U"); // 强制更新
    }

    println!("正在解析Maven依赖...");

    // 执行Maven命令
    let output = Command::new("mvn")
        .args(&mvn_args)
        .current_dir(project_dir)
        .output()
        .context("执行Maven命令失败")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Maven执行失败: {}", error));
    }

    println!("Maven依赖解析完成");
    println!("正在下载依赖...");

    let download_output = Command::new("mvn")
        .arg("dependency:copy-dependencies")
        .current_dir(project_dir)
        .output()
        .context("下载Maven依赖失败")?;

    if !download_output.status.success() {
        let error = String::from_utf8_lossy(&download_output.stderr);
        return Err(anyhow::anyhow!("依赖下载失败: {}", error));
    }

    println!("依赖下载完成");
    println!("Maven依赖安装完成");
    Ok(())
}

fn install_from_gradle(project_dir: &Path, _production: bool, force: bool) -> Result<()> {
    println!("使用Gradle安装依赖...");

    // 检查Gradle是否安装
    if !check_command_exists("gradle") {
        return Err(anyhow::anyhow!("Gradle未安装，请先安装Gradle"));
    }

    // 构建Gradle命令
    let mut gradle_args = vec!["dependencies"];

    if force {
        gradle_args.push("--refresh-dependencies");
    }

    println!("正在解析Gradle依赖...");

    // 执行Gradle命令
    let output = Command::new("gradle")
        .args(&gradle_args)
        .current_dir(project_dir)
        .output()
        .context("执行Gradle命令失败")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Gradle执行失败: {}", error));
    }

    println!("Gradle依赖解析完成");
    println!("正在下载依赖...");

    let download_output = Command::new("gradle")
        .arg("build")
        .current_dir(project_dir)
        .output()
        .context("下载Gradle依赖失败")?;

    if !download_output.status.success() {
        let error = String::from_utf8_lossy(&download_output.stderr);
        return Err(anyhow::anyhow!("依赖下载失败: {}", error));
    }

    println!("依赖下载完成");
    println!("Gradle依赖安装完成");
    Ok(())
}

fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
