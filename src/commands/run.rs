use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn execute(main_class: Option<String>, args: Vec<String>) -> Result<()> {
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

    println!("🚀 运行项目...");
    
    let class_to_run = if let Some(ref class) = main_class {
        class.clone()
    } else {
        // 从配置文件获取主类
        get_main_class_from_config(&current_dir, config_file)?
    };
    
    println!("主类: {}", class_to_run);
    if !args.is_empty() {
        println!("参数: {}", args.join(" "));
    }

    // 根据配置文件类型运行项目
    let result = match config_file {
        "jx.toml" => run_jx_project(&current_dir, &class_to_run, &args),
        "pom.xml" => run_maven_project(&current_dir, &class_to_run, &args),
        "build.gradle" => run_gradle_project(&current_dir, &class_to_run, &args),
        _ => Err(anyhow::anyhow!("不支持的配置文件类型")),
    };

    match result {
        Ok(_) => {
            println!("✅ 项目运行完成!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 运行失败: {}", e);
            Err(e)
        }
    }
}

fn get_main_class_from_config(project_dir: &Path, config_file: &str) -> Result<String> {
    match config_file {
        "jx.toml" => {
            let config_path = project_dir.join("jx.toml");
            let config_content = std::fs::read_to_string(&config_path)?;
            
            // 从jx.toml中提取主类
            for line in config_content.lines() {
                if line.trim().starts_with("main_class = \"") {
                    let class = line.trim()
                        .trim_start_matches("main_class = \"")
                        .trim_end_matches("\"");
                    return Ok(class.to_string());
                }
            }
            Ok("com.example.Main".to_string()) // 默认主类
        }
        "pom.xml" => {
            // Maven项目通常使用exec插件或默认主类
            Ok("com.example.Main".to_string())
        }
        "build.gradle" => {
            // Gradle项目通常使用application插件
            let build_gradle_path = project_dir.join("build.gradle");
            let build_content = std::fs::read_to_string(&build_gradle_path)?;
            
            for line in build_content.lines() {
                if line.trim().starts_with("mainClass = '") {
                    let class = line.trim()
                        .trim_start_matches("mainClass = '")
                        .trim_end_matches("'");
                    return Ok(class.to_string());
                }
            }
            Ok("com.example.Main".to_string()) // 默认主类
        }
        _ => Ok("com.example.Main".to_string()),
    }
}

fn run_jx_project(project_dir: &Path, main_class: &str, args: &[String]) -> Result<()> {
    // 检查jx.toml中的项目类型
    let config_path = project_dir.join("jx.toml");
    let config_content = std::fs::read_to_string(&config_path)?;
    
    if config_content.contains("type = \"maven\"") {
        run_maven_project(project_dir, main_class, args)
    } else if config_content.contains("type = \"gradle\"") {
        run_gradle_project(project_dir, main_class, args)
    } else {
        Err(anyhow::anyhow!("在jx.toml中找不到有效的项目类型"))
    }
}

fn run_maven_project(project_dir: &Path, main_class: &str, args: &[String]) -> Result<()> {
    println!("使用Maven运行项目...");
    
    if !check_command_exists("mvn") {
        return Err(anyhow::anyhow!("Maven未安装，请先安装Maven"));
    }
    
    // 先编译项目
    println!("编译项目...");
    let compile_output = Command::new("mvn")
        .arg("compile")
        .current_dir(project_dir)
        .output()
        .context("Maven编译失败")?;
    
    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Maven编译失败: {}", error));
    }
    
    // 运行项目
    println!("运行项目...");
    let mut mvn_args = vec!["exec:java", "-Dexec.mainClass", main_class];
    
    let args_str = if !args.is_empty() {
        args.join(" ")
    } else {
        String::new()
    };
    
    if !args.is_empty() {
        mvn_args.push("-Dexec.args");
        mvn_args.push(&args_str);
    }
    
    let run_output = Command::new("mvn")
        .args(&mvn_args)
        .current_dir(project_dir)
        .output()
        .context("Maven运行失败")?;
    
    if !run_output.status.success() {
        let error = String::from_utf8_lossy(&run_output.stderr);
        return Err(anyhow::anyhow!("Maven运行失败: {}", error));
    }
    
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    println!("程序输出:");
    println!("{}", stdout);
    
    Ok(())
}

fn run_gradle_project(project_dir: &Path, main_class: &str, args: &[String]) -> Result<()> {
    println!("使用Gradle运行项目...");
    
    if !check_command_exists("gradle") {
        return Err(anyhow::anyhow!("Gradle未安装，请先安装Gradle"));
    }
    
    // 先编译项目
    println!("编译项目...");
    let compile_output = Command::new("gradle")
        .arg("compileJava")
        .current_dir(project_dir)
        .output()
        .context("Gradle编译失败")?;
    
    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Gradle编译失败: {}", error));
    }
    
    // 运行项目
    println!("运行项目...");
    let mut gradle_args = vec!["run"];
    
    let args_str = if !args.is_empty() {
        args.join(" ")
    } else {
        String::new()
    };
    
    if !args.is_empty() {
        gradle_args.push("--args");
        gradle_args.push(&args_str);
    }
    
    let run_output = Command::new("gradle")
        .args(&gradle_args)
        .current_dir(project_dir)
        .output()
        .context("Gradle运行失败")?;
    
    if !run_output.status.success() {
        let error = String::from_utf8_lossy(&run_output.stderr);
        return Err(anyhow::anyhow!("Gradle运行失败: {}", error));
    }
    
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    println!("程序输出:");
    println!("{}", stdout);
    
    Ok(())
}

fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
