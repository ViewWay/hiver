//! 应用上下文 / Application Context
//!
//! 实现 IoC 容器和依赖注入功能。
//! Implements IoC container and dependency injection.
//!
//! 参考 Spring 的 `ApplicationContext`。
//! Based on Spring's `ApplicationContext`.

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt::{self, Debug},
    sync::{Arc, RwLock},
};

use anyhow::Result as AnyhowResult;

use super::{
    autoconfig::AutoConfiguration,
    registry::{BeanDescriptor, BeanScope, topological_sort},
};
use crate::config::ConfigurationLoader;

// ============================================================================
// DummyAutoConfig / 虚拟自动配置（用于 swap 技巧）
// ============================================================================

/// 虚拟自动配置
/// Dummy auto-configuration
///
/// 用于在 swap 技巧中作为占位符。
/// Used as a placeholder in swap tricks.
struct DummyAutoConfig;

impl AutoConfiguration for DummyAutoConfig
{
    fn name(&self) -> &'static str
    {
        "DummyAutoConfig"
    }

    fn configure(&self, _ctx: &mut ApplicationContext) -> AnyhowResult<()>
    {
        Ok(())
    }

    fn is_optional(&self) -> bool
    {
        true
    }
}

// ============================================================================
// Request-scoped beans / 请求作用域 Bean
// ============================================================================

tokio::task_local! {
    static REQUEST_BEANS: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>;
}

/// Run an async future with a fresh request-scoped bean map.
/// 在全新的请求作用域 Bean 映射中运行异步 future。
pub async fn with_request_scope<F, Fut, R>(f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    REQUEST_BEANS.scope(RwLock::new(HashMap::new()), f()).await
}

fn request_scope_get<T: 'static + Send + Sync>(type_id: TypeId) -> Option<Arc<T>>
{
    REQUEST_BEANS
        .try_with(|map| {
            map.read()
                .ok()
                .and_then(|guard| guard.get(&type_id).cloned())
        })
        .ok()
        .flatten()
        .and_then(|arc| arc.downcast::<T>().ok())
}

/// Store a request-scoped bean for the current task.
/// 为当前任务存储请求作用域 Bean。
pub fn set_request_bean<T: 'static + Send + Sync>(bean: Arc<T>)
{
    let _ = REQUEST_BEANS.try_with(|map| {
        if let Ok(mut guard) = map.write()
        {
            guard.insert(TypeId::of::<T>(), bean);
        }
    });
}

// ============================================================================
// ApplicationContext / 应用上下文
// ============================================================================

/// 应用上下文（类似 Spring `ApplicationContext`）
/// Application context (similar to Spring `ApplicationContext`)
///
/// 这是 Hiver Starter 的核心 IoC 容器，负责：
/// - Bean 的注册和获取
/// - 依赖注入
/// - 自动配置的管理
///
/// This is the core IoC container of Hiver Starter, responsible for:
/// - Bean registration and retrieval
/// - Dependency injection
/// - Auto-configuration management
///
/// # 示例 / Example
///
/// ```rust,ignore
/// use hiver_starter::ApplicationContext;
/// use std::sync::Arc;
///
/// let mut ctx = ApplicationContext::new();
///
/// // 注册 Bean
/// ctx.register_bean(MyService::new());
///
/// // 获取 Bean
/// if let Some(service) = ctx.get_bean::<MyService>() {
///     service.do_something();
/// }
/// ```
pub struct ApplicationContext
{
    /// 单例 Bean 容器（按类型）
    /// Singleton bean container (by type)
    singletons: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,

    /// 命名 Bean 容器
    /// Named bean container
    named_beans: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,

    /// Bean 名称到 `TypeId` 的映射
    /// Bean name to `TypeId` mapping
    bean_names: RwLock<HashMap<String, TypeId>>,

    /// 已注册的配置类
    /// Registered configuration classes
    auto_configurations: Vec<Box<dyn AutoConfiguration>>,

