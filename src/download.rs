use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;

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

        let cache_path = format!(
            "{}/{}/{}/{}",
            self.cache_dir, group_id, artifact_id, filename
        );
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

        // 创建HTTP客户端
        let client = reqwest::Client::new();

        // 发送GET请求
        let response = client.get(&url).send().await.context("发送HTTP请求失败")?;

        // 检查响应状态
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP请求失败，状态码: {}",
                response.status()
            ));
        }

        // 获取文件大小
        let total_size = response
            .content_length()
            .ok_or_else(|| anyhow::anyhow!("无法获取文件大小"))?;

        // 创建进度条
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
            .map_err(|e| anyhow::anyhow!("设置进度条模板失败: {}", e))?
            .progress_chars("#>-"),
        );
        pb.set_message(format!("下载 {}", filename));

        // 创建文件
        let mut file = tokio::fs::File::create(&cache_path)
            .await
            .context("创建文件失败")?;

        // 下载并写入文件
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(item) = stream.next().await {
            let chunk = item.context("下载数据失败")?;
            file.write_all(&chunk).await.context("写入文件失败")?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        // 关闭文件
        file.flush().await.context("刷新文件缓冲区失败")?;

        // 完成进度条
        pb.finish_with_message(format!("下载完成 {}", filename));

        println!("下载完成");

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
