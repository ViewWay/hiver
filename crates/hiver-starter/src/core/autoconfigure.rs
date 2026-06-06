//! 自动配置注册与排序系统 / Auto-Configuration Registration & Ordering System
//!
//! 提供基于条件的自动配置注册、排序和评估能力。
//! Provides condition-based auto-configuration registration, ordering, and evaluation.
//!
//! 参考 Spring Boot 的 `@AutoConfigureBefore`/`@AutoConfigureAfter` 注解以及
//! `spring-boot-autoconfigure` 的 `AutoConfigurationImportSelector`。
//! Based on Spring Boot's `@AutoConfigureBefore`/`@AutoConfigureAfter` annotations and
//! `spring-boot-autoconfigure`'s `AutoConfigurationImportSelector`.
//!
//! # 功能 / Features
//!
//! - 注册带条件的自动配置条目
//! - 按优先级和 `@AutoConfigureBefore`/`@AutoConfigureAfter` 排序
//! - 评估所有条件并返回适用的配置列表
//!
//! # 示例 / Example
//!
//! ```rust,ignore
//! use hiver_starter::core::autoconfigure::*;
//!
//! let mut registry = AutoConfigurationRegistry::new();
//! registry.register(AutoConfigurationEntry::new("MyConfig", factory_fn));
//! registry.register_conditional(
//!     AutoConfigurationEntry::new("ConditionalConfig", factory_fn)
//!         .with_condition(Box::new(ConditionalOnProperty::new("feature.enabled"))),
//! );
//!
//! let applicable = registry.evaluate(&context);
//! ```

use std::{any::TypeId, fmt};

use super::container::ApplicationContext;

// ============================================================================
// AutoConfigurationEntry / 自动配置条目
// ============================================================================

/// 自动配置条目
/// Auto-configuration entry
///
/// 描述一个可注册的自动配置，包含名称、工厂函数、条件和优先级。
/// Describes a registrable auto-configuration with name, factory function, condition, and priority.
///
/// # 字段 / Fields
///
/// - `name`: 配置名称，用于日志和调试 / Configuration name for logging and debugging
/// - `factory`: 工厂函数，负责在 `ApplicationContext` 中注册 Bean / Factory function that registers
///   beans
/// - `condition`: 可选条件，不满足时跳过此配置 / Optional condition; skip if not met
/// - `priority`: 排序优先级，数字越小优先级越高 / Sort priority (lower = higher priority)
pub struct AutoConfigurationEntry
{
    /// 配置名称
    /// Configuration name
    name: String,

    /// 工厂函数：接收 `&mut ApplicationContext`，执行配置逻辑
    /// Factory function: receives `&mut ApplicationContext`, executes configuration logic
    factory: Box<dyn Fn(&mut ApplicationContext) -> anyhow::Result<()> + Send + Sync>,

    /// 可选条件
    /// Optional condition
    condition: Option<Box<dyn Condition>>,

    /// 排序优先级（数字越小优先级越高）
    /// Sort priority (lower number = higher priority)
    priority: i32,
}

impl AutoConfigurationEntry
{
    /// 创建新的自动配置条目
    /// Create a new auto-configuration entry
    ///
    /// # 参数 / Parameters
    ///
    /// - `name`: 配置名称 / Configuration name
    /// - `factory`: 工厂函数 / Factory function
    pub fn new<F>(name: impl Into<String>, factory: F) -> Self
    where
        F: Fn(&mut ApplicationContext) -> anyhow::Result<()> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            factory: Box::new(factory),
            condition: None,
            priority: 0,
        }
    }

    /// 设置条件
    /// Set condition
    ///
    /// # 参数 / Parameters
    ///
    /// - `condition`: 条件对象 / Condition object
    pub fn with_condition(mut self, condition: Box<dyn Condition>) -> Self
    {
        self.condition = Some(condition);
        self
    }

    /// 设置优先级
    /// Set priority
    ///
    /// # 参数 / Parameters
    ///
    /// - `priority`: 优先级数值 / Priority value
    pub fn with_priority(mut self, priority: i32) -> Self
    {
        self.priority = priority;
        self
    }

    /// 获取配置名称
    /// Get configuration name
    pub fn name(&self) -> &str
    {
        &self.name
    }

    /// 获取优先级
    /// Get priority
    pub fn priority(&self) -> i32
    {
        self.priority
    }

    /// 检查条件是否满足（无条件时默认为 true）
    /// Check if condition is met (defaults to true when no condition)
    pub fn matches(&self, ctx: &ApplicationContext) -> bool
    {
        self.condition.as_ref().is_none_or(|c| c.matches(ctx))
    }

    /// 执行工厂函数
    /// Execute factory function
    pub fn configure(&self, ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        (self.factory)(ctx)
    }
}

