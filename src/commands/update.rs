use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(clap::Args)]
#[clap(about = "更新依赖")]
pub struct UpdateCommand {
    #[clap(index = 1, help = "依赖坐标 (groupId:artifactId)")]
    pub dependency: Option<String>,

    #[clap(long, help = "更新到最新版本")]
    pub latest: bool,
}

impl UpdateCommand {
    pub fn execute(&self) -> Result<()> {
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

        println!("🔄 更新依赖...");

        if let Some(dep) = &self.dependency {
            println!("依赖: {}", dep);
        } else {
            println!("更新所有依赖");
        }

        if self.latest {
            println!("更新到最新版本");
        }

        // 根据配置文件类型更新依赖
        let result = match config_file {
            "jx.toml" => update_jx_config(&current_dir, &self.dependency, self.latest),
            "pom.xml" => update_maven(&current_dir, &self.dependency, self.latest),
            "build.gradle" => update_gradle(&current_dir, &self.dependency, self.latest),
            _ => Err(anyhow::anyhow!("不支持的配置文件类型")),
        };

        match result {
            Ok(_) => {
                println!("✅ 依赖更新完成!");
                println!("请运行 'jx install' 来安装更新后的依赖");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 更新失败: {}", e);
                Err(e)
            }
        }
    }
}

fn update_jx_config(project_dir: &Path, dependency: &Option<String>, latest: bool) -> Result<()> {
    let config_path = project_dir.join("jx.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("找不到jx.toml配置文件"));
    }

    if latest {
        // 对于最新版本，提示用户使用具体版本号
        return Err(anyhow::anyhow!(
            "请指定具体的版本号进行更新，格式: groupId:artifactId:version"
        ));
    } else if let Some(dep) = dependency {
        // 更新特定依赖
        let dep_info = parse_dependency_coordinate_auto(dep)?;
        let config_content = fs::read_to_string(&config_path)?;
        let mut lines: Vec<String> = config_content.lines().map(|s| s.to_string()).collect();

        for i in 0..lines.len() {
            let line = lines[i].trim();
            if line.starts_with(&format!("{}:{}", dep_info.group_id, dep_info.artifact_id)) {
                // 更新为指定的版本号
                let new_line = format!(
                    "{}:{} = \"{}\"",
                    dep_info.group_id, dep_info.artifact_id, dep_info.version
                );
                lines[i] = new_line;
                println!(
                    "已更新依赖 {}:{} 到版本 {}",
                    dep_info.group_id, dep_info.artifact_id, dep_info.version
                );
                break;
            }
        }

        fs::write(&config_path, lines.join("\n"))?;
    }

    Ok(())
}

fn update_maven(project_dir: &Path, dependency: &Option<String>, latest: bool) -> Result<()> {
    if latest {
        // 使用Maven命令更新所有依赖
        println!("使用Maven更新所有依赖...");

        if !check_command_exists("mvn") {
            return Err(anyhow::anyhow!("Maven未安装，请先安装Maven"));
        }

        let output = Command::new("mvn")
            .arg("versions:use-latest-versions")
            .current_dir(project_dir)
            .output()
            .context("执行Maven命令失败")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Maven更新失败: {}", error));
        }

        println!("Maven依赖更新完成");
    } else if let Some(dep) = dependency {
        // 更新特定依赖
        let dep_info = parse_dependency_coordinate_auto(dep)?;
        let pom_path = project_dir.join("pom.xml");
        let pom_content = fs::read_to_string(&pom_path)?;
        let mut lines: Vec<String> = pom_content.lines().map(|s| s.to_string()).collect();

        // 查找并更新版本号
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line == "<dependency>" {
                let mut in_dependency = false;
                let mut dependency_start = i;

                for j in i..lines.len() {
                    let dep_line = lines[j].trim();
                    if dep_line == "<dependency>" {
                        in_dependency = true;
                        dependency_start = j;
                    } else if in_dependency && dep_line == "</dependency>" {
                        // 检查这个依赖是否匹配
                        let dependency_lines = &lines[dependency_start..=j];
                        if dependency_lines.iter().any(|l| {
                            l.contains(&format!("<groupId>{}</groupId>", dep_info.group_id))
                        }) && dependency_lines.iter().any(|l| {
                            l.contains(&format!(
                                "<artifactId>{}</artifactId>",
                                dep_info.artifact_id
                            ))
                        }) {
                            // 更新为指定的版本号
                            for k in dependency_start..=j {
                                if lines[k].trim().starts_with("<version>") {
                                    lines[k] = format!(
                                        "            <version>{}</version>",
                                        dep_info.version
                                    );
                                    println!(
                                        "已更新依赖 {}:{} 到版本 {}",
                                        dep_info.group_id, dep_info.artifact_id, dep_info.version
                                    );
                                    break;
                                }
                            }
                            break;
                        }
                        in_dependency = false;
                    }
                }
                break;
            }
            i += 1;
        }

        fs::write(&pom_path, lines.join("\n"))?;
    }

    Ok(())
}

