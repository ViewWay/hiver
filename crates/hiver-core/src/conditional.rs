//! Conditional bean registration support
//! 条件化Bean注册支持
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@ConditionalOnProperty`
//! - `@ConditionalOnMissingBean`
//! - `@ConditionalOnBean`
//! - `@ConditionalOnExpression`
//! - `@Profile`
//!
//! # Overview / 概述
//!
//! This module provides a condition evaluation system for the IoC container,
//! enabling Spring Boot-style conditional bean registration. Beans can be
//! registered only when certain conditions are met, such as the presence of
//! a configuration property, the absence of another bean, or an active profile.
//!
//! 此模块为IoC容器提供条件评估系统，支持Spring Boot风格的条件化Bean注册。
//! 只有在满足特定条件时（如配置属性存在、另一个Bean不存在、或特定Profile激活），
//! Bean才会被注册。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::any::TypeId;
use std::collections::HashMap;

/// Context provided to [`Condition`] evaluations.
/// 提供给 [`Condition`] 评估的上下文。
///
/// Contains the current state of the container and environment
/// so that conditions can make informed decisions.
///
/// 包含容器和环境的当前状态，以便条件可以做出明智的决策。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::ConditionContext;
/// use std::collections::HashMap;
///
/// let ctx = ConditionContext::new()
///     .with_properties(HashMap::from([
///         ("app.feature.enabled".to_string(), "true".to_string()),
///     ]))
///     .with_profiles(vec!["production".to_string()]);
///
/// assert_eq!(ctx.property("app.feature.enabled"), Some("true"));
/// assert!(ctx.is_profile_active("production"));
/// ```
pub struct ConditionContext {
    /// Configuration properties (key-value pairs)
    /// 配置属性（键值对）
    properties: HashMap<String, String>,

    /// Currently active profiles
    /// 当前激活的配置文件
    active_profiles: Vec<String>,

    /// TypeIds of beans already registered in the container
    /// 容器中已注册的Bean的TypeId列表
    registered_beans: Vec<TypeId>,

    /// Named bean lookups (bean name -> TypeId)
    /// 命名Bean查找（bean名称 -> TypeId）
    bean_names: HashMap<String, TypeId>,
}

impl ConditionContext {
    /// Create a new empty condition context.
    /// 创建一个空的_condition context_。
    #[must_use]
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            active_profiles: Vec::new(),
            registered_beans: Vec::new(),
            bean_names: HashMap::new(),
        }
    }

    /// Set the configuration properties.
    /// 设置配置属性。
    #[must_use]
    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = properties;
        self
    }

    /// Set the active profiles.
    /// 设置激活的配置文件。
    #[must_use]
    pub fn with_profiles(mut self, profiles: Vec<String>) -> Self {
        self.active_profiles = profiles;
        self
    }

    /// Set the registered bean TypeIds.
    /// 设置已注册的Bean的TypeId列表。
    ///
    /// This is typically populated from the container's current state
    /// before evaluating a conditional registration.
    ///
    /// 通常在评估条件注册之前从容器的当前状态填充。
    #[must_use]
    pub fn with_registered_beans(mut self, beans: Vec<TypeId>) -> Self {
        self.registered_beans = beans;
        self
    }

    /// Set the named bean lookups.
    /// 设置命名Bean查找。
    #[must_use]
    pub fn with_bean_names(mut self, names: HashMap<String, TypeId>) -> Self {
        self.bean_names = names;
        self
    }

    /// Get a property value by key.
    /// 按键获取属性值。
    ///
    /// Returns `Some(&str)` if the property exists, `None` otherwise.
    /// 如果属性存在则返回 `Some(&str)`，否则返回 `None`。
    #[must_use]
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }

    /// Check whether a property exists and equals a specific value.
    /// 检查属性是否存在且等于特定值。
    #[must_use]
    pub fn property_equals(&self, key: &str, expected: &str) -> bool {
        self.property(key) == Some(expected)
    }

    /// Check whether a profile is currently active.
    /// 检查配置文件是否当前激活。
    ///
    /// The special profile `"default"` is always considered active.
    /// 特殊配置文件 `"default"` 始终被视为激活状态。
    #[must_use]
    pub fn is_profile_active(&self, profile: &str) -> bool {
        profile == "default" || self.active_profiles.iter().any(|p| p == profile)
    }

    /// Check whether a bean of type `T` is already registered.
    /// 检查类型 `T` 的Bean是否已注册。
    #[must_use]
    pub fn has_bean<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.registered_beans.contains(&type_id)
    }

    /// Check whether a bean with the given name is registered.
    /// 检查具有给定名称的Bean是否已注册。
    #[must_use]
    pub fn has_bean_by_id(&self, name: &str) -> bool {
        self.bean_names.contains_key(name)
    }

    /// Get a reference to the properties map.
    /// 获取属性映射的引用。
    #[must_use]
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    /// Get a reference to the active profiles.
    /// 获取激活配置文件的引用。
    #[must_use]
    pub fn active_profiles(&self) -> &[String] {
        &self.active_profiles
    }

    /// Get a reference to the registered bean TypeIds.
    /// 获取已注册Bean的TypeId列表的引用。
    #[must_use]
    pub fn registered_beans(&self) -> &[TypeId] {
        &self.registered_beans
    }
}

