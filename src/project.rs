use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub project_type: ProjectType,
    pub java_version: String,
    pub source_dirs: Vec<String>,
    pub test_dirs: Vec<String>,
    pub resource_dirs: Vec<String>,
    pub target_dir: String,
    pub main_class: Option<String>,
    pub test_class: Option<String>,
    pub dependencies: Vec<ProjectDependency>,
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProjectType {
    Maven,
    Gradle,
    Jx,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: DependencyScope,
    pub optional: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DependencyScope {
    Compile,
    Runtime,
    Test,
    Provided,
    System,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Project {
    pub fn new(name: &str, project_type: ProjectType) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: Some(format!("A Java project created with jx")),
            project_type,
            java_version: "11".to_string(),
            source_dirs: vec!["src/main/java".to_string()],
            test_dirs: vec!["src/test/java".to_string()],
            resource_dirs: vec!["src/main/resources".to_string()],
            target_dir: "target".to_string(),
            main_class: Some("com.example.Main".to_string()),
            test_class: Some("com.example.MainTest".to_string()),
            dependencies: Vec::new(),
            repositories: vec![Repository {
                name: "Maven Central".to_string(),
                url: "https://repo1.maven.org/maven2/".to_string(),
                username: None,
                password: None,
            }],
        }
    }

    pub fn from_directory(dir: &Path) -> Result<Self> {
        // 尝试从不同的配置文件加载项目信息
        let jx_config = dir.join("jx.toml");
        let pom_xml = dir.join("pom.xml");
        let build_gradle = dir.join("build.gradle");

        if jx_config.exists() {
            Self::from_jx_config(&jx_config)
        } else if pom_xml.exists() {
            Self::from_maven_pom(&pom_xml)
        } else if build_gradle.exists() {
            Self::from_gradle_build(&build_gradle)
        } else {
            Err(anyhow::anyhow!("找不到项目配置文件"))
        }
    }

    fn from_jx_config(config_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(config_path)?;
        let config: toml::Value = toml::from_str(&content)?;

        // 解析jx.toml配置
        let project = config
            .get("project")
            .ok_or_else(|| anyhow::anyhow!("缺少project部分"))?;

        let name = project
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = project
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();

        let project_type = project.get("type").and_then(|t| t.as_str()).unwrap_or("jx");

        let project_type = match project_type {
            "maven" => ProjectType::Maven,
            "gradle" => ProjectType::Gradle,
            _ => ProjectType::Jx,
        };

        Ok(Self::new(&name, project_type))
    }

    fn from_maven_pom(pom_path: &Path) -> Result<Self> {
        // 简单的XML解析
        let content = fs::read_to_string(pom_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut name = "unknown".to_string();
        let mut version = "1.0.0".to_string();

        for line in lines {
            let line = line.trim();
            if line.starts_with("<artifactId>") && line.ends_with("</artifactId>") {
                name = line[12..line.len() - 13].to_string();
            } else if line.starts_with("<version>") && line.ends_with("</version>") {
                version = line[9..line.len() - 10].to_string();
            }
        }

        Ok(Self::new(&name, ProjectType::Maven))
    }

    fn from_gradle_build(build_path: &Path) -> Result<Self> {
        // 简单的Gradle解析
        let content = fs::read_to_string(build_path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut name = "unknown".to_string();
        let mut version = "1.0.0".to_string();

        for line in lines {
            let line = line.trim();
            if line.starts_with("rootProject.name") {
                if let Some(quote_start) = line.find('\'') {
                    if let Some(quote_end) = line.rfind('\'') {
                        name = line[quote_start + 1..quote_end].to_string();
                    }
                }
            }
        }

        Ok(Self::new(&name, ProjectType::Gradle))
    }

    pub fn add_dependency(&mut self, dependency: ProjectDependency) {
        self.dependencies.push(dependency);
    }

    pub fn remove_dependency(&mut self, group_id: &str, artifact_id: &str) -> bool {
        let initial_len = self.dependencies.len();
        self.dependencies
            .retain(|d| !(d.group_id == group_id && d.artifact_id == artifact_id));
        self.dependencies.len() < initial_len
    }

    pub fn get_dependency(&self, group_id: &str, artifact_id: &str) -> Option<&ProjectDependency> {
        self.dependencies
            .iter()
            .find(|d| d.group_id == group_id && d.artifact_id == artifact_id)
    }

    pub fn has_dependency(&self, group_id: &str, artifact_id: &str) -> bool {
        self.get_dependency(group_id, artifact_id).is_some()
    }

    pub fn get_source_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for source_dir in &self.source_dirs {
            let dir_path = Path::new(source_dir);
            if dir_path.exists() {
                self.collect_java_files(dir_path, &mut files)?;
            }
        }

        Ok(files)
    }

    pub fn get_test_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for test_dir in &self.test_dirs {
            let dir_path = Path::new(test_dir);
            if dir_path.exists() {
                self.collect_java_files(dir_path, &mut files)?;
            }
        }

        Ok(files)
    }

    fn collect_java_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("java") {
                    files.push(path);
                } else if path.is_dir() {
                    self.collect_java_files(&path, files)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_classpath(&self) -> Vec<String> {
        let mut classpath = vec![self.target_dir.clone()];

        // 添加依赖的classpath
        for dep in &self.dependencies {
            let jar_name = format!("{}-{}.jar", dep.artifact_id, dep.version);
            classpath.push(format!("lib/{}", jar_name));
        }

        classpath
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("项目名称不能为空"));
        }

        if self.version.is_empty() {
            return Err(anyhow::anyhow!("项目版本不能为空"));
        }

        if self.source_dirs.is_empty() {
            return Err(anyhow::anyhow!("至少需要一个源码目录"));
        }

        Ok(())
    }
}

impl ProjectDependency {
    pub fn new(group_id: &str, artifact_id: &str, version: &str, scope: DependencyScope) -> Self {
        Self {
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: version.to_string(),
            scope,
            optional: false,
        }
    }

    pub fn coordinate(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}