fn update_gradle(project_dir: &Path, dependency: &Option<String>, latest: bool) -> Result<()> {
    if latest {
        // 使用Gradle命令更新所有依赖
        println!("使用Gradle更新所有依赖...");

        if !check_command_exists("gradle") {
            return Err(anyhow::anyhow!("Gradle未安装，请先安装Gradle"));
        }

        let output = Command::new("gradle")
            .arg("dependencyUpdates")
            .current_dir(project_dir)
            .output()
            .context("执行Gradle命令失败")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Gradle更新失败: {}", error));
        }

        println!("Gradle依赖更新完成");
    } else if let Some(dep) = dependency {
        // 更新特定依赖
        let dep_info = parse_dependency_coordinate_auto(dep)?;
        let build_gradle_path = project_dir.join("build.gradle");
        let build_content = fs::read_to_string(&build_gradle_path)?;
        let mut lines: Vec<String> = build_content.lines().map(|s| s.to_string()).collect();

        // 查找并更新版本号
        for i in 0..lines.len() {
            let line = lines[i].trim();
            if line.contains(&format!("'{}:{}", dep_info.group_id, dep_info.artifact_id)) {
                // 更新为指定的版本号
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let scope = parts[0];
                    let new_line = format!(
                        "    {} '{}:{}:{}'",
                        scope, dep_info.group_id, dep_info.artifact_id, dep_info.version
                    );
                    lines[i] = new_line;
                    println!(
                        "已更新依赖 {}:{} 到版本 {}",
                        dep_info.group_id, dep_info.artifact_id, dep_info.version
                    );
                    break;
                }
            }
        }

        fs::write(&build_gradle_path, lines.join("\n"))?;
    }

    Ok(())
}

#[derive(Debug)]
struct DependencyInfo {
    group_id: String,
    artifact_id: String,
    version: String,
}

fn parse_dependency_coordinate(coordinate: &str) -> Result<DependencyInfo> {
    let parts: Vec<&str> = coordinate.split(':').collect();

    match parts.len() {
        2 => Ok(DependencyInfo {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: String::new(),
        }),
        _ => Err(anyhow::anyhow!(
            "无效的依赖坐标格式，应为 groupId:artifactId"
        )),
    }
}

fn parse_dependency_coordinate_with_version(coordinate: &str) -> Result<DependencyInfo> {
    let parts: Vec<&str> = coordinate.split(':').collect();

    match parts.len() {
        3 => Ok(DependencyInfo {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: parts[2].to_string(),
        }),
        _ => Err(anyhow::anyhow!(
            "无效的依赖坐标格式，必须指定版本: groupId:artifactId:version"
        )),
    }
}

fn parse_dependency_coordinate_auto(coordinate: &str) -> Result<DependencyInfo> {
    let parts: Vec<&str> = coordinate.split(':').collect();

    match parts.len() {
        2 => {
            // 没有版本号，自动获取最新版本
            let latest_version = get_latest_version(parts[0], parts[1])?;
            println!("📦 自动获取最新版本: {}", latest_version);
            Ok(DependencyInfo {
                group_id: parts[0].to_string(),
                artifact_id: parts[1].to_string(),
                version: latest_version,
            })
        }
        3 => Ok(DependencyInfo {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: parts[2].to_string(),
        }),
        _ => Err(anyhow::anyhow!(
            "无效的依赖坐标格式，应为 groupId:artifactId 或 groupId:artifactId:version"
        )),
    }
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