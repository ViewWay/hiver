//! Module dependency graph — detect cycles, topological ordering.
//! 模块依赖图 — 检测循环依赖、拓扑排序。

use std::collections::{HashMap, HashSet, VecDeque};

/// A declared dependency between two modules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleDependency
{
    /// Source module (depends on target).
    pub source: String,
    /// Target module (depended upon).
    pub target: String,
}

impl ModuleDependency
{
    /// Create a new dependency: `source` depends on `target`.
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self
    {
        Self { source: source.into(), target: target.into() }
    }
}

/// Module dependency graph — tracks and validates inter-module dependencies.
pub struct DependencyGraph
{
    /// Adjacency list: module → set of modules it depends on.
    dependencies: HashMap<String, HashSet<String>>,
}

impl DependencyGraph
{
    /// Create a new empty dependency graph.
    pub fn new() -> Self
    {
        Self { dependencies: HashMap::new() }
    }

    /// Add a dependency: `source` depends on `target`.
    pub fn add_dependency(&mut self, dep: ModuleDependency)
    {
        self.dependencies
            .entry(dep.source.clone())
            .or_default()
            .insert(dep.target.clone());
        self.dependencies.entry(dep.target).or_default();
    }

    /// Get all modules in the graph.
    pub fn modules(&self) -> Vec<&str>
    {
        self.dependencies.keys().map(String::as_str).collect()
    }

    /// Get direct dependencies of a module.
    pub fn dependencies_of(&self, module: &str) -> Vec<&str>
    {
        self.dependencies
            .get(module)
            .map(|deps| deps.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Detect circular dependencies using DFS.
    pub fn detect_cycles(&self) -> Vec<Vec<String>>
    {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut path_set = HashSet::new();

        for module in self.dependencies.keys()
        {
            self.dfs_cycles(module, &mut visited, &mut path, &mut path_set, &mut cycles);
        }
        cycles
    }

    /// Check if the graph is acyclic (no circular dependencies).
    pub fn is_acyclic(&self) -> bool
    {
        self.detect_cycles().is_empty()
    }

    /// Topological sort — returns modules in dependency order.
    /// Returns `None` if the graph has cycles.
    pub fn topological_sort(&self) -> Option<Vec<String>>
    {
        if !self.is_acyclic()
        {
            return None;
        }

        // Build reverse adjacency and in-degrees.
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_deg: HashMap<String, usize> = HashMap::new();

        for module in self.dependencies.keys()
        {
            in_deg.insert(module.clone(), 0);
        }

        for (source, targets) in &self.dependencies
        {
            in_deg.insert(source.clone(), targets.len());
            for target in targets
            {
                dependents.entry(target.clone()).or_default().push(source.clone());
            }
        }

        let mut queue: VecDeque<String> = in_deg
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(m, _)| m.clone())
            .collect();

        let mut result = Vec::new();
        while let Some(module) = queue.pop_front()
        {
            result.push(module.clone());
            if let Some(deps_of) = dependents.get(&module)
            {
                for dependent in deps_of
                {
                    let deg = in_deg.get_mut(dependent).unwrap();
                    *deg -= 1;
                    if *deg == 0
                    {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        if result.len() == self.dependencies.len()
        {
            Some(result)
        }
        else
        {
            None
        }
    }

    fn dfs_cycles(
        &self,
        module: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        path_set: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
    )
    {
        if visited.contains(module)
        {
            return;
        }

        path.push(module.to_string());
        path_set.insert(module.to_string());

        if let Some(deps) = self.dependencies.get(module)
        {
            for dep in deps
            {
                if path_set.contains(dep.as_str())
                {
                    let start = path.iter().position(|m| m == dep).unwrap();
                    let mut cycle: Vec<String> = path[start..].to_vec();
                    cycle.push(dep.clone());
                    cycles.push(cycle);
                }
                else if !visited.contains(dep)
                {
                    self.dfs_cycles(dep, visited, path, path_set, cycles);
                }
            }
        }

        path.pop();
        path_set.remove(module);
        visited.insert(module.to_string());
    }
}

impl Default for DependencyGraph
{
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_empty_graph_is_acyclic()
    {
        assert!(DependencyGraph::new().is_acyclic());
    }

    #[test]
    fn test_simple_chain()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("app", "user"));
        graph.add_dependency(ModuleDependency::new("user", "common"));

        assert!(graph.is_acyclic());
        let order = graph.topological_sort().unwrap();
        assert_eq!(order.len(), 3);
        let common_pos = order.iter().position(|m| m == "common").unwrap();
        let user_pos = order.iter().position(|m| m == "user").unwrap();
        let app_pos = order.iter().position(|m| m == "app").unwrap();
        assert!(common_pos < user_pos);
        assert!(user_pos < app_pos);
    }

    #[test]
    fn test_circular_dependency()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("a", "b"));
        graph.add_dependency(ModuleDependency::new("b", "c"));
        graph.add_dependency(ModuleDependency::new("c", "a"));

        assert!(!graph.is_acyclic());
        assert!(graph.topological_sort().is_none());
        assert!(!graph.detect_cycles().is_empty());
    }

    #[test]
    fn test_diamond_dependency()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("app", "user"));
        graph.add_dependency(ModuleDependency::new("app", "order"));
        graph.add_dependency(ModuleDependency::new("user", "common"));
        graph.add_dependency(ModuleDependency::new("order", "common"));

        assert!(graph.is_acyclic());
        let order = graph.topological_sort().unwrap();
        assert_eq!(order.len(), 4);
        assert!(order.iter().position(|m| m == "common").unwrap() < order.iter().position(|m| m == "user").unwrap());
        assert!(order.iter().position(|m| m == "common").unwrap() < order.iter().position(|m| m == "order").unwrap());
        assert!(order.iter().position(|m| m == "user").unwrap() < order.iter().position(|m| m == "app").unwrap());
    }

    #[test]
    fn test_dependencies_of()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("app", "user"));
        graph.add_dependency(ModuleDependency::new("app", "order"));
        assert_eq!(graph.dependencies_of("app").len(), 2);
        assert_eq!(graph.dependencies_of("user").len(), 0);
    }

    #[test]
    fn test_modules_list()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("a", "b"));
        assert_eq!(graph.modules().len(), 2);
    }
}
