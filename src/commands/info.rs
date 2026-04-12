use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn execute() -> Result<()> {
    println!("ℹ️ 项目信息...");

    let current_dir = std::env::current_dir()?;

    // 检测项目类型
    let project_type = detect_project_type(&current_dir)?;
    println!("项目类型: {}", project_type);

    // 获取项目基本信息
    let project_info = get_project_info(&current_dir, &project_type)?;

    // 显示项目基本信息
    display_project_info(&project_info);

    // 显示依赖信息
    display_dependencies(&current_dir, &project_type)?;

    // 显示构建信息
    display_build_info(&current_dir, &project_type)?;

    // 显示文件统计信息
    display_file_stats(&current_dir)?;

    // 显示环境信息
    display_environment_info()?;

    Ok(())
}

#[derive(Debug)]
struct ProjectInfo {
    name: String,
    version: String,
    description: Option<String>,
    group_id: Option<String>,
    artifact_id: Option<String>,
    packaging: Option<String>,
    java_version: Option<String>,
    source_encoding: Option<String>,
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

fn get_project_info(project_dir: &Path, project_type: &str) -> Result<ProjectInfo> {
    match project_type {
        "Maven" | "Maven + Gradle" => get_maven_project_info(project_dir),
        "Gradle" => get_gradle_project_info(project_dir),
        "jx" => get_jx_project_info(project_dir),
        _ => get_generic_project_info(project_dir),
    }
}

fn get_maven_project_info(project_dir: &Path) -> Result<ProjectInfo> {
    let pom_path = project_dir.join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)?;

    let mut info = ProjectInfo {
        name: "未知".to_string(),
        version: "未知".to_string(),
        description: None,
        group_id: None,
        artifact_id: None,
        packaging: None,
        java_version: None,
        source_encoding: None,
    };

    let lines: Vec<&str> = pom_content.lines().collect();

    for line in lines {
        let line = line.trim();

        if line.starts_with("<groupId>") && line.ends_with("</groupId>") {
            info.group_id = Some(line[10..line.len() - 11].to_string());
        } else if line.starts_with("<artifactId>") && line.ends_with("</artifactId>") {
            info.artifact_id = Some(line[13..line.len() - 14].to_string());
            info.name = line[13..line.len() - 14].to_string();
        } else if line.starts_with("<version>") && line.ends_with("</version>") {
            info.version = line[9..line.len() - 10].to_string();
        } else if line.starts_with("<packaging>") && line.ends_with("</packaging>") {
            info.packaging = Some(line[11..line.len() - 12].to_string());
        } else if line.starts_with("<description>") && line.ends_with("</description>") {
            info.description = Some(line[13..line.len() - 14].to_string());
        } else if line.starts_with("<maven.compiler.source>")
            && line.ends_with("</maven.compiler.source>")
        {
            let start = "<maven.compiler.source>".len();
            let end = line.len() - "</maven.compiler.source>".len();
            if start < end {
                info.java_version = Some(line[start..end].to_string());
            }
        } else if line.starts_with("<project.build.sourceEncoding>")
            && line.ends_with("</project.build.sourceEncoding>")
        {
            let start = "<project.build.sourceEncoding>".len();
            let end = line.len() - "</project.build.sourceEncoding>".len();
            if start < end {
                info.source_encoding = Some(line[start..end].to_string());
            }
        }
    }

    Ok(info)
}