impl fmt::Debug for AutoConfigurationEntry
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("AutoConfigurationEntry")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("has_condition", &self.condition.is_some())
            .finish()
    }
}

// ============================================================================
// AutoConfigurationRegistry / 自动配置注册表
// ============================================================================

/// 自动配置注册表
/// Auto-configuration registry
///
/// 管理所有已注册的自动配置条目，支持条件评估和优先级排序。
/// Manages all registered auto-configuration entries, supports condition evaluation and priority
/// sorting.
///
/// 参考 Spring Boot 的 `AutoConfigurationImportSelector` 和 `AutoConfigurationMetadata`。
/// Based on Spring Boot's `AutoConfigurationImportSelector` and `AutoConfigurationMetadata`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::autoconfigure::*;
///
/// let mut registry = AutoConfigurationRegistry::new();
/// registry.register(AutoConfigurationEntry::new("ConfigA", factory_a));
/// registry.register_conditional(
///     AutoConfigurationEntry::new("ConfigB", factory_b)
///         .with_condition(Box::new(ConditionalOnClass::new::<MyType>())),
/// );
///
/// // 评估所有条件，返回适用的配置
/// let applicable = registry.evaluate(&context);
/// ```
#[derive(Debug, Default)]
pub struct AutoConfigurationRegistry
{
    /// 已注册的自动配置条目
    /// Registered auto-configuration entries
    entries: Vec<AutoConfigurationEntry>,

    /// 排序约束：Before/After 关系
    /// Ordering constraints: Before/After relationships
    order_constraints: Vec<OrderConstraint>,
}

/// 排序约束
/// Ordering constraint
#[derive(Debug, Clone)]
struct OrderConstraint
{
    /// 源配置名称（应在前面）
    /// Source configuration name (should come first)
    before: String,

    /// 目标配置名称（应在后面）
    /// Target configuration name (should come later)
    after: String,
}

impl AutoConfigurationRegistry
{
    /// 创建新的注册表
    /// Create a new registry
    pub fn new() -> Self
    {
        Self::default()
    }

    /// 注册自动配置（无条件）
    /// Register auto-configuration (no condition)
    ///
    /// # 参数 / Parameters
    ///
    /// - `entry`: 自动配置条目 / Auto-configuration entry
    pub fn register(&mut self, entry: AutoConfigurationEntry)
    {
        self.entries.push(entry);
    }

    /// 注册带条件的自动配置
    /// Register auto-configuration with condition
    ///
    /// # 参数 / Parameters
    ///
    /// - `entry`: 自动配置条目（应已通过 `with_condition` 设置条件） / Auto-configuration entry
    ///   (should have condition set via `with_condition`)
    pub fn register_conditional(&mut self, entry: AutoConfigurationEntry)
    {
        self.entries.push(entry);
    }

    /// 添加排序约束：`before_name` 必须在 `after_name` 之前执行
    /// Add ordering constraint: `before_name` must execute before `after_name`
    ///
    /// 等价于 Spring Boot 的 `@AutoConfigureBefore`。
    /// Equivalent to Spring Boot's `@AutoConfigureBefore`.
    ///
    /// # 参数 / Parameters
    ///
    /// - `before_name`: 需要在前的配置名称 / Name of the configuration that should come before
    /// - `after_name`: 需要在后的配置名称 / Name of the configuration that should come after
    pub fn before(&mut self, before_name: &str, after_name: &str)
    {
        self.order_constraints.push(OrderConstraint {
            before: before_name.to_string(),
            after: after_name.to_string(),
        });
    }

