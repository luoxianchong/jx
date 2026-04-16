use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "运行测试")]
pub struct TestCommand {
    #[clap(index = 1, help = "测试类名")]
    pub test_class: Option<String>,

    #[clap(long, help = "测试方法名")]
    pub method: Option<String>,
}

impl TestCommand {
    pub fn execute(&self) -> Result<()> {
        println!("🧪 运行测试...");

        let current_dir = std::env::current_dir()?;

        // 检测项目类型
        let project_type = detect_project_type(&current_dir)?;
        println!("项目类型: {}", project_type);

        // 获取测试配置
        let test_config = get_test_config(&current_dir, &project_type)?;

        // 显示测试信息
        display_test_info(&test_config, &self.test_class, &self.method);

        // 运行测试
        let result = match project_type.as_str() {
            "Maven" | "Maven + Gradle" => {
                run_maven_tests(&current_dir, &test_config, &self.test_class, &self.method)
            }
            "Gradle" => run_gradle_tests(&current_dir, &test_config, &self.test_class, &self.method),
            "jx" => run_jx_tests(&current_dir, &test_config, &self.test_class, &self.method),
            _ => run_generic_tests(&current_dir, &test_config, &self.test_class, &self.method),
        };

        match result {
            Ok(_) => {
                println!("✅ 测试执行完成!");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 测试执行失败: {}", e);
                Err(e)
            }
        }
    }
}

#[derive(Debug)]
struct TestConfig {
    test_framework: String,
    test_source_dir: PathBuf,
    test_class_dir: PathBuf,
    main_class: Option<String>,
    test_class: Option<String>,
    java_version: Option<String>,
    #[allow(dead_code)]
    dependencies: Vec<String>,
}

fn detect_project_type(project_dir: &Path) -> Result<String> {
    let has_pom = project_dir.join("pom.xml").exists();
    let has_gradle = project_dir.join("build.gradle").exists();
    let has_settings_gradle = project_dir.join("settings.gradle").exists();
    let has_jx = project_dir.join("jx.toml").exists();

    if has_jx {
        Ok("jx".to_string())
    } else if has_pom && (has_gradle || has_settings_gradle) {
        Ok("Maven + Gradle".to_string())
    } else if has_pom {
        Ok("Maven".to_string())
    } else if has_gradle || has_settings_gradle {
        Ok("Gradle".to_string())
    } else {
        Ok("未知".to_string())
    }
}

fn get_test_config(project_dir: &Path, project_type: &str) -> Result<TestConfig> {
    match project_type {
        "Maven" | "Maven + Gradle" => get_maven_test_config(project_dir),
        "Gradle" => get_gradle_test_config(project_dir),
        "jx" => get_jx_test_config(project_dir),
        _ => get_generic_test_config(project_dir),
    }
}

fn get_maven_test_config(project_dir: &Path) -> Result<TestConfig> {
    let pom_path = project_dir.join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)?;

    let mut config = TestConfig {
        test_framework: "JUnit".to_string(),
        test_source_dir: project_dir.join("src/test/java"),
        test_class_dir: project_dir.join("target/test-classes"),
        main_class: None,
        test_class: None,
        java_version: Some("11".to_string()),
        dependencies: Vec::new(),
    };

    let lines: Vec<&str> = pom_content.lines().collect();

    for line in lines {
        let line = line.trim();

        if line.starts_with("<maven.compiler.source>") && line.ends_with("</maven.compiler.source>")
        {
            let start = "<maven.compiler.source>".len();
            let end = line.len() - "</maven.compiler.source>".len();
            if start < end {
                config.java_version = Some(line[start..end].to_string());
            }
        } else if line.starts_with("<maven.compiler.target>")
            && line.ends_with("</maven.compiler.target>")
        {
            let start = "<maven.compiler.target>".len();
            let end = line.len() - "</maven.compiler.target>".len();
            if start < end {
                config.java_version = Some(line[start..end].to_string());
            }
        }
    }

    // 检测测试框架
    if pom_content.contains("junit") {
        config.test_framework = "JUnit".to_string();
    } else if pom_content.contains("testng") {
        config.test_framework = "TestNG".to_string();
    }

    Ok(config)
}