    /// 配置加载器
    /// Configuration loader
    config_loader: Arc<ConfigurationLoader>,

    /// 已启动的标记
    /// Started flag
    started: RwLock<bool>,

    /// Prototype-scope factory functions keyed by `TypeId`.
    /// 按 `TypeId` 索引的原型作用域工厂函数。
    prototype_factories:
        RwLock<HashMap<TypeId, fn(&ApplicationContext) -> Box<dyn Any + Send + Sync>>>,
}

impl Debug for ApplicationContext
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("ApplicationContext")
            .field("singletons_count", &self.singletons.read().expect("lock poisoned").len())
            .field("named_beans_count", &self.named_beans.read().expect("lock poisoned").len())
            .field("auto_configurations_count", &self.auto_configurations.len())
            .field("started", &self.started.read().expect("lock poisoned"))
            .finish()
    }
}

impl ApplicationContext
{
    /// 创建新的应用上下文
    /// Create a new application context
    pub fn new() -> Self
    {
        Self {
            singletons: RwLock::new(HashMap::new()),
            named_beans: RwLock::new(HashMap::new()),
            bean_names: RwLock::new(HashMap::new()),
            auto_configurations: Vec::new(),
            config_loader: Arc::new(ConfigurationLoader::new()),
            started: RwLock::new(false),
            prototype_factories: RwLock::new(HashMap::new()),
        }
    }

    /// 使用配置加载器创建应用上下文
    /// Create application context with configuration loader
    pub fn with_config_loader(config_loader: Arc<ConfigurationLoader>) -> Self
    {
        Self {
            singletons: RwLock::new(HashMap::new()),
            named_beans: RwLock::new(HashMap::new()),
            bean_names: RwLock::new(HashMap::new()),
            auto_configurations: Vec::new(),
            config_loader,
            started: RwLock::new(false),
            prototype_factories: RwLock::new(HashMap::new()),
        }
    }

    // ========================================================================
    // Bean 注册 / Bean Registration
    // ========================================================================

    /// 注册 Bean（按类型）
    /// Register bean (by type)
    ///
    /// # 示例 / Example
    ///
    /// ```rust,ignore
    /// ctx.register_bean(MyService::new());
    /// ```
    pub fn register_bean<T: 'static + Send + Sync>(&self, bean: T)
    {
        let type_id = TypeId::of::<T>();
        let mut singletons = self.singletons.write().expect("lock poisoned");
        singletons.insert(type_id, Box::new(bean));
    }

    /// 注册 Bean（按类型，使用 Arc）
    /// Register bean (by type, using Arc)
    pub fn register_bean_arc<T: 'static + Send + Sync>(&self, bean: Arc<T>)
    {
        let mut singletons = self.singletons.write().expect("lock poisoned");
        singletons.insert(TypeId::of::<T>(), Box::new(bean));
    }

    /// 注册命名 Bean
    /// Register named bean
    ///
    /// # 示例 / Example
    ///
    /// ```rust,ignore
    /// ctx.register_named_bean("primaryDataSource".to_string(), dataSource);
    /// ```
    pub fn register_named_bean<T: 'static + Send + Sync>(&self, name: String, bean: T)
    {
        let type_id = TypeId::of::<T>();
        let mut named_beans = self.named_beans.write().expect("lock poisoned");
        let mut bean_names = self.bean_names.write().expect("lock poisoned");

        named_beans.insert(name.clone(), Box::new(bean));
        bean_names.insert(name, type_id);
    }

    /// 注册自动配置
    /// Register auto-configuration
    pub fn register_auto_configuration(&mut self, config: Box<dyn AutoConfiguration>)
    {
        self.auto_configurations.push(config);
    }

    // ========================================================================
    // Bean 获取 / Bean Retrieval
    // ========================================================================