    /// 添加排序约束：`after_name` 必须在 `before_name` 之后执行
    /// Add ordering constraint: `after_name` must execute after `before_name`
    ///
    /// 等价于 Spring Boot 的 `@AutoConfigureAfter`。
    /// Equivalent to Spring Boot's `@AutoConfigureAfter`.
    ///
    /// # 参数 / Parameters
    ///
    /// - `after_name`: 需要在后的配置名称 / Name of the configuration that should come after
    /// - `before_name`: 需要在前的配置名称 / Name of the configuration that should come before
    pub fn after(&mut self, after_name: &str, before_name: &str)
    {
        self.order_constraints.push(OrderConstraint {
            before: before_name.to_string(),
            after: after_name.to_string(),
        });
    }

    /// 评估所有条件并返回适用的配置列表
    /// Evaluate all conditions and return the list of applicable configurations
    ///
    /// 遍历所有已注册的条目，检查每个条目的条件是否满足，
    /// 然后按优先级和 Before/After 约束排序后返回。
    /// Iterates over all registered entries, checks each entry's condition,
    /// then returns them sorted by priority and Before/After constraints.
    ///
    /// # 参数 / Parameters
    ///
    /// - `ctx`: 应用上下文 / Application context
    ///
    /// # 返回 / Returns
    ///
    /// 返回所有条件满足的配置条目（已排序）。
    /// Returns all entries whose conditions are met (sorted).
    pub fn evaluate(&self, ctx: &ApplicationContext) -> Vec<&AutoConfigurationEntry>
    {
        let mut applicable: Vec<&AutoConfigurationEntry> = self
            .entries
            .iter()
            .filter(|entry| entry.matches(ctx))
            .collect();

        self.sort_entries(&mut applicable);
        applicable
    }

    /// 按优先级和 Before/After 约束排序
    /// Sort by priority and Before/After constraints
    ///
    /// 排序策略：
    /// 1. 先按 priority 升序排列（数字越小越靠前）
    /// 2. 再根据 Before/After 约束调整顺序
    ///
    /// Sorting strategy:
    /// 1. Sort by priority ascending (lower number comes first)
    /// 2. Adjust order based on Before/After constraints
    pub fn sort_by_priority(&mut self)
    {
        // Step 1: sort by numeric priority
        self.entries.sort_by_key(|e| e.priority);

        // Step 2: apply Before/After constraint adjustments
        self.apply_order_constraints_owned();
    }

    /// 获取所有已注册的条目（只读）
    /// Get all registered entries (read-only)
    pub fn entries(&self) -> &[AutoConfigurationEntry]
    {
        &self.entries
    }

    /// 获取已注册条目数量
    /// Get the number of registered entries
    pub fn len(&self) -> usize
    {
        self.entries.len()
    }

    /// 检查是否为空
    /// Check if empty
    pub fn is_empty(&self) -> bool
    {
        self.entries.is_empty()
    }

    /// 按名称查找条目
    /// Find entry by name
    pub fn find(&self, name: &str) -> Option<&AutoConfigurationEntry>
    {
        self.entries.iter().find(|e| e.name() == name)
    }

    // ----------------------------------------------------------------
    // 内部排序实现 / Internal sorting implementation
    // ----------------------------------------------------------------

    /// 对引用切片进行排序（用于 evaluate 返回结果）
    /// Sort a slice of references (used for evaluate return value)
    fn sort_entries(&self, entries: &mut Vec<&AutoConfigurationEntry>)
    {
        entries.sort_by_key(|e| e.priority);
        self.apply_order_constraints_ref(entries);
    }