fn get_gradle_test_config(project_dir: &Path) -> Result<TestConfig> {
    let build_gradle_path = project_dir.join("build.gradle");
    let build_content = fs::read_to_string(&build_gradle_path)?;

    let mut config = TestConfig {
        test_framework: "JUnit".to_string(),
        test_source_dir: project_dir.join("src/test/java"),
        test_class_dir: project_dir.join("build/classes/java/test"),
        main_class: None,
        test_class: None,
        java_version: Some("11".to_string()),
        dependencies: Vec::new(),
    };

    let lines: Vec<&str> = build_content.lines().collect();

    for line in lines {
        let line = line.trim();

        if line.starts_with("sourceCompatibility") {
            if let Some(quote_start) = line.find('\'') {
                if let Some(quote_end) = line.rfind('\'') {
                    config.java_version = Some(line[quote_start + 1..quote_end].to_string());
                }
            }
        } else if line.starts_with("mainClass") {
            if let Some(quote_start) = line.find('\'') {
                if let Some(quote_end) = line.rfind('\'') {
                    config.main_class = Some(line[quote_start + 1..quote_end].to_string());
                }
            }
        }
    }

    // 检测测试框架
    if build_content.contains("junit") {
        config.test_framework = "JUnit".to_string();
    } else if build_content.contains("testng") {
        config.test_framework = "TestNG".to_string();
    }

    Ok(config)
}

fn get_jx_test_config(project_dir: &Path) -> Result<TestConfig> {
    let jx_path = project_dir.join("jx.toml");
    let jx_content = fs::read_to_string(&jx_path)?;

    let mut config = TestConfig {
        test_framework: "JUnit".to_string(),
        test_source_dir: project_dir.join("src/test/java"),
        test_class_dir: project_dir.join("target/test-classes"),
        main_class: None,
        test_class: None,
        java_version: Some("11".to_string()),
        dependencies: Vec::new(),
    };

    let lines: Vec<&str> = jx_content.lines().collect();

    for line in lines {
        let line = line.trim();

        if line.starts_with("main_class = \"") {
            let start = "main_class = \"".len();
            let end = line.len() - 1;
            if start < end {
                config.main_class = Some(line[start..end].to_string());
            }
        } else if line.starts_with("test_class = \"") {
            let start = "test_class = \"".len();
            let end = line.len() - 1;
            if start < end {
                config.test_class = Some(line[start..end].to_string());
            }
        } else if line.starts_with("java_version = \"") {
            let start = "java_version = \"".len();
            let end = line.len() - 1;
            if start < end {
                config.java_version = Some(line[start..end].to_string());
            }
        }
    }

    Ok(config)
}

fn get_generic_test_config(project_dir: &Path) -> Result<TestConfig> {
    Ok(TestConfig {
        test_framework: "JUnit".to_string(),
        test_source_dir: project_dir.join("src/test/java"),
        test_class_dir: project_dir.join("target/test-classes"),
        main_class: None,
        test_class: None,
        java_version: Some("11".to_string()),
        dependencies: Vec::new(),
    })
}

fn display_test_info(config: &TestConfig, test_class: &Option<String>, method: &Option<String>) {
    println!("\n📋 测试配置:");
    println!("{}", "─".repeat(40));
    println!("测试框架: {}", config.test_framework);
    println!("测试源码目录: {}", config.test_source_dir.display());
    println!("测试类目录: {}", config.test_class_dir.display());

    if let Some(ref java_version) = config.java_version {
        println!("Java版本: {}", java_version);
    }

    if let Some(ref main_class) = config.main_class {
        println!("主类: {}", main_class);
    }

    if let Some(ref test_class) = config.test_class {
        println!("默认测试类: {}", test_class);
    }

    if let Some(ref class) = test_class {
        println!("指定测试类: {}", class);
    }

    if let Some(ref m) = method {
        println!("指定测试方法: {}", m);
    }
}

