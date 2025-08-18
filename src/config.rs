use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct JxConfig {
    pub project: ProjectConfig,
    pub build: BuildConfig,
    pub dependencies: DependenciesConfig,
    pub repositories: RepositoriesConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildConfig {
    pub source_dir: Option<String>,
    pub target_dir: Option<String>,
    pub main_class: Option<String>,
    pub test_class: Option<String>,
    pub java_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependenciesConfig {
    pub compile: Option<Vec<Dependency>>,
    pub runtime: Option<Vec<Dependency>>,
    pub test: Option<Vec<Dependency>>,
    pub provided: Option<Vec<Dependency>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: Option<String>,
    pub classifier: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoriesConfig {
    pub maven_central: Option<String>,
    pub jcenter: Option<String>,
    pub custom: Option<Vec<CustomRepository>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomRepository {
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for JxConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "my-java-project".to_string(),
                version: "1.0.0".to_string(),
                description: Some("A Java project created with jx".to_string()),
                authors: Some(vec!["jx team".to_string()]),
                license: Some("MIT".to_string()),
                homepage: None,
                repository: None,
            },
            build: BuildConfig {
                source_dir: Some("src/main/java".to_string()),
                target_dir: Some("target".to_string()),
                main_class: Some("com.example.Main".to_string()),
                test_class: Some("com.example.MainTest".to_string()),
                java_version: Some("11".to_string()),
            },
            dependencies: DependenciesConfig {
                compile: Some(vec![]),
                runtime: Some(vec![]),
                test: Some(vec![]),
                provided: Some(vec![]),
            },
            repositories: RepositoriesConfig {
                maven_central: Some("https://repo1.maven.org/maven2/".to_string()),
                jcenter: Some("https://jcenter.bintray.com/".to_string()),
                custom: Some(vec![]),
            },
        }
    }
}

pub fn load_config(config_path: &PathBuf) -> Result<JxConfig> {
    if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        let config: JxConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(JxConfig::default())
    }
}

pub fn save_config(config: &JxConfig, config_path: &PathBuf) -> Result<()> {
    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

pub fn get_config_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("jx.toml");
    path
}