    /// 将 Before/After 约束应用到 owned 条目列表
    /// Apply Before/After constraints to an owned entry list
    fn apply_order_constraints_owned(&mut self)
    {
        // 使用多轮冒泡来满足所有 Before/After 约束
        // Use multiple passes of bubble adjustments to satisfy all constraints
        for _ in 0..self.order_constraints.len()
        {
            let mut swapped = false;
            for constraint in &self.order_constraints
            {
                let before_idx = self
                    .entries
                    .iter()
                    .position(|e| e.name() == constraint.before);
                let after_idx = self
                    .entries
                    .iter()
                    .position(|e| e.name() == constraint.after);

                if let (Some(bi), Some(ai)) = (before_idx, after_idx)
                {
                    if bi > ai
                    {
                        // before 在 after 后面，需要移动
                        // before is after after, need to move
                        let entry = self.entries.remove(bi);
                        self.entries.insert(ai, entry);
                        swapped = true;
                    }
                }
            }
            if !swapped
            {
                break;
            }
        }
    }

    /// 将 Before/After 约束应用到引用切片
    /// Apply Before/After constraints to a reference slice
    fn apply_order_constraints_ref(&self, entries: &mut Vec<&AutoConfigurationEntry>)
    {
        for _ in 0..self.order_constraints.len()
        {
            let mut swapped = false;
            for constraint in &self.order_constraints
            {
                let before_idx = entries.iter().position(|e| e.name() == constraint.before);
                let after_idx = entries.iter().position(|e| e.name() == constraint.after);

                if let (Some(bi), Some(ai)) = (before_idx, after_idx)
                {
                    if bi > ai
                    {
                        let entry = entries.remove(bi);
                        entries.insert(ai, entry);
                        swapped = true;
                    }
                }
            }
            if !swapped
            {
                break;
            }
        }
    }
}

// ============================================================================
// Condition Trait / 条件 Trait
// ============================================================================

/// 条件 trait
/// Condition trait
///
/// 用于评估自动配置条目是否应该被应用。
/// Used to evaluate whether an auto-configuration entry should be applied.
///
/// 与 `condition.rs` 中的 `Conditional` trait 独立，此 trait 直接操作
/// `ApplicationContext`，专为自动配置注册表设计。
/// Independent from the `Conditional` trait in `condition.rs`, this trait
/// operates directly on `ApplicationContext`, designed for the auto-configuration registry.
pub trait Condition: Send + Sync + fmt::Debug
{
    /// 检查条件是否满足
    /// Check if the condition is met
    ///
    /// # 参数 / Parameters
    ///
    /// - `ctx`: 应用上下文 / Application context
    fn matches(&self, ctx: &ApplicationContext) -> bool;
}

// ============================================================================
// ConditionalOnClass / 类存在条件
// ============================================================================

/// 类存在条件
/// Class presence condition
///
/// 检查某个类型是否存在（通过 `TypeId`）。
/// Checks if a type exists (via `TypeId`).
///
/// 在 Rust 中，由于没有运行时反射，这个条件通常用于检查编译时特征。
/// 通过存储已知类型的 `TypeId` 集合，在运行时检查类型是否被"注册"。
///
/// In Rust, since there is no runtime reflection, this condition is typically used
/// to check compile-time features. By storing a set of known `TypeId`s, it checks
/// at runtime whether a type has been "registered".
///
/// 等价于 Spring Boot 的 `@ConditionalOnClass`。
/// Equivalent to Spring Boot's `@ConditionalOnClass`.
#[derive(Debug, Clone)]
pub struct ConditionalOnClass
{
    /// 类型名称（用于日志和环境变量检查）
    /// Type name (for logging and environment variable checking)
    type_name: &'static str,
}

