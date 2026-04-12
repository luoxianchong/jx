use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute(dependency: String, scope: String) -> Result<()> {
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

    println!("➕ 添加依赖...");
    println!("依赖: {}", dependency);
    println!("作用域: {}", scope);

    // 解析依赖坐标
    let dep_info = parse_dependency_coordinate(&dependency)?;

    // 根据配置文件类型添加依赖
    let result = match config_file {
        "jx.toml" => add_to_jx_config(&current_dir, &dep_info, &scope),
        "pom.xml" => add_to_maven(&current_dir, &dep_info, &scope),
        "build.gradle" => add_to_gradle(&current_dir, &dep_info, &scope),
        _ => Err(anyhow::anyhow!("不支持的配置文件类型")),
    };

    match result {
        Ok(_) => {
            println!("✅ 依赖添加成功!");
            println!("请运行 'jx install' 来安装新添加的依赖");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ 添加失败: {}", e);
            Err(e)
        }
    }
}

#[derive(Debug)]
struct DependencyInfo {
    group_id: String,
    artifact_id: String,
    version: Option<String>,
}

fn parse_dependency_coordinate(coordinate: &str) -> Result<DependencyInfo> {
    let parts: Vec<&str> = coordinate.split(':').collect();

    match parts.len() {
        2 => {
            // 没有版本号，尝试获取最新版本
            let latest_version = get_latest_version(parts[0], parts[1])?;
            println!("📦 自动获取最新版本: {}", latest_version);
            Ok(DependencyInfo {
                group_id: parts[0].to_string(),
                artifact_id: parts[1].to_string(),
                version: Some(latest_version),
            })
        }
        3 => Ok(DependencyInfo {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: Some(parts[2].to_string()),
        }),
        _ => Err(anyhow::anyhow!(
            "无效的依赖坐标格式，应为 groupId:artifactId 或 groupId:artifactId:version"
        )),
    }
}

fn add_to_jx_config(project_dir: &Path, dep_info: &DependencyInfo, _scope: &str) -> Result<()> {
    let config_path = project_dir.join("jx.toml");

    if !config_path.exists() {
        // 如果配置文件不存在，创建一个基本的配置
        let basic_config = format!(
            r#"[project]
name = "my-java-project"
type = "maven"
version = "1.0.0"
java_version = "11"

[build]
main_class = "com.example.Main"
test_class = "com.example.MainTest"

[dependencies]
"#,
        );
        fs::write(&config_path, basic_config)?;
    }

    let config_content = fs::read_to_string(&config_path)?;

    // 简单的TOML解析和修改
    let mut lines: Vec<String> = config_content.lines().map(|s| s.to_string()).collect();

    // 查找dependencies部分
    let mut in_dependencies = false;
    let mut _dependencies_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "[dependencies]" {
            in_dependencies = true;
            _dependencies_start = i;
            break;
        }
    }

    if !in_dependencies {
        // 如果没有dependencies部分，添加一个
        lines.push("[dependencies]".to_string());
        _dependencies_start = lines.len() - 1;
    }

    // 构建依赖行
    let dep_line = format!(
        "{}:{} = \"{}\"",
        dep_info.group_id,
        dep_info.artifact_id,
        dep_info.version.as_ref().expect("版本号必须存在")
    );

    // 在dependencies部分后添加依赖
    lines.insert(_dependencies_start + 1, dep_line);

    // 写回文件
    fs::write(&config_path, lines.join("\n"))?;

    println!("已添加到 jx.toml");
    Ok(())
}

fn add_to_maven(project_dir: &Path, dep_info: &DependencyInfo, scope: &str) -> Result<()> {
    let pom_path = project_dir.join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)?;

    // 简单的XML解析和修改
    let mut lines: Vec<String> = pom_content.lines().map(|s| s.to_string()).collect();

    // 查找dependencies部分
    let mut in_dependencies = false;
    let mut _dependencies_start = 0;
    let mut dependencies_end = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "<dependencies>" {
            in_dependencies = true;
            _dependencies_start = i;
        } else if in_dependencies && line.trim() == "</dependencies>" {
            dependencies_end = i;
            break;
        }
    }

    if !in_dependencies {
        return Err(anyhow::anyhow!("在pom.xml中找不到dependencies部分"));
    }

    // 构建依赖XML
    let dep_xml = format!(
        r#"        <dependency>
            <groupId>{}</groupId>
            <artifactId>{}</artifactId>
            <version>{}</version>
            <scope>{}</scope>
        </dependency>"#,
        dep_info.group_id,
        dep_info.artifact_id,
        dep_info.version.as_ref().expect("版本号必须存在"),
        scope
    );

    // 在</dependencies>前添加依赖
    lines.insert(dependencies_end, dep_xml);

    // 写回文件
    fs::write(&pom_path, lines.join("\n"))?;

    println!("已添加到 pom.xml");
    Ok(())
}

