use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn execute(mode: String, no_test: bool) -> Result<()> {
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

    println!("🔨 构建项目...");
    println!("构建模式: {}", mode);
    if no_test {
        println!("跳过测试");
    }

    // 根据配置文件类型构建项目
    let result = match config_file {
        "jx.toml" => build_jx_project(&current_dir, &mode, no_test),
        "pom.xml" => build_maven_project(&current_dir, &mode, no_test),
        "build.gradle" => build_gradle_project(&current_dir, &mode, no_test),
        _ => Err(anyhow::anyhow!("不支持的配置文件类型")),
    };

    match result {
        Ok(_) => {
            println!("✅ 项目构建完成!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 构建失败: {}", e);
            Err(e)
        }
    }
}

fn build_jx_project(project_dir: &Path, mode: &str, no_test: bool) -> Result<()> {
    // 检查jx.toml中的项目类型
    let config_path = project_dir.join("jx.toml");
    let config_content = std::fs::read_to_string(&config_path)?;
    
    if config_content.contains("type = \"maven\"") {
        build_maven_project(project_dir, mode, no_test)
    } else if config_content.contains("type = \"gradle\"") {
        build_gradle_project(project_dir, mode, no_test)
    } else {
        Err(anyhow::anyhow!("在jx.toml中找不到有效的项目类型"))
    }
}

fn build_maven_project(project_dir: &Path, mode: &str, no_test: bool) -> Result<()> {
    println!("使用Maven构建项目...");
    
    if !check_command_exists("mvn") {
        return Err(anyhow::anyhow!("Maven未安装，请先安装Maven"));
    }
    
    // 构建Maven命令
    let mut mvn_args = vec!["clean"];
    
    match mode {
        "release" => mvn_args.push("package"),
        "debug" => mvn_args.push("compile"),
        _ => mvn_args.push("compile"),
    }
    
    if no_test {
        mvn_args.push("-DskipTests");
    }
    
    println!("执行Maven命令: mvn {}", mvn_args.join(" "));
    
    // 执行Maven命令
    let output = Command::new("mvn")
        .args(&mvn_args)
        .current_dir(project_dir)
        .output()
        .context("执行Maven命令失败")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Maven构建失败: {}", error));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Maven构建输出:");
    println!("{}", stdout);
    
    println!("Maven构建完成");
    Ok(())
}

fn build_gradle_project(project_dir: &Path, mode: &str, no_test: bool) -> Result<()> {
    println!("使用Gradle构建项目...");
    
    if !check_command_exists("gradle") {
        return Err(anyhow::anyhow!("Gradle未安装，请先安装Gradle"));
    }
    
    // 构建Gradle命令
    let mut gradle_args = vec!["clean"];
    
    match mode {
        "release" => gradle_args.push("build"),
        "debug" => gradle_args.push("compileJava"),
        _ => gradle_args.push("compileJava"),
    }
    
    if no_test {
        gradle_args.push("-x");
        gradle_args.push("test");
    }
    
    println!("执行Gradle命令: gradle {}", gradle_args.join(" "));
    
    // 执行Gradle命令
    let output = Command::new("gradle")
        .args(&gradle_args)
        .current_dir(project_dir)
        .output()
        .context("执行Gradle命令失败")?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Gradle构建失败: {}", error));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Gradle构建输出:");
    println!("{}", stdout);
    
    println!("Gradle构建完成");
    Ok(())
}

fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