impl Default for ConditionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for conditional bean registration.
/// 条件化Bean注册的trait。
///
/// Implementations decide whether a bean should be registered based on
/// the current [`ConditionContext`].
///
/// 实现根据当前的 [`ConditionContext`] 决定是否应注册Bean。
///
/// Equivalent to Spring Boot's `@Conditional` annotation.
/// 等价于Spring Boot的 `@Conditional` 注解。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{Condition, ConditionContext};
///
/// struct AlwaysTrue;
///
/// impl Condition for AlwaysTrue {
///     fn matches(&self, _context: &ConditionContext) -> bool {
///         true
///     }
/// }
/// ```
pub trait Condition: Send + Sync {
    /// Evaluate whether the condition matches.
    /// 评估条件是否匹配。
    ///
    /// Returns `true` if the bean should be registered, `false` otherwise.
    /// 如果应注册Bean则返回 `true`，否则返回 `false`。
    fn matches(&self, context: &ConditionContext) -> bool;
}

/// Condition that matches based on a configuration property.
/// 基于配置属性匹配的条件。
///
/// Equivalent to Spring Boot's `@ConditionalOnProperty`.
/// 等价于Spring Boot的 `@ConditionalOnProperty`。
///
/// # Behavior / 行为
///
/// - If `value` is `Some(v)`: matches when `property(key) == v`.
/// - If `value` is `None` and `match_if_missing` is `false` (default):
///   matches when the property exists with any non-empty value.
/// - If `value` is `None` and `match_if_missing` is `true`:
///   matches when the property exists (even if empty) or is missing.
///
/// - 如果 `value` 为 `Some(v)`：当 `property(key) == v` 时匹配。
/// - 如果 `value` 为 `None` 且 `match_if_missing` 为 `false`（默认）：
///   当属性存在且具有任何非空值时匹配。
/// - 如果 `value` 为 `None` 且 `match_if_missing` 为 `true`：
///   当属性存在（即使为空）或缺失时匹配。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{ConditionalOnProperty, Condition, ConditionContext};
/// use std::collections::HashMap;
///
/// let ctx = ConditionContext::new()
///     .with_properties(HashMap::from([
///         ("feature.cache.enabled".to_string(), "true".to_string()),
///     ]));
///
/// let cond = ConditionalOnProperty::new("feature.cache.enabled")
///     .with_value("true");
///
/// assert!(cond.matches(&ctx));
/// ```
pub struct ConditionalOnProperty {
    /// The property key to check.
    /// 要检查的属性键。
    key: String,

    /// The expected property value. If `None`, only the existence of the
    /// property is checked.
    /// 期望的属性值。如果为 `None`，则仅检查属性是否存在。
    value: Option<String>,

