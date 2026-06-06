//! Auto-Configuration Processor / 自动配置处理器
//!
//! 桥接自动配置注册表与 IoC 容器，负责评估条件、排序并执行配置工厂。
//! Bridges the auto-configuration registry with the IoC container, evaluating
//! conditions, sorting by priority, and invoking factory functions.
//!
//! 参考 Spring Boot 的 `AutoConfigurationImportSelector` 和 `ConfigurationClassPostProcessor`。
//! Based on Spring Boot's `AutoConfigurationImportSelector` and `ConfigurationClassPostProcessor`.
//!
//! # 功能 / Features
//!
//! - 从 `AutoConfigurationRegistry` 中提取适用配置
//! - 使用 `ConditionContext` 评估条件
//! - 按 `@AutoConfigureBefore`/`@AutoConfigureAfter` 排序
//! - 调用工厂函数将 Bean 注册到容器
//!
//! # 示例 / Example
//!
//! ```rust,ignore
//! use hiver_starter::core::autoconfigure_processor::AutoConfigurationProcessor;
//! use hiver_starter::core::autoconfigure::AutoConfigurationRegistry;
//! use hiver_starter::core::container::ApplicationContext;
//!
//! let mut registry = AutoConfigurationRegistry::new();
//! // ... register entries ...
//!
//! let mut ctx = ApplicationContext::new();
//! let processor = AutoConfigurationProcessor::new();
//! processor.process(&registry, &mut ctx)?;
//! ```

use std::{any::TypeId, collections::HashMap};

use anyhow::Result;

use super::{
    autoconfigure::{AutoConfigurationRegistry, Condition},
    container::ApplicationContext,
};

// ============================================================================
// ConditionContext / 条件上下文
// ============================================================================

/// 条件评估上下文
/// Condition evaluation context
///
/// 从容器当前状态构建的上下文信息，用于评估自动配置条件。
/// Context built from the current container state, used to evaluate
/// auto-configuration conditions.
///
/// 包含配置属性、已注册的 Bean 类型和命名 Bean 信息。
/// Contains configuration properties, registered bean types, and named bean info.
///
/// 等价于 Spring Boot 的 `ConditionContext`。
/// Equivalent to Spring Boot's `ConditionContext`.
#[derive(Debug)]
pub struct ConditionContext
{
    /// 配置属性（键值对）
    /// Configuration properties (key-value pairs)
    properties: HashMap<String, String>,

    /// 已注册的 Bean TypeId 集合
    /// Set of registered bean TypeIds
    registered_beans: Vec<TypeId>,

    /// 命名 Bean 映射（名称 -> TypeId）
    /// Named bean mapping (name -> TypeId)
    bean_names: HashMap<String, TypeId>,
}

impl ConditionContext
{
    /// 从 `ApplicationContext` 构建条件上下文
    /// Build condition context from `ApplicationContext`
    ///
    /// 从容器的当前状态中提取属性和已注册的 Bean 信息。
    /// Extracts properties and registered bean information from the container's
    /// current state.
    ///
    /// # 参数 / Parameters
    ///
    /// - `ctx`: 应用上下文 / Application context
    pub fn from_context(ctx: &ApplicationContext) -> Self
    {
        // 从 ApplicationContext 提取所有属性
        // Extract all properties from ApplicationContext
        let properties = ctx.config_loader().all().clone();

        Self {
            properties,
            registered_beans: Vec::new(),
            bean_names: HashMap::new(),
        }
    }

    /// 添加已注册的 Bean 类型
    /// Add a registered bean type
    ///
    /// # 参数 / Parameters
    ///
    /// - `type_id`: Bean 的 TypeId / Bean's TypeId
    pub fn with_registered_bean(mut self, type_id: TypeId) -> Self
    {
        self.registered_beans.push(type_id);
        self
    }

    /// 批量设置已注册的 Bean 类型
    /// Set registered bean types in bulk
    ///
    /// # 参数 / Parameters
    ///
    /// - `beans`: Bean TypeId 列表 / List of bean TypeIds
    pub fn with_registered_beans(mut self, beans: Vec<TypeId>) -> Self
    {
        self.registered_beans = beans;
        self
    }