impl ConditionalOnClass
{
    /// 创建新的类存在条件
    /// Create a new class presence condition
    ///
    /// # 类型参数 / Type Parameters
    ///
    /// - `T`: 要检查的类型 / Type to check
    pub fn new<T: 'static>() -> Self
    {
        Self {
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl Condition for ConditionalOnClass
{
    fn matches(&self, _ctx: &ApplicationContext) -> bool
    {
        // 在 Rust 中，如果类型 `T` 被引用，编译器会确保其存在。
        // 因此 `TypeId::of::<T>()` 能编译就意味着类型存在。
        //
        // 对于 feature-gated 的类型，我们使用环境变量来模拟。
        // In Rust, if type `T` is referenced, the compiler ensures it exists.
        // So `TypeId::of::<T>()` compiling means the type exists.
        //
        // For feature-gated types, we use environment variables to simulate.
        let env_key = format!(
            "HIVER_CONDITIONAL_ON_CLASS_{}",
            self.type_name.replace("::", "_").replace(['<', '>'], "_")
        );
        std::env::var(env_key).map_or(true, |v| v == "true" || v == "1")
    }
}

// ============================================================================
// ConditionalOnMissingClass / 类不存在条件
// ============================================================================

/// 类不存在条件
/// Class absence condition
///
/// 检查某个类型是否不存在。
/// Checks if a type does not exist.
///
/// 等价于 Spring Boot 的 `@ConditionalOnMissingClass`。
/// Equivalent to Spring Boot's `@ConditionalOnMissingClass`.
#[derive(Debug, Clone)]
pub struct ConditionalOnMissingClass
{
    /// 类型名称（用于日志和环境变量检查）
    /// Type name (for logging and environment variable checking)
    type_name: &'static str,
}

impl ConditionalOnMissingClass
{
    /// 创建新的类不存在条件
    /// Create a new class absence condition
    ///
    /// # 类型参数 / Type Parameters
    ///
    /// - `T`: 要检查不存在的类型 / Type to check for absence
    pub fn new<T: 'static>() -> Self
    {
        Self {
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl Condition for ConditionalOnMissingClass
{
    fn matches(&self, ctx: &ApplicationContext) -> bool
    {
        !ConditionalOnClass {
            type_name: self.type_name,
        }
        .matches(ctx)
    }
}

// ============================================================================
// ConditionalOnProperty (AutoConfigure 版) / Property Condition
// ============================================================================

/// 属性条件（用于自动配置注册表）
/// Property condition (for auto-configuration registry)
///
/// 检查配置属性是否存在或有特定值。
/// Checks if a configuration property exists or has a specific value.
///
/// 这是 `condition.rs` 中 `ConditionalOnProperty` 的轻量版本，
/// 直接实现 `Condition` trait 而非 `Conditional` trait。
///
/// This is a lightweight version of `ConditionalOnProperty` in `condition.rs`,
/// implementing the `Condition` trait directly instead of the `Conditional` trait.
///
/// 等价于 Spring Boot 的 `@ConditionalOnProperty`。
/// Equivalent to Spring Boot's `@ConditionalOnProperty`.
#[derive(Debug, Clone)]
pub struct ConditionalOnPropertyCondition
{
    /// 属性键
    /// Property key
    key: String,

    /// 期望的值（None 表示只需存在）
    /// Expected value (None means property just needs to exist)
    expected_value: Option<String>,

    /// 如果属性不存在时的默认匹配行为
    /// Default match behavior when property is absent
    match_if_missing: bool,
}

impl ConditionalOnPropertyCondition
{
    /// 创建新的属性条件
    /// Create a new property condition
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键 / Property key
    pub fn new(key: impl Into<String>) -> Self
    {
        Self {
            key: key.into(),
            expected_value: None,
            match_if_missing: false,
        }
    }

    /// 设置期望的值
    /// Set expected value
    pub fn having_value(mut self, value: impl Into<String>) -> Self
    {
        self.expected_value = Some(value.into());
        self
    }

    /// 设置属性缺失时的匹配行为
    /// Set match behavior when property is absent
    pub fn match_if_missing(mut self, match_if_missing: bool) -> Self
    {
        self.match_if_missing = match_if_missing;
        self
    }
}

impl Condition for ConditionalOnPropertyCondition
{
    fn matches(&self, ctx: &ApplicationContext) -> bool
    {
        if let Some(actual) = ctx.get_property(&self.key)
        {
            if let Some(ref expected) = self.expected_value
            {
                actual == *expected
            }
            else
            {
                !actual.is_empty()
            }
        }
        else
        {
            self.match_if_missing
        }
    }
}

// ============================================================================
// ConditionalOnMissingBean (AutoConfigure 版) / Bean Missing Condition
// ============================================================================

/// Bean 缺失条件（用于自动配置注册表）
/// Bean missing condition (for auto-configuration registry)
///
/// 检查容器中是否不存在指定类型的 Bean。
/// Checks if a bean of the specified type does not exist in the container.
///
/// 等价于 Spring Boot 的 `@ConditionalOnMissingBean`。
/// Equivalent to Spring Boot's `@ConditionalOnMissingBean`.
#[derive(Debug, Clone)]
pub struct ConditionalOnMissingBeanCondition
{
    /// Bean 类型 ID
    /// Bean type ID
    type_id: TypeId,
}

impl ConditionalOnMissingBeanCondition
{
    /// 创建新的 Bean 缺失条件
    /// Create a new bean missing condition
    ///
    /// # 类型参数 / Type Parameters
    ///
    /// - `T`: 要检查缺失的 Bean 类型 / Bean type to check for absence
    pub fn new<T: 'static>() -> Self
    {
        Self {
            type_id: TypeId::of::<T>(),
        }
    }
}

impl Condition for ConditionalOnMissingBeanCondition
{
    fn matches(&self, ctx: &ApplicationContext) -> bool
    {
        !ctx.contains_bean_by_id(self.type_id)
    }
}

// ============================================================================
// ConditionalOnBean (AutoConfigure 版) / Bean Present Condition
// ============================================================================

/// Bean 存在条件（用于自动配置注册表）
/// Bean present condition (for auto-configuration registry)
///
/// 检查容器中是否存在指定类型的 Bean。
/// Checks if a bean of the specified type exists in the container.
///
/// 等价于 Spring Boot 的 `@ConditionalOnBean`。
/// Equivalent to Spring Boot's `@ConditionalOnBean`.
#[derive(Debug, Clone)]
pub struct ConditionalOnBeanCondition
{
    /// Bean 类型 ID
    /// Bean type ID
    type_id: TypeId,
}

impl ConditionalOnBeanCondition
{
    /// 创建新的 Bean 存在条件
    /// Create a new bean present condition
    ///
    /// # 类型参数 / Type Parameters
    ///
    /// - `T`: 要检查存在的 Bean 类型 / Bean type to check for presence
    pub fn new<T: 'static>() -> Self
    {
        Self {
            type_id: TypeId::of::<T>(),
        }
    }
}

impl Condition for ConditionalOnBeanCondition
{
    fn matches(&self, ctx: &ApplicationContext) -> bool
    {
        ctx.contains_bean_by_id(self.type_id)
    }
}

// ============================================================================
// AutoConfigureOrder / 自动配置顺序
// ============================================================================

/// 自动配置顺序枚举
/// Auto-configure order enum
///
/// 用于声明配置之间的相对执行顺序。
/// Used to declare relative execution order between configurations.
///
/// 等价于 Spring Boot 的 `@AutoConfigureBefore` 和 `@AutoConfigureAfter`。
/// Equivalent to Spring Boot's `@AutoConfigureBefore` and `@AutoConfigureAfter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoConfigureOrder
{
    /// 在指定配置之前执行
    /// Execute before the specified configuration
    Before,

    /// 在指定配置之后执行
    /// Execute after the specified configuration
    After,
}

// ============================================================================
// EnableAutoConfiguration / 启用自动配置标记
// ============================================================================

/// 启用自动配置标记结构体
/// Enable auto-configuration marker struct
///
/// 当此类型存在于注册表中时，表示启用了自动配置功能。
/// When this type exists in the registry, auto-configuration is enabled.
///
/// 等价于 Spring Boot 的 `@EnableAutoConfiguration` 注解。
/// Equivalent to Spring Boot's `@EnableAutoConfiguration` annotation.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::autoconfigure::EnableAutoConfiguration;
///
/// // 此标记通常由框架内部使用
/// // This marker is typically used internally by the framework
/// let _marker = EnableAutoConfiguration;
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct EnableAutoConfiguration;

impl EnableAutoConfiguration
{
    /// 创建新的标记实例
    /// Create a new marker instance
    pub fn new() -> Self
    {
        Self
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use std::sync::Arc;

    use super::*;

    fn noop_factory(_ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        Ok(())
    }

    fn register_test_bean(ctx: &mut ApplicationContext) -> anyhow::Result<()>
    {
        ctx.register_bean(42i32);
        Ok(())
    }

    #[test]
    fn test_entry_creation()
    {
        let entry = AutoConfigurationEntry::new("TestConfig", noop_factory);
        assert_eq!(entry.name(), "TestConfig");
        assert_eq!(entry.priority(), 0);
        assert!(entry.condition.is_none());
    }

    #[test]
    fn test_entry_with_priority()
    {
        let entry =
            AutoConfigurationEntry::new("HighPriorityConfig", noop_factory).with_priority(-100);
        assert_eq!(entry.priority(), -100);
    }

    #[test]
    fn test_entry_matches_no_condition()
    {
        let ctx = ApplicationContext::new();
        let entry = AutoConfigurationEntry::new("AlwaysConfig", noop_factory);
        assert!(entry.matches(&ctx));
    }

    #[test]
    fn test_entry_matches_with_condition()
    {
        let ctx = ApplicationContext::new();
        let entry = AutoConfigurationEntry::new("ConditionalConfig", noop_factory)
            .with_condition(Box::new(ConditionalOnPropertyCondition::new("nonexistent.key")));

        // 属性不存在且 match_if_missing=false，应返回 false
        // Property absent and match_if_missing=false, should return false
        assert!(!entry.matches(&ctx));
    }

    #[test]
    fn test_entry_configure()
    {
        let mut ctx = ApplicationContext::new();
        let entry = AutoConfigurationEntry::new("RegisterConfig", register_test_bean);
        entry.configure(&mut ctx).unwrap();
        assert!(ctx.contains_bean::<i32>());
    }

    #[test]
    fn test_registry_register()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("ConfigA", noop_factory));
        registry.register(AutoConfigurationEntry::new("ConfigB", noop_factory));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_register_conditional()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register_conditional(
            AutoConfigurationEntry::new("ConditionalConfig", noop_factory)
                .with_condition(Box::new(ConditionalOnPropertyCondition::new("feature.x"))),
        );
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_evaluate_no_conditions()
    {
        let ctx = ApplicationContext::new();
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("ConfigA", noop_factory));
        registry.register(AutoConfigurationEntry::new("ConfigB", noop_factory));

        let applicable = registry.evaluate(&ctx);
        assert_eq!(applicable.len(), 2);
    }

    #[test]
    fn test_registry_evaluate_with_condition()
    {
        let ctx = ApplicationContext::new();
        let mut registry = AutoConfigurationRegistry::new();

        // 无条件配置
        // Unconditional configuration
        registry.register(AutoConfigurationEntry::new("AlwaysConfig", noop_factory));

        // 有条件配置（属性不存在）
        // Conditional configuration (property absent)
        registry.register_conditional(
            AutoConfigurationEntry::new("ConditionalConfig", noop_factory)
                .with_condition(Box::new(ConditionalOnPropertyCondition::new("missing.key"))),
        );

        let applicable = registry.evaluate(&ctx);
        assert_eq!(applicable.len(), 1);
        assert_eq!(applicable[0].name(), "AlwaysConfig");
    }

    #[test]
    fn test_registry_sort_by_priority()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry
            .register(AutoConfigurationEntry::new("LowPriority", noop_factory).with_priority(100));
        registry.register(
            AutoConfigurationEntry::new("HighPriority", noop_factory).with_priority(-100),
        );
        registry.register(AutoConfigurationEntry::new("DefaultPriority", noop_factory));

        registry.sort_by_priority();

        let entries = registry.entries();
        assert_eq!(entries[0].name(), "HighPriority");
        assert_eq!(entries[1].name(), "DefaultPriority");
        assert_eq!(entries[2].name(), "LowPriority");
    }

    #[test]
    fn test_registry_before_after_constraints()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("ConfigA", noop_factory).with_priority(10));
        registry.register(AutoConfigurationEntry::new("ConfigB", noop_factory).with_priority(5));

        // ConfigA (priority 10) 应该在 ConfigB (priority 5) 之前
        // 虽然按优先级 ConfigB 更高，但 before 约束覆盖
        // ConfigA (priority 10) should come before ConfigB (priority 5)
        // Even though ConfigB has higher priority numerically, the before constraint overrides
        registry.before("ConfigA", "ConfigB");

        let ctx = ApplicationContext::new();
        let applicable = registry.evaluate(&ctx);

        assert_eq!(applicable.len(), 2);
        // ConfigA should come before ConfigB due to constraint
        assert_eq!(applicable[0].name(), "ConfigA");
        assert_eq!(applicable[1].name(), "ConfigB");
    }

    #[test]
    fn test_registry_find()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("TargetConfig", noop_factory));
        registry.register(AutoConfigurationEntry::new("OtherConfig", noop_factory));

        let found = registry.find("TargetConfig");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "TargetConfig");

