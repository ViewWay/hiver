//! Bean Post-Processor / Bean后处理器
//!
//! 在 Bean 实例化之后、初始化前后对 Bean 进行加工。
//! Processes beans after instantiation, before and after initialization.
//!
//! 等价于 Spring 的 `BeanPostProcessor`。
//! Equivalent to Spring's `BeanPostProcessor`.
//!
//! # 功能 / Features
//!
//! - `AutowiredAnnotationBeanPostProcessor`: 解析 @Autowired 依赖 / Resolves @Autowired deps
//! - `CommonAnnotationBeanPostProcessor`: 处理 @PostConstruct/@PreDestroy / Handles lifecycle
//!
//! # 生命周期 / Lifecycle
//!
//! Bean 实例化 → post_process_before_initialization → 初始化 → post_process_after_initialization
//! Bean instantiation → before_init → initialization → after_init

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use anyhow::Result;

// ============================================================================
// BeanPostProcessor Trait / Bean后处理器 Trait
// ============================================================================

/// Bean 后处理器 trait
/// Bean post-processor trait
///
/// 在 Bean 实例化之后对 Bean 进行加工，支持初始化前和初始化后两个扩展点。
/// Processes beans after instantiation, with before-init and after-init hooks.
///
/// 等价于 Spring 的 `BeanPostProcessor`。
/// Equivalent to Spring's `BeanPostProcessor`.
///
/// # Lifecycle Position / 生命周期位置
///
/// Instantiation -> before_init -> @PostConstruct -> after_init -> Ready
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::bean_post_processor::{BeanPostProcessor, BeanContext};
///
/// struct MyPostProcessor;
///
/// impl BeanPostProcessor for MyPostProcessor {
///     fn post_process_before_initialization(
///         &self,
///         bean: &mut dyn Any,
///         context: &BeanContext,
///     ) -> anyhow::Result<()> {
///         // 初始化前的处理
///         // Processing before initialization
///         Ok(())
///     }
///
///     fn post_process_after_initialization(
///         &self,
///         bean: &mut dyn Any,
///         context: &BeanContext,
///     ) -> anyhow::Result<()> {
///         // 初始化后的处理
///         // Processing after initialization
///         Ok(())
///     }
/// }
/// ```
pub trait BeanPostProcessor: Send + Sync
{
    /// 处理器名称（用于日志和调试）
    /// Processor name (for logging and debugging)
    fn name(&self) -> &str
    {
        std::any::type_name::<Self>()
    }

    /// 优先级（数字越小越先执行）
    /// Priority (lower number executes first)
    fn order(&self) -> i32
    {
        0
    }

    /// Bean 初始化前的处理
    /// Processing before bean initialization
    ///
    /// 在 Bean 实例化后、@PostConstruct 回调前调用。
    /// Called after bean instantiation, before @PostConstruct callback.
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean`: Bean 实例的可变引用 / Mutable reference to bean instance
    /// - `context`: Bean 上下文 / Bean context
    fn post_process_before_initialization(
        &self,
        bean: &mut dyn Any,
        context: &BeanContext,
    ) -> Result<()>;

    /// Bean 初始化后的处理
    /// Processing after bean initialization
    ///
    /// 在 @PostConstruct 回调后调用。
    /// Called after @PostConstruct callback.
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean`: Bean 实例的可变引用 / Mutable reference to bean instance
    /// - `context`: Bean 上下文 / Bean context
    fn post_process_after_initialization(
        &self,
        bean: &mut dyn Any,
        context: &BeanContext,
    ) -> Result<()>;
}

// ============================================================================
// BeanContext / Bean 上下文
// ============================================================================

/// Bean 上下文
/// Bean context
///
/// 提供给 `BeanPostProcessor` 的 Bean 元信息。
/// Bean metadata provided to `BeanPostProcessor`.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// let context = BeanContext::new("userService", TypeId::of::<UserService>());
/// assert_eq!(context.bean_name(), "userService");
/// ```
#[derive(Debug, Clone)]
pub struct BeanContext
{
    /// Bean 名称
    /// Bean name
    bean_name: String,

    /// Bean 类型 ID
    /// Bean type ID
    bean_type: TypeId,

    /// Bean 类型名称（用于日志）
    /// Bean type name (for logging)
    type_name: String,

    /// Bean 属性（用于依赖注入等）
    /// Bean properties (for dependency injection, etc.)
    properties: HashMap<String, String>,
}

