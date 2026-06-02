//! Module registry — tracks all registered modules.

use crate::module::{Module, ModuleMetadata};
use std::collections::HashMap;
use std::sync::RwLock;

/// Registry of application modules.
pub struct ModuleRegistry {
    modules: RwLock<HashMap<String, ModuleMetadata>>,
}

impl ModuleRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            modules: RwLock::new(HashMap::new()),
        }
    }

    /// Register a module.
    pub fn register<M: Module>(&self, module: &M) {
        let meta = ModuleMetadata::from_module(module);
        self.modules
            .write()
            .unwrap()
            .insert(meta.name.clone(), meta);
    }

    /// Get a module by name.
    pub fn get(&self, name: &str) -> Option<ModuleMetadata> {
        self.modules.read().unwrap().get(name).cloned()
    }

    /// List all registered module names.
    pub fn module_names(&self) -> Vec<String> {
        self.modules.read().unwrap().keys().cloned().collect()
    }

    /// List all module metadata.
    pub fn all_modules(&self) -> Vec<ModuleMetadata> {
        self.modules.read().unwrap().values().cloned().collect()
    }

    /// Number of registered modules.
    pub fn len(&self) -> usize {
        self.modules.read().unwrap().len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.modules.read().unwrap().is_empty()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CustomerModule;
    impl Module for CustomerModule {
        fn name(&self) -> &str {
            "customer"
        }
        fn description(&self) -> &'static str {
            "Customer management"
        }
    }

    struct OrderModule;
    impl Module for OrderModule {
        fn name(&self) -> &str {
            "order"
        }
        fn dependencies(&self) -> Vec<&str> {
            vec!["customer"]
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = ModuleRegistry::new();
        registry.register(&CustomerModule);
        registry.register(&OrderModule);

        assert_eq!(registry.len(), 2);
        assert!(registry.get("customer").is_some());
        assert!(registry.get("order").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_module_names() {
        let registry = ModuleRegistry::new();
        registry.register(&CustomerModule);
        registry.register(&OrderModule);

        let mut names = registry.module_names();
        names.sort();
        assert_eq!(names, vec!["customer", "order"]);
    }
}