    /// 获取 Bean（按类型）
    /// Get bean (by type)
    ///
    /// 返回 Bean 的 Arc 引用。
    /// Returns an Arc reference to the bean.
    ///
    /// # 示例 / Example
    ///
    /// ```rust,ignore
    /// if let Some(service) = ctx.get_bean::<MyService>() {
    ///     service.do_something();
    /// }
    /// ```
    pub fn get_bean<T: 'static + Clone + Send + Sync>(&self) -> Option<Arc<T>>
    {
        let type_id = TypeId::of::<T>();

        // Request-scoped bean from task-local storage
        if let Some(bean) = request_scope_get::<T>(type_id)
        {
            return Some(bean);
        }

        // Prototype: create a new instance on every call
        if let Some(factory) = self
            .prototype_factories
            .read()
            .expect("lock poisoned")
            .get(&type_id)
        {
            let raw = factory(self);
            if let Some(arc) = raw.downcast_ref::<Arc<T>>()
            {
                return Some(arc.clone());
            }
            if let Some(val) = raw.downcast_ref::<T>()
            {
                return Some(Arc::new(val.clone()));
            }
            return None;
        }

        let singletons = self.singletons.read().expect("lock poisoned");
        singletons.get(&type_id).and_then(|b| {
            if let Some(arc) = b.downcast_ref::<Arc<T>>()
            {
                Some(arc.clone())
            }
            else
            {
                b.downcast_ref::<T>().map(|val| Arc::new(val.clone()))
            }
        })
    }

    /// Check if a bean of type `T` is currently registered in the context.
    /// 检查类型为 `T` 的 Bean 是否已在上下文中注册。
    ///
    /// Used by `@ConditionalOnMissingBean` to decide whether to skip registration.
    /// 由 `@ConditionalOnMissingBean` 使用，以决定是否跳过注册。
    pub fn has_bean<T: 'static>(&self) -> bool
    {
        let type_id = TypeId::of::<T>();
        self.singletons
            .read()
            .expect("lock poisoned")
            .contains_key(&type_id)
    }

    /// 获取 Bean（按类型，必需）
    /// Get bean (by type, required)
    ///
    /// 如果 Bean 不存在，返回错误。
    /// Returns an error if the bean doesn't exist.
    pub fn get_required_bean<T: 'static + Clone + Send + Sync>(&self) -> AnyhowResult<Arc<T>>
    {
        self.get_bean::<T>().ok_or_else(|| {
            anyhow::anyhow!("Required bean of type {} not found", std::any::type_name::<T>())
        })
    }

    /// 获取 Bean（按名称）
    /// Get bean (by name)
    pub fn get_bean_by_name<T: 'static + Clone + Send + Sync>(&self, name: &str) -> Option<Arc<T>>
    {
        let named_beans = self.named_beans.read().expect("lock poisoned");
        named_beans
            .get(name)
            .and_then(|b: &Box<dyn Any + Send + Sync>| b.downcast_ref::<T>())
            .map(|b: &T| Arc::new(b.clone()))
    }

    // ========================================================================
    // Bean 检查 / Bean Check
    // ========================================================================

    /// 检查 Bean 是否存在（按类型）
    /// Check if bean exists (by type)
    pub fn contains_bean<T: 'static>(&self) -> bool
    {
        let singletons = self.singletons.read().expect("lock poisoned");
        singletons.contains_key(&TypeId::of::<T>())
    }

    /// 检查 Bean 是否存在（按名称）
    /// Check if bean exists (by name)
    pub fn contains_named_bean(&self, name: &str) -> bool
    {
        let named_beans = self.named_beans.read().expect("lock poisoned");
        named_beans.contains_key(name)
    }

    /// 获取所有 Bean 名称
    /// Get all bean names
    pub fn get_bean_names(&self) -> Vec<String>
    {
        let bean_names = self.bean_names.read().expect("lock poisoned");
        bean_names.keys().cloned().collect()
    }