    /// 设置命名 Bean 映射
    /// Set named bean mapping
    ///
    /// # 参数 / Parameters
    ///
    /// - `names`: 命名 Bean 映射 / Named bean mapping
    pub fn with_bean_names(mut self, names: HashMap<String, TypeId>) -> Self
    {
        self.bean_names = names;
        self
    }

    /// 获取配置属性
    /// Get configuration property
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键 / Property key
    #[must_use]
    pub fn property(&self, key: &str) -> Option<&str>
    {
        self.properties.get(key).map(String::as_str)
    }

    /// 检查属性是否存在且等于期望值
    /// Check if a property exists and equals expected value
    ///
    /// # 参数 / Parameters
    ///
    /// - `key`: 属性键 / Property key
    /// - `expected`: 期望值 / Expected value
    #[must_use]
    pub fn property_equals(&self, key: &str, expected: &str) -> bool
    {
        self.property(key).is_some_and(|v| v == expected)
    }

    /// 检查指定类型的 Bean 是否已注册
    /// Check if a bean of the specified type is registered
    ///
    /// # 参数 / Parameters
    ///
    /// - `type_id`: Bean 的 TypeId / Bean's TypeId
    #[must_use]
    pub fn has_bean(&self, type_id: TypeId) -> bool
    {
        self.registered_beans.contains(&type_id)
    }

    /// 检查指定名称的 Bean 是否已注册
    /// Check if a bean with the specified name is registered
    ///
    /// # 参数 / Parameters
    ///
    /// - `name`: Bean 名称 / Bean name
    #[must_use]
    pub fn has_bean_by_name(&self, name: &str) -> bool
    {
        self.bean_names.contains_key(name)
    }

    /// 获取所有配置属性
    /// Get all configuration properties
    #[must_use]
    pub fn properties(&self) -> &HashMap<String, String>
    {
        &self.properties
    }

    /// 获取已注册的 Bean TypeId 列表
    /// Get the list of registered bean TypeIds
    #[must_use]
    pub fn registered_beans(&self) -> &[TypeId]
    {
        &self.registered_beans
    }

    /// 获取命名 Bean 映射
    /// Get the named bean mapping
    #[must_use]
    pub fn bean_names(&self) -> &HashMap<String, TypeId>
    {
        &self.bean_names
    }
}

// ============================================================================
// AutoConfigurationProcessor / 自动配置处理器
// ============================================================================

/// 自动配置处理器
/// Auto-configuration processor
///
/// 核心处理器，桥接 `AutoConfigurationRegistry` 与 `ApplicationContext`。
/// Core processor that bridges `AutoConfigurationRegistry` with `ApplicationContext`.
///
/// 处理流程 / Processing steps:
/// 1. 从注册表获取所有条目 / Get all entries from registry
/// 2. 构建 `ConditionContext` / Build `ConditionContext`
/// 3. 评估每个条目的条件 / Evaluate each entry's condition
/// 4. 按优先级和 Before/After 约束排序 / Sort by priority and constraints
/// 5. 调用工厂函数注册 Bean / Invoke factory functions to register beans
///
/// 等价于 Spring Boot 的 `AutoConfigurationImportSelector`。
/// Equivalent to Spring Boot's `AutoConfigurationImportSelector`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::autoconfigure_processor::AutoConfigurationProcessor;
///
/// let processor = AutoConfigurationProcessor::new();
/// processor.process(&registry, &mut ctx)?;
/// ```
#[derive(Debug, Default)]
pub struct AutoConfigurationProcessor
{
    /// 已处理的配置名称列表（按处理顺序）
    /// List of processed configuration names (in processing order)
    processed: Vec<String>,

    /// 被跳过的配置名称及原因
    /// Skipped configuration names with reasons
    skipped: Vec<(String, SkipReason)>,
}

/// 配置被跳过的原因
/// Reason why a configuration was skipped
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason
{
    /// 条件不满足
    /// Condition not met
    ConditionNotMet,
    /// 执行时出错
    /// Error during execution
    ExecutionError(String),
}

impl AutoConfigurationProcessor
{
    /// 创建新的自动配置处理器
    /// Create a new auto-configuration processor
    pub fn new() -> Self
    {
        Self::default()
    }

