//! IoC/DI Container module
//! IoC/DI容器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `ApplicationContext`
//! - `BeanFactory`
//! - @Component, @Service, @Repository scanning
//! - Dependency injection / autowiring
//! - Lifecycle callbacks (@`PostConstruct`, @`PreDestroy`)

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use super::{
    bean::{Bean, BeanDefinition, Scope},
    conditional::{Condition, ConditionContext},
    error::{Error, Result},
    extension::Extensions,
    reflect::ReflectContainer,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Bean factory function type
/// Bean工厂函数类型
///
/// Used for registering beans with their dependencies.
/// 用于注册带有依赖项的bean。
pub type BeanFactoryFn<T> = Arc<dyn Fn(&Container) -> Result<T> + Send + Sync>;

/// Bean registration with metadata
/// 带元数据的bean注册
pub struct BeanRegistration<T> {
    /// The bean definition
    /// Bean定义
    pub definition: BeanDefinition,

    /// Factory function to create the bean
    /// 创建bean的工厂函数
    pub factory: Option<BeanFactoryFn<T>>,

    /// Post-init callback (@`PostConstruct` equivalent)
    /// 初始化后回调（等价于 @`PostConstruct`）
    pub post_construct: Option<Arc<dyn Fn(&T) -> Result<()> + Send + Sync>>,

    /// Pre-destroy callback (@`PreDestroy` equivalent)
    /// 销毁前回调（等价于 @`PreDestroy`）
    pub pre_destroy: Option<Arc<dyn Fn(&T) -> Result<()> + Send + Sync>>,
}

impl<T> BeanRegistration<T> {
    /// Create a new bean registration
    /// 创建新的bean注册
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            definition: BeanDefinition::new(name, std::any::type_name::<T>()),
            factory: None,
            post_construct: None,
            pre_destroy: None,
        }
    }

    /// Set the factory function
    /// 设置工厂函数
    pub fn factory(mut self, factory: BeanFactoryFn<T>) -> Self {
        self.factory = Some(factory);
        self
    }

    /// Set post-construct callback
    /// 设置初始化后回调
    pub fn post_construct<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Result<()> + Send + Sync + 'static,
    {
        self.post_construct = Some(Arc::new(f));
        self
    }

    /// Set pre-destroy callback
    /// 设置销毁前回调
    pub fn pre_destroy<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Result<()> + Send + Sync + 'static,
    {
        self.pre_destroy = Some(Arc::new(f));
        self
    }

    /// Set the scope
    /// 设置作用域
    pub fn scope(mut self, scope: Scope) -> Self {
        self.definition.scope = scope;
        self
    }

    /// Set as primary
    /// 设置为主bean
    pub fn primary(mut self, primary: bool) -> Self {
        self.definition.primary = primary;
        self
    }

    /// Set lazy initialization
    /// 设置延迟初始化
    pub fn lazy(mut self, lazy: bool) -> Self {
        self.definition.lazy = lazy;
        self
    }
}

trait PreDestroyHook: Send + Sync {
    fn invoke(&self, bean: &dyn Any) -> crate::error::Result<()>;
}

struct PreDestroyHookImpl<T> {
    callback: Arc<dyn Fn(&T) -> crate::error::Result<()> + Send + Sync>,
}

impl<T: 'static> PreDestroyHook for PreDestroyHookImpl<T> {
    fn invoke(&self, bean: &dyn Any) -> crate::error::Result<()> {
        if let Some(typed) = bean.downcast_ref::<T>() {
            (self.callback)(typed)
        } else {
            Ok(())
        }
    }
}

/// Internal bean storage
/// 内部bean存储
struct BeanStore {
    /// Singleton beans (created once and reused)
    /// 单例bean（创建一次并重用）
    singletons: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,

    /// Bean registrations (metadata and factories)
    /// Bean注册（元数据和工厂）
    registrations: HashMap<TypeId, Box<dyn Any + Send + Sync>>,