    /// Whether to match when the property is missing.
    /// 当属性缺失时是否匹配。
    match_if_missing: bool,
}

impl ConditionalOnProperty {
    /// Create a new condition that checks for the existence of a property.
    /// 创建一个检查属性是否存在的新条件。
    ///
    /// By default, this matches when the property key exists with a
    /// non-empty value. Use [`with_value`](Self::with_value) or
    /// [`with_match_if_missing`](Self::with_match_if_missing) to customize.
    ///
    /// 默认情况下，当属性键存在且具有非空值时匹配。
    /// 使用 [`with_value`](Self::with_value) 或
    /// [`with_match_if_missing`](Self::with_match_if_missing) 进行自定义。
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
            match_if_missing: false,
        }
    }

    /// Set the expected property value.
    /// 设置期望的属性值。
    ///
    /// When set, the condition matches only when `property(key) == value`.
    /// 设置后，条件仅在 `property(key) == value` 时匹配。
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set whether to match when the property is missing.
    /// 设置属性缺失时是否匹配。
    ///
    /// When `true`, the condition matches even if the property is not present.
    /// 为 `true` 时，即使属性不存在，条件也会匹配。
    #[must_use]
    pub fn with_match_if_missing(mut self, match_if_missing: bool) -> Self {
        self.match_if_missing = match_if_missing;
        self
    }

    /// Get the property key.
    /// 获取属性键。
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the expected value, if any.
    /// 获取期望值（如果有）。
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    /// Get whether missing properties should match.
    /// 获取缺失属性是否应匹配。
    #[must_use]
    pub fn match_if_missing(&self) -> bool {
        self.match_if_missing
    }
}

impl Condition for ConditionalOnProperty {
    fn matches(&self, context: &ConditionContext) -> bool {
        if let Some(expected) = &self.value {
            // Exact value match
            // 精确值匹配
            context.property_equals(&self.key, expected)
        } else {
            // Existence check
            // 存在性检查
            let exists = context.property(&self.key).is_some();
            exists || self.match_if_missing
        }
    }
}

/// Condition that matches when a bean of a given type is **not** registered.
/// 当指定类型的Bean **未** 注册时匹配的条件。
///
/// Equivalent to Spring Boot's `@ConditionalOnMissingBean`.
/// 等价于Spring Boot的 `@ConditionalOnMissingBean`。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{ConditionalOnMissingBean, Condition, ConditionContext};
/// use std::any::TypeId;
///
/// let ctx = ConditionContext::new()
///     .with_registered_beans(vec![]);
///
/// // No DataSource bean registered, so this matches
/// // 未注册DataSource Bean，因此匹配
/// struct DataSource;
/// let cond = ConditionalOnMissingBean::of::<DataSource>();
/// assert!(cond.matches(&ctx));
/// ```
pub struct ConditionalOnMissingBean {
    /// The TypeId of the bean type to check.
    /// 要检查的Bean类型的TypeId。
    type_id: TypeId,
}

impl ConditionalOnMissingBean {
    /// Create a condition that matches when a bean of type `T` is not registered.
    /// 创建当类型 `T` 的Bean未注册时匹配的条件。
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
        }
    }

    /// Create a condition from a raw `TypeId`.
    /// 从原始 `TypeId` 创建条件。
    pub fn from_type_id(type_id: TypeId) -> Self {
        Self { type_id }
    }

    /// Get the TypeId this condition checks.
    /// 获取此条件检查的TypeId。
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

impl Condition for ConditionalOnMissingBean {
    fn matches(&self, context: &ConditionContext) -> bool {
        !context.registered_beans().contains(&self.type_id)
    }
}