fn run_maven_tests(
    project_dir: &Path,
    _config: &TestConfig,
    test_class: &Option<String>,
    method: &Option<String>,
) -> Result<()> {
    println!("\n🔨 使用Maven运行测试...");

    if !check_command_exists("mvn") {
        return Err(anyhow::anyhow!("Maven未安装，请先安装Maven"));
    }

    // 先编译项目
    println!("编译项目...");
    let compile_output = Command::new("mvn")
        .arg("compile")
        .arg("test-compile")
        .current_dir(project_dir)
        .output()
        .context("Maven编译失败")?;

    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Maven编译失败: {}", error));
    }

    // 构建测试命令
    let mut mvn_args = Vec::new();
    mvn_args.push("test".to_string());

    if let Some(ref class) = test_class {
        mvn_args.push(format!("-Dtest={}", class));
    }

    if let Some(ref m) = method {
        mvn_args.push(format!("-Dmethods={}", m));
    }

    println!("执行Maven测试命令: mvn {}", mvn_args.join(" "));

    // 执行测试
    let test_output = Command::new("mvn")
        .args(&mvn_args)
        .current_dir(project_dir)
        .output()
        .context("Maven测试执行失败")?;

    let stdout = String::from_utf8_lossy(&test_output.stdout);
    let stderr = String::from_utf8_lossy(&test_output.stderr);

    println!("测试输出:");
    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    if !stderr.is_empty() {
        println!("错误输出:");
        println!("{}", stderr);
    }

    if !test_output.status.success() {
        return Err(anyhow::anyhow!(
            "Maven测试失败，退出码: {}",
            test_output.status
        ));
    }

    println!("Maven测试完成");
    Ok(())
}

fn run_gradle_tests(
    project_dir: &Path,
    _config: &TestConfig,
    test_class: &Option<String>,
    method: &Option<String>,
) -> Result<()> {
    println!("\n🔨 使用Gradle运行测试...");

    if !check_command_exists("gradle") {
        return Err(anyhow::anyhow!("Gradle未安装，请先安装Gradle"));
    }

    // 先编译项目
    println!("编译项目...");
    let compile_output = Command::new("gradle")
        .arg("compileJava")
        .arg("compileTestJava")
        .current_dir(project_dir)
        .output()
        .context("Gradle编译失败")?;

    if !compile_output.status.success() {
        let error = String::from_utf8_lossy(&compile_output.stderr);
        return Err(anyhow::anyhow!("Gradle编译失败: {}", error));
    }

    // 构建测试命令
    let mut gradle_args = vec!["test"];
    let mut test_method = None;

    if let Some(ref m) = method {
        if let Some(ref class) = test_class {
            test_method = Some(format!("{}.{}", class, m));
        }
    }

    if let Some(ref class) = test_class {
        gradle_args.push("--tests");
        gradle_args.push(class);
    }

    if let Some(ref method_name) = test_method {
        gradle_args.push("--tests");
        gradle_args.push(method_name);
    }

    println!("执行Gradle测试命令: gradle {}", gradle_args.join(" "));

    // 执行测试
    let test_output = Command::new("gradle")
        .args(&gradle_args)
        .current_dir(project_dir)
        .output()
        .context("Gradle测试执行失败")?;

    if !test_output.status.success() {
        let error = String::from_utf8_lossy(&test_output.stderr);
        return Err(anyhow::anyhow!("Gradle测试失败: {}", error));
    }

    let stdout = String::from_utf8_lossy(&test_output.stdout);
    println!("测试输出:");
    println!("{}", stdout);

    println!("Gradle测试完成");
    Ok(())
}

fn run_jx_tests(
    project_dir: &Path,
    config: &TestConfig,
    test_class: &Option<String>,
    method: &Option<String>,
) -> Result<()> {
    println!("\n🔨 使用jx运行测试...");

    // 检查jx.toml中的项目类型
    let jx_path = project_dir.join("jx.toml");
    let jx_content = fs::read_to_string(&jx_path)?;

    if jx_content.contains("type = \"maven\"") {
        run_maven_tests(project_dir, config, test_class, method)
    } else if jx_content.contains("type = \"gradle\"") {
        run_gradle_tests(project_dir, config, test_class, method)
    } else {
        run_generic_tests(project_dir, config, test_class, method)
    }
}

