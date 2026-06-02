//! 条件评估器 / Condition Evaluator
//!
//! 提供集中式的条件评估能力，将 `AutoConfigurationRegistry` 和
//! `ApplicationContext` 结合，评估所有自动配置条目的条件并返回适用列表。
//! Provides centralized condition evaluation, combining `AutoConfigurationRegistry`
//! and `ApplicationContext` to evaluate all auto-configuration entries and return
//! the applicable list.
//!
//! 参考 Spring Boot 的 `ConditionEvaluator` 和 `AutoConfigurationImportSelector`。
//! Based on Spring Boot's `ConditionEvaluator` and `AutoConfigurationImportSelector`.
//!
//! # 功能 / Features
//!
//! - 集中评估所有注册的自动配置条件
//! - 返回带有名称和优先级的适用配置列表
//! - 支持排除过滤
//!
//! # 示例 / Example
//!
//! ```rust,ignore
//! use hiver_starter::core::autoconfigure::AutoConfigurationRegistry;
//! use hiver_starter::core::condition_evaluator::ConditionEvaluator;
//!
//! let registry = AutoConfigurationRegistry::new();
//! let ctx = ApplicationContext::new();
//!
//! let evaluator = ConditionEvaluator::new(&registry);
//! let applicable = evaluator.evaluate(&ctx);
//!
//! for config in &applicable {
//!     println!("Applying: {} (priority: {})", config.name, config.priority);
//! }
//! ```

use std::collections::HashSet;
use std::fmt;

use super::autoconfigure::{AutoConfigurationEntry, AutoConfigurationRegistry};
use super::container::ApplicationContext;

// ============================================================================
// ApplicableConfig / 适用配置
// ============================================================================

/// 适用配置
/// Applicable configuration
///
/// 描述一个通过条件评估的自动配置条目。
/// Describes an auto-configuration entry that passed condition evaluation.
///
/// 包含配置的名称和优先级信息，用于排序和执行。
/// Contains the configuration's name and priority for sorting and execution.
#[derive(Debug, Clone)]
pub struct ApplicableConfig {
    /// 配置名称
    /// Configuration name
    pub name: String,

    /// 配置优先级
    /// Configuration priority
    pub priority: i32,
}

impl ApplicableConfig {
    /// 创建新的适用配置
    /// Create a new applicable configuration
    ///
    /// # 参数 / Parameters
    ///
    /// - `name`: 配置名称 / Configuration name
    /// - `priority`: 配置优先级 / Configuration priority
    pub fn new(name: impl Into<String>, priority: i32) -> Self {
        Self {
            name: name.into(),
            priority,
        }
    }
}

impl fmt::Display for ApplicableConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApplicableConfig[name={}, priority={}]", self.name, self.priority)
    }
}

impl PartialEq for ApplicableConfig {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ApplicableConfig {}

impl std::hash::Hash for ApplicableConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

// ============================================================================
// ConditionEvaluator / 条件评估器
// ============================================================================

/// 条件评估器
/// Condition evaluator
///
/// 集中评估 `AutoConfigurationRegistry` 中所有条目的条件。
/// Centrally evaluates conditions of all entries in an `AutoConfigurationRegistry`.
///
/// 参考 Spring Boot 的 `ConditionEvaluator`。
/// Based on Spring Boot's `ConditionEvaluator`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::autoconfigure::AutoConfigurationRegistry;
/// use hiver_starter::core::condition_evaluator::ConditionEvaluator;
///
/// let registry = AutoConfigurationRegistry::new();
/// // ... 注册条目 ...
///
/// let evaluator = ConditionEvaluator::new(&registry);
///
/// // 评估所有条件
/// let applicable = evaluator.evaluate(&ctx);
///
/// // 排除特定配置
/// let mut evaluator = ConditionEvaluator::new(&registry);
/// evaluator.exclude("ExcludedConfig");
/// let filtered = evaluator.evaluate(&ctx);
/// ```
pub struct ConditionEvaluator<'a> {
    /// 自动配置注册表引用
    /// Reference to the auto-configuration registry
    registry: &'a AutoConfigurationRegistry,

    /// 排除的配置名称集合
    /// Set of excluded configuration names
    excluded: HashSet<String>,
}

impl<'a> ConditionEvaluator<'a> {
    /// 创建新的条件评估器
    /// Create a new condition evaluator
    ///
    /// # 参数 / Parameters
    ///
    /// - `registry`: 自动配置注册表 / Auto-configuration registry
    pub fn new(registry: &'a AutoConfigurationRegistry) -> Self {
        Self {
            registry,
            excluded: HashSet::new(),
        }
    }

    /// 排除指定名称的配置
    /// Exclude a configuration by name
    ///
    /// # 参数 / Parameters
    ///
    /// - `name`: 要排除的配置名称 / Name of the configuration to exclude
    pub fn exclude(mut self, name: impl Into<String>) -> Self {
        self.excluded.insert(name.into());
        self
    }