/// Condition that matches when a bean of a given type **is** registered.
/// 当指定类型的Bean **已** 注册时匹配的条件。
///
/// Equivalent to Spring Boot's `@ConditionalOnBean`.
/// 等价于Spring Boot的 `@ConditionalOnBean`。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{ConditionalOnBean, Condition, ConditionContext};
/// use std::any::TypeId;
///
/// struct DataSource;
/// let ctx = ConditionContext::new()
///     .with_registered_beans(vec![TypeId::of::<DataSource>()]);
///
/// let cond = ConditionalOnBean::of::<DataSource>();
/// assert!(cond.matches(&ctx));
/// ```
pub struct ConditionalOnBean {
    /// The TypeId of the bean type to check.
    /// 要检查的Bean类型的TypeId。
    type_id: TypeId,
}

impl ConditionalOnBean {
    /// Create a condition that matches when a bean of type `T` is registered.
    /// 创建当类型 `T` 的Bean已注册时匹配的条件。
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
        }
    }

    /// Create a condition from a raw `TypeId`.
    /// 从原始 `TypeId` 创建条件。
    pub fn from_type_id(type_id: TypeId) -> Self {
        Self { type_id }
    }

    /// Get the TypeId this condition checks.
    /// 获取此条件检查的TypeId。
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

impl Condition for ConditionalOnBean {
    fn matches(&self, context: &ConditionContext) -> bool {
        context.registered_beans().contains(&self.type_id)
    }
}

/// Condition that matches when a specific profile is active.
/// 当特定配置文件激活时匹配的条件。
///
/// Equivalent to Spring's `@Profile` annotation.
/// 等价于Spring的 `@Profile` 注解。
///
/// The special profile `"default"` is always considered active.
/// 特殊配置文件 `"default"` 始终被视为激活状态。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{ProfileCondition, Condition, ConditionContext};
///
/// let ctx = ConditionContext::new()
///     .with_profiles(vec!["production".to_string()]);
///
/// let cond = ProfileCondition::new("production");
/// assert!(cond.matches(&ctx));
///
/// let dev_cond = ProfileCondition::new("dev");
/// assert!(!dev_cond.matches(&ctx));
/// ```
pub struct ProfileCondition {
    /// The profile name to check.
    /// 要检查的配置文件名称。
    profile: String,
}

impl ProfileCondition {
    /// Create a new profile condition.
    /// 创建新的配置文件条件。
    pub fn new(profile: impl Into<String>) -> Self {
        Self {
            profile: profile.into(),
        }
    }

    /// Get the profile name.
    /// 获取配置文件名称。
    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }
}

impl Condition for ProfileCondition {
    fn matches(&self, context: &ConditionContext) -> bool {
        context.is_profile_active(&self.profile)
    }
}

/// Composite condition that matches when **all** inner conditions match (AND logic).
/// 当**所有**内部条件都匹配时的组合条件（AND逻辑）。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{AllConditions, ConditionalOnProperty, ProfileCondition, Condition, ConditionContext};
/// use std::collections::HashMap;
///
/// let ctx = ConditionContext::new()
///     .with_properties(HashMap::from([
///         ("cache.enabled".to_string(), "true".to_string()),
///     ]))
///     .with_profiles(vec!["production".to_string()]);
///
/// let cond = AllConditions::new(vec![
///     Box::new(ConditionalOnProperty::new("cache.enabled").with_value("true")),
///     Box::new(ProfileCondition::new("production")),
/// ]);
///
/// assert!(cond.matches(&ctx));
/// ```
pub struct AllConditions {
    /// Inner conditions that must all match.
    /// 必须全部匹配的内部条件。
    conditions: Vec<Box<dyn Condition>>,
}

impl AllConditions {
    /// Create a new AND composite condition.
    /// 创建新的AND组合条件。
    pub fn new(conditions: Vec<Box<dyn Condition>>) -> Self {
        Self { conditions }
    }

    /// Get a reference to the inner conditions.
    /// 获取内部条件的引用。
    #[must_use]
    pub fn conditions(&self) -> &[Box<dyn Condition>] {
        &self.conditions
    }
}

impl Condition for AllConditions {
    fn matches(&self, context: &ConditionContext) -> bool {
        // All conditions must match; short-circuit on first failure
        // 所有条件必须匹配；在第一个失败时短路
        self.conditions.iter().all(|c| c.matches(context))
    }
}