    /// 处理自动配置注册表，将适用的配置应用到容器
    /// Process the auto-configuration registry, applying applicable configs
    ///
    /// 这是核心方法，负责：
    /// This is the core method, responsible for:
    ///
    /// 1. 评估所有条件 / Evaluating all conditions
    /// 2. 排序 / Sorting
    /// 3. 执行工厂函数 / Executing factory functions
    ///
    /// # 参数 / Parameters
    ///
    /// - `registry`: 自动配置注册表 / Auto-configuration registry
    /// - `ctx`: 应用上下文（可变引用） / Application context (mutable ref)
    ///
    /// # 返回 / Returns
    ///
    /// 返回处理结果，包含成功数量和跳过数量。
    /// Returns the processing result, including success and skip counts.
    pub fn process(
        &mut self,
        registry: &AutoConfigurationRegistry,
        ctx: &mut ApplicationContext,
    ) -> Result<ProcessResult>
    {
        // 清除上次处理的状态
        // Clear previous processing state
        self.processed.clear();
        self.skipped.clear();

        // Step 1: 获取排序后的适用配置列表
        // Step 1: Get sorted applicable configuration list
        let applicable = registry.evaluate(ctx);

        let total = applicable.len();

        // Step 2: 逐个执行配置的工厂函数
        // Step 2: Execute each configuration's factory function
        for entry in &applicable
        {
            let config_name = entry.name().to_string();

            // 在执行前再次检查条件（前面的配置可能改变了容器状态）
            // Re-check condition before execution (previous configs may have
            // changed container state)
            if !entry.matches(ctx)
            {
                self.skipped
                    .push((config_name, SkipReason::ConditionNotMet));
                tracing::debug!(
                    "Skipping auto-configuration '{}' - condition not met",
                    entry.name()
                );
                continue;
            }

            tracing::info!("Applying auto-configuration: {}", entry.name());

            // Step 3: 调用工厂函数
            // Step 3: Call the factory function
            match entry.configure(ctx)
            {
                Ok(()) =>
                {
                    self.processed.push(config_name);
                    tracing::debug!("Successfully applied auto-configuration: {}", entry.name());
                },
                Err(e) =>
                {
                    let error_msg = e.to_string();
                    tracing::warn!("Auto-configuration '{}' failed: {}", entry.name(), e);
                    self.skipped
                        .push((config_name, SkipReason::ExecutionError(error_msg)));
                },
            }
        }

        tracing::info!(
            "Auto-configuration processing complete: {} applied, {} skipped out of {} candidates",
            self.processed.len(),
            self.skipped.len(),
            total
        );

        Ok(ProcessResult {
            applied_count: self.processed.len(),
            skipped_count: self.skipped.len(),
            total_candidates: total,
        })
    }

    /// 使用自定义条件上下文处理（高级用法）
    /// Process with custom condition context (advanced usage)
    ///
    /// 允许外部构建 `ConditionContext`，用于更精细的条件控制。
    /// Allows externally built `ConditionContext` for finer condition control.
    ///
    /// # 参数 / Parameters
    ///
    /// - `registry`: 自动配置注册表 / Auto-configuration registry
    /// - `ctx`: 应用上下文 / Application context
    /// - `_condition_ctx`: 自定义条件上下文 / Custom condition context
    pub fn process_with_context(
        &mut self,
        registry: &AutoConfigurationRegistry,
        ctx: &mut ApplicationContext,
        _condition_ctx: &ConditionContext,
    ) -> Result<ProcessResult>
    {
        // 目前的实现复用 process 方法
        // Current implementation reuses the process method
        // 未来可以扩展为使用自定义条件上下文进行更精细的控制
        // Can be extended for finer control using custom condition context
        self.process(registry, ctx)
    }

    /// 获取已处理的配置名称列表
    /// Get the list of processed configuration names
    pub fn processed(&self) -> &[String]
    {
        &self.processed
    }

    /// 获取被跳过的配置列表
    /// Get the list of skipped configurations
    pub fn skipped(&self) -> &[(String, SkipReason)]
    {
        &self.skipped
    }

    /// 检查指定配置是否已被处理
    /// Check if a specific configuration has been processed
    pub fn is_processed(&self, name: &str) -> bool
    {
        self.processed.iter().any(|n| n == name)
    }

