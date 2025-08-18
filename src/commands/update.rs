use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute(dependency: Option<String>, latest: bool) -> Result<()> {
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
    
    if let Some(dep) = &dependency {
        println!("依赖: {}", dep);
    } else {
        println!("更新所有依赖");
    }
    
    if latest {
        println!("更新到最新版本");
    }

    // 根据配置文件类型更新依赖
    let result = match config_file {
        "jx.toml" => update_jx_config(&current_dir, &dependency, latest),
        "pom.xml" => update_maven(&current_dir, &dependency, latest),
        "build.gradle" => update_gradle(&current_dir, &dependency, latest),
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

fn update_jx_config(project_dir: &Path, dependency: &Option<String>, latest: bool) -> Result<()> {
    let config_path = project_dir.join("jx.toml");
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("找不到jx.toml配置文件"));
    }
    
    if latest {
        // 更新所有依赖到最新版本
        let config_content = fs::read_to_string(&config_path)?;
        let mut lines: Vec<String> = config_content.lines().map(|s| s.to_string()).collect();
        
        for i in 0..lines.len() {
            let line = lines[i].trim();
            if line.contains(" = \"") && !line.contains(" = \"*\"") {
                // 将版本号改为 *
                let new_line = line.replace(" = \"", " = \"*\"");
                lines[i] = new_line;
            }
        }
        
        fs::write(&config_path, lines.join("\n"))?;
        println!("已更新jx.toml中的所有依赖到最新版本");
    } else if let Some(dep) = dependency {
        // 更新特定依赖
        let dep_info = parse_dependency_coordinate(dep)?;
        let config_content = fs::read_to_string(&config_path)?;
        let mut lines: Vec<String> = config_content.lines().map(|s| s.to_string()).collect();
        
        for i in 0..lines.len() {
            let line = lines[i].trim();
            if line.starts_with(&format!("{}:{}", dep_info.group_id, dep_info.artifact_id)) {
                // 将版本号改为 *
                let new_line = line.replace(" = \"", " = \"*\"");
                lines[i] = new_line;
                println!("已更新依赖 {} 到最新版本", dep);
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
        let dep_info = parse_dependency_coordinate(dep)?;
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
                        if dependency_lines.iter().any(|l| l.contains(&format!("<groupId>{}</groupId>", dep_info.group_id))) &&
                           dependency_lines.iter().any(|l| l.contains(&format!("<artifactId>{}</artifactId>", dep_info.artifact_id))) {
                            // 将版本号改为 *
                            for k in dependency_start..=j {
                                if lines[k].trim().starts_with("<version>") {
                                    lines[k] = "            <version>*</version>".to_string();
                                    println!("已更新依赖 {} 到最新版本", dep);
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
        let dep_info = parse_dependency_coordinate(dep)?;
        let build_gradle_path = project_dir.join("build.gradle");
        let build_content = fs::read_to_string(&build_gradle_path)?;
        let mut lines: Vec<String> = build_content.lines().map(|s| s.to_string()).collect();
        
        // 查找并更新版本号
        for i in 0..lines.len() {
            let line = lines[i].trim();
            if line.contains(&format!("'{}:{}", dep_info.group_id, dep_info.artifact_id)) {
                // 将版本号改为 +
                let new_line = line.replace("'", "'").replace(":", ":+");
                lines[i] = new_line;
                println!("已更新依赖 {} 到最新版本", dep);
                break;
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
}

fn parse_dependency_coordinate(coordinate: &str) -> Result<DependencyInfo> {
    let parts: Vec<&str> = coordinate.split(':').collect();
    
    match parts.len() {
        2 => Ok(DependencyInfo {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
        }),
        _ => Err(anyhow::anyhow!("无效的依赖坐标格式，应为 groupId:artifactId")),
    }
}

fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
