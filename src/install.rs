use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct Installer {
    lib_dir: String,
}

impl Installer {
    pub fn new() -> Self {
        let lib_dir = "lib".to_string();
        Self { lib_dir }
    }

    pub async fn install_dependencies(
        &self,
        dependencies: &[crate::dependency::Dependency],
    ) -> Result<()> {
        println!("📦 开始安装依赖...");

        // 创建lib目录
        fs::create_dir_all(&self.lib_dir)?;

        println!("正在安装 {} 个依赖...", dependencies.len());

        for (i, dep) in dependencies.iter().enumerate() {
            println!(
                "[{}/{}] 安装 {}",
                i + 1,
                dependencies.len(),
                dep.coordinate()
            );

            // 下载依赖
            let downloader = crate::download::Downloader::new();
            let cache_path = downloader
                .download_dependency(
                    &dep.group_id,
                    &dep.artifact_id,
                    &dep.version,
                    dep.classifier.as_deref(),
                )
                .await?;

            // 复制到lib目录
            let lib_path = format!("{}/{}", self.lib_dir, dep.filename());
            fs::copy(&cache_path, &lib_path)?;

            println!("✅ 已安装: {}", dep.coordinate());
        }

        println!("✅ 所有依赖安装完成!");

        Ok(())
    }

    pub fn get_installed_dependencies(&self) -> Result<Vec<String>> {
        if !Path::new(&self.lib_dir).exists() {
            return Ok(Vec::new());
        }

        let mut dependencies = Vec::new();
        let entries = fs::read_dir(&self.lib_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    dependencies.push(filename.to_string());
                }
            }
        }

        Ok(dependencies)
    }

    pub fn uninstall_dependency(&self, dependency_name: &str) -> Result<()> {
        let lib_path = format!("{}/{}", self.lib_dir, dependency_name);
        let path = Path::new(&lib_path);

        if path.exists() {
            fs::remove_file(path)?;
            println!("已卸载: {}", dependency_name);
        } else {
            println!("未找到依赖: {}", dependency_name);
        }

        Ok(())
    }

    pub fn clean_lib_directory(&self) -> Result<()> {
        if Path::new(&self.lib_dir).exists() {
            fs::remove_dir_all(&self.lib_dir)?;
            println!("lib目录已清理");
        }
        Ok(())
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self::new()
    }
}
