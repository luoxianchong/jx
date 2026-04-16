use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(clap::Args)]
#[clap(about = "移除依赖")]
pub struct RemoveCommand {
    #[clap(index = 1, required = true, help = "依赖坐标 (groupId:artifactId)")]
    pub dependency: String,
}

impl RemoveCommand {
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

        println!("🗑️ 移除依赖...");
        println!("依赖: {}", self.dependency);

        // 解析依赖坐标
        let dep_info = parse_dependency_coordinate(&self.dependency)?;

        // 根据配置文件类型移除依赖
        let result = match config_file {
            "jx.toml" => remove_from_jx_config(&current_dir, &dep_info),
            "pom.xml" => remove_from_maven(&current_dir, &dep_info),
            "build.gradle" => remove_from_gradle(&current_dir, &dep_info),
            _ => Err(anyhow::anyhow!("不支持的配置文件类型")),
        };

        match result {
            Ok(_) => {
                println!("✅ 依赖移除成功!");
                println!("请运行 'jx install' 来更新依赖");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 移除失败: {}", e);
                Err(e)
            }
        }
    }
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
        _ => Err(anyhow::anyhow!(
            "无效的依赖坐标格式，应为 groupId:artifactId"
        )),
    }
}

fn remove_from_jx_config(project_dir: &Path, dep_info: &DependencyInfo) -> Result<()> {
    let config_path = project_dir.join("jx.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("找不到jx.toml配置文件"));
    }

    let config_content = fs::read_to_string(&config_path)?;
    let mut lines: Vec<String> = config_content.lines().map(|s| s.to_string()).collect();

    // 查找并移除依赖
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with(&format!("{}:{}", dep_info.group_id, dep_info.artifact_id)) {
            lines.remove(i);
            println!("已从jx.toml中移除");
            break;
        }
        i += 1;
    }

    // 写回文件
    fs::write(&config_path, lines.join("\n"))?;
    Ok(())
}

fn remove_from_maven(project_dir: &Path, dep_info: &DependencyInfo) -> Result<()> {
    let pom_path = project_dir.join("pom.xml");
    let pom_content = fs::read_to_string(&pom_path)?;
    let mut lines: Vec<String> = pom_content.lines().map(|s| s.to_string()).collect();

    // 查找并移除依赖
    let mut i = 0;
    let mut in_dependency = false;
    let mut dependency_start = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line == "<dependency>" {
            in_dependency = true;
            dependency_start = i;
        } else if in_dependency && line == "</dependency>" {
            // 检查这个依赖是否匹配
            let dependency_lines = &lines[dependency_start..=i];
            if dependency_lines
                .iter()
                .any(|l| l.contains(&format!("<groupId>{}</groupId>", dep_info.group_id)))
                && dependency_lines.iter().any(|l| {
                    l.contains(&format!(
                        "<artifactId>{}</artifactId>",
                        dep_info.artifact_id
                    ))
                })
            {
                // 移除整个依赖块
                for _ in dependency_start..=i {
                    lines.remove(dependency_start);
                }
                println!("已从pom.xml中移除");
                break;
            }
            in_dependency = false;
        }

        i += 1;
    }

    // 写回文件
    fs::write(&pom_path, lines.join("\n"))?;
    Ok(())
}

fn remove_from_gradle(project_dir: &Path, dep_info: &DependencyInfo) -> Result<()> {
    let build_gradle_path = project_dir.join("build.gradle");
    let build_content = fs::read_to_string(&build_gradle_path)?;
    let mut lines: Vec<String> = build_content.lines().map(|s| s.to_string()).collect();

    // 查找并移除依赖行
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.contains(&format!("'{}:{}", dep_info.group_id, dep_info.artifact_id)) {
            lines.remove(i);
            println!("已从build.gradle中移除");
            break;
        }
        i += 1;
    }

    // 写回文件
    fs::write(&build_gradle_path, lines.join("\n"))?;
    Ok(())
}