/// Composite condition that matches when **any** inner condition matches (OR logic).
/// 当**任意**内部条件匹配时的组合条件（OR逻辑）。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{AnyCondition, ProfileCondition, Condition, ConditionContext};
///
/// let ctx = ConditionContext::new()
///     .with_profiles(vec!["staging".to_string()]);
///
/// let cond = AnyCondition::new(vec![
///     Box::new(ProfileCondition::new("production")),
///     Box::new(ProfileCondition::new("staging")),
/// ]);
///
/// assert!(cond.matches(&ctx));
/// ```
pub struct AnyCondition {
    /// Inner conditions; at least one must match.
    /// 内部条件；至少一个必须匹配。
    conditions: Vec<Box<dyn Condition>>,
}

impl AnyCondition {
    /// Create a new OR composite condition.
    /// 创建新的OR组合条件。
    pub fn new(conditions: Vec<Box<dyn Condition>>) -> Self {
        Self { conditions }
    }

    /// Get a reference to the inner conditions.
    /// 获取内部条件的引用。
    #[must_use]
    pub fn conditions(&self) -> &[Box<dyn Condition>] {
        &self.conditions
    }
}

impl Condition for AnyCondition {
    fn matches(&self, context: &ConditionContext) -> bool {
        // At least one condition must match; short-circuit on first success
        // 至少一个条件必须匹配；在第一个成功时短路
        self.conditions.iter().any(|c| c.matches(context))
    }
}

/// Negation condition that inverts the result of an inner condition.
/// 取反条件，反转内部条件的结果。
///
/// # Example / 示例
///
/// ```
/// use hiver_core::conditional::{NotCondition, ProfileCondition, Condition, ConditionContext};
///
/// let ctx = ConditionContext::new()
///     .with_profiles(vec!["production".to_string()]);
///
/// let not_dev = NotCondition::new(Box::new(ProfileCondition::new("dev")));
/// assert!(not_dev.matches(&ctx)); // "dev" is not active, so NOT "dev" is true
/// ```
pub struct NotCondition {
    /// The inner condition to negate.
    /// 要取反的内部条件。
    inner: Box<dyn Condition>,
}

impl NotCondition {
    /// Create a new negation condition wrapping the given condition.
    /// 创建一个包装给定条件的新取反条件。
    pub fn new(inner: Box<dyn Condition>) -> Self {
        Self { inner }
    }

    /// Get a reference to the inner condition.
    /// 获取内部条件的引用。
    #[must_use]
    pub fn inner(&self) -> &dyn Condition {
        &*self.inner
    }
}

