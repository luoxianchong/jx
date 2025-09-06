use crate::dependency::Dependency;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

pub struct DependencyResolver {
    resolved: HashMap<String, Dependency>,
    unresolved: HashSet<String>,
    in_progress: HashSet<String>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            resolved: HashMap::new(),
            unresolved: HashSet::new(),
            in_progress: HashSet::new(),
        }
    }

    pub async fn resolve_dependencies(
        &mut self,
        dependencies: &[Dependency],
    ) -> Result<Vec<Dependency>> {
        let mut resolved_deps = Vec::new();

        for dep in dependencies {
            let resolved = self.resolve_dependency(dep).await?;
            resolved_deps.extend(resolved);
        }

        Ok(resolved_deps)
    }

    async fn resolve_dependency(&mut self, dependency: &Dependency) -> Result<Vec<Dependency>> {
        let key = dependency.coordinate();

        // 检查是否已经解析过
        if let Some(resolved) = self.resolved.get(&key) {
            return Ok(vec![resolved.clone()]);
        }

        // 检查是否正在解析中（循环依赖检测）
        if self.in_progress.contains(&key) {
            return Err(anyhow::anyhow!("检测到循环依赖: {}", key));
        }

        // 标记为正在解析
        self.in_progress.insert(key.clone());

        let mut resolved_deps = vec![dependency.clone()];

        // 解析传递依赖
        let transitive_deps = self.resolve_transitive_dependencies(dependency).await?;
        resolved_deps.extend(transitive_deps);

        // 标记为已解析
        self.resolved.insert(key.clone(), dependency.clone());
        self.in_progress.remove(&key);

        Ok(resolved_deps)
    }

    async fn resolve_transitive_dependencies(
        &self,
        dependency: &Dependency,
    ) -> Result<Vec<Dependency>> {
        // TODO: 实现传递依赖解析
        // 这里应该查询Maven Central或其他仓库来获取传递依赖信息

        // 临时返回空向量
        Ok(Vec::new())
    }

    pub fn get_resolution_order(&self) -> Vec<String> {
        // 使用拓扑排序确定依赖解析顺序
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        for key in self.resolved.keys() {
            if !visited.contains(key) {
                if let Err(e) =
                    self.topological_sort(key, &mut visited, &mut temp_visited, &mut order)
                {
                    eprintln!("警告: 依赖排序失败: {}", e);
                    // 继续处理其他依赖
                }
            }
        }

        order.reverse();
        order
    }

    fn topological_sort(
        &self,
        key: &str,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if temp_visited.contains(key) {
            return Err(anyhow::anyhow!("检测到循环依赖: {}", key));
        }

        if visited.contains(key) {
            return Ok(());
        }

        temp_visited.insert(key.to_string());

        // TODO: 获取依赖的传递依赖
        // let transitive = self.get_transitive_dependencies(key)?;
        // for dep_key in transitive {
        //     self.topological_sort(&dep_key, visited, temp_visited, order)?;
        // }

        temp_visited.remove(key);
        visited.insert(key.to_string());
        order.push(key.to_string());

        Ok(())
    }

    pub fn detect_conflicts(&self) -> Vec<DependencyConflict> {
        let mut conflicts = Vec::new();
        let mut version_map: HashMap<String, HashMap<String, String>> = HashMap::new();

        for dep in self.resolved.values() {
            let key = format!("{}:{}", dep.group_id, dep.artifact_id);
            let versions = version_map.entry(key.clone()).or_insert_with(HashMap::new);

            if let Some(existing_version) = versions.get(&dep.version) {
                if existing_version != &dep.version {
                    conflicts.push(DependencyConflict {
                        group_id: dep.group_id.clone(),
                        artifact_id: dep.artifact_id.clone(),
                        versions: vec![existing_version.clone(), dep.version.clone()],
                        conflict_type: ConflictType::VersionConflict,
                    });
                }
            } else {
                versions.insert(dep.version.clone(), dep.version.clone());
            }
        }

        conflicts
    }

    pub fn get_dependency_tree(&self) -> Vec<DependencyTreeNode> {
        let mut tree = Vec::new();
        let mut visited = HashSet::new();

        for dep in self.resolved.values() {
            let key = dep.coordinate();
            if !visited.contains(&key) {
                let node = self.build_tree_node(dep, &mut visited, 0);
                tree.push(node);
            }
        }

        tree
    }

    fn build_tree_node(
        &self,
        dep: &Dependency,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> DependencyTreeNode {
        visited.insert(dep.coordinate());

        let mut node = DependencyTreeNode {
            dependency: dep.clone(),
            children: Vec::new(),
            depth,
        };

        // TODO: 添加传递依赖节点
        // let transitive = self.get_transitive_dependencies(&dep.coordinate())?;
        // for dep_key in transitive {
        //     if let Some(child_dep) = self.resolved.get(&dep_key) {
        //         if !visited.contains(&dep_key) {
        //             let child_node = self.build_tree_node(child_dep, visited, depth + 1);
        //             node.children.push(child_node);
        //         }
        //     }
        // }

        node
    }

    pub fn clear(&mut self) {
        self.resolved.clear();
        self.unresolved.clear();
        self.in_progress.clear();
    }
}

#[derive(Debug)]
pub struct DependencyConflict {
    pub group_id: String,
    pub artifact_id: String,
    pub versions: Vec<String>,
    pub conflict_type: ConflictType,
}

#[derive(Debug)]
pub enum ConflictType {
    VersionConflict,
    ScopeConflict,
    OptionalConflict,
}

#[derive(Debug)]
pub struct DependencyTreeNode {
    pub dependency: Dependency,
    pub children: Vec<DependencyTreeNode>,
    pub depth: usize,
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

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}
