use crate::environment::{prepare_project_environment, EnvConfig, prepend_to_path};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "运行项目")]
pub struct RunCommand {
    #[clap(index = 1, help = "主类名")]
    pub main_class: Option<String>,

    #[clap(index = 2, help = "程序参数")]
    pub args: Vec<String>,
}

impl RunCommand {
    pub fn execute(&self) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        // 检测并准备项目环境
        let env_config = prepare_project_environment()?;

        if let Some(config) = env_config {
            // 使用 .jx/ 环境执行
            self.run_with_env(&current_dir, &config)?;
        } else {
            // 原有逻辑（无 .jx/ 环境）
            self.run_without_env(&current_dir)?;
        }
        Ok(())
    }

    fn run_with_env(&self, project_dir: &Path, config: &EnvConfig) -> Result<()> {
        println!("🚀 运行项目（使用 .jx 环境）...");

        // 查找项目配置文件
        let config_file = self.find_config_file(project_dir)?;

        let class_to_run = if let Some(ref class) = self.main_class {
            class.clone()
        } else {
            get_main_class_from_config(project_dir, &config_file)?
        };

        println!("主类: {}", class_to_run);
        if !self.args.is_empty() {
            println!("参数: {}", self.args.join(" "));
        }

        // 根据配置文件类型运行项目（带环境变量）
        let result = match config_file {
            "jx.toml" => run_jx_project_with_env(project_dir, &class_to_run, &self.args, config),
            "pom.xml" => run_maven_project_with_env(project_dir, &class_to_run, &self.args, config),
            "build.gradle" => run_gradle_project_with_env(project_dir, &class_to_run, &self.args, config),
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

    fn run_without_env(&self, project_dir: &Path) -> Result<()> {
        println!("🚀 运行项目...");

        // 查找项目配置文件
        let config_file = self.find_config_file(project_dir)?;

        let class_to_run = if let Some(ref class) = self.main_class {
            class.clone()
        } else {
            get_main_class_from_config(project_dir, &config_file)?
        };

        println!("主类: {}", class_to_run);
        if !self.args.is_empty() {
            println!("参数: {}", self.args.join(" "));
        }

        // 根据配置文件类型运行项目（原有逻辑）
        let result = match config_file {
            "jx.toml" => run_jx_project(project_dir, &class_to_run, &self.args),
            "pom.xml" => run_maven_project(project_dir, &class_to_run, &self.args),
            "build.gradle" => run_gradle_project(project_dir, &class_to_run, &self.args),
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

    fn find_config_file(&self, project_dir: &Path) -> Result<&str> {
        if project_dir.join("jx.toml").exists() {
            Ok("jx.toml")
        } else if project_dir.join("pom.xml").exists() {
            Ok("pom.xml")
        } else if project_dir.join("build.gradle").exists() {
            Ok("build.gradle")
        } else {
            Err(anyhow::anyhow!("找不到项目配置文件，请先运行 'jx init'"))
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
                    let class = line
                        .trim()
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
                    let class = line
                        .trim()
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

    // 运行项目 - 使用 spawn 和 status 来直接输出到控制台
    println!("运行项目...");
    println!("──────────────────────────────────────────");

    let mut mvn_cmd = Command::new("mvn");
    mvn_cmd.arg("exec:java");
    mvn_cmd.arg(format!("-Dexec.mainClass={}", main_class));

    if !args.is_empty() {
        let args_str = args.join(" ");
        mvn_cmd.arg(format!("-Dexec.args={}", args_str));
    }

    mvn_cmd.current_dir(project_dir);

    // 使用 status() 而不是 output() 以便实时显示输出
    let status = mvn_cmd.status().context("Maven运行失败")?;

    println!("──────────────────────────────────────────");

    if !status.success() {
        return Err(anyhow::anyhow!("程序执行失败"));
    }

    Ok(())
}

fn run_gradle_project(project_dir: &Path, _main_class: &str, args: &[String]) -> Result<()> {
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

    // 运行项目 - 使用 spawn 和 status 来直接输出到控制台
    println!("运行项目...");
    println!("──────────────────────────────────────────");

    let mut gradle_cmd = Command::new("gradle");
    gradle_cmd.arg("run");

    if !args.is_empty() {
        let args_str = args.join(" ");
        gradle_cmd.arg("--args");
        gradle_cmd.arg(&args_str);
    }

    gradle_cmd.current_dir(project_dir);

    // 使用 status() 而不是 output() 以便实时显示输出
    let status = gradle_cmd.status().context("Gradle运行失败")?;

    println!("──────────────────────────────────────────");

    if !status.success() {
        return Err(anyhow::anyhow!("程序执行失败"));
    }

    Ok(())
}

fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_jx_project_with_env(project_dir: &Path, main_class: &str, args: &[String], config: &EnvConfig) -> Result<()> {
    let config_path = project_dir.join("jx.toml");
    let config_content = std::fs::read_to_string(&config_path)?;

    if config_content.contains("type = \"maven\"") {
        run_maven_project_with_env(project_dir, main_class, args, config)
    } else if config_content.contains("type = \"gradle\"") {
        run_gradle_project_with_env(project_dir, main_class, args, config)
    } else {
        Err(anyhow::anyhow!("在jx.toml中找不到有效的项目类型"))
    }
}

fn run_maven_project_with_env(project_dir: &Path, main_class: &str, args: &[String], config: &EnvConfig) -> Result<()> {
    println!("使用Maven运行项目...");

    // 先编译项目
    println!("编译项目...");
    let mut compile_cmd = Command::new("mvn");
    compile_cmd.arg("compile");
    compile_cmd.env("PATH", prepend_to_path(&config.bin_dir));
    if let Some(java_home) = &config.java_home {
        compile_cmd.env("JAVA_HOME", java_home);
    }
    compile_cmd.current_dir(project_dir);

    let compile_output = compile_cmd.output().context("Maven编译失败")?;

    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Maven编译失败: {}", error));
    }

    // 运行项目
    println!("运行项目...");
    println!("──────────────────────────────────────────");

    let mut mvn_cmd = Command::new("mvn");
    mvn_cmd.arg("exec:java");
    mvn_cmd.arg(format!("-Dexec.mainClass={}", main_class));
    mvn_cmd.env("PATH", prepend_to_path(&config.bin_dir));
    if let Some(java_home) = &config.java_home {
        mvn_cmd.env("JAVA_HOME", java_home);
    }

    if !args.is_empty() {
        let args_str = args.join(" ");
        mvn_cmd.arg(format!("-Dexec.args={}", args_str));
    }

    mvn_cmd.current_dir(project_dir);

    let status = mvn_cmd.status().context("Maven运行失败")?;

    println!("──────────────────────────────────────────");

    if !status.success() {
        return Err(anyhow::anyhow!("程序执行失败"));
    }

    Ok(())
}

fn run_gradle_project_with_env(project_dir: &Path, _main_class: &str, args: &[String], config: &EnvConfig) -> Result<()> {
    println!("使用Gradle运行项目...");

    // 先编译项目
    println!("编译项目...");
    let mut compile_cmd = Command::new("gradle");
    compile_cmd.arg("compileJava");
    compile_cmd.env("PATH", prepend_to_path(&config.bin_dir));
    if let Some(java_home) = &config.java_home {
        compile_cmd.env("JAVA_HOME", java_home);
    }
    compile_cmd.current_dir(project_dir);

    let compile_output = compile_cmd.output().context("Gradle编译失败")?;

    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Gradle编译失败: {}", error));
    }

    // 运行项目
    println!("运行项目...");
    println!("──────────────────────────────────────────");

    let mut gradle_cmd = Command::new("gradle");
    gradle_cmd.arg("run");
    gradle_cmd.env("PATH", prepend_to_path(&config.bin_dir));
    if let Some(java_home) = &config.java_home {
        gradle_cmd.env("JAVA_HOME", java_home);
    }

    if !args.is_empty() {
        let args_str = args.join(" ");
        gradle_cmd.arg("--args");
        gradle_cmd.arg(&args_str);
    }

    gradle_cmd.current_dir(project_dir);

    let status = gradle_cmd.status().context("Gradle运行失败")?;

    println!("──────────────────────────────────────────");

    if !status.success() {
        return Err(anyhow::anyhow!("程序执行失败"));
    }

    Ok(())
}