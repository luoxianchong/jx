use anyhow::Result;
use std::fs;
use std::path::Path;

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
        2 => Ok(DependencyInfo {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: None,
        }),
        3 => Ok(DependencyInfo {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: Some(parts[2].to_string()),
        }),
        _ => Err(anyhow::anyhow!("无效的依赖坐标格式，应为 groupId:artifactId 或 groupId:artifactId:version")),
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
    let dep_line = if let Some(version) = &dep_info.version {
        format!("{}:{} = \"{}\"", dep_info.group_id, dep_info.artifact_id, version)
    } else {
        format!("{}:{} = \"*\"", dep_info.group_id, dep_info.artifact_id)
    };
    
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
        dep_info.version.as_ref().unwrap_or(&"*".to_string()),
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
    let dep_line = if let Some(version) = &dep_info.version {
        format!("    {} '{}:{}:{}'", scope, dep_info.group_id, dep_info.artifact_id, version)
    } else {
        format!("    {} '{}:{}'", scope, dep_info.group_id, dep_info.artifact_id)
    };
    
    // 在dependencies部分后添加依赖
    lines.insert(_dependencies_start + 1, dep_line);
    
    // 写回文件
    fs::write(&build_gradle_path, lines.join("\n"))?;
    
    println!("已添加到 build.gradle");
    Ok(())
}