fn get_gradle_project_info(project_dir: &Path) -> Result<ProjectInfo> {
    let build_gradle_path = project_dir.join("build.gradle");
    let build_content = fs::read_to_string(&build_gradle_path)?;

    let mut info = ProjectInfo {
        name: "未知".to_string(),
        version: "未知".to_string(),
        description: None,
        group_id: None,
        artifact_id: None,
        packaging: None,
        java_version: None,
        source_encoding: None,
    };

    let lines: Vec<&str> = build_content.lines().collect();

    for line in lines {
        let line = line.trim();

        if line.starts_with("rootProject.name") {
            if let Some(quote_start) = line.find('\'') {
                if let Some(quote_end) = line.rfind('\'') {
                    info.name = line[quote_start + 1..quote_end].to_string();
                }
            }
        } else if line.starts_with("version") {
            if let Some(quote_start) = line.find('\'') {
                if let Some(quote_end) = line.rfind('\'') {
                    info.version = line[quote_start + 1..quote_end].to_string();
                }
            }
        } else if line.starts_with("group") {
            if let Some(quote_start) = line.find('\'') {
                if let Some(quote_end) = line.rfind('\'') {
                    info.group_id = Some(line[quote_start + 1..quote_end].to_string());
                }
            }
        } else if line.starts_with("sourceCompatibility") {
            if let Some(quote_start) = line.find('\'') {
                if let Some(quote_end) = line.rfind('\'') {
                    info.java_version = Some(line[quote_start + 1..quote_end].to_string());
                }
            }
        }
    }

    Ok(info)
}

fn get_jx_project_info(project_dir: &Path) -> Result<ProjectInfo> {
    let jx_path = project_dir.join("jx.toml");
    let jx_content = fs::read_to_string(&jx_path)?;

    let mut info = ProjectInfo {
        name: "未知".to_string(),
        version: "未知".to_string(),
        description: None,
        group_id: None,
        artifact_id: None,
        packaging: None,
        java_version: None,
        source_encoding: None,
    };

    let lines: Vec<&str> = jx_content.lines().collect();

    for line in lines {
        let line = line.trim();

        if line.starts_with("name = \"") {
            info.name = line[8..line.len() - 1].to_string();
        } else if line.starts_with("version = \"") {
            info.version = line[11..line.len() - 1].to_string();
        } else if line.starts_with("description = \"") {
            info.description = Some(line[15..line.len() - 1].to_string());
        } else if line.starts_with("java_version = \"") {
            info.java_version = Some(line[16..line.len() - 1].to_string());
        }
    }

    Ok(info)
}

fn get_generic_project_info(project_dir: &Path) -> Result<ProjectInfo> {
    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("未知")
        .to_string();

    Ok(ProjectInfo {
        name,
        version: "未知".to_string(),
        description: None,
        group_id: None,
        artifact_id: None,
        packaging: None,
        java_version: None,
        source_encoding: None,
    })
}

fn display_project_info(info: &ProjectInfo) {
    println!("\n📋 项目基本信息:");
    println!("{}", "─".repeat(40));
    println!("名称: {}", info.name);
    println!("版本: {}", info.version);

    if let Some(ref desc) = info.description {
        println!("描述: {}", desc);
    }

    if let Some(ref group_id) = info.group_id {
        println!("Group ID: {}", group_id);
    }

    if let Some(ref artifact_id) = info.artifact_id {
        println!("Artifact ID: {}", artifact_id);
    }

    if let Some(ref packaging) = info.packaging {
        println!("打包类型: {}", packaging);
    }

    if let Some(ref java_version) = info.java_version {
        println!("Java版本: {}", java_version);
    }

    if let Some(ref encoding) = info.source_encoding {
        println!("源码编码: {}", encoding);
    }
}