    /// 排除多个配置
    /// Exclude multiple configurations
    ///
    /// # 参数 / Parameters
    ///
    /// - `names`: 要排除的配置名称列表 / List of configuration names to exclude
    pub fn exclude_many(mut self, names: &[&str]) -> Self {
        for name in names {
            self.excluded.insert((*name).to_string());
        }
        self
    }

    /// 评估所有条件并返回适用的配置列表
    /// Evaluate all conditions and return the list of applicable configurations
    ///
    /// 遍历注册表中的所有条目，检查每个条目的条件是否满足，
    /// 排除被排除的条目，然后按优先级排序返回。
    /// Iterates over all entries in the registry, checks each entry's condition,
    /// excludes excluded entries, then returns them sorted by priority.
    ///
    /// # 参数 / Parameters
    ///
    /// - `ctx`: 应用上下文 / Application context
    ///
    /// # 返回 / Returns
    ///
    /// 返回通过条件评估且未被排除的配置列表（按优先级排序）。
    /// Returns configurations that passed condition evaluation and were not excluded
    /// (sorted by priority).
    pub fn evaluate(&self, ctx: &ApplicationContext) -> Vec<ApplicableConfig> {
        self.registry
            .entries()
            .iter()
            .filter(|entry| !self.excluded.contains(entry.name()))
            .filter(|entry| entry.matches(ctx))
            .map(|entry| ApplicableConfig::new(entry.name(), entry.priority()))
            .collect()
    }

    /// 评估所有条件并返回适用的条目引用列表
    /// Evaluate all conditions and return the list of applicable entry references
    ///
    /// 与 `evaluate` 类似，但返回条目的引用而非 `ApplicableConfig`。
    /// Similar to `evaluate`, but returns entry references instead of `ApplicableConfig`.
    ///
    /// # 参数 / Parameters
    ///
    /// - `ctx`: 应用上下文 / Application context
    pub fn evaluate_entries(&self, ctx: &ApplicationContext) -> Vec<&AutoConfigurationEntry> {
        self.registry
            .entries()
            .iter()
            .filter(|entry| !self.excluded.contains(entry.name()))
            .filter(|entry| entry.matches(ctx))
            .collect()
    }

    /// 获取被排除的配置数量
    /// Get the number of excluded configurations
    pub fn excluded_count(&self) -> usize {
        self.excluded.len()
    }

    /// 检查指定配置是否被排除
    /// Check if a specific configuration is excluded
    pub fn is_excluded(&self, name: &str) -> bool {
        self.excluded.contains(name)
    }

    /// 获取注册表中未被排除的总条目数
    /// Get total number of entries in the registry (before evaluation)
    pub fn total_entries(&self) -> usize {
        self.registry.len()
    }
}

// ============================================================================
// 便捷函数 / Convenience Functions
// ============================================================================

