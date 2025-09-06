use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    pub version: String,
    pub dependencies: HashMap<String, LockedDependency>,
    pub metadata: LockMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub classifier: Option<String>,
    pub scope: String,
    pub checksum: String,
    pub url: String,
    pub dependencies: Vec<String>, // 传递依赖的坐标
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub total_dependencies: usize,
    pub total_size: u64,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            dependencies: HashMap::new(),
            metadata: LockMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                total_dependencies: 0,
                total_size: 0,
            },
        }
    }

    pub fn add_dependency(&mut self, dep: LockedDependency) {
        let key = format!("{}:{}:{}", dep.group_id, dep.artifact_id, dep.version);
        self.dependencies.insert(key, dep);
        self.metadata.total_dependencies = self.dependencies.len();
        self.metadata.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn remove_dependency(&mut self, group_id: &str, artifact_id: &str, version: &str) -> bool {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        let removed = self.dependencies.remove(&key).is_some();
        if removed {
            self.metadata.total_dependencies = self.dependencies.len();
            self.metadata.updated_at = chrono::Utc::now().to_rfc3339();
        }
        removed
    }

    pub fn get_dependency(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Option<&LockedDependency> {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.dependencies.get(&key)
    }

    pub fn has_dependency(&self, group_id: &str, artifact_id: &str, version: &str) -> bool {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        self.dependencies.contains_key(&key)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let lock_file: LockFile = toml::from_str(&content)?;
            Ok(lock_file)
        } else {
            Ok(LockFile::new())
        }
    }

    pub fn update_checksum(
        &mut self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        checksum: &str,
    ) -> Result<()> {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        if let Some(dep) = self.dependencies.get_mut(&key) {
            dep.checksum = checksum.to_string();
            self.metadata.updated_at = chrono::Utc::now().to_rfc3339();
        }
        Ok(())
    }

    pub fn update_url(
        &mut self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        url: &str,
    ) -> Result<()> {
        let key = format!("{}:{}:{}", group_id, artifact_id, version);
        if let Some(dep) = self.dependencies.get_mut(&key) {
            dep.url = url.to_string();
            self.metadata.updated_at = chrono::Utc::now().to_rfc3339();
        }
        Ok(())
    }

    pub fn get_dependency_tree(&self) -> Vec<DependencyTreeNode> {
        let mut tree = Vec::new();
        let mut visited = HashMap::new();

        for (key, dep) in &self.dependencies {
            if !visited.contains_key(key) {
                let node = self.build_tree_node(dep, &mut visited, 0);
                tree.push(node);
            }
        }

        tree
    }

    fn build_tree_node(
        &self,
        dep: &LockedDependency,
        visited: &mut HashMap<String, bool>,
        depth: usize,
    ) -> DependencyTreeNode {
        visited.insert(dep.coordinate(), true);

        let mut node = DependencyTreeNode {
            dependency: dep.clone(),
            children: Vec::new(),
            depth,
        };

        // 添加传递依赖
        for dep_coord in &dep.dependencies {
            if let Some(child_dep) = self.dependencies.get(dep_coord) {
                if !visited.contains_key(dep_coord) {
                    let child_node = self.build_tree_node(child_dep, visited, depth + 1);
                    node.children.push(child_node);
                }
            }
        }

        node
    }
}

#[derive(Debug, Clone)]
pub struct DependencyTreeNode {
    pub dependency: LockedDependency,
    pub children: Vec<DependencyTreeNode>,
    pub depth: usize,
}

impl LockedDependency {
    pub fn coordinate(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }

    pub fn filename(&self) -> String {
        let mut filename = format!("{}-{}.jar", self.artifact_id, self.version);
        if let Some(ref classifier) = self.classifier {
            filename = format!("{}-{}-{}.jar", self.artifact_id, self.version, classifier);
        }
        filename
    }
}

impl DependencyTreeNode {
    pub fn print_tree(&self) {
        let indent = "  ".repeat(self.depth);
        println!("{}{}", indent, self.dependency.coordinate());

        for child in &self.children {
            child.print_tree();
        }
    }
}