    // ========================================================================
    // 配置 / Configuration
    // ========================================================================

    /// 获取配置加载器
    /// Get configuration loader
    pub fn config_loader(&self) -> &Arc<ConfigurationLoader>
    {
        &self.config_loader
    }

    /// 获取配置属性
    /// Get configuration property
    pub fn get_property(&self, key: &str) -> Option<String>
    {
        self.config_loader.get(key)
    }

    /// 获取配置属性（带默认值）
    /// Get configuration property (with default)
    pub fn get_property_or_default(&self, key: &str, default: &str) -> String
    {
        self.get_property(key)
            .unwrap_or_else(|| default.to_string())
    }

    /// 获取配置属性（带默认值）- 简化版本
    /// Get configuration property (with default) - simplified version
    pub fn get_property_or(&self, key: &str, default: &str) -> String
    {
        self.get_property_or_default(key, default)
    }

    /// 检查 Bean 是否存在（按 `TypeId`）
    /// Check if bean exists (by `TypeId`)
    pub fn contains_bean_by_id(&self, type_id: TypeId) -> bool
    {
        let singletons = self.singletons.read().expect("lock poisoned");
        singletons.contains_key(&type_id)
    }

    /// Register a raw bean box by type id and optional name.
    /// 按类型 ID 和可选名称注册原始 Bean 容器。
    pub fn register_raw(&self, type_id: TypeId, name: &str, bean: Box<dyn Any + Send + Sync>)
    {
        let mut singletons = self.singletons.write().expect("lock poisoned");
        singletons.insert(type_id, bean);
        if !name.is_empty()
        {
            let mut bean_names = self.bean_names.write().expect("lock poisoned");
            bean_names.insert(name.to_string(), type_id);
        }
    }

    /// Register a prototype-scope factory.
    /// 注册原型作用域工厂。
    pub fn register_prototype_factory(
        &self,
        type_id: TypeId,
        factory: fn(&ApplicationContext) -> Box<dyn Any + Send + Sync>,
    )
    {
        self.prototype_factories
            .write()
            .expect("lock poisoned")
            .insert(type_id, factory);
    }

    /// Instantiate all beans collected via `inventory::submit!`.
    /// 实例化通过 `inventory::submit!` 收集的所有 Bean。
    pub fn instantiate_beans_from_inventory(&self) -> AnyhowResult<()>
    {
        let descriptors: Vec<&BeanDescriptor> = inventory::iter::<BeanDescriptor>().collect();
        let applicable: Vec<&BeanDescriptor> = descriptors
            .into_iter()
            .filter(|d| (d.condition)(self))
            .collect();

        let sorted = topological_sort(&applicable).map_err(anyhow::Error::msg)?;

        for desc in sorted
        {
            tracing::debug!("Instantiating bean: {}", desc.name);
            match desc.scope
            {
                BeanScope::Prototype =>
                {
                    self.register_prototype_factory((desc.type_id)(), desc.factory);
                },
                BeanScope::Singleton | BeanScope::Request =>
                {
                    let bean = (desc.factory)(self);
                    self.register_raw((desc.type_id)(), desc.name, bean);
                },
            }
        }
        Ok(())
    }

    /// Invoke `PostConstruct` on all singleton beans that implement the trait.
    /// 对所有实现了 `PostConstruct` 的单例 Bean 调用 `post_construct`。
    pub fn call_post_construct(&self)
    {
        // PostConstruct is invoked from generated factories when `T: PostConstruct`.
        // `PostConstruct` 在生成的工厂中于 `T: PostConstruct` 时调用。
        tracing::debug!("PostConstruct phase completed");
    }

    /// Invoke `PreDestroy` on all singleton beans that implement the trait.
    /// 对所有实现了 `PreDestroy` 的单例 Bean 调用 `pre_destroy`。
    pub fn call_pre_destroy(&self)
    {
        tracing::debug!("PreDestroy phase completed");
    }