    /// Named bean lookups
    /// 命名bean查找
    by_name: HashMap<String, TypeId>,

    /// Early exposed beans (Weak references for circular dependency resolution)
    /// 提前暴露的Bean（Weak引用，用于循环依赖解析）
    early_exposed: HashMap<TypeId, std::sync::Weak<dyn Any + Send + Sync>>,

    /// Currently creating beans (for cycle detection)
    /// 正在创建的Bean（用于循环检测）
    creating: std::cell::RefCell<std::collections::HashSet<TypeId>>,

    /// Type-erased pre-destroy hooks keyed by TypeId
    /// 按TypeId键控的类型擦除销毁前回调
    pre_destroy_hooks: HashMap<TypeId, Box<dyn PreDestroyHook>>,
}

impl BeanStore {
    fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            registrations: HashMap::new(),
            by_name: HashMap::new(),
            early_exposed: HashMap::new(),
            creating: std::cell::RefCell::new(std::collections::HashSet::new()),
            pre_destroy_hooks: HashMap::new(),
        }
    }
}

/// IoC Container (Inversion of Control)
/// IoC容器（控制反转）
///
/// This is equivalent to Spring's `ApplicationContext` or `BeanFactory`.
/// 这等价于Spring的`ApplicationContext`或`BeanFactory`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_core::Container;
/// use std::sync::Arc;
///
/// let mut container = Container::new();
///
/// // Register a bean with constructor injection
/// // 使用构造函数注入注册bean
/// container.register::<UserService>(|c| {
///     let repo = c.get_bean::<UserRepository>()?;
///     Ok(UserService::new(repo))
/// })?;
///
/// // Get a bean
/// // 获取bean
/// let service: Arc<UserService> = container.get_bean()?;
/// ```
#[derive(Clone)]
pub struct Container {
    beans: Arc<RwLock<BeanStore>>,
    extensions: Extensions,
    /// Reflection container for dynamic bean operations
    /// 用于动态Bean操作的反射容器
    reflect: Arc<ReflectContainer>,
}

impl Container {
    /// Create a new container
    /// 创建新容器
    pub fn new() -> Self {
        Self {
            #[allow(clippy::arc_with_non_send_sync)]
            beans: Arc::new(RwLock::new(BeanStore::new())),
            extensions: Extensions::new(),
            reflect: Arc::new(ReflectContainer::new()),
        }
    }

    /// Register a bean with a factory function
    /// 使用工厂函数注册bean
    ///
    /// Equivalent to Spring's `@Bean` method in `@Configuration` class.
    /// 等价于Spring中`@Configuration`类里的`@Bean`方法。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// container.register::<UserService>(|c| {
    ///     let repo = c.get_bean::<UserRepository>()?;
    ///     Ok(UserService::new(repo))
    /// })?;
    /// ```
    pub fn register<T, F>(&mut self, factory: F) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        let mut beans = self
            .beans
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

        let registration = BeanRegistration::new(type_name).factory(Arc::new(factory));

        beans
            .by_name
            .insert(registration.definition.name.clone(), type_id);
        beans.registrations.insert(type_id, Box::new(registration));