fn display_dependencies(project_dir: &Path, project_type: &str) -> Result<()> {
    println!("\n📦 依赖信息:");
    println!("{}", "─".repeat(40));

    let dependencies = match project_type {
        "Maven" | "Maven + Gradle" => read_maven_dependencies(project_dir)?,
        "Gradle" => read_gradle_dependencies(project_dir)?,
        "jx" => read_jx_dependencies(project_dir)?,
        _ => Vec::new(),
    };

    if dependencies.is_empty() {
        println!("暂无依赖");
    } else {
        println!("总依赖数: {}", dependencies.len());

        // 按作用域分组
        let mut scope_groups: HashMap<String, Vec<&str>> = HashMap::new();
        for dep in &dependencies {
            let scope = scope_groups
                .entry(dep.scope.clone())
                .or_insert_with(Vec::new);
            scope.push(&dep.coordinate);
        }

        for (scope, deps) in scope_groups {
            println!("\n{} 作用域 ({}个):", get_scope_icon(&scope), deps.len());
            for dep in deps {
                println!("  {}", dep);
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct DependencyInfo {
    coordinate: String,
    scope: String,
}

fn read_maven_dependencies(project_dir: &Path) -> Result<Vec<DependencyInfo>> {
    let pom_path = project_dir.join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)?;

    let mut dependencies = Vec::new();
    let lines: Vec<&str> = pom_content.lines().collect();

    let mut in_dependencies = false;
    let mut current_dep: Option<HashMap<String, String>> = None;

    for line in lines {
        let line = line.trim();

        if line == "<dependencies>" {
            in_dependencies = true;
        } else if line == "</dependencies>" {
            in_dependencies = false;
            break;
        } else if in_dependencies {
            if line == "<dependency>" {
                current_dep = Some(HashMap::new());
            } else if line == "</dependency>" {
                if let Some(dep) = current_dep.take() {
                    if let (Some(group_id), Some(artifact_id), Some(version)) = (
                        dep.get("groupId"),
                        dep.get("artifactId"),
                        dep.get("version"),
                    ) {
                        let scope = dep.get("scope").unwrap_or(&"compile".to_string()).clone();
                        let coordinate = format!("{}:{}:{}", group_id, artifact_id, version);
                        dependencies.push(DependencyInfo { coordinate, scope });
                    }
                }
            } else if line.starts_with("<") && line.ends_with(">") && !line.starts_with("</") {
                if let Some(dep) = &mut current_dep {
                    let content = line.trim_start_matches('<').trim_end_matches('>');
                    if let Some(colon_pos) = content.find('>') {
                        let tag_name = &content[..colon_pos];
                        let value = &content[colon_pos + 1..];

                        if !tag_name.is_empty() && !value.is_empty() {
                            dep.insert(tag_name.to_string(), value.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

fn read_gradle_dependencies(project_dir: &Path) -> Result<Vec<DependencyInfo>> {
    let build_gradle_path = project_dir.join("build.gradle");
    let build_content = fs::read_to_string(&build_gradle_path)?;

    let mut dependencies = Vec::new();
    let lines: Vec<&str> = build_content.lines().collect();

    let mut in_dependencies = false;

    for line in lines {
        let line = line.trim();

        if line == "dependencies {" {
            in_dependencies = true;
        } else if line == "}" && in_dependencies {
            in_dependencies = false;
            break;
        } else if in_dependencies && line.contains("'") {
            let parts: Vec<&str> = line.split('\'').collect();
            if parts.len() >= 2 {
                let dep_coord = parts[1];
                let coord_parts: Vec<&str> = dep_coord.split(':').collect();

                if coord_parts.len() >= 2 {
                    let group_id = coord_parts[0];
                    let artifact_id = coord_parts[1];
                    let version = coord_parts.get(2).unwrap_or(&"*");

                    let scope = if line.contains("implementation") {
                        "implementation"
                    } else if line.contains("compileOnly") {
                        "compileOnly"
                    } else if line.contains("runtimeOnly") {
                        "runtimeOnly"
                    } else if line.contains("testImplementation") {
                        "testImplementation"
                    } else {
                        "implementation"
                    };

                    let coordinate = format!("{}:{}:{}", group_id, artifact_id, version);
                    dependencies.push(DependencyInfo {
                        coordinate,
                        scope: scope.to_string(),
                    });
                }
            }
        }
    }

    Ok(dependencies)
}

fn read_jx_dependencies(project_dir: &Path) -> Result<Vec<DependencyInfo>> {
    let jx_path = project_dir.join("jx.toml");
    let jx_content = fs::read_to_string(&jx_path)?;

    let mut dependencies = Vec::new();
    let lines: Vec<&str> = jx_content.lines().collect();

    let mut in_dependencies = false;

    for line in lines {
        let line = line.trim();

        if line == "[dependencies]" {
            in_dependencies = true;
        } else if line.starts_with('[') && line != "[dependencies]" {
            in_dependencies = false;
        } else if in_dependencies && line.contains('=') {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let dep_coord = parts[0].trim();
                let version = parts[1].trim().trim_matches('"');

                let coordinate = format!("{}:{}", dep_coord, version);
                dependencies.push(DependencyInfo {
                    coordinate,
                    scope: "compile".to_string(),
                });
            }
        }
    }

    Ok(dependencies)
}

fn get_scope_icon(scope: &str) -> &str {
    match scope {
        "compile" => "📦 编译",
        "runtime" => "🔄 运行时",
        "test" => "🧪 测试",
        "provided" => "⚡ 提供",
        "system" => "💻 系统",
        "implementation" => "📦 实现",
        "compileOnly" => "📝 仅编译",
        "runtimeOnly" => "🔄 仅运行时",
        "testImplementation" => "🧪 测试实现",
        _ => "📦 其他",
    }
}

fn display_build_info(project_dir: &Path, project_type: &str) -> Result<()> {
    println!("\n🔨 构建信息:");
    println!("{}", "─".repeat(40));

    match project_type {
        "Maven" | "Maven + Gradle" => {
            let target_dir = project_dir.join("target");
            if target_dir.exists() {
                let target_size = calculate_directory_size(&target_dir)?;
                println!(
                    "Maven target目录: {} ({} bytes)",
                    target_dir.display(),
                    target_size
                );
            } else {
                println!("Maven target目录: 不存在");
            }
        }
        "Gradle" => {
            let build_dir = project_dir.join("build");
            if build_dir.exists() {
                let build_size = calculate_directory_size(&build_dir)?;
                println!(
                    "Gradle build目录: {} ({} bytes)",
                    build_dir.display(),
                    build_size
                );
            } else {
                println!("Gradle build目录: 不存在");
            }

            let gradle_dir = project_dir.join(".gradle");
            if gradle_dir.exists() {
                let gradle_size = calculate_directory_size(&gradle_dir)?;
                println!(
                    "Gradle缓存目录: {} ({} bytes)",
                    gradle_dir.display(),
                    gradle_size
                );
            }
        }
        _ => {}
    }

    let lib_dir = project_dir.join("lib");
    if lib_dir.exists() {
        let lib_size = calculate_directory_size(&lib_dir)?;
        println!("依赖库目录: {} ({} bytes)", lib_dir.display(), lib_size);
    }

    Ok(())
}

fn display_file_stats(project_dir: &Path) -> Result<()> {
    println!("\n📊 文件统计:");
    println!("{}", "─".repeat(40));

    let mut java_files = 0;
    let mut xml_files = 0;
    let mut gradle_files = 0;
    let mut toml_files = 0;
    let mut total_files = 0;

    for entry in walkdir::WalkDir::new(project_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            total_files += 1;

            if let Some(extension) = path.extension() {
                match extension.to_str() {
                    Some("java") => java_files += 1,
                    Some("xml") => xml_files += 1,
                    Some("gradle") => gradle_files += 1,
                    Some("toml") => toml_files += 1,
                    _ => {}
                }
            }
        }
    }

    println!("总文件数: {}", total_files);
    println!("Java源文件: {}", java_files);
    println!("XML配置文件: {}", xml_files);
    println!("Gradle文件: {}", gradle_files);
    println!("TOML配置文件: {}", toml_files);

    Ok(())
}

fn display_environment_info() -> Result<()> {
    println!("\n🌍 环境信息:");
    println!("{}", "─".repeat(40));

    println!("操作系统: {}", std::env::consts::OS);
    println!("架构: {}", std::env::consts::ARCH);
    println!("当前目录: {}", std::env::current_dir()?.display());

    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        println!("JAVA_HOME: {}", java_home);
    }

    if let Ok(maven_home) = std::env::var("MAVEN_HOME") {
        println!("MAVEN_HOME: {}", maven_home);
    }

    if let Ok(gradle_home) = std::env::var("GRADLE_HOME") {
        println!("GRADLE_HOME: {}", gradle_home);
    }

    Ok(())
}

fn calculate_directory_size(dir_path: &Path) -> Result<u64> {
    let mut total_size = 0;

    for entry in walkdir::WalkDir::new(dir_path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                total_size += metadata.len();
            }
        }
    }

    Ok(total_size)
}