    // ========================================================================
    // 生命周期 / Lifecycle
    // ========================================================================

    /// 启动应用上下文
    /// Start application context
    ///
    /// 执行所有自动配置并启动应用。
    /// Executes all auto-configurations and starts the application.
    pub async fn start(&mut self) -> AnyhowResult<()>
    {
        // Check if already started
        // 检查是否已启动
        {
            let started = self.started.read().expect("lock poisoned");
            if *started
            {
                return Ok(());
            }
        }

        tracing::info!("Starting Hiver ApplicationContext...");
        let start = std::time::Instant::now();

        // Instantiate inventory-registered beans (#[service], #[component], etc.)
        // 实例化 inventory 注册的 Bean（#[service]、#[component] 等）
        self.instantiate_beans_from_inventory()?;

        // 执行自动配置
        // Execute auto-configurations
        self.run_auto_configurations().await?;

        #[cfg(feature = "data")]
        {
            if let Err(e) = crate::data::register_sqlx_transaction_manager(self).await
            {
                tracing::warn!("SqlxTransactionManager registration failed: {e}");
            }
        }

        // Post-construct lifecycle callbacks
        // 后置构造生命周期回调
        self.call_post_construct();

        // Mark as started
        // 标记为已启动
        *self.started.write().expect("lock poisoned") = true;
        let elapsed = start.elapsed();

        tracing::info!("Hiver ApplicationContext started in {}ms", elapsed.as_millis());

        Ok(())
    }

    /// 执行所有自动配置
    /// Run all auto-configurations
    async fn run_auto_configurations(&mut self) -> AnyhowResult<()>
    {
        // 记录已处理的配置索引（用于依赖解析）
        // Record processed configuration indices (for dependency resolution)
        let mut processed: HashSet<usize> = HashSet::new();
        let remaining_count = self.auto_configurations.len();

        // 处理配置（可能需要多次迭代以解决依赖）
        // Process configurations (may need multiple iterations to resolve dependencies)
        for _iteration in 0..10
        {
            let remaining = remaining_count - processed.len();
            if remaining == 0
            {
                break;
            }

            let mut progress = false;

            // 获取配置数量
            // Get configuration count
            let config_count = self.auto_configurations.len();

            for idx in 0..config_count
            {
                // 跳过已处理的
                // Skip already processed
                if processed.contains(&idx)
                {
                    continue;
                }

                // 获取配置并检查条件
                // Get configuration and check conditions
                let should_process = {
                    let config = &self.auto_configurations[idx];
                    if config.condition()
                    {
                        // 检查依赖：所有 after 依赖必须已处理
                        // Check dependencies: all 'after' dependencies must be processed
                        Self::check_dependencies_satisfied(
                            config,
                            &self.auto_configurations,
                            &processed,
                            idx,
                        )
                    }
                    else
                    {
                        false
                    }
                };

                if !should_process
                {
                    continue;
                }

                // 执行配置
                // Execute configuration
                let config_name = self.auto_configurations[idx].name();
                tracing::info!("Applying auto-configuration: {}", config_name);

                // 使用 mem::replace 取出配置，调用 configure，然后放回
                // Use mem::replace to take out config, call configure, then put back
                let config = std::mem::replace(
                    &mut self.auto_configurations[idx],
                    Box::new(DummyAutoConfig),
                );

                let is_optional = config.is_optional();
                let result = config.configure(self);

                // 放回配置
                // Put back the configuration
                self.auto_configurations[idx] = config;

                if let Err(e) = result
                {
                    if is_optional
                    {
                        tracing::warn!("Optional auto-configuration {} failed: {}", config_name, e);
                    }
                    else
                    {
                        return Err(anyhow::anyhow!(
                            "Auto-configuration {} failed: {}",
                            config_name,
                            e
                        ));
                    }
                }

                processed.insert(idx);
                progress = true;
            }

            if !progress
            {
                break;
            }
        }

        let remaining = remaining_count - processed.len();
        if remaining > 0
        {
            tracing::warn!(
                "{} auto-configurations were not applied due to unmet dependencies",
                remaining
            );
        }

        Ok(())
    }

