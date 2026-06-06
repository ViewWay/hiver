//! Module boundary verification — validates dependency graph.

use crate::registry::ModuleRegistry;

/// Result of module verification.
#[derive(Debug)]
pub struct VerificationResult
{
    /// Whether all verifications passed.
    pub valid: bool,
    /// Errors found during verification.
    pub errors: Vec<String>,
    /// Warnings found during verification.
    pub warnings: Vec<String>,
}

impl VerificationResult
{
    fn ok() -> Self
    {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Verify module dependency graph: no missing deps, no cycles.
pub fn verify_modules(registry: &ModuleRegistry) -> VerificationResult
{
    let modules = registry.all_modules();
    let names: std::collections::HashSet<&str> = modules.iter().map(|m| m.name.as_str()).collect();
    let mut result = VerificationResult::ok();

    // 1. Check that all dependencies exist
    for module in &modules
    {
        for dep in &module.dependencies
        {
            if !names.contains(dep.as_str())
            {
                result.valid = false;
                result.errors.push(format!(
                    "Module '{}' depends on '{}', which is not registered",
                    module.name, dep
                ));
            }
        }
    }

    // 2. Check for circular dependencies using DFS
    let module_map: std::collections::HashMap<String, Vec<String>> = modules
        .iter()
        .map(|m| (m.name.clone(), m.dependencies.clone()))
        .collect();

    let mut visited = std::collections::HashSet::<String>::new();
    let mut in_stack = std::collections::HashSet::<String>::new();

    for module in &modules
    {
        if let Some(cycle) =
            detect_cycle(&module.name, &module_map, &mut visited, &mut in_stack, &mut Vec::new())
        {
            result.valid = false;
            result
                .errors
                .push(format!("Circular dependency detected: {}", cycle.join(" -> ")));
        }
    }

    // 3. Warn about self-dependencies
    for module in &modules
    {
        if module.dependencies.contains(&module.name)
        {
            result
                .warnings
                .push(format!("Module '{}' has a self-dependency", module.name));
        }
    }

    result
}

fn detect_cycle(
    node: &str,
    graph: &std::collections::HashMap<String, Vec<String>>,
    visited: &mut std::collections::HashSet<String>,
    in_stack: &mut std::collections::HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>>
{
    if in_stack.contains(node)
    {
        let cycle_start = path.iter().position(|n| n == node).unwrap_or(0);
        let mut cycle: Vec<String> = path[cycle_start..].to_vec();
        cycle.push(node.to_string());
        return Some(cycle);
    }

    if visited.contains(node)
    {
        return None;
    }

    visited.insert(node.to_string());
    in_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(deps) = graph.get(node)
    {
        for dep in deps
        {
            if let Some(cycle) = detect_cycle(dep, graph, visited, in_stack, path)
            {
                return Some(cycle);
            }
        }
    }

    path.pop();
    in_stack.remove(node);
    None
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::module::Module;

    struct ModA;
    impl Module for ModA
    {
        fn name(&self) -> &'static str
        {
            "a"
        }
    }

    struct ModB;
    impl Module for ModB
    {
        fn name(&self) -> &'static str
        {
            "b"
        }

        fn dependencies(&self) -> Vec<&str>
        {
            vec!["a"]
        }
    }

    struct ModC;
    impl Module for ModC
    {
        fn name(&self) -> &'static str
        {
            "c"
        }

        fn dependencies(&self) -> Vec<&str>
        {
            vec!["b"]
        }
    }

    #[test]
    fn test_valid_modules()
    {
        let registry = ModuleRegistry::new();
        registry.register(&ModA);
        registry.register(&ModB);
        registry.register(&ModC);

        let result = verify_modules(&registry);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_missing_dependency()
    {
        let registry = ModuleRegistry::new();
        registry.register(&ModB);

        let result = verify_modules(&registry);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("not registered")));
    }

    struct ModX;
    impl Module for ModX
    {
        fn name(&self) -> &'static str
        {
            "x"
        }

        fn dependencies(&self) -> Vec<&str>
        {
            vec!["y"]
        }
    }

    struct ModY;
    impl Module for ModY
    {
        fn name(&self) -> &'static str
        {
            "y"
        }

        fn dependencies(&self) -> Vec<&str>
        {
            vec!["x"]
        }
    }

    #[test]
    fn test_circular_dependency()
    {
        let registry = ModuleRegistry::new();
        registry.register(&ModX);
        registry.register(&ModY);

        let result = verify_modules(&registry);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Circular")));
    }

    #[test]
    fn test_empty_registry()
    {
        let registry = ModuleRegistry::new();
        let result = verify_modules(&registry);
        assert!(result.valid);
    }
}