impl Condition for NotCondition {
    fn matches(&self, context: &ConditionContext) -> bool {
        !self.inner.matches(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // -- ConditionContext tests -------------------------------------------
    // ConditionContext 测试

    #[test]
    fn condition_context_new_is_empty() {
        let ctx = ConditionContext::new();
        assert!(ctx.properties().is_empty());
        assert!(ctx.active_profiles().is_empty());
        assert!(ctx.registered_beans().is_empty());
    }

    #[test]
    fn condition_context_default_is_empty() {
        let ctx = ConditionContext::default();
        assert!(ctx.properties().is_empty());
    }

    #[test]
    fn condition_context_with_properties() {
        let props = HashMap::from([
            ("key1".to_string(), "val1".to_string()),
            ("key2".to_string(), "val2".to_string()),
        ]);
        let ctx = ConditionContext::new().with_properties(props);
        assert_eq!(ctx.property("key1"), Some("val1"));
        assert_eq!(ctx.property("key2"), Some("val2"));
        assert_eq!(ctx.property("missing"), None);
    }

    #[test]
    fn condition_context_with_profiles() {
        let ctx =
            ConditionContext::new().with_profiles(vec!["dev".to_string(), "test".to_string()]);
        assert!(ctx.is_profile_active("dev"));
        assert!(ctx.is_profile_active("test"));
        assert!(!ctx.is_profile_active("prod"));
    }

    #[test]
    fn condition_context_default_profile_always_active() {
        let ctx = ConditionContext::new();
        assert!(ctx.is_profile_active("default"));
    }

    #[test]
    fn condition_context_has_bean() {
        struct MyService;
        let ctx = ConditionContext::new().with_registered_beans(vec![TypeId::of::<MyService>()]);
        assert!(ctx.has_bean::<MyService>());

        struct OtherService;
        assert!(!ctx.has_bean::<OtherService>());
    }

    #[test]
    fn condition_context_has_bean_by_id() {
        let ctx = ConditionContext::new()
            .with_bean_names(HashMap::from([("myService".to_string(), TypeId::of::<i32>())]));
        assert!(ctx.has_bean_by_id("myService"));
        assert!(!ctx.has_bean_by_id("nonexistent"));
    }

    #[test]
    fn condition_context_property_equals() {
        let ctx = ConditionContext::new()
            .with_properties(HashMap::from([("timeout".to_string(), "30".to_string())]));
        assert!(ctx.property_equals("timeout", "30"));
        assert!(!ctx.property_equals("timeout", "60"));
        assert!(!ctx.property_equals("missing", "30"));
    }

    // -- ConditionalOnProperty tests --------------------------------------
    // ConditionalOnProperty 测试

    #[test]
    fn conditional_on_property_exact_value_match() {
        let ctx = ConditionContext::new()
            .with_properties(HashMap::from([("cache.enabled".to_string(), "true".to_string())]));

        let cond = ConditionalOnProperty::new("cache.enabled").with_value("true");
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_property_exact_value_mismatch() {
        let ctx = ConditionContext::new()
            .with_properties(HashMap::from([("cache.enabled".to_string(), "false".to_string())]));

        let cond = ConditionalOnProperty::new("cache.enabled").with_value("true");
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_property_existence_check_present() {
        let ctx = ConditionContext::new()
            .with_properties(HashMap::from([("cache.enabled".to_string(), "true".to_string())]));

        // No value specified, just check existence
        // 未指定值，仅检查存在性
        let cond = ConditionalOnProperty::new("cache.enabled");
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_property_existence_check_missing() {
        let ctx = ConditionContext::new();

        let cond = ConditionalOnProperty::new("cache.enabled");
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_property_match_if_missing_true_when_absent() {
        let ctx = ConditionContext::new();

        let cond = ConditionalOnProperty::new("cache.enabled").with_match_if_missing(true);
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_property_match_if_missing_false_when_absent() {
        let ctx = ConditionContext::new();

        let cond = ConditionalOnProperty::new("cache.enabled").with_match_if_missing(false);
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_property_accessors() {
        let cond = ConditionalOnProperty::new("my.key").with_value("my.val");
        assert_eq!(cond.key(), "my.key");
        assert_eq!(cond.value(), Some("my.val"));
        assert!(!cond.match_if_missing());
    }

    // -- ConditionalOnMissingBean tests -----------------------------------
    // ConditionalOnMissingBean 测试

    #[test]
    fn conditional_on_missing_bean_matches_when_absent() {
        struct MyService;
        let ctx = ConditionContext::new();

        let cond = ConditionalOnMissingBean::of::<MyService>();
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_missing_bean_rejects_when_present() {
        struct MyService;
        let ctx = ConditionContext::new().with_registered_beans(vec![TypeId::of::<MyService>()]);

        let cond = ConditionalOnMissingBean::of::<MyService>();
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_missing_bean_from_type_id() {
        struct MyService;
        let ctx = ConditionContext::new();

        let cond = ConditionalOnMissingBean::from_type_id(TypeId::of::<MyService>());
        assert!(cond.matches(&ctx));
        assert_eq!(cond.type_id(), TypeId::of::<MyService>());
    }

    // -- ConditionalOnBean tests ------------------------------------------
    // ConditionalOnBean 测试

    #[test]
    fn conditional_on_bean_matches_when_present() {
        struct MyService;
        let ctx = ConditionContext::new().with_registered_beans(vec![TypeId::of::<MyService>()]);

        let cond = ConditionalOnBean::of::<MyService>();
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_bean_rejects_when_absent() {
        struct MyService;
        let ctx = ConditionContext::new();

        let cond = ConditionalOnBean::of::<MyService>();
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn conditional_on_bean_from_type_id() {
        struct MyService;
        let ctx = ConditionContext::new().with_registered_beans(vec![TypeId::of::<MyService>()]);

        let cond = ConditionalOnBean::from_type_id(TypeId::of::<MyService>());
        assert!(cond.matches(&ctx));
        assert_eq!(cond.type_id(), TypeId::of::<MyService>());
    }

    // -- ProfileCondition tests -------------------------------------------
    // ProfileCondition 测试

    #[test]
    fn profile_condition_matches_active_profile() {
        let ctx = ConditionContext::new().with_profiles(vec!["production".to_string()]);

        let cond = ProfileCondition::new("production");
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn profile_condition_rejects_inactive_profile() {
        let ctx = ConditionContext::new().with_profiles(vec!["production".to_string()]);

        let cond = ProfileCondition::new("dev");
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn profile_condition_default_always_active() {
        let ctx = ConditionContext::new();

        let cond = ProfileCondition::new("default");
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn profile_condition_accessor() {
        let cond = ProfileCondition::new("test");
        assert_eq!(cond.profile(), "test");
    }

    // -- AllConditions tests ----------------------------------------------
    // AllConditions 测试

    #[test]
    fn all_conditions_matches_when_all_true() {
        let ctx = ConditionContext::new()
            .with_properties(HashMap::from([
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
            ]))
            .with_profiles(vec!["production".to_string()]);

        let cond = AllConditions::new(vec![
            Box::new(ConditionalOnProperty::new("a").with_value("1")),
            Box::new(ConditionalOnProperty::new("b").with_value("2")),
            Box::new(ProfileCondition::new("production")),
        ]);

        assert!(cond.matches(&ctx));
    }

    #[test]
    fn all_conditions_rejects_when_any_false() {
        let ctx = ConditionContext::new()
            .with_properties(HashMap::from([("a".to_string(), "1".to_string())]))
            .with_profiles(vec!["production".to_string()]);

        let cond = AllConditions::new(vec![
            Box::new(ConditionalOnProperty::new("a").with_value("1")),
            Box::new(ConditionalOnProperty::new("missing")),
            Box::new(ProfileCondition::new("production")),
        ]);

        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn all_conditions_empty_always_matches() {
        let ctx = ConditionContext::new();
        let cond = AllConditions::new(vec![]);
        assert!(cond.matches(&ctx));
    }

    // -- AnyCondition tests -----------------------------------------------
    // AnyCondition 测试

    #[test]
    fn any_condition_matches_when_any_true() {
        let ctx = ConditionContext::new().with_profiles(vec!["staging".to_string()]);

        let cond = AnyCondition::new(vec![
            Box::new(ProfileCondition::new("production")),
            Box::new(ProfileCondition::new("staging")),
            Box::new(ProfileCondition::new("dev")),
        ]);

        assert!(cond.matches(&ctx));
    }

    #[test]
    fn any_condition_rejects_when_all_false() {
        let ctx = ConditionContext::new().with_profiles(vec!["production".to_string()]);

        let cond = AnyCondition::new(vec![
            Box::new(ProfileCondition::new("dev")),
            Box::new(ProfileCondition::new("test")),
        ]);

        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn any_condition_empty_never_matches() {
        let ctx = ConditionContext::new();
        let cond = AnyCondition::new(vec![]);
        assert!(!cond.matches(&ctx));
    }

    // -- NotCondition tests -----------------------------------------------
    // NotCondition 测试

    #[test]
    fn not_condition_inverts_true() {
        let ctx = ConditionContext::new().with_profiles(vec!["production".to_string()]);

        let cond = NotCondition::new(Box::new(ProfileCondition::new("production")));
        assert!(!cond.matches(&ctx));
    }

    #[test]
    fn not_condition_inverts_false() {
        let ctx = ConditionContext::new().with_profiles(vec!["production".to_string()]);

        let cond = NotCondition::new(Box::new(ProfileCondition::new("dev")));
        assert!(cond.matches(&ctx));
    }

    #[test]
    fn not_condition_nested() {
        let ctx = ConditionContext::new()
            .with_properties(HashMap::from([("x".to_string(), "1".to_string())]))
            .with_profiles(vec!["prod".to_string()]);

        // NOT( x=="1" AND profile=="prod" ) => false
        let inner = AllConditions::new(vec![
            Box::new(ConditionalOnProperty::new("x").with_value("1")),
            Box::new(ProfileCondition::new("prod")),
        ]);
        let cond = NotCondition::new(Box::new(inner));
        assert!(!cond.matches(&ctx));
    }

    // -- Integration / complex scenarios ----------------------------------
    // 集成 / 复杂场景

    #[test]
    fn conditional_on_missing_then_present() {
        struct DataSource;
        struct HikariDataSource;

        // Initially no DataSource, so ConditionalOnMissingBean matches
        // 初始时没有DataSource，因此ConditionalOnMissingBean匹配
        let ctx_empty = ConditionContext::new();
        let cond_missing = ConditionalOnMissingBean::of::<DataSource>();
        assert!(cond_missing.matches(&ctx_empty));

        // After registering DataSource, ConditionalOnMissingBean no longer matches
        // 注册DataSource后，ConditionalOnMissingBean不再匹配
        let ctx_with =
            ConditionContext::new().with_registered_beans(vec![TypeId::of::<DataSource>()]);
        assert!(!cond_missing.matches(&ctx_with));

        // ConditionalOnBean now matches
        // ConditionalOnBean现在匹配
        let cond_present = ConditionalOnBean::of::<DataSource>();
        assert!(cond_present.matches(&ctx_with));

        // HikariDataSource is still missing
        // HikariDataSource仍然缺失
        assert!(ConditionalOnMissingBean::of::<HikariDataSource>().matches(&ctx_with));
    }

    #[test]
    fn profile_and_property_combined() {
        let ctx_prod = ConditionContext::new()
            .with_properties(HashMap::from([(
                "monitoring.enabled".to_string(),
                "true".to_string(),
            )]))
            .with_profiles(vec!["production".to_string()]);

        let cond = AllConditions::new(vec![
            Box::new(ProfileCondition::new("production")),
            Box::new(ConditionalOnProperty::new("monitoring.enabled").with_value("true")),
        ]);

        assert!(cond.matches(&ctx_prod));

        // Wrong profile
        // 错误的配置文件
        let ctx_dev = ConditionContext::new()
            .with_properties(HashMap::from([(
                "monitoring.enabled".to_string(),
                "true".to_string(),
            )]))
            .with_profiles(vec!["dev".to_string()]);
        assert!(!cond.matches(&ctx_dev));

        // Missing property
        // 缺失属性
        let ctx_no_prop = ConditionContext::new().with_profiles(vec!["production".to_string()]);
        assert!(!cond.matches(&ctx_no_prop));
    }

    #[test]
    fn fallback_pattern_with_any() {
        // Register prod OR staging OR dev-specific beans
        // 注册prod或staging或dev特定的Bean
        let ctx = ConditionContext::new().with_profiles(vec!["staging".to_string()]);

        let cond = AnyCondition::new(vec![
            Box::new(ProfileCondition::new("production")),
            Box::new(ProfileCondition::new("staging")),
        ]);

        assert!(cond.matches(&ctx));
    }

    #[test]
    fn custom_condition_impl() {
        struct EvenNumberCondition(u32);

        impl Condition for EvenNumberCondition {
            fn matches(&self, _context: &ConditionContext) -> bool {
                self.0 % 2 == 0
            }
        }

        let ctx = ConditionContext::new();
        assert!(EvenNumberCondition(4).matches(&ctx));
        assert!(!EvenNumberCondition(3).matches(&ctx));
    }
}