fn run_generic_tests(
    project_dir: &Path,
    config: &TestConfig,
    test_class: &Option<String>,
    method: &Option<String>,
) -> Result<()> {
    println!("\n🔨 使用通用方式运行测试...");

    // 检查是否有编译好的测试类
    if !config.test_class_dir.exists() {
        println!("测试类目录不存在，尝试编译...");

        // 尝试使用javac编译
        if check_command_exists("javac") {
            compile_test_classes(project_dir, config)?;
        } else {
            return Err(anyhow::anyhow!("未找到Java编译器，无法运行测试"));
        }
    }

    // 查找测试类
    let test_classes = find_test_classes(&config.test_class_dir, test_class)?;

    if test_classes.is_empty() {
        println!("⚠️ 未找到测试类");
        return Ok(())
    }

    // 运行测试
    for test_class in test_classes {
        run_single_test_class(&test_class, method)?;
    }

    Ok(())
}

fn compile_test_classes(project_dir: &Path, config: &TestConfig) -> Result<()> {
    println!("使用javac编译测试类...");

    let mut javac_args = Vec::new();

    // 添加基本参数
    javac_args.push("-cp".to_string());
    javac_args.push(".".to_string());
    javac_args.push("-d".to_string());
    javac_args.push(config.test_class_dir.to_str().unwrap().to_string());

    // 添加依赖路径
    let lib_dir = project_dir.join("lib");
    if lib_dir.exists() {
        let mut classpath = ".".to_string();
        for entry in fs::read_dir(&lib_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar") {
                classpath.push_str(&format!(":{}", path.display()));
            }
        }
        javac_args.push("-cp".to_string());
        javac_args.push(classpath);
    }

    // 添加测试源码目录
    javac_args.push("-sourcepath".to_string());
    javac_args.push(config.test_source_dir.to_str().unwrap().to_string());

    // 查找所有Java测试文件
    let test_files = find_java_files(&config.test_source_dir)?;
    for test_file in test_files {
        javac_args.push(test_file.to_str().unwrap().to_string());
    }

    println!("执行编译命令: javac {}", javac_args.join(" "));

    let output = Command::new("javac")
        .args(&javac_args)
        .current_dir(project_dir)
        .output()
        .context("javac编译失败")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("javac编译失败: {}", error));
    }

    println!("测试类编译完成");
    Ok(())
}

fn find_test_classes(
    test_class_dir: &Path,
    specified_class: &Option<String>,
) -> Result<Vec<PathBuf>> {
    let mut test_classes = Vec::new();

    if let Some(ref class) = specified_class {
        // 查找指定的测试类
        let class_file = test_class_dir.join(format!("{}.class", class.replace('.', "/")));
        if class_file.exists() {
            test_classes.push(class_file);
        }
    } else {
        // 查找所有测试类
        for entry in walkdir::WalkDir::new(test_class_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("class") {
                test_classes.push(path.to_path_buf());
            }
        }
    }

    Ok(test_classes)
}

fn find_java_files(source_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut java_files = Vec::new();

    if source_dir.exists() {
        for entry in walkdir::WalkDir::new(source_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("java") {
                java_files.push(path.to_path_buf());
            }
        }
    }

    Ok(java_files)
}

fn run_single_test_class(test_class_path: &Path, method: &Option<String>) -> Result<()> {
    let class_name = test_class_path
        .strip_prefix(test_class_path.parent().unwrap())
        .unwrap()
        .with_extension("")
        .to_string_lossy()
        .replace('/', ".")
        .trim_start_matches('.')
        .to_string();

    println!("运行测试类: {}", class_name);

    if !check_command_exists("java") {
        return Err(anyhow::anyhow!("Java运行时未安装"));
    }

    let mut java_args = vec!["-cp", ".", &class_name];

    if let Some(ref m) = method {
        java_args.push(m);
    }

    let output = Command::new("java")
        .args(&java_args)
        .output()
        .context("Java测试执行失败")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("输出: {}", stdout);
    }

    if !stderr.is_empty() {
        println!("错误: {}", stderr);
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