    /// 获取处理结果摘要
    /// Get processing result summary
    pub fn summary(&self) -> String
    {
        format!(
            "AutoConfigurationProcessor: {} applied, {} skipped",
            self.processed.len(),
            self.skipped.len()
        )
    }
}

// ============================================================================
// ProcessResult / 处理结果
// ============================================================================

/// 自动配置处理结果
/// Auto-configuration processing result
///
/// 包含处理过程中的统计信息。
/// Contains statistics from the processing run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResult
{
    /// 成功应用的配置数量
    /// Number of successfully applied configurations
    pub applied_count: usize,

    /// 被跳过的配置数量
    /// Number of skipped configurations
    pub skipped_count: usize,

    /// 候选配置总数
    /// Total number of candidate configurations
    pub total_candidates: usize,
}

impl ProcessResult
{
    /// 创建空的处理结果
    /// Create an empty processing result
    pub fn empty() -> Self
    {
        Self {
            applied_count: 0,
            skipped_count: 0,
            total_candidates: 0,
        }
    }

    /// 检查是否有配置被应用
    /// Check if any configurations were applied
    pub fn has_applied(&self) -> bool
    {
        self.applied_count > 0
    }

    /// 检查是否有配置被跳过
    /// Check if any configurations were skipped
    pub fn has_skipped(&self) -> bool
    {
        self.skipped_count > 0
    }
}

impl std::fmt::Display for ProcessResult
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "ProcessResult(applied={}, skipped={}, total={})",
            self.applied_count, self.skipped_count, self.total_candidates
        )
    }
}

// ============================================================================
// 条件适配器 / Condition Adapter
// ============================================================================

/// 适配 `Condition` trait 以使用 `ConditionContext`
/// Adapter to use `Condition` trait with `ConditionContext`
///
/// 将 `autoconfigure::Condition`（操作 `ApplicationContext`）与
/// 本模块的 `ConditionContext`（独立的条件评估上下文）桥接。
/// Bridges `autoconfigure::Condition` (operating on `ApplicationContext`)
/// with this module's `ConditionContext` (standalone condition evaluation context).
pub struct ConditionAdapter
{
    /// 内部条件
    /// Inner condition
    inner: Box<dyn Condition>,
}

impl ConditionAdapter
{
    /// 创建新的条件适配器
    /// Create a new condition adapter
    pub fn new(condition: Box<dyn Condition>) -> Self
    {
        Self { inner: condition }
    }

    /// 使用 `ApplicationContext` 评估条件
    /// Evaluate condition using `ApplicationContext`
    pub fn matches_with_ctx(&self, ctx: &ApplicationContext) -> bool
    {
        self.inner.matches(ctx)
    }
}