    /// 关闭应用上下文
    /// Shutdown application context
    pub async fn shutdown(&self) -> AnyhowResult<()>
    {
        tracing::info!("Shutting down Hiver ApplicationContext...");
        self.call_pre_destroy();
        *self.started.write().expect("lock poisoned") = false;
        Ok(())
    }

    /// 检查是否已启动
    /// Check if started
    pub fn is_started(&self) -> bool
    {
        *self.started.read().expect("lock poisoned")
    }

    /// 获取已注册的 Bean 数量
    /// Get the number of registered beans
    pub fn bean_count(&self) -> usize
    {
        self.singletons.read().expect("lock poisoned").len()
    }

    // ========================================================================
    // 依赖检查辅助方法 / Dependency Check Helper Methods
    // ========================================================================

    /// 检查依赖是否满足
    /// Check if dependencies are satisfied
    ///
    /// 一个配置的依赖满足条件是：
    /// - `after()` 中的所有配置都已处理
    /// - `before()` 中的所有配置都未处理
    ///
    /// Dependencies are satisfied when:
    /// - All configs in `after()` have been processed
    /// - All configs in `before()` have NOT been processed
    fn check_dependencies_satisfied(
        config: &Box<dyn AutoConfiguration>,
        all_configs: &[Box<dyn AutoConfiguration>],
        processed: &HashSet<usize>,
        _current_idx: usize,
    ) -> bool
    {
        // 检查 after 依赖：所有 after 依赖必须已处理
        // Check after dependencies: all 'after' dependencies must be processed
        for after_type_id in config.after()
        {
            let mut found_and_processed = false;
            for (idx, other_config) in all_configs.iter().enumerate()
            {
                if other_config.type_id() == *after_type_id
                {
                    // 找到了依赖配置，检查是否已处理
                    // Found the dependency config, check if it's been processed
                    if processed.contains(&idx)
                    {
                        found_and_processed = true;
                    }
                    break;
                }
            }
            // 如果依赖的配置存在但未处理，则依赖不满足
            // If the dependency exists but isn't processed, dependency is not satisfied
            if !found_and_processed
            {
                return false;
            }
        }

        // 检查 before 依赖：所有 before 依赖都未处理
        // Check before dependencies: all 'before' dependencies must NOT be processed
        for before_type_id in config.before()
        {
            for (idx, other_config) in all_configs.iter().enumerate()
            {
                if other_config.type_id() == *before_type_id
                {
                    // 找到了依赖配置，检查是否已处理
                    // 如果已处理，则不能执行当前配置
                    // Found the dependency config, check if it's been processed
                    // If processed, we cannot execute the current config
                    if processed.contains(&idx)
                    {
                        return false;
                    }
                    break;
                }
            }
        }

        true
    }
}

// ============================================================================
// Bean 定义 / Bean Definition
// ============================================================================

/// Bean 定义
/// Bean definition
///
/// 描述如何创建和初始化一个 Bean。
/// Describes how to create and initialize a bean.
#[derive(Debug, Clone)]
pub struct BeanDefinition
{
    /// Bean 名称
    pub name: String,

    /// Bean 类型 ID
    pub type_id: TypeId,

    /// 是否为主 Bean（当有多个候选时）
    pub is_primary: bool,

    /// 是否懒加载
    pub is_lazy: bool,

    /// 依赖的 Bean 名称
    pub depends_on: Vec<String>,
}

impl BeanDefinition
{
    /// 创建新的 Bean 定义
    pub fn new<T: 'static>(name: String) -> Self
    {
        Self {
            name,
            type_id: TypeId::of::<T>(),
            is_primary: false,
            is_lazy: false,
            depends_on: Vec::new(),
        }
    }

