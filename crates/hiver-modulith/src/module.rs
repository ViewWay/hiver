//! Module definition trait and metadata.
//! 模块定义 trait 和元数据。

use std::any::TypeId;

/// A module in the modular monolith.
/// 模块化单体中的模块。
pub trait Module: Send + Sync + 'static
{
    /// Module name (must be unique across the application).
    fn name(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> &'static str
    {
        ""
    }

    /// Names of modules this module depends on.
    fn dependencies(&self) -> Vec<&str>
    {
        Vec::new()
    }

    /// Packages/namespaces this module owns (for boundary verification).
    fn packages(&self) -> Vec<&str>
    {
        vec![self.name()]
    }
}

/// Static metadata about a registered module.
#[derive(Debug, Clone)]
pub struct ModuleMetadata
{
    /// Module name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Dependencies.
    pub dependencies: Vec<String>,
    /// Owned packages.
    pub packages: Vec<String>,
    /// Runtime type ID.
    pub type_id: TypeId,
}

impl ModuleMetadata
{
    /// Build metadata from a Module trait object.
    pub fn from_module<M: Module>(module: &M) -> Self
    {
        Self {
            name: module.name().to_string(),
            description: module.description().to_string(),
            dependencies: module
                .dependencies()
                .iter()
                .map(ToString::to_string)
                .collect(),
            packages: module.packages().iter().map(ToString::to_string).collect(),
            type_id: TypeId::of::<M>(),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    struct OrderModule;
    impl Module for OrderModule
    {
        fn name(&self) -> &'static str
        {
            "order"
        }

        fn description(&self) -> &'static str
        {
            "Order management"
        }

        fn dependencies(&self) -> Vec<&str>
        {
            vec!["customer", "product"]
        }

        fn packages(&self) -> Vec<&str>
        {
            vec!["order", "order.item"]
        }
    }

    #[test]
    fn test_module_metadata()
    {
        let m = OrderModule;
        let meta = ModuleMetadata::from_module(&m);
        assert_eq!(meta.name, "order");
        assert_eq!(meta.description, "Order management");
        assert_eq!(meta.dependencies, vec!["customer", "product"]);
        assert_eq!(meta.packages, vec!["order", "order.item"]);
    }

    #[test]
    fn test_default_dependencies()
    {
        struct SimpleMod;
        impl Module for SimpleMod
        {
            fn name(&self) -> &'static str
            {
                "simple"
            }
        }
        let m = SimpleMod;
        assert!(m.dependencies().is_empty());
    }
}