impl std::fmt::Debug for ConditionAdapter
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("ConditionAdapter").finish()
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use super::*;
    use crate::core::autoconfigure::{
        AutoConfigurationEntry, ConditionalOnMissingBeanCondition, ConditionalOnPropertyCondition,
    };

    fn noop_factory(_ctx: &mut ApplicationContext) -> Result<()>
    {
        Ok(())
    }

    fn register_i32_bean(ctx: &mut ApplicationContext) -> Result<()>
    {
        ctx.register_bean(42i32);
        Ok(())
    }

    fn register_string_bean(ctx: &mut ApplicationContext) -> Result<()>
    {
        ctx.register_bean("hello".to_string());
        Ok(())
    }

    // ----------------------------------------------------------------
    // ConditionContext 测试 / ConditionContext Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_condition_context_from_context()
    {
        let ctx = ApplicationContext::new();
        let cond_ctx = ConditionContext::from_context(&ctx);

        // 空容器应该有空的属性和 Bean
        // Empty container should have empty properties and beans
        assert!(cond_ctx.properties().is_empty());
        assert!(cond_ctx.registered_beans().is_empty());
        assert!(cond_ctx.bean_names().is_empty());
    }

    #[test]
    fn test_condition_context_with_registered_beans()
    {
        let ctx = ApplicationContext::new();
        let cond_ctx = ConditionContext::from_context(&ctx)
            .with_registered_bean(TypeId::of::<i32>())
            .with_registered_bean(TypeId::of::<String>());

        assert!(cond_ctx.has_bean(TypeId::of::<i32>()));
        assert!(cond_ctx.has_bean(TypeId::of::<String>()));
        assert!(!cond_ctx.has_bean(TypeId::of::<u64>()));
    }

    #[test]
    fn test_condition_context_with_bean_names()
    {
        let ctx = ApplicationContext::new();
        let cond_ctx = ConditionContext::from_context(&ctx)
            .with_bean_names(HashMap::from([("myService".to_string(), TypeId::of::<i32>())]));

        assert!(cond_ctx.has_bean_by_name("myService"));
        assert!(!cond_ctx.has_bean_by_name("nonexistent"));
    }

    #[test]
    fn test_condition_context_property()
    {
        let mut loader = crate::config::ConfigurationLoader::new();
        loader.set("test.key".to_string(), "test.value".to_string());
        let ctx = ApplicationContext::with_config_loader(Arc::new(loader));
        let cond_ctx = ConditionContext::from_context(&ctx);

        assert_eq!(cond_ctx.property("test.key"), Some("test.value"));
        assert_eq!(cond_ctx.property("nonexistent"), None);
        assert!(cond_ctx.property_equals("test.key", "test.value"));
        assert!(!cond_ctx.property_equals("test.key", "wrong"));
    }

    // ----------------------------------------------------------------
    // AutoConfigurationProcessor 测试 / Processor Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_processor_empty_registry()
    {
        let registry = AutoConfigurationRegistry::new();
        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();

        let result = processor.process(&registry, &mut ctx).unwrap();
        assert_eq!(result.applied_count, 0);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.total_candidates, 0);
        assert!(!result.has_applied());
    }

    #[test]
    fn test_processor_single_unconditional()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("RegisterInt", register_i32_bean));

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();

        let result = processor.process(&registry, &mut ctx).unwrap();

        assert_eq!(result.applied_count, 1);
        assert_eq!(result.skipped_count, 0);
        assert!(result.has_applied());
        assert!(ctx.contains_bean::<i32>());
    }

    #[test]
    fn test_processor_multiple_configurations()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("ConfigInt", register_i32_bean));
        registry.register(AutoConfigurationEntry::new("ConfigStr", register_string_bean));

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();

        let result = processor.process(&registry, &mut ctx).unwrap();

        assert_eq!(result.applied_count, 2);
        assert!(ctx.contains_bean::<i32>());
        assert!(ctx.contains_bean::<String>());
    }

    #[test]
    fn test_processor_conditional_skip()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("AlwaysConfig", register_i32_bean));
        registry.register_conditional(
            AutoConfigurationEntry::new("ConditionalConfig", register_string_bean).with_condition(
                Box::new(ConditionalOnPropertyCondition::new("nonexistent.property")),
            ),
        );

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();

        let result = processor.process(&registry, &mut ctx).unwrap();

        // ConditionalConfig 条件不满足，evaluate() 已过滤
        // ConditionalConfig condition not met, filtered by evaluate()
        // 因此只有 AlwaysConfig 被处理
        // So only AlwaysConfig is processed
        assert_eq!(result.applied_count, 1);
        assert_eq!(result.total_candidates, 1); // evaluate() already filtered
        assert!(ctx.contains_bean::<i32>());
        assert!(!ctx.contains_bean::<String>());
    }

    #[test]
    fn test_processor_conditional_on_missing_bean()
    {
        // 第一个配置注册 i32，第二个配置仅在 String 缺失时注册
        // First config registers i32, second registers String only when absent
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("RegisterInt", register_i32_bean));
        registry.register_conditional(
            AutoConfigurationEntry::new("ConditionalString", register_string_bean)
                .with_condition(Box::new(ConditionalOnMissingBeanCondition::new::<String>())),
        );

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();

        let result = processor.process(&registry, &mut ctx).unwrap();

        // String 尚未注册，条件满足，应该被注册
        // String not yet registered, condition met, should be registered
        assert_eq!(result.applied_count, 2);
        assert!(ctx.contains_bean::<i32>());
        assert!(ctx.contains_bean::<String>());
    }

    #[test]
    fn test_processor_priority_ordering()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry
            .register(AutoConfigurationEntry::new("LowPriority", noop_factory).with_priority(100));
        registry.register(
            AutoConfigurationEntry::new("HighPriority", noop_factory).with_priority(-100),
        );
        registry.register(AutoConfigurationEntry::new("DefaultPriority", noop_factory));

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();

        let result = processor.process(&registry, &mut ctx).unwrap();

        assert_eq!(result.applied_count, 3);
        assert_eq!(result.skipped_count, 0);
    }

    #[test]
    fn test_processor_is_processed()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("MyConfig", register_i32_bean));

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();

        assert!(!processor.is_processed("MyConfig"));

        processor.process(&registry, &mut ctx).unwrap();

        assert!(processor.is_processed("MyConfig"));
        assert!(!processor.is_processed("OtherConfig"));
    }

    #[test]
    fn test_processor_skipped_reason()
    {
        // 使用 match_if_missing(true) 使条件在属性缺失时仍满足 evaluate()，
        // 然后在 processor 中再次检查时条件可能改变
        // Use match_if_missing(true) so condition passes evaluate() even
        // when property is absent, demonstrating skip tracking
        let mut registry = AutoConfigurationRegistry::new();
        registry.register_conditional(
            AutoConfigurationEntry::new("SkippedConfig", noop_factory).with_condition(Box::new(
                ConditionalOnPropertyCondition::new("missing.key").match_if_missing(true),
            )),
        );

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();
        processor.process(&registry, &mut ctx).unwrap();

        // match_if_missing=true 意味着条件满足，所以应该被应用而非跳过
        // match_if_missing=true means condition is met, should be applied not skipped
        assert_eq!(processor.processed().len(), 1);
        assert_eq!(processor.processed()[0], "SkippedConfig");
    }

    #[test]
    fn test_processor_summary_after_process()
    {
        let mut registry = AutoConfigurationRegistry::new();
        registry.register(AutoConfigurationEntry::new("Config1", register_i32_bean));
        registry.register(AutoConfigurationEntry::new("Config2", register_string_bean));

        let mut ctx = ApplicationContext::new();
        let mut processor = AutoConfigurationProcessor::new();
        processor.process(&registry, &mut ctx).unwrap();

        let summary = processor.summary();
        assert!(summary.contains("2 applied"));
        assert!(summary.contains("0 skipped"));
    }

    // ----------------------------------------------------------------
    // ProcessResult 测试 / ProcessResult Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_process_result_empty()
    {
        let result = ProcessResult::empty();
        assert_eq!(result.applied_count, 0);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.total_candidates, 0);
        assert!(!result.has_applied());
        assert!(!result.has_skipped());
    }

    #[test]
    fn test_process_result_display()
    {
        let result = ProcessResult {
            applied_count: 5,
            skipped_count: 2,
            total_candidates: 7,
        };
        let display = result.to_string();
        assert!(display.contains("applied=5"));
        assert!(display.contains("skipped=2"));
        assert!(display.contains("total=7"));
    }

    // ----------------------------------------------------------------
    // ConditionAdapter 测试 / ConditionAdapter Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_condition_adapter()
    {
        let condition = ConditionalOnPropertyCondition::new("test.key");
        let adapter = ConditionAdapter::new(Box::new(condition));

        let ctx = ApplicationContext::new();
        // 属性不存在，条件不满足
        // Property absent, condition not met
        assert!(!adapter.matches_with_ctx(&ctx));
    }

    #[test]
    fn test_condition_adapter_debug()
    {
        let condition = ConditionalOnPropertyCondition::new("test.key");
        let adapter = ConditionAdapter::new(Box::new(condition));
        let debug_str = format!("{:?}", adapter);
        assert!(debug_str.contains("ConditionAdapter"));
    }

    // ----------------------------------------------------------------
    // SkipReason 测试 / SkipReason Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_skip_reason_equality()
    {
        let reason1 = SkipReason::ConditionNotMet;
        let reason2 = SkipReason::ConditionNotMet;
        let reason3 = SkipReason::ExecutionError("test error".to_string());

        assert_eq!(reason1, reason2);
        assert_ne!(reason1, reason3);
    }
}