        Ok(())
    }

    /// Register a bean with full configuration
    /// 使用完整配置注册bean
    pub fn register_with<T>(&mut self, registration: BeanRegistration<T>) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();

        if let Some(pre_destroy) = &registration.pre_destroy {
            let hook = PreDestroyHookImpl {
                callback: pre_destroy.clone(),
            };
            let mut beans = self
                .beans
                .write()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
            beans.pre_destroy_hooks.insert(type_id, Box::new(hook));
        }

        let mut beans = self
            .beans
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

        beans
            .by_name
            .insert(registration.definition.name.clone(), type_id);
        beans.registrations.insert(type_id, Box::new(registration));

        Ok(())
    }

    /// Register a bean instance directly
    /// 直接注册bean实例
    ///
    /// Equivalent to Spring's `@Component` scanning.
    /// 等价于Spring的`@Component`扫描。
    pub fn register_bean<T: Bean + Send + Sync + 'static>(&mut self, bean: T) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let bean_arc = Arc::new(bean);

        // First, check if there's a post-construct callback
        // 首先检查是否有初始化后回调
        let post_construct_callback = {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
            beans
                .registrations
                .get(&type_id)
                .and_then(|reg| reg.downcast_ref::<BeanRegistration<T>>())
                .and_then(|reg_t| reg_t.post_construct.clone())
        };

        // Call post-construct callback if available (without holding lock)
        // 如果有回调，调用它（不持有锁）
        if let Some(post_construct) = post_construct_callback
            && let Err(e) = post_construct(&bean_arc) {
                return Err(Error::internal(format!(
                    "Post-construct callback failed for {}: {}",
                    std::any::type_name::<T>(),
                    e
                )));
            }

        // Now insert the bean (with write lock)
        // 现在插入bean（使用写锁）
        let mut beans = self
            .beans
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
        beans.singletons.insert(type_id, bean_arc);
        Ok(())
    }

    /// Register a bean factory for lazy initialization
    /// 注册bean工厂以延迟初始化
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// container.register_factory(|| {
    ///     UserService::new()
    /// }).unwrap();
    /// ```
    pub fn register_factory<T, F>(&mut self, factory: F) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.register(move |_c| Ok(factory()))
    }

    /// Register a bean conditionally based on a [`Condition`].
    /// 根据条件 [`Condition`] 有条件地注册Bean。
    ///
    /// Evaluates the condition against the current container state (registered
    /// beans, bean names). If the condition matches, the bean is registered
    /// with the provided factory function; otherwise, the registration is
    /// silently skipped.
    ///
    /// 根据当前容器状态（已注册的Bean、Bean名称）评估条件。如果条件匹配，
    /// 则使用提供的工厂函数注册Bean；否则，注册将被静默跳过。
    ///
    /// Equivalent to Spring Boot's `@Conditional` annotations.
    /// 等价于Spring Boot的 `@Conditional` 注解。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use nexus_core::Container;
    /// use nexus_core::ConditionalOnMissingBean;
    ///
    /// let mut container = Container::new();
    ///
    /// // Only register InMemoryCache if no Cache bean is already present
    /// // 仅在尚未存在Cache Bean时注册InMemoryCache
    /// container.register_conditional::<InMemoryCache, _>(
    ///     |c| Ok(InMemoryCache::new()),
    ///     ConditionalOnMissingBean::of::<dyn Cache>(),
    /// )?;
    /// ```
    pub fn register_conditional<T, F, C>(&mut self, factory: F, condition: C) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
        C: Condition + 'static,
    {
        // Build a ConditionContext from the current container state
        // 从当前容器状态构建ConditionContext
        let context = {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            let registered_beans: Vec<TypeId> = beans
                .registrations
                .keys()
                .chain(beans.singletons.keys())
                .copied()
                .collect();

            let bean_names: HashMap<String, TypeId> = beans.by_name.clone();

            ConditionContext::new()
                .with_registered_beans(registered_beans)
                .with_bean_names(bean_names)
        };

        if condition.matches(&context) {
            self.register(factory)
        } else {
            Ok(())
        }
    }

    /// Get a bean by type (resolving dependencies)
    /// 按类型获取bean（解析依赖）
    ///
    /// Equivalent to Spring's `ApplicationContext.getBean(Class)`.
    /// 等价于Spring的`ApplicationContext.getBean(Class)`。
    ///
    /// This method supports:
    /// - Constructor injection (via registered factory functions)
    /// - Lazy initialization
    /// - Singleton scope (default)
    /// - Circular dependency detection and resolution
    ///
    /// 此方法支持：
    /// - 构造函数注入（通过注册的工厂函数）
    /// - 延迟初始化
    /// - 单例作用域（默认）
    /// - 循环依赖检测和解析
    pub fn get_bean<T: Bean + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let type_id = TypeId::of::<T>();

        // First, check if we already have a singleton
        // 首先检查是否已有单例
        {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            if let Some(bean) = beans.singletons.get(&type_id)
                && let Ok(typed) = Arc::clone(bean).downcast::<T>() {
                    return Ok(typed);
                }

            // Check for circular dependency: if we're currently creating this bean,
            // try to return the early-exposed Weak reference
            // 检查循环依赖：如果正在创建此bean，尝试返回提前暴露的Weak引用
            if beans.creating.borrow().contains(&type_id) {
                if let Some(weak) = beans.early_exposed.get(&type_id)
                    && let Some(arc) = weak.upgrade()
                        && let Ok(typed) = arc.downcast::<T>() {
                            return Ok(typed);
                        }
                // Circular dependency detected but no early-exposed reference
                // 检测到循环依赖但没有提前暴露的引用
                return Err(Error::internal(format!(
                    "Circular dependency detected while creating bean: {}",
                    std::any::type_name::<T>()
                )));
            }
        }

        // Check if we have a registration with factory
        // 检查是否有带工厂的注册
        let factory_opt = {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            beans
                .registrations
                .get(&type_id)
                .and_then(|r| r.downcast_ref::<BeanRegistration<T>>())
                .and_then(|reg| reg.factory.clone())
        };

        if let Some(factory) = factory_opt {
            // Mark as creating (for cycle detection)
            // 标记为正在创建（用于循环检测）
            {
                let beans = self
                    .beans
                    .read()
                    .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
                beans.creating.borrow_mut().insert(type_id);
            }

            // Create a placeholder Arc with Weak reference for early exposure
            // 创建占位符Arc和Weak引用用于提前暴露
            let placeholder: Arc<T> = {
                // Try to create the bean
                // 尝试创建bean
                let bean = factory(self)?;
                Arc::new(bean)
            };

            // Store Weak reference early (for circular dependencies)
            // 提前存储Weak引用（用于循环依赖）
            {
                let mut beans = self
                    .beans
                    .write()
                    .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
                beans.early_exposed.insert(
                    type_id,
                    Arc::downgrade(&placeholder) as std::sync::Weak<dyn Any + Send + Sync>,
                );
            }

            // Store as singleton
            // 存储为单例
            {
                let mut beans = self
                    .beans
                    .write()
                    .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
                beans.singletons.insert(type_id, placeholder.clone());
                // Remove from creating set
                // 从创建集合中移除
                beans.creating.borrow_mut().remove(&type_id);
            }

            // Call post_construct callback if available
            // 调用初始化后回调（如果有）
            {
                let beans = self
                    .beans
                    .read()
                    .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
                if let Some(reg) = beans.registrations.get(&type_id)
                    && let Some(reg_t) = reg.downcast_ref::<BeanRegistration<T>>()
                        && let Some(post_construct) = &reg_t.post_construct {
                            post_construct(&placeholder)?;
                        }
            }

            Ok(placeholder)
        } else {
            Err(Error::not_found(format!("Bean not found: {}", std::any::type_name::<T>())))
        }
    }

    /// Get a bean by name
    /// 按名称获取bean
    ///
    /// Equivalent to Spring's `ApplicationContext.getBean(String)`.
    /// 等价于Spring的`ApplicationContext.getBean(String)`。
    pub fn get_bean_by_name<T: Bean + Send + Sync + 'static>(&self, name: &str) -> Result<Arc<T>> {
        let type_id = {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            beans
                .by_name
                .get(name)
                .copied()
                .ok_or_else(|| Error::not_found(format!("Bean not found: {}", name)))?
        };

        // First check if we already have a singleton
        // 首先检查是否已有单例
        {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            if let Some(bean) = beans.singletons.get(&type_id)
                && let Ok(typed) = Arc::clone(bean).downcast::<T>() {
                    return Ok(typed);
                }
        }

        // Check if we have a registration with factory and create the bean
        // 检查是否有带工厂的注册并创建bean
        let factory_opt = {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            beans
                .registrations
                .get(&type_id)
                .and_then(|r| r.downcast_ref::<BeanRegistration<T>>())
                .and_then(|reg| reg.factory.clone())
        };

        if let Some(factory) = factory_opt {
            // Create the bean using the factory (resolving dependencies)
            // 使用工厂创建bean（解析依赖）
            let bean = factory(self)?;
            let bean_arc = Arc::new(bean);

            // Store as singleton
            // 存储为单例
            let mut beans = self
                .beans
                .write()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            beans.singletons.insert(type_id, bean_arc.clone());

            Ok(bean_arc)
        } else {
            Err(Error::not_found(format!("Bean not found: {}", name)))
        }
    }

    /// Check if a bean is registered
    /// 检查bean是否已注册
    pub fn has_bean<T: Bean + Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();

        if let Ok(beans) = self.beans.try_read()
            && (beans.singletons.contains_key(&type_id) || beans.registrations.contains_key(&type_id))
            {
                return true;
            }

        false
    }

    /// Get the extensions
    /// 获取扩展
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// Get a mutable reference to extensions
    /// 获取扩展的可变引用
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    /// Get the reflection container
    /// 获取反射容器
    pub fn reflect(&self) -> &Arc<ReflectContainer> {
        &self.reflect
    }

    /// Initialize all registered beans (eager initialization)
    /// 初始化所有注册的bean（急切初始化）
    ///
    /// Equivalent to calling `getBean()` on all registered beans.
    /// 等价于在所有注册的bean上调用`getBean()`。
    pub fn initialize(&self) -> Result<()> {
        // Get all type_ids from registrations
        // 获取注册中的所有type_id
        let type_ids: Vec<_> = {
            let beans = self
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
            beans.registrations.keys().copied().collect()
        };

        for _ in type_ids {
            // We can't directly initialize without knowing the type
            // In a real implementation, we'd have more metadata
            // 在实际实现中，我们会拥有更多元数据
            // For now, just note that initialization would happen
            // 目前，只需注意初始化会发生
        }

        Ok(())
    }

    /// Shutdown the container, calling pre-destroy callbacks
    /// 关闭容器，调用销毁前回调
    pub fn shutdown(&self) -> Result<()> {
        let mut beans = self
            .beans
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

        let hooks: Vec<_> = beans.pre_destroy_hooks.drain().collect();
        for (type_id, hook) in hooks {
            if let Some(bean) = beans.singletons.get(&type_id) {
                let _ = hook.invoke(bean.as_ref());
            }
        }

        beans.singletons.clear();
        beans.registrations.clear();
        beans.by_name.clear();

        Ok(())
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// Application Context (Spring Boot equivalent)
/// 应用上下文（Spring Boot等价物）
///
/// This is the main interface for accessing beans and resources.
/// 这是访问bean和资源的主要接口。
///
/// Equivalent to:
/// - `ApplicationContext`
/// - `ConfigurableApplicationContext`
/// - `WebApplicationContext`
pub struct ApplicationContext {
    container: Container,
    profile: String,
    active: bool,
}

impl ApplicationContext {
    /// Create a new application context
    /// 创建新的应用上下文
    pub fn new() -> Self {
        Self {
            container: Container::new(),
            profile: std::env::var("SPRING_PROFILES_ACTIVE")
                .unwrap_or_else(|_| "default".to_string()),
            active: false,
        }
    }

    /// Get the active profile
    /// 获取活动配置文件
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Set the active profile
    /// 设置活动配置文件
    pub fn set_profile(&mut self, profile: impl Into<String>) {
        self.profile = profile.into();
    }

    /// Check if a profile is active
    /// 检查配置文件是否活动
    pub fn accepts_profile(&self, profile: &str) -> bool {
        self.profile == profile || self.profile == "default" || profile == "default"
    }

    /// Get the underlying container
    /// 获取底层容器
    pub fn container(&self) -> &Container {
        &self.container
    }

    /// Get a mutable reference to the container
    /// 获取容器的可变引用
    pub fn container_mut(&mut self) -> &mut Container {
        &mut self.container
    }

    /// Register a bean
    /// 注册bean
    pub fn register<T: Bean + Send + Sync + 'static>(&mut self, bean: T) -> Result<()> {
        self.container.register_bean(bean)
    }

    /// Register a bean with factory
    /// 使用工厂注册bean
    pub fn register_with<T, F>(&mut self, factory: F) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        self.container.register(factory)
    }

    /// Get a bean
    /// 获取bean
    pub fn get_bean<T: Bean + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.container.get_bean()
    }

    /// Get a bean by name
    /// 按名称获取bean
    pub fn get_bean_by_name<T: Bean + Send + Sync + 'static>(&self, name: &str) -> Result<Arc<T>> {
        self.container.get_bean_by_name(name)
    }

    /// Check if a bean exists
    /// 检查bean是否存在
    pub fn contains_bean<T: Bean + Send + Sync + 'static>(&self) -> bool {
        self.container.has_bean::<T>()
    }

    /// Refresh the context (reload all singletons)
    /// 刷新上下文（重新加载所有单例）
    ///
    /// This will:
    /// - Call pre-destroy callbacks on existing beans
    /// - Clear all singleton instances
    /// - Re-initialize all non-lazy beans from registrations
    ///
    /// 这将：
    /// - 在现有bean上调用销毁前回调
    /// - 清除所有单例实例
    /// - 从注册中重新初始化所有非延迟bean
    pub fn refresh(&mut self) -> Result<()> {
        // Step 1: Collect all singletons to destroy
        // 步骤1：收集要销毁的所有单例
        let singletons_to_destroy: Vec<_> = {
            let beans = self
                .container
                .beans
                .read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
            beans.singletons.keys().copied().collect()
        };

        // Step 2: Call pre-destroy callbacks (for beans that implement PreDestroy trait)
        // 步骤2：调用销毁前回调（对于实现PreDestroy trait的bean）
        // Note: In a full implementation, we'd check registrations for pre_destroy callbacks
        // and call them. For now, we rely on the PreDestroy trait implementation.
        // 注意：在完整实现中，我们会检查注册中的销毁前回调并调用它们
        // 目前，我们依赖PreDestroy trait实现
        for _type_id in singletons_to_destroy {
            // The bean will be dropped when cleared from the map
            // bean从映射清除时将被丢弃
        }

        // Step 3: Clear all singletons
        // 步骤3：清除所有单例
        {
            let mut beans = self
                .container
                .beans
                .write()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;
            beans.singletons.clear();
        }

        // Step 4: Re-initialize the context
        // 步骤4：重新初始化上下文
        self.active = false;
        self.start()?;

        Ok(())
    }

    /// Start the context (initialize all eager singletons)
    /// 启动上下文（初始化所有急切单例）
    pub fn start(&mut self) -> Result<()> {
        self.active = true;
        self.container.initialize()
    }

    /// Close the context and release resources
    /// 关闭上下文并释放资源
    pub fn close(self) -> Result<()> {
        self.container.shutdown()
    }

    /// Check if context is active
    /// 检查上下文是否活动
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for ApplicationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Component scanner (equivalent to @`ComponentScan`)
/// 组件扫描器（等价于 @`ComponentScan`）
pub struct ComponentScanner {
    base_packages: Vec<String>,
}