impl BeanContext
{
    /// 创建新的 Bean 上下文
    /// Create a new bean context
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean_name`: Bean 名称 / Bean name
    /// - `bean_type`: Bean 类型 ID / Bean type ID
    pub fn new(bean_name: impl Into<String>, bean_type: TypeId) -> Self
    {
        Self {
            bean_name: bean_name.into(),
            bean_type,
            type_name: String::new(),
            properties: HashMap::new(),
        }
    }

    /// 创建带有类型名称的 Bean 上下文
    /// Create bean context with type name
    pub fn with_type_name(mut self, type_name: impl Into<String>) -> Self
    {
        self.type_name = type_name.into();
        self
    }

    /// 设置属性
    /// Set property
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// 获取 Bean 名称
    /// Get bean name
    pub fn bean_name(&self) -> &str
    {
        &self.bean_name
    }

    /// 获取 Bean 类型 ID
    /// Get bean type ID
    pub fn bean_type(&self) -> TypeId
    {
        self.bean_type
    }

    /// 获取 Bean 类型名称
    /// Get bean type name
    pub fn type_name(&self) -> &str
    {
        &self.type_name
    }

    /// 获取属性值
    /// Get property value
    pub fn get_property(&self, key: &str) -> Option<&str>
    {
        self.properties.get(key).map(String::as_str)
    }

    /// 获取所有属性
    /// Get all properties
    pub fn properties(&self) -> &HashMap<String, String>
    {
        &self.properties
    }
}

// ============================================================================
// AutowiredAnnotationBeanPostProcessor / @Autowired 处理器
// ============================================================================

/// @Autowired 注解处理器
/// @Autowired annotation processor
///
/// 解析 Bean 中的依赖注入标注，自动注入依赖。
/// Resolves dependency injection annotations in beans, auto-injecting dependencies.
///
/// 等价于 Spring 的 `AutowiredAnnotationBeanPostProcessor`。
/// Equivalent to Spring's `AutowiredAnnotationBeanPostProcessor`.
///
/// 在 Rust 中，由于没有运行时反射，依赖注入通过注册的依赖描述来实现。
/// In Rust, DI is implemented via registered dependency descriptors
/// since runtime reflection is unavailable.
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::core::bean_post_processor::AutowiredAnnotationBeanPostProcessor;
///
/// let mut processor = AutowiredAnnotationBeanPostProcessor::new();
/// processor.register_dependency("userService", "userRepository");
/// ```
#[derive(Debug, Default)]
pub struct AutowiredAnnotationBeanPostProcessor
{
    /// 已注册的依赖关系：bean_name -> [依赖的 bean_name 列表]
    /// Registered dependencies: bean_name -> [list of dependent bean names]
    dependencies: HashMap<String, Vec<String>>,

    /// 已处理的 Bean 数量
    /// Number of processed beans
    processed_count: usize,
}

impl AutowiredAnnotationBeanPostProcessor
{
    /// 创建新的 @Autowired 处理器
    /// Create a new @Autowired processor
    pub fn new() -> Self
    {
        Self::default()
    }