/// 便捷评估函数：评估注册表中的所有条件
/// Convenience evaluation function: evaluate all conditions in a registry
///
/// 这是最简单的使用方式，不需要创建 `ConditionEvaluator` 实例。
/// This is the simplest usage, no need to create a `ConditionEvaluator` instance.
///
/// # 参数 / Parameters
///
/// - `registry`: 自动配置注册表 / Auto-configuration registry
/// - `ctx`: 应用上下文 / Application context
///
/// # 返回 / Returns
///
/// 返回通过条件评估的配置列表（按优先级排序）。
/// Returns configurations that passed condition evaluation (sorted by priority).
///
/// # 示例 / Example
///
/// ```rust,ignore
/// let applicable = evaluate_conditions(&registry, &ctx);
/// for config in &applicable {
///     println!("Applying: {}", config.name);
/// }
/// ```
pub fn evaluate_conditions(
    registry: &AutoConfigurationRegistry,
    ctx: &ApplicationContext,
) -> Vec<ApplicableConfig> {
    let evaluator = ConditionEvaluator::new(registry);
    evaluator.evaluate(ctx)
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::autoconfigure::AutoConfigurationEntry;

    fn noop_factory(_ctx: &mut ApplicationContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn create_test_registry() -> AutoConfigurationRegistry {
        let mut registry = AutoConfigurationRegistry::new();
        registry
            .register(AutoConfigurationEntry::new("CoreConfig", noop_factory).with_priority(-100));
        registry.register(AutoConfigurationEntry::new("WebConfig", noop_factory).with_priority(0));
        registry
            .register(AutoConfigurationEntry::new("DataConfig", noop_factory).with_priority(50));
        registry
    }

    #[test]
    fn test_applicable_config_creation() {
        let config = ApplicableConfig::new("TestConfig", 42);
        assert_eq!(config.name, "TestConfig");
        assert_eq!(config.priority, 42);
    }

    #[test]
    fn test_applicable_config_display() {
        let config = ApplicableConfig::new("MyConfig", -10);
        let display = config.to_string();
        assert!(display.contains("MyConfig"));
        assert!(display.contains("-10"));
    }

    #[test]
    fn test_applicable_config_equality() {
        let a = ApplicableConfig::new("SameName", 1);
        let b = ApplicableConfig::new("SameName", 2);
        let c = ApplicableConfig::new("OtherName", 1);

        // 相等性仅基于名称
        // Equality is based on name only
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_evaluator_new() {
        let registry = create_test_registry();
        let evaluator = ConditionEvaluator::new(&registry);
        assert_eq!(evaluator.total_entries(), 3);
        assert_eq!(evaluator.excluded_count(), 0);
    }

    #[test]
    fn test_evaluator_evaluate_all() {
        let registry = create_test_registry();
        let ctx = ApplicationContext::new();
        let evaluator = ConditionEvaluator::new(&registry);

        let applicable = evaluator.evaluate(&ctx);
        assert_eq!(applicable.len(), 3);

        // 按优先级排序
        // Sorted by priority
        assert_eq!(applicable[0].name, "CoreConfig");
        assert_eq!(applicable[1].name, "WebConfig");
        assert_eq!(applicable[2].name, "DataConfig");
    }

    #[test]
    fn test_evaluator_exclude_single() {
        let registry = create_test_registry();
        let ctx = ApplicationContext::new();
        let evaluator = ConditionEvaluator::new(&registry).exclude("DataConfig");

        assert!(evaluator.is_excluded("DataConfig"));
        assert!(!evaluator.is_excluded("WebConfig"));

        let applicable = evaluator.evaluate(&ctx);
        assert_eq!(applicable.len(), 2);
        assert!(!applicable.iter().any(|c| c.name == "DataConfig"));
    }

    #[test]
    fn test_evaluator_exclude_many() {
        let registry = create_test_registry();
        let ctx = ApplicationContext::new();
        let evaluator =
            ConditionEvaluator::new(&registry).exclude_many(&["DataConfig", "WebConfig"]);

        assert_eq!(evaluator.excluded_count(), 2);

        let applicable = evaluator.evaluate(&ctx);
        assert_eq!(applicable.len(), 1);
        assert_eq!(applicable[0].name, "CoreConfig");
    }

    #[test]
    fn test_evaluator_evaluate_entries() {
        let registry = create_test_registry();
        let ctx = ApplicationContext::new();
        let evaluator = ConditionEvaluator::new(&registry);

        let entries = evaluator.evaluate_entries(&ctx);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name(), "CoreConfig");
    }

    #[test]
    fn test_evaluate_with_condition() {
        let mut registry = AutoConfigurationRegistry::new();

        registry.register(AutoConfigurationEntry::new("AlwaysConfig", noop_factory));
        registry.register_conditional(
            AutoConfigurationEntry::new("ConditionalConfig", noop_factory).with_condition(
                Box::new(crate::core::autoconfigure::ConditionalOnPropertyCondition::new(
                    "missing.prop",
                )),
            ),
        );

        let ctx = ApplicationContext::new();
        let evaluator = ConditionEvaluator::new(&registry);

        let applicable = evaluator.evaluate(&ctx);
        assert_eq!(applicable.len(), 1);
        assert_eq!(applicable[0].name, "AlwaysConfig");
    }

    #[test]
    fn test_evaluate_with_condition_met() {
        let mut registry = AutoConfigurationRegistry::new();

        registry.register(AutoConfigurationEntry::new("AlwaysConfig", noop_factory));
        registry.register_conditional(
            AutoConfigurationEntry::new("ConditionalConfig", noop_factory).with_condition(
                Box::new(
                    crate::core::autoconfigure::ConditionalOnPropertyCondition::new("exists.prop")
                        .match_if_missing(true),
                ),
            ),
        );

        let ctx = ApplicationContext::new();
        let evaluator = ConditionEvaluator::new(&registry);

        let applicable = evaluator.evaluate(&ctx);
        assert_eq!(applicable.len(), 2);
    }

    #[test]
    fn test_convenience_evaluate_conditions() {
        let registry = create_test_registry();
        let ctx = ApplicationContext::new();

        let applicable = evaluate_conditions(&registry, &ctx);
        assert_eq!(applicable.len(), 3);
    }

    #[test]
    fn test_empty_registry() {
        let registry = AutoConfigurationRegistry::new();
        let ctx = ApplicationContext::new();
        let evaluator = ConditionEvaluator::new(&registry);

        let applicable = evaluator.evaluate(&ctx);
        assert!(applicable.is_empty());
        assert_eq!(evaluator.total_entries(), 0);
    }

    #[test]
    fn test_exclude_all() {
        let registry = create_test_registry();
        let ctx = ApplicationContext::new();
        let evaluator = ConditionEvaluator::new(&registry).exclude_many(&[
            "CoreConfig",
            "WebConfig",
            "DataConfig",
        ]);

        let applicable = evaluator.evaluate(&ctx);
        assert!(applicable.is_empty());
    }

    #[test]
    fn test_applicable_config_hash() {
        let a = ApplicableConfig::new("ConfigA", 1);
        let b = ApplicableConfig::new("ConfigA", 2);
        let c = ApplicableConfig::new("ConfigB", 1);

        let mut set: HashSet<ApplicableConfig> = HashSet::new();
        set.insert(a);
        // b 和 a 同名，不会被添加（HashSet 去重）
        // b has same name as a, won't be added (HashSet dedup)
        set.insert(b);
        set.insert(c);

        assert_eq!(set.len(), 2);
    }
}