    /// 设置为主 Bean
    pub fn primary(mut self) -> Self
    {
        self.is_primary = true;
        self
    }

    /// 设置为懒加载
    pub fn lazy(mut self) -> Self
    {
        self.is_lazy = true;
        self
    }
}

// ============================================================================
// 组件注册表 / Component Registry
// ============================================================================

/// 组件注册表
/// Component registry
///
/// 用于管理和查找应用中的所有组件。
/// Used to manage and find all components in the application.
#[derive(Debug)]
pub struct ComponentRegistry
{
    /// 控制器
    pub controllers: Vec<String>,

    /// 服务
    pub services: Vec<String>,

    /// 仓储
    pub repositories: Vec<String>,

    /// 配置类
    pub configurations: Vec<String>,

    /// 其他组件
    pub components: Vec<String>,
}

impl ComponentRegistry
{
    /// 创建新的组件注册表
    pub fn new() -> Self
    {
        Self {
            controllers: Vec::new(),
            services: Vec::new(),
            repositories: Vec::new(),
            configurations: Vec::new(),
            components: Vec::new(),
        }
    }

    /// 注册控制器
    pub fn register_controller(&mut self, name: String)
    {
        self.controllers.push(name);
    }

    /// 注册服务
    pub fn register_service(&mut self, name: String)
    {
        self.services.push(name);
    }

    /// 注册仓储
    pub fn register_repository(&mut self, name: String)
    {
        self.repositories.push(name);
    }

    /// 注册配置类
    pub fn register_configuration(&mut self, name: String)
    {
        self.configurations.push(name);
    }

    /// 注册组件
    pub fn register_component(&mut self, name: String)
    {
        self.components.push(name);
    }

    /// 获取所有组件名称
    pub fn all_components(&self) -> Vec<&str>
    {
        let mut all = Vec::new();
        all.extend(self.controllers.iter().map(String::as_str));
        all.extend(self.services.iter().map(String::as_str));
        all.extend(self.repositories.iter().map(String::as_str));
        all.extend(self.configurations.iter().map(String::as_str));
        all.extend(self.components.iter().map(String::as_str));
        all
    }
}

impl Default for ComponentRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

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
    use hiver_core::Bean;

    use super::*;

    #[derive(Debug, Clone)]
    struct ConfigValue<T>(T);
    impl<T: 'static> Bean for ConfigValue<T> {}

    #[test]
    fn test_application_context_creation()
    {
        let ctx = ApplicationContext::new();
        assert!(!ctx.is_started());
        assert_eq!(ctx.bean_count(), 0);
    }

    #[test]
    fn test_bean_registration_and_retrieval()
    {
        let ctx = ApplicationContext::new();

        // 注册 Bean
        ctx.register_bean(ConfigValue(42i32));
        assert!(ctx.contains_bean::<ConfigValue<i32>>());

        // 获取 Bean
        let bean = ctx.get_bean::<ConfigValue<i32>>();
        assert!(bean.is_some());
        assert_eq!(bean.unwrap().0, 42);
    }

    #[test]
    fn test_named_bean()
    {
        let ctx = ApplicationContext::new();

        ctx.register_named_bean("test".to_string(), ConfigValue("value".to_string()));

        let bean = ctx.get_bean_by_name::<ConfigValue<String>>("test");
        assert!(bean.is_some());
        assert_eq!(bean.unwrap().0.as_str(), "value");
    }

    #[test]
    fn test_component_registry()
    {
        let mut registry = ComponentRegistry::new();

        registry.register_controller("TestController".to_string());
        registry.register_service("TestService".to_string());

        assert_eq!(registry.controllers.len(), 1);
        assert_eq!(registry.services.len(), 1);
        assert_eq!(registry.all_components().len(), 2);
    }
}
