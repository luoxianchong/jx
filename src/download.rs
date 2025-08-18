use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct Downloader {
    cache_dir: String,
}

impl Downloader {
    pub fn new() -> Self {
        let cache_dir = format!("{}/.jx/cache", dirs::home_dir().unwrap().display());
        Self { cache_dir }
    }

    pub async fn download_dependency(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        classifier: Option<&str>,
    ) -> Result<String> {
        // 创建缓存目录
        fs::create_dir_all(&self.cache_dir)?;

        // 构建文件名
        let filename = if let Some(c) = classifier {
            format!("{}-{}-{}.jar", artifact_id, version, c)
        } else {
            format!("{}-{}.jar", artifact_id, version)
        };

        let cache_path = format!("{}/{}/{}/{}", self.cache_dir, group_id, artifact_id, filename);
        let cache_file = Path::new(&cache_path);

        // 检查缓存
        if cache_file.exists() {
            println!("从缓存加载: {}", filename);
            return Ok(cache_path);
        }

        // 创建目录
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // 从Maven Central下载
        let url = self.build_maven_central_url(group_id, artifact_id, version, classifier);
        println!("下载: {}", url);

        println!("正在下载...");

        // 使用curl下载
        let output = Command::new("curl")
            .args(&["-L", "-o", &cache_path, &url])
            .output()
            .context("执行curl命令失败")?;

        println!("下载完成");

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("下载失败: {}", error));
        }

        Ok(cache_path)
    }

    fn build_maven_central_url(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        classifier: Option<&str>,
    ) -> String {
        let group_path = group_id.replace('.', "/");
        let mut url = format!(
            "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}",
            group_path, artifact_id, version, artifact_id, version
        );

        if let Some(c) = classifier {
            url.push_str(&format!("-{}", c));
        }

        url.push_str(".jar");
        url
    }

    pub fn clear_cache(&self) -> Result<()> {
        if Path::new(&self.cache_dir).exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            println!("缓存已清理");
        }
        Ok(())
    }

    pub fn get_cache_size(&self) -> Result<u64> {
        if !Path::new(&self.cache_dir).exists() {
            return Ok(0);
        }

        let mut total_size = 0;
        self.calculate_dir_size(&self.cache_dir, &mut total_size)?;
        Ok(total_size)
    }

    fn calculate_dir_size(&self, dir_path: &str, total_size: &mut u64) -> Result<()> {
        let entries = fs::read_dir(dir_path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    *total_size += metadata.len();
                }
            } else if path.is_dir() {
                self.calculate_dir_size(path.to_str().unwrap(), total_size)?;
            }
        }
        
        Ok(())
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}