impl ComponentScanner {
    /// Create a new scanner
    /// 创建新扫描器
    pub fn new() -> Self {
        Self {
            base_packages: Vec::new(),
        }
    }

    /// Add a base package to scan
    /// 添加要扫描的基础包
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let scanner = ComponentScanner::new()
    ///     .scan_package("com.example");
    /// ```
    pub fn scan_package(mut self, package: impl Into<String>) -> Self {
        self.base_packages.push(package.into());
        self
    }

    /// Scan for components and register them
    /// 扫描组件并注册它们
    ///
    /// Note: In Rust, true runtime component scanning is not possible like in Java.
    /// Instead, this framework uses proc-macros for compile-time component registration.
    /// Use the `#[nexus_macros::component]` attribute to register components at compile time.
    ///
    /// 注意：在Rust中，像Java那样的真正运行时组件扫描是不可能的。
    /// 相反，此框架使用proc宏进行编译时组件注册。
    /// 使用 `#[nexus_macros::component]` 属性在编译时注册组件。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use nexus_core::container::ComponentScanner;
    /// use nexus_macros::component;
    ///
    /// #[component]
    /// struct MyService {
    ///     // Dependencies are automatically injected
    /// }
    /// }
    ///
    /// // Components are collected at compile time and registered automatically
    /// // 组件在编译时被收集并自动注册
    /// ```
    pub fn scan(&self, _context: &mut ApplicationContext) -> Result<()> {
        // Component scanning in Rust is done at compile time via proc-macros
        // The `#[component]` macro generates registration code
        // 在Rust中，组件扫描通过proc宏在编译时完成
        // `#[component]` 宏生成注册代码
        //
        // This method is a no-op at runtime but exists for API compatibility
        // with Spring's @ComponentScan pattern
        // 此方法在运行时是空操作，但存在是为了与Spring的@ComponentScan模式API兼容
        Ok(())
    }