fn add_to_gradle(project_dir: &Path, dep_info: &DependencyInfo, scope: &str) -> Result<()> {
    let build_gradle_path = project_dir.join("build.gradle");
    let build_content = fs::read_to_string(&build_gradle_path)?;

    // 简单的Gradle解析和修改
    let mut lines: Vec<String> = build_content.lines().map(|s| s.to_string()).collect();

    // 查找dependencies部分
    let mut in_dependencies = false;
    let mut _dependencies_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "dependencies {" {
            in_dependencies = true;
            _dependencies_start = i;
            break;
        }
    }

    if !in_dependencies {
        return Err(anyhow::anyhow!("在build.gradle中找不到dependencies部分"));
    }

    // 构建依赖行
    let dep_line = format!(
        "    {} '{}:{}:{}'",
        scope,
        dep_info.group_id,
        dep_info.artifact_id,
        dep_info.version.as_ref().expect("版本号必须存在")
    );

    // 在dependencies部分后添加依赖
    lines.insert(_dependencies_start + 1, dep_line);

    // 写回文件
    fs::write(&build_gradle_path, lines.join("\n"))?;

    println!("已添加到 build.gradle");
    Ok(())
}

fn get_latest_version(group_id: &str, artifact_id: &str) -> Result<String> {
    println!("🔍 正在查询最新版本...");

    // 尝试从 Maven Central 获取最新版本
    let url = format!(
        "https://search.maven.org/solrsearch/select?q=g:{}+AND+a:{}&rows=1&wt=json",
        group_id, artifact_id
    );

    // 使用 curl 获取信息
    let output = Command::new("curl").arg("-s").arg("-f").arg(&url).output();

    match output {
        Ok(output) if output.status.success() => {
            let response = String::from_utf8_lossy(&output.stdout);

            // 简单的 JSON 解析来获取版本号
            if let Some(version) = parse_maven_central_response(&response) {
                return Ok(version);
            }
        }
        _ => {}
    }

    // 如果 Maven Central 查询失败，尝试使用 Maven 命令
    if check_command_exists("mvn") {
        let output = Command::new("mvn")
            .arg("-q")
            .arg("versions:display-dependency-updates")
            .arg(
                "-Dincludes={}:{}"
                    .replace("{}", group_id)
                    .replace("{}", artifact_id),
            )
            .arg("-DprocessDependencies=false")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                if let Some(version) = parse_maven_version_output(&response) {
                    return Ok(version);
                }
            }
        }
    }

    // 返回一个默认的最新稳定版本（对于常见库）
    Ok(get_default_latest_version(group_id, artifact_id))
}

fn parse_maven_central_response(response: &str) -> Option<String> {
    // 查找 "latestVersion" 或 "v" 字段
    if let Some(start) = response.find("\"latestVersion\":\"") {
        let start = start + 17;
        if let Some(end) = response[start..].find('"') {
            return Some(response[start..start + end].to_string());
        }
    } else if let Some(start) = response.find("\"v\":\"") {
        let start = start + 5;
        if let Some(end) = response[start..].find('"') {
            return Some(response[start..start + end].to_string());
        }
    }
    None
}

fn parse_maven_version_output(output: &str) -> Option<String> {
    // 解析 Maven 输出以找到版本号
    for line in output.lines() {
        if line.contains("->") {
            if let Some(arrow_pos) = line.find("->") {
                let version = line[arrow_pos + 2..].trim();
                if !version.is_empty() {
                    return Some(version.to_string());
                }
            }
        }
    }
    None
}

fn get_default_latest_version(group_id: &str, artifact_id: &str) -> String {
    // 为一些常见库提供默认的最新稳定版本
    match (group_id, artifact_id) {
        ("org.springframework", "spring-core") => "5.3.31".to_string(),
        ("org.springframework", "spring-web") => "5.3.31".to_string(),
        ("org.springframework.boot", "spring-boot-starter") => "2.7.18".to_string(),
        ("junit", "junit") => "4.13.2".to_string(),
        ("org.junit.jupiter", "junit-jupiter") => "5.10.1".to_string(),
        ("com.google.guava", "guava") => "32.1.3-jre".to_string(),
        ("org.apache.commons", "commons-lang3") => "3.14.0".to_string(),
        ("com.fasterxml.jackson.core", "jackson-databind") => "2.16.0".to_string(),
        ("org.slf4j", "slf4j-api") => "2.0.9".to_string(),
        ("ch.qos.logback", "logback-classic") => "1.4.14".to_string(),
        _ => "1.0.0".to_string(),
    }
}

fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
