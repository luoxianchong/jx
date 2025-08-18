use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: DependencyScope,
    pub classifier: Option<String>,
    pub exclusions: Vec<Exclusion>,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyScope {
    Compile,
    Runtime,
    Test,
    Provided,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exclusion {
    pub group_id: String,
    pub artifact_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub dependency: Dependency,
    pub children: Vec<DependencyNode>,
    pub depth: usize,
}

impl Dependency {
    pub fn new(group_id: &str, artifact_id: &str, version: &str) -> Self {
        Self {
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: version.to_string(),
            scope: DependencyScope::Compile,
            classifier: None,
            exclusions: Vec::new(),
            optional: false,
        }
    }

    pub fn with_scope(mut self, scope: DependencyScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_classifier(mut self, classifier: &str) -> Self {
        self.classifier = Some(classifier.to_string());
        self
    }

    pub fn with_exclusions(mut self, exclusions: Vec<Exclusion>) -> Self {
        self.exclusions = exclusions;
        self
    }

    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

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

impl DependencyNode {
    pub fn new(dependency: Dependency, depth: usize) -> Self {
        Self {
            dependency,
            children: Vec::new(),
            depth,
        }
    }

    pub fn add_child(&mut self, child: DependencyNode) {
        self.children.push(child);
    }

    pub fn print_tree(&self) {
        let indent = "  ".repeat(self.depth);
        println!("{}{}", indent, self.dependency.coordinate());
        
        for child in &self.children {
            child.print_tree();
        }
    }
}

pub fn resolve_dependencies(dependencies: &[Dependency]) -> Result<Vec<DependencyNode>> {
    let mut resolved = Vec::new();
    let mut visited = HashMap::new();

    for dep in dependencies {
        if !visited.contains_key(&dep.coordinate()) {
            let node = build_dependency_tree(dep, dependencies, &mut visited, 0)?;
            resolved.push(node);
        }
    }

    Ok(resolved)
}

fn build_dependency_tree(
    dependency: &Dependency,
    all_dependencies: &[Dependency],
    visited: &mut HashMap<String, bool>,
    depth: usize,
) -> Result<DependencyNode> {
    visited.insert(dependency.coordinate(), true);
    
    let mut node = DependencyNode::new(dependency.clone(), depth);
    
    // TODO: 实现传递依赖解析
    // 这里应该查询Maven Central或其他仓库来获取传递依赖
    
    Ok(node)
}