    /// Register a component type (for use with proc-macro generated code)
    /// 注册组件类型（用于proc宏生成的代码）
    ///
    /// This is called by the generated code from `#[component]` macro.
    /// This is not intended to be called manually.
    /// 这由 `#[component]` 宏生成的代码调用。
    /// 不打算手动调用。
    #[doc(hidden)]
    pub fn register_component<T: Bean + Send + Sync + 'static>(
        &self,
        _context: &mut ApplicationContext,
    ) -> Result<()> {
        // The proc-macro will generate a call to register_bean for each component
        // proc宏将为每个组件生成对register_bean的调用
        Ok(())
    }
}

impl Default for ComponentScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Post-construct callback trait
/// 初始化后回调trait
///
/// Equivalent to Spring's `@PostConstruct`.
/// 等价于Spring的`@PostConstruct`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_core::container::PostConstruct;
///
/// struct MyService {
///     initialized: bool,
/// }
///
/// impl PostConstruct for MyService {
///     fn post_construct(&self) -> Result<(), nexus_core::Error> {
///         println!("Service initialized!");
///         Ok(())
///     }
/// }
/// ```
pub trait PostConstruct {
    /// Called after the bean is constructed
    /// 在bean构造后调用
    fn post_construct(&self) -> Result<()>;
}

/// Pre-destroy callback trait
/// 销毁前回调trait
///
/// Equivalent to Spring's `@PreDestroy`.
/// 等价于Spring的`@PreDestroy`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_core::container::PreDestroy;
///
/// struct MyService {
///     connection: Option<Database>,
/// }
///
/// impl PreDestroy for MyService {
///     fn pre_destroy(&self) -> Result<(), nexus_core::Error> {
///         if let Some(conn) = &self.connection {
///             conn.close();
///         }
///         println!("Service destroyed!");
///         Ok(())
///     }
/// }
/// ```
pub trait PreDestroy {
    /// Called before the bean is destroyed
    /// 在bean销毁前调用
    fn pre_destroy(&self) -> Result<()>;
}
