use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn execute(query: String, limit: usize) -> Result<()> {
    println!("🔍 搜索依赖...");
    println!("搜索关键词: {}", query);
    println!("最大结果数: {}", limit);

    let current_dir = std::env::current_dir()?;

    // 检测项目类型
    let project_type = detect_project_type(&current_dir)?;
    println!("项目类型: {}", project_type);

    // 本地搜索
    let local_results = search_local_dependencies(&current_dir, &query, limit)?;
    if !local_results.is_empty() {
        println!("\n📁 本地依赖搜索结果:");
        for (i, result) in local_results.iter().enumerate() {
            println!(
                "  {}. {}:{}:{}",
                i + 1,
                result.group_id,
                result.artifact_id,
                result.version
            );
        }
    }

    // 配置文件搜索
    let config_results = search_in_config_files(&current_dir, &query, limit)?;
    if !config_results.is_empty() {
        println!("\n⚙️ 配置文件搜索结果:");
        for (i, result) in config_results.iter().enumerate() {
            println!(
                "  {}. {}:{}:{}",
                i + 1,
                result.group_id,
                result.artifact_id,
                result.version
            );
        }
    }

    // Maven Central搜索（模拟）
    let central_results = search_maven_central(&query, limit)?;
    if !central_results.is_empty() {
        println!("\n🌐 Maven Central搜索结果:");
        for (i, result) in central_results.iter().enumerate() {
            println!(
                "  {}. {}:{}:{}",
                i + 1,
                result.group_id,
                result.artifact_id,
                result.version
            );
            if let Some(desc) = &result.description {
                println!("     描述: {}", desc);
            }
        }
    }

    let total_results = local_results.len() + config_results.len() + central_results.len();
    if total_results == 0 {
        println!("\n❌ 未找到匹配的依赖");
        println!("💡 提示:");
        println!("  - 检查搜索关键词是否正确");
        println!("  - 尝试使用更通用的关键词");
        println!("  - 确保项目已正确配置");
    } else {
        println!("\n✅ 搜索完成，共找到 {} 个结果", total_results);
    }

    Ok(())
}

#[derive(Debug)]
struct DependencyResult {
    group_id: String,
    artifact_id: String,
    version: String,
    description: Option<String>,
    #[allow(dead_code)]
    source: String,
}

fn detect_project_type(project_dir: &Path) -> Result<String> {
    let has_pom = project_dir.join("pom.xml").exists();
    let has_gradle = project_dir.join("build.gradle").exists();
    let has_settings_gradle = project_dir.join("settings.gradle").exists();

    if has_pom && (has_gradle || has_settings_gradle) {
        Ok("Maven + Gradle".to_string())
    } else if has_pom {
        Ok("Maven".to_string())
    } else if has_gradle || has_settings_gradle {
        Ok("Gradle".to_string())
    } else {
        Ok("未知".to_string())
    }
}