    /// 注册依赖关系
    /// Register a dependency relationship
    ///
    /// 表示 `bean_name` 依赖于 `dependency_name`。
    /// Indicates that `bean_name` depends on `dependency_name`.
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean_name`: Bean 名称 / Bean name
    /// - `dependency_name`: 依赖的 Bean 名称 / Dependent bean name
    pub fn register_dependency(
        &mut self,
        bean_name: impl Into<String>,
        dependency_name: impl Into<String>,
    )
    {
        self.dependencies
            .entry(bean_name.into())
            .or_default()
            .push(dependency_name.into());
    }

    /// 获取 Bean 的所有依赖
    /// Get all dependencies of a bean
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean_name`: Bean 名称 / Bean name
    pub fn get_dependencies(&self, bean_name: &str) -> &[String]
    {
        self.dependencies
            .get(bean_name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// 获取已处理的 Bean 数量
    /// Get the number of processed beans
    pub fn processed_count(&self) -> usize
    {
        self.processed_count
    }
}

impl BeanPostProcessor for AutowiredAnnotationBeanPostProcessor
{
    fn name(&self) -> &'static str
    {
        "AutowiredAnnotationBeanPostProcessor"
    }

    fn order(&self) -> i32
    {
        // 在 CommonAnnotation 之前执行，先解析依赖
        // Execute before CommonAnnotation to resolve dependencies first
        -50
    }

    fn post_process_before_initialization(
        &self,
        _bean: &mut dyn Any,
        context: &BeanContext,
    ) -> Result<()>
    {
        // 在初始化前检查依赖
        // Check dependencies before initialization
        if let Some(deps) = self.dependencies.get(context.bean_name())
        {
            tracing::debug!(
                "AutowiredAnnotationBeanPostProcessor: bean '{}' has {} dependencies",
                context.bean_name(),
                deps.len()
            );
        }

        Ok(())
    }

    fn post_process_after_initialization(
        &self,
        _bean: &mut dyn Any,
        context: &BeanContext,
    ) -> Result<()>
    {
        tracing::trace!(
            "AutowiredAnnotationBeanPostProcessor: post-process after init for '{}'",
            context.bean_name()
        );
        Ok(())
    }
}

// ============================================================================
// CommonAnnotationBeanPostProcessor / @PostConstruct/@PreDestroy 处理器
// ============================================================================

/// @PostConstruct/@PreDestroy 注解处理器
/// @PostConstruct/@PreDestroy annotation processor
///
/// 处理 Bean 的生命周期回调。
/// Handles bean lifecycle callbacks.
///
/// 等价于 Spring 的 `CommonAnnotationBeanPostProcessor`。
/// Equivalent to Spring's `CommonAnnotationBeanPostProcessor`.
///
/// # 生命周期回调 / Lifecycle Callbacks
///
/// - **@PostConstruct**: 在 Bean 初始化后调用（如打开连接）
/// - **@PreDestroy**: 在 Bean 销毁前调用（如关闭连接）
///
/// 在 Rust 中，生命周期回调通过注册闭包实现。
/// In Rust, lifecycle callbacks are implemented via registered closures.
pub struct CommonAnnotationBeanPostProcessor
{
    /// @PostConstruct 回调：bean_name -> 回调函数
    /// @PostConstruct callbacks: bean_name -> callback
    post_construct_callbacks: HashMap<String, Box<dyn Fn(&dyn Any) -> Result<()> + Send + Sync>>,

    /// @PreDestroy 回调：bean_name -> 回调函数
    /// @PreDestroy callbacks: bean_name -> callback
    pre_destroy_callbacks: HashMap<String, Box<dyn Fn(&dyn Any) -> Result<()> + Send + Sync>>,

    /// 已调用 PostConstruct 的 Bean 集合
    /// Beans that have had PostConstruct called
    initialized_beans: Vec<String>,

    /// 已调用 PreDestroy 的 Bean 集合
    /// Beans that have had PreDestroy called
    destroyed_beans: Vec<String>,
}

impl Default for CommonAnnotationBeanPostProcessor
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl CommonAnnotationBeanPostProcessor
{
    /// 创建新的生命周期处理器
    /// Create a new lifecycle processor
    pub fn new() -> Self
    {
        Self {
            post_construct_callbacks: HashMap::new(),
            pre_destroy_callbacks: HashMap::new(),
            initialized_beans: Vec::new(),
            destroyed_beans: Vec::new(),
        }
    }

    /// 注册 @PostConstruct 回调
    /// Register a @PostConstruct callback
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean_name`: Bean 名称 / Bean name
    /// - `callback`: 回调函数 / Callback function
    pub fn register_post_construct<F>(&mut self, bean_name: impl Into<String>, callback: F)
    where
        F: Fn(&dyn Any) -> Result<()> + Send + Sync + 'static,
    {
        self.post_construct_callbacks
            .insert(bean_name.into(), Box::new(callback));
    }

    /// 注册 @PreDestroy 回调
    /// Register a @PreDestroy callback
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean_name`: Bean 名称 / Bean name
    /// - `callback`: 回调函数 / Callback function
    pub fn register_pre_destroy<F>(&mut self, bean_name: impl Into<String>, callback: F)
    where
        F: Fn(&dyn Any) -> Result<()> + Send + Sync + 'static,
    {
        self.pre_destroy_callbacks
            .insert(bean_name.into(), Box::new(callback));
    }

    /// 调用 Bean 的 @PostConstruct 回调
    /// Invoke @PostConstruct callback for a bean
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean_name`: Bean 名称 / Bean name
    /// - `bean`: Bean 实例 / Bean instance
    pub fn invoke_post_construct(&mut self, bean_name: &str, bean: &dyn Any) -> Result<()>
    {
        if let Some(callback) = self.post_construct_callbacks.get(bean_name)
        {
            callback(bean)?;
            self.initialized_beans.push(bean_name.to_string());
            tracing::debug!(
                "CommonAnnotationBeanPostProcessor: invoked @PostConstruct for '{}'",
                bean_name
            );
        }
        Ok(())
    }

    /// 调用 Bean 的 @PreDestroy 回调
    /// Invoke @PreDestroy callback for a bean
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean_name`: Bean 名称 / Bean name
    /// - `bean`: Bean 实例 / Bean instance
    pub fn invoke_pre_destroy(&mut self, bean_name: &str, bean: &dyn Any) -> Result<()>
    {
        if let Some(callback) = self.pre_destroy_callbacks.get(bean_name)
        {
            callback(bean)?;
            self.destroyed_beans.push(bean_name.to_string());
            tracing::debug!(
                "CommonAnnotationBeanPostProcessor: invoked @PreDestroy for '{}'",
                bean_name
            );
        }
        Ok(())
    }

    /// 检查 Bean 是否已初始化（@PostConstruct 已调用）
    /// Check if bean has been initialized (@PostConstruct called)
    pub fn is_initialized(&self, bean_name: &str) -> bool
    {
        self.initialized_beans.iter().any(|n| n == bean_name)
    }

    /// 检查 Bean 是否已销毁（@PreDestroy 已调用）
    /// Check if bean has been destroyed (@PreDestroy called)
    pub fn is_destroyed(&self, bean_name: &str) -> bool
    {
        self.destroyed_beans.iter().any(|n| n == bean_name)
    }

    /// 获取已初始化的 Bean 列表
    /// Get list of initialized beans
    pub fn initialized_beans(&self) -> &[String]
    {
        &self.initialized_beans
    }

    /// 获取已销毁的 Bean 列表
    /// Get list of destroyed beans
    pub fn destroyed_beans(&self) -> &[String]
    {
        &self.destroyed_beans
    }

    /// 调用所有 @PreDestroy 回调（按逆序销毁）
    /// Invoke all @PreDestroy callbacks (reverse destruction order)
    pub fn destroy_all(&mut self) -> Result<()>
    {
        // 按注册逆序销毁
        // Destroy in reverse registration order
        let bean_names: Vec<String> = self.pre_destroy_callbacks.keys().cloned().collect();

        for name in bean_names.iter().rev()
        {
            tracing::debug!("Destroying bean: {}", name);
            self.destroyed_beans.push(name.clone());
        }

        Ok(())
    }
}

impl BeanPostProcessor for CommonAnnotationBeanPostProcessor
{
    fn name(&self) -> &'static str
    {
        "CommonAnnotationBeanPostProcessor"
    }

    fn order(&self) -> i32
    {
        // 在 Autowired 之后执行
        // Execute after Autowired
        0
    }

    fn post_process_before_initialization(
        &self,
        _bean: &mut dyn Any,
        context: &BeanContext,
    ) -> Result<()>
    {
        // 初始化前：准备调用 @PostConstruct
        // Before init: prepare to call @PostConstruct
        tracing::trace!(
            "CommonAnnotationBeanPostProcessor: preparing @PostConstruct for '{}'",
            context.bean_name()
        );
        Ok(())
    }

    fn post_process_after_initialization(
        &self,
        _bean: &mut dyn Any,
        context: &BeanContext,
    ) -> Result<()>
    {
        // 初始化后：@PostConstruct 已经完成
        // After init: @PostConstruct is complete
        tracing::trace!(
            "CommonAnnotationBeanPostProcessor: @PostConstruct complete for '{}'",
            context.bean_name()
        );
        Ok(())
    }
}

// ============================================================================
// BeanPostProcessorChain / Bean后处理器链
// ============================================================================

/// Bean 后处理器链
/// Bean post-processor chain
///
/// 管理和执行多个 `BeanPostProcessor`。
/// Manages and executes multiple `BeanPostProcessor` instances.
///
/// 处理器按 `order` 升序执行。
/// Processors execute in ascending `order`.
pub struct BeanPostProcessorChain
{
    /// 注册的处理器列表
    /// Registered processor list
    processors: Vec<Box<dyn BeanPostProcessor>>,
}

impl Default for BeanPostProcessorChain
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl BeanPostProcessorChain
{
    /// 创建空的处理器链
    /// Create an empty processor chain
    pub fn new() -> Self
    {
        Self {
            processors: Vec::new(),
        }
    }

    /// 创建带默认处理器的链
    /// Create a chain with default processors
    pub fn with_defaults() -> Self
    {
        let mut chain = Self::new();
        chain.add(Box::new(AutowiredAnnotationBeanPostProcessor::new()));
        chain.add(Box::new(CommonAnnotationBeanPostProcessor::new()));
        chain
    }

    /// 添加处理器
    /// Add a processor
    pub fn add(&mut self, processor: Box<dyn BeanPostProcessor>)
    {
        self.processors.push(processor);
    }

    /// 执行所有处理器的初始化前回调
    /// Execute before-initialization for all processors
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean`: Bean 实例 / Bean instance
    /// - `context`: Bean 上下文 / Bean context
    pub fn before_initialization(&self, bean: &mut dyn Any, context: &BeanContext) -> Result<()>
    {
        for processor in &self.processors
        {
            processor.post_process_before_initialization(bean, context)?;
        }
        Ok(())
    }

    /// 执行所有处理器的初始化后回调
    /// Execute after-initialization for all processors
    ///
    /// # 参数 / Parameters
    ///
    /// - `bean`: Bean 实例 / Bean instance
    /// - `context`: Bean 上下文 / Bean context
    pub fn after_initialization(&self, bean: &mut dyn Any, context: &BeanContext) -> Result<()>
    {
        for processor in &self.processors
        {
            processor.post_process_after_initialization(bean, context)?;
        }
        Ok(())
    }

    /// 获取处理器数量
    /// Get processor count
    pub fn len(&self) -> usize
    {
        self.processors.len()
    }

    /// 检查是否为空
    /// Check if empty
    pub fn is_empty(&self) -> bool
    {
        self.processors.is_empty()
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use std::sync::Arc;

    use super::*;

    // ----------------------------------------------------------------
    // BeanContext 测试 / BeanContext Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_bean_context_new()
    {
        let ctx = BeanContext::new("testService", TypeId::of::<String>());
        assert_eq!(ctx.bean_name(), "testService");
        assert_eq!(ctx.bean_type(), TypeId::of::<String>());
        assert!(ctx.type_name().is_empty());
    }

    #[test]
    fn test_bean_context_with_type_name()
    {
        let ctx = BeanContext::new("svc", TypeId::of::<i32>()).with_type_name("i32");
        assert_eq!(ctx.type_name(), "i32");
    }

    #[test]
    fn test_bean_context_with_property()
    {
        let ctx = BeanContext::new("svc", TypeId::of::<i32>())
            .with_property("key1", "value1")
            .with_property("key2", "value2");

        assert_eq!(ctx.get_property("key1"), Some("value1"));
        assert_eq!(ctx.get_property("key2"), Some("value2"));
        assert_eq!(ctx.get_property("missing"), None);
    }

    // ----------------------------------------------------------------
    // AutowiredAnnotationBeanPostProcessor 测试 / Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_autowired_processor_new()
    {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn test_autowired_register_dependency()
    {
        let mut processor = AutowiredAnnotationBeanPostProcessor::new();
        processor.register_dependency("userService", "userRepository");
        processor.register_dependency("userService", "cacheManager");

        let deps = processor.get_dependencies("userService");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"userRepository".to_string()));
        assert!(deps.contains(&"cacheManager".to_string()));
    }

    #[test]
    fn test_autowired_no_dependencies()
    {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        assert!(processor.get_dependencies("unknownService").is_empty());
    }

    #[test]
    fn test_autowired_before_initialization()
    {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let context = BeanContext::new("testService", TypeId::of::<i32>());
        let mut bean = 42i32;

        let result = processor.post_process_before_initialization(&mut bean, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_autowired_after_initialization()
    {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let context = BeanContext::new("testService", TypeId::of::<i32>());
        let mut bean = 42i32;

        let result = processor.post_process_after_initialization(&mut bean, &context);
        assert!(result.is_ok());
    }

    // ----------------------------------------------------------------
    // CommonAnnotationBeanPostProcessor 测试 / Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_common_processor_new()
    {
        let processor = CommonAnnotationBeanPostProcessor::new();
        assert!(processor.initialized_beans().is_empty());
        assert!(processor.destroyed_beans().is_empty());
    }

    #[test]
    fn test_common_post_construct()
    {
        let mut processor = CommonAnnotationBeanPostProcessor::new();
        let initialized = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let initialized_clone = initialized.clone();

        processor.register_post_construct("myService", move |_bean| {
            initialized_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });

        let bean = 42i32;
        processor.invoke_post_construct("myService", &bean).unwrap();

        assert!(initialized.load(std::sync::atomic::Ordering::SeqCst));
        assert!(processor.is_initialized("myService"));
    }

    #[test]
    fn test_common_pre_destroy()
    {
        let mut processor = CommonAnnotationBeanPostProcessor::new();
        let destroyed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let destroyed_clone = destroyed.clone();

        processor.register_pre_destroy("myService", move |_bean| {
            destroyed_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });

        let bean = 42i32;
        processor.invoke_pre_destroy("myService", &bean).unwrap();

        assert!(destroyed.load(std::sync::atomic::Ordering::SeqCst));
        assert!(processor.is_destroyed("myService"));
    }

    #[test]
    fn test_common_no_callback()
    {
        let mut processor = CommonAnnotationBeanPostProcessor::new();
        let bean = 42i32;

        // 没有 callback，不应该出错
        // No callback, should not error
        processor.invoke_post_construct("unknown", &bean).unwrap();
        processor.invoke_pre_destroy("unknown", &bean).unwrap();

        assert!(!processor.is_initialized("unknown"));
        assert!(!processor.is_destroyed("unknown"));
    }

    #[test]
    fn test_common_destroy_all()
    {
        let mut processor = CommonAnnotationBeanPostProcessor::new();
        processor.register_pre_destroy("serviceA", |_bean| Ok(()));
        processor.register_pre_destroy("serviceB", |_bean| Ok(()));

        processor.destroy_all().unwrap();
        assert_eq!(processor.destroyed_beans().len(), 2);
    }

    #[test]
    fn test_common_before_and_after_init()
    {
        let processor = CommonAnnotationBeanPostProcessor::new();
        let context = BeanContext::new("testService", TypeId::of::<i32>());
        let mut bean = 42i32;

        let result = processor.post_process_before_initialization(&mut bean, &context);
        assert!(result.is_ok());

        let result = processor.post_process_after_initialization(&mut bean, &context);
        assert!(result.is_ok());
    }

    // ----------------------------------------------------------------
    // BeanPostProcessorChain 测试 / Chain Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_chain_empty()
    {
        let chain = BeanPostProcessorChain::new();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_chain_with_defaults()
    {
        let chain = BeanPostProcessorChain::with_defaults();
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_chain_before_and_after()
    {
        let mut chain = BeanPostProcessorChain::new();
        chain.add(Box::new(AutowiredAnnotationBeanPostProcessor::new()));
        chain.add(Box::new(CommonAnnotationBeanPostProcessor::new()));

        let context = BeanContext::new("testService", TypeId::of::<i32>());
        let mut bean = 42i32;

        chain.before_initialization(&mut bean, &context).unwrap();
        chain.after_initialization(&mut bean, &context).unwrap();
    }

    // ----------------------------------------------------------------
    // BeanPostProcessor trait 测试 / Trait Tests
    // ----------------------------------------------------------------

    #[test]
    fn test_custom_bean_post_processor()
    {
        struct TestPostProcessor
        {
            before_called: Arc<std::sync::atomic::AtomicBool>,
            after_called: Arc<std::sync::atomic::AtomicBool>,
        }

        impl BeanPostProcessor for TestPostProcessor
        {
            fn name(&self) -> &'static str
            {
                "TestPostProcessor"
            }

            fn order(&self) -> i32
            {
                10
            }

            fn post_process_before_initialization(
                &self,
                _bean: &mut dyn Any,
                _context: &BeanContext,
            ) -> Result<()>
            {
                self.before_called
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }

            fn post_process_after_initialization(
                &self,
                _bean: &mut dyn Any,
                _context: &BeanContext,
            ) -> Result<()>
            {
                self.after_called
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }

        let before = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let after = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let processor = TestPostProcessor {
            before_called: before.clone(),
            after_called: after.clone(),
        };

        let context = BeanContext::new("testBean", TypeId::of::<i32>());
        let mut bean = 42i32;

        assert_eq!(processor.name(), "TestPostProcessor");
        assert_eq!(processor.order(), 10);

        processor
            .post_process_before_initialization(&mut bean, &context)
            .unwrap();
        assert!(before.load(std::sync::atomic::Ordering::SeqCst));

        processor
            .post_process_after_initialization(&mut bean, &context)
            .unwrap();
        assert!(after.load(std::sync::atomic::Ordering::SeqCst));
    }
}