        let not_found = registry.find("NonExistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_is_empty()
    {
        let registry = AutoConfigurationRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_conditional_on_missing_bean_condition()
    {
        let ctx = ApplicationContext::new();
        let condition = ConditionalOnMissingBeanCondition::new::<i32>();

        // Bean 不存在，条件满足
        // Bean absent, condition met
        assert!(condition.matches(&ctx));

        // 注册 Bean
        // Register bean
        ctx.register_bean(42i32);

        // Bean 存在，条件不满足
        // Bean present, condition not met
        assert!(!condition.matches(&ctx));
    }

    #[test]
    fn test_conditional_on_bean_condition()
    {
        let ctx = ApplicationContext::new();
        let condition = ConditionalOnBeanCondition::new::<i32>();

        // Bean 不存在，条件不满足
        // Bean absent, condition not met
        assert!(!condition.matches(&ctx));

        // 注册 Bean
        // Register bean
        ctx.register_bean(42i32);

        // Bean 存在，条件满足
        // Bean present, condition met
        assert!(condition.matches(&ctx));
    }

    #[test]
    fn test_conditional_on_property_condition()
    {
        let ctx = ApplicationContext::new();

        // 属性不存在，match_if_missing=false
        // Property absent, match_if_missing=false
        let cond = ConditionalOnPropertyCondition::new("test.key");
        assert!(!cond.matches(&ctx));

        // match_if_missing=true
        let cond = ConditionalOnPropertyCondition::new("test.key").match_if_missing(true);
        assert!(cond.matches(&ctx));

        // 有值匹配
        // Value matches
        let mut loader = crate::config::ConfigurationLoader::new();
        loader.set("test.key".to_string(), "enabled".to_string());
        let ctx_with_prop = ApplicationContext::with_config_loader(Arc::new(loader));

        let cond = ConditionalOnPropertyCondition::new("test.key").having_value("enabled");
        assert!(cond.matches(&ctx_with_prop));

        // 有值不匹配
        // Value does not match
        let cond = ConditionalOnPropertyCondition::new("test.key").having_value("disabled");
        assert!(!cond.matches(&ctx_with_prop));
    }

    #[test]
    fn test_enable_auto_configuration_marker()
    {
        let marker = EnableAutoConfiguration::new();
        let _default = EnableAutoConfiguration;
        // 标记存在且可用
        // Marker exists and is usable
        assert_eq!(format!("{:?}", marker), "EnableAutoConfiguration");
    }

    #[test]
    fn test_auto_configure_order()
    {
        let before = AutoConfigureOrder::Before;
        let after = AutoConfigureOrder::After;
        assert_ne!(before, after);
        assert_eq!(before, AutoConfigureOrder::Before);
    }

    #[test]
    fn test_conditional_on_class()
    {
        let ctx = ApplicationContext::new();
        let condition = ConditionalOnClass::new::<String>();
        // 默认行为：返回 true（无环境变量覆盖）
        // Default behavior: returns true (no env var override)
        assert!(condition.matches(&ctx));
    }

    #[test]
    fn test_conditional_on_missing_class()
    {
        let ctx = ApplicationContext::new();
        let condition = ConditionalOnMissingClass::new::<String>();
        // 默认：ConditionalOnClass 返回 true，因此 MissingClass 返回 false
        // Default: ConditionalOnClass returns true, so MissingClass returns false
        assert!(!condition.matches(&ctx));
    }

    #[test]
    fn test_entry_debug_format()
    {
        let entry = AutoConfigurationEntry::new("DebugTest", noop_factory).with_priority(42);
        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("DebugTest"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("has_condition: false"));
    }
}