fn search_local_dependencies(
    project_dir: &Path,
    query: &str,
    limit: usize,
) -> Result<Vec<DependencyResult>> {
    let mut results = Vec::new();

    // 搜索lib目录中的jar文件
    let lib_dir = project_dir.join("lib");
    if lib_dir.exists() {
        for entry in fs::read_dir(&lib_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.to_lowercase().contains(&query.to_lowercase()) {
                        // 尝试从文件名解析依赖信息
                        if let Some(dep_info) = parse_jar_filename(filename) {
                            results.push(DependencyResult {
                                group_id: dep_info.group_id,
                                artifact_id: dep_info.artifact_id,
                                version: dep_info.version,
                                description: Some("本地jar文件".to_string()),
                                source: "local".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // 限制结果数量
    results.truncate(limit);
    Ok(results)
}

fn search_in_config_files(
    project_dir: &Path,
    query: &str,
    limit: usize,
) -> Result<Vec<DependencyResult>> {
    let mut results = Vec::new();

    // 搜索pom.xml
    let pom_path = project_dir.join("pom.xml");
    if pom_path.exists() {
        let pom_content = fs::read_to_string(&pom_path)?;
        let pom_results = parse_maven_dependencies(&pom_content, query)?;
        results.extend(pom_results);
    }

    // 搜索build.gradle
    let gradle_path = project_dir.join("build.gradle");
    if gradle_path.exists() {
        let gradle_content = fs::read_to_string(&gradle_path)?;
        let gradle_results = parse_gradle_dependencies(&gradle_content, query)?;
        results.extend(gradle_results);
    }

    // 搜索jx.toml
    let jx_path = project_dir.join("jx.toml");
    if jx_path.exists() {
        let jx_content = fs::read_to_string(&jx_path)?;
        let jx_results = parse_jx_dependencies(&jx_content, query)?;
        results.extend(jx_results);
    }

    // 限制结果数量
    results.truncate(limit);
    Ok(results)
}

fn search_maven_central(query: &str, limit: usize) -> Result<Vec<DependencyResult>> {
    // 实现真实的Maven Central搜索
    let mut results = Vec::new();

    // 构建搜索URL
    let search_url = format!(
        "https://search.maven.org/solrsearch/select?q={}&rows={}&wt=json",
        urlencoding::encode(query),
        limit
    );

    println!("🔍 正在搜索Maven Central: {}", search_url);

    // 使用curl进行HTTP请求
    let output = std::process::Command::new("curl")
        .args(&["-s", "-H", "Accept: application/json", &search_url])
        .output()
        .context("执行curl命令失败")?;

    if !output.status.success() {
        println!("⚠️ Maven Central搜索失败，使用本地模拟结果");
        return search_maven_central_fallback(query, limit);
    }

    let response = String::from_utf8_lossy(&output.stdout);

    // 解析JSON响应
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
        if let Some(response_obj) = json.get("response") {
            if let Some(docs) = response_obj.get("docs") {
                if let Some(docs_array) = docs.as_array() {
                    for doc in docs_array {
                        if let (Some(group_id), Some(artifact_id), Some(version)) = (
                            doc.get("g").and_then(|v| v.as_str()),
                            doc.get("a").and_then(|v| v.as_str()),
                            doc.get("v").and_then(|v| v.as_str()),
                        ) {
                            let description = doc
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            results.push(DependencyResult {
                                group_id: group_id.to_string(),
                                artifact_id: artifact_id.to_string(),
                                version: version.to_string(),
                                description,
                                source: "maven-central".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // 如果解析失败或没有结果，使用备用方案
    if results.is_empty() {
        println!("⚠️ JSON解析失败，使用本地模拟结果");
        return search_maven_central_fallback(query, limit);
    }

    Ok(results)
}

fn search_maven_central_fallback(query: &str, limit: usize) -> Result<Vec<DependencyResult>> {
    // 本地模拟搜索结果作为备用方案
    let mut results = Vec::new();

    // 扩展的模拟搜索结果
    let mock_results = [
        (
            "org.springframework",
            "spring-core",
            "5.3.0",
            "Spring Framework核心模块",
        ),
        (
            "org.springframework",
            "spring-web",
            "5.3.0",
            "Spring Web模块",
        ),
        (
            "org.springframework",
            "spring-boot-starter",
            "2.7.0",
            "Spring Boot启动器",
        ),
        (
            "org.springframework",
            "spring-context",
            "5.3.0",
            "Spring上下文模块",
        ),
        (
            "org.springframework",
            "spring-beans",
            "5.3.0",
            "Spring Beans模块",
        ),
        (
            "com.fasterxml.jackson.core",
            "jackson-databind",
            "2.13.0",
            "Jackson数据绑定",
        ),
        (
            "com.fasterxml.jackson.core",
            "jackson-core",
            "2.13.0",
            "Jackson核心模块",
        ),
        (
            "org.apache.commons",
            "commons-lang3",
            "3.12.0",
            "Apache Commons Lang3",
        ),
        (
            "org.apache.commons",
            "commons-io",
            "2.11.0",
            "Apache Commons IO",
        ),
        ("junit", "junit", "4.13.2", "JUnit测试框架"),
        ("org.mockito", "mockito-core", "4.5.1", "Mockito模拟框架"),
        (
            "org.mockito",
            "mockito-junit-jupiter",
            "4.5.1",
            "Mockito JUnit5支持",
        ),
        ("org.slf4j", "slf4j-api", "1.7.36", "SLF4J日志门面"),
        (
            "ch.qos.logback",
            "logback-classic",
            "1.2.11",
            "Logback经典模块",
        ),
        (
            "org.hibernate",
            "hibernate-core",
            "5.6.0",
            "Hibernate核心模块",
        ),
        ("mysql", "mysql-connector-java", "8.0.27", "MySQL连接器"),
        ("org.postgresql", "postgresql", "42.3.1", "PostgreSQL连接器"),
        (
            "org.mongodb",
            "mongodb-driver-sync",
            "4.4.0",
            "MongoDB同步驱动",
        ),
        (
            "org.elasticsearch.client",
            "elasticsearch-rest-high-level-client",
            "7.17.0",
            "Elasticsearch高级客户端",
        ),
        ("org.apache.kafka", "kafka-clients", "3.0.0", "Kafka客户端"),
        (
            "org.apache.spark",
            "spark-core_2.12",
            "3.2.0",
            "Spark核心模块",
        ),
    ];

    for (group_id, artifact_id, version, description) in &mock_results {
        if group_id.to_lowercase().contains(&query.to_lowercase())
            || artifact_id.to_lowercase().contains(&query.to_lowercase())
        {
            results.push(DependencyResult {
                group_id: group_id.to_string(),
                artifact_id: artifact_id.to_string(),
                version: version.to_string(),
                description: Some(description.to_string()),
                source: "maven-central-fallback".to_string(),
            });
        }
    }

    // 限制结果数量
    results.truncate(limit);
    Ok(results)
}

#[derive(Debug)]
struct JarDependencyInfo {
    group_id: String,
    artifact_id: String,
    version: String,
}

fn parse_jar_filename(filename: &str) -> Option<JarDependencyInfo> {
    // 尝试从jar文件名解析依赖信息
    // 格式通常是: artifactId-version.jar 或 groupId-artifactId-version.jar

    if !filename.ends_with(".jar") {
        return None;
    }

    let name_without_ext = &filename[..filename.len() - 4];
    let parts: Vec<&str> = name_without_ext.split('-').collect();

    if parts.len() >= 2 {
        let artifact_id = parts[0].to_string();
        let version = parts[1].to_string();
        let group_id = if parts.len() > 2 {
            parts[..parts.len() - 2].join(".")
        } else {
            "unknown".to_string()
        };

        Some(JarDependencyInfo {
            group_id,
            artifact_id,
            version,
        })
    } else {
        None
    }
}

fn parse_maven_dependencies(pom_content: &str, query: &str) -> Result<Vec<DependencyResult>> {
    let mut results = Vec::new();
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
                        if group_id.to_lowercase().contains(&query.to_lowercase())
                            || artifact_id.to_lowercase().contains(&query.to_lowercase())
                        {
                            results.push(DependencyResult {
                                group_id: group_id.clone(),
                                artifact_id: artifact_id.clone(),
                                version: version.clone(),
                                description: Some("Maven依赖".to_string()),
                                source: "pom.xml".to_string(),
                            });
                        }
                    }
                }
            } else if line.starts_with("<") && line.ends_with(">") {
                if let Some(dep) = &mut current_dep {
                    let content = line.trim_start_matches('<').trim_end_matches('>');
                    if let Some(tag_name) = content.split('>').next() {
                        if let Some(value) = content.split('>').nth(1) {
                            dep.insert(tag_name.to_string(), value.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

fn parse_gradle_dependencies(gradle_content: &str, query: &str) -> Result<Vec<DependencyResult>> {
    let mut results = Vec::new();
    let lines: Vec<&str> = gradle_content.lines().collect();

    let mut in_dependencies = false;

    for line in lines {
        let line = line.trim();

        if line == "dependencies {" {
            in_dependencies = true;
        } else if line == "}" && in_dependencies {
            in_dependencies = false;
            break;
        } else if in_dependencies && line.contains("'") {
            // 解析Gradle依赖行，格式通常是: implementation 'groupId:artifactId:version'
            let parts: Vec<&str> = line.split('\'').collect();
            if parts.len() >= 2 {
                let dep_coord = parts[1];
                let coord_parts: Vec<&str> = dep_coord.split(':').collect();

                if coord_parts.len() >= 2 {
                    let group_id = coord_parts[0];
                    let artifact_id = coord_parts[1];
                    let version = coord_parts.get(2).unwrap_or(&"*");

                    if group_id.to_lowercase().contains(&query.to_lowercase())
                        || artifact_id.to_lowercase().contains(&query.to_lowercase())
                    {
                        results.push(DependencyResult {
                            group_id: group_id.to_string(),
                            artifact_id: artifact_id.to_string(),
                            version: version.to_string(),
                            description: Some("Gradle依赖".to_string()),
                            source: "build.gradle".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}

fn parse_jx_dependencies(jx_content: &str, query: &str) -> Result<Vec<DependencyResult>> {
    let mut results = Vec::new();
    let lines: Vec<&str> = jx_content.lines().collect();

    let mut in_dependencies = false;

    for line in lines {
        let line = line.trim();

        if line == "[dependencies]" {
            in_dependencies = true;
        } else if line.starts_with('[') && line != "[dependencies]" {
            in_dependencies = false;
        } else if in_dependencies && line.contains('=') {
            // 解析jx.toml依赖行，格式通常是: groupId:artifactId = "version"
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let dep_coord = parts[0].trim();
                let version = parts[1].trim().trim_matches('"');

                let coord_parts: Vec<&str> = dep_coord.split(':').collect();
                if coord_parts.len() == 2 {
                    let group_id = coord_parts[0];
                    let artifact_id = coord_parts[1];

                    if group_id.to_lowercase().contains(&query.to_lowercase())
                        || artifact_id.to_lowercase().contains(&query.to_lowercase())
                    {
                        results.push(DependencyResult {
                            group_id: group_id.to_string(),
                            artifact_id: artifact_id.to_string(),
                            version: version.to_string(),
                            description: Some("jx配置依赖".to_string()),
                            source: "jx.toml".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}
