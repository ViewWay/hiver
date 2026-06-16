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

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use super::{
    bean::{Bean, BeanDefinition, BeanState, DependencyInfo, Scope},
    conditional::{Condition, ConditionContext},
    error::{Error, Result},
    extension::Extensions,
    lifecycle::BeanPostProcessor,
    reflect::ReflectContainer,
};

/// Bean factory function type
/// Bean工厂函数类型
///
/// Used for registering beans with their dependencies.
/// 用于注册带有依赖项的bean。
pub type BeanFactoryFn<T> = Arc<dyn Fn(&Container) -> Result<T> + Send + Sync>;

/// Bean registration with metadata
/// 带元数据的bean注册
pub struct BeanRegistration<T>
{
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

impl<T> BeanRegistration<T>
{
    /// Create a new bean registration
    /// 创建新的bean注册
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            definition: BeanDefinition::new(name, std::any::type_name::<T>()),
            factory: None,
            post_construct: None,
            pre_destroy: None,
        }
    }

    /// Set the factory function
    /// 设置工厂函数
    pub fn factory(mut self, factory: BeanFactoryFn<T>) -> Self
    {
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
    pub fn scope(mut self, scope: Scope) -> Self
    {
        self.definition.scope = scope;
        self
    }

    /// Set as primary
    /// 设置为主bean
    pub fn primary(mut self, primary: bool) -> Self
    {
        self.definition.primary = primary;
        self
    }

    /// Set lazy initialization
    /// 设置延迟初始化
    pub fn lazy(mut self, lazy: bool) -> Self
    {
        self.definition.lazy = lazy;
        self
    }
}

trait PreDestroyHook: Send + Sync
{
    fn invoke(&self, bean: &dyn Any) -> Result<()>;
}

struct PreDestroyHookImpl<T>
{
    callback: Arc<dyn Fn(&T) -> Result<()> + Send + Sync>,
}

impl<T: 'static> PreDestroyHook for PreDestroyHookImpl<T>
{
    fn invoke(&self, bean: &dyn Any) -> Result<()>
    {
        if let Some(typed) = bean.downcast_ref::<T>()
        {
            (self.callback)(typed)
        }
        else
        {
            Ok(())
        }
    }
}
/// Internal bean storage — unified by bean name.
/// 内部bean存储 — 以bean名称统一管理。
///
/// Root cause fix: eliminates dual TypeId/name storage by using a single
/// `HashMap<String, BeanEntry>` for ALL beans, with `type_index` for reverse lookup.
/// 根本修复：通过使用单个 `HashMap<String, BeanEntry>` 存储所有 bean，
/// 并用 `type_index` 做反向查找，消除了 TypeId/名称 双重存储。
struct BeanStore
{
    /// All beans keyed by unique name.
    /// 所有bean以唯一名称为键。
    beans: HashMap<String, BeanEntry>,

    /// TypeId → default bean name (for `get_bean::<T>()` fast path).
    /// TypeId → 默认bean名称（用于 `get_bean::<T>()` 快速路径）。
    type_index: HashMap<TypeId, String>,

    /// TypeId → all bean names of this type (for multi-bean resolution).
    /// TypeId → 此类型的所有bean名称（用于多bean解析）。
    type_to_names: HashMap<TypeId, Vec<String>>,

    /// Currently creating beans (for cycle detection).
    /// 正在创建的bean（用于循环检测）。
    creating: std::cell::RefCell<std::collections::HashSet<String>>,
}

/// A single bean's full lifecycle data.
/// 单个bean的完整生命周期数据。
struct BeanEntry
{
    /// The bean's concrete TypeId.
    /// bean的具体TypeId。
    type_id: TypeId,

    /// The type-erased registration (`BeanRegistration<T>`).
    /// 类型擦除的注册（`BeanRegistration<T>`）。
    registration: Box<dyn Any + Send + Sync>,

    /// Cached singleton instance (None before first creation).
    /// 缓存的单例实例（首次创建前为None）。
    instance: Option<Arc<dyn Any + Send + Sync>>,

    /// Early exposed Weak ref for circular dependency resolution.
    /// 提前暴露的Weak引用，用于循环依赖解析。
    early_exposed: Option<std::sync::Weak<dyn Any + Send + Sync>>,

    /// Lifecycle state.
    /// 生命周期状态。
    state: BeanState,

    /// Whether this bean is lazily initialized.
    /// 是否延迟初始化。
    lazy: bool,

    /// Whether this bean is the primary candidate for its type.
    /// 是否为其类型的主候选bean。
    primary: bool,

    /// Bean scope (Singleton or Prototype).
    /// Bean作用域（Singleton或Prototype）。
    scope: Scope,

    /// Type-erased pre-destroy hook.
    /// 类型擦除的销毁前回调。
    pre_destroy_hook: Option<Box<dyn PreDestroyHook>>,

    /// Eager init function (used by `initialize()`).
    /// 急切初始化函数（由 `initialize()` 使用）。
    eager_init_fn: Option<Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>>,

    /// Declared dependencies for startup verification.
    /// 声明的依赖，用于启动验证。
    dependencies: Vec<super::bean::DependencyInfo>,
}

impl BeanStore
{
    fn new() -> Self
    {
        Self {
            beans: HashMap::new(),
            type_index: HashMap::new(),
            type_to_names: HashMap::new(),
            creating: std::cell::RefCell::new(std::collections::HashSet::new()),
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
/// use hiver_core::Container;
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
pub struct Container
{
    beans: Arc<RwLock<BeanStore>>,
    extensions: Extensions,
    /// Reflection container for dynamic bean operations
    /// 用于动态Bean操作的反射容器
    reflect: Arc<ReflectContainer>,
    /// Registered BeanPostProcessors (Spring BeanPostProcessor)
    /// 注册的BeanPostProcessors（Spring BeanPostProcessor）
    post_processors: Arc<RwLock<Vec<Box<dyn BeanPostProcessor>>>>,
}

impl Container
{
    /// Create a new container
    /// 创建新容器
    pub fn new() -> Self
    {
        Self {
            #[allow(clippy::arc_with_non_send_sync)]
            beans: Arc::new(RwLock::new(BeanStore::new())),
            extensions: Extensions::new(),
            reflect: Arc::new(ReflectContainer::new()),
            post_processors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Acquire read lock on bean store.
    /// 获取bean存储的读锁。
    fn read_beans(&self) -> Result<std::sync::RwLockReadGuard<'_, BeanStore>>
    {
        self.beans
            .read()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))
    }

    /// Acquire write lock on bean store.
    /// 获取bean存储的写锁。
    fn write_beans(&self) -> Result<std::sync::RwLockWriteGuard<'_, BeanStore>>
    {
        self.beans
            .write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))
    }

    // ── BeanPostProcessor support ──────────────────────────────────────

    /// Add a `BeanPostProcessor` to the container.
    /// 向容器添加 `BeanPostProcessor`。
    ///
    /// Equivalent to Spring's `beanFactory.addBeanPostProcessor()`.
    /// 等价于 Spring 的 `beanFactory.addBeanPostProcessor()`。
    ///
    /// Post-processors are applied during bean creation in
    /// `get_bean_by_name`.
    /// 后处理器在 `get_bean_by_name` 中的bean创建期间应用。
    pub fn add_bean_post_processor<P>(&self, processor: P)
    where
        P: BeanPostProcessor + 'static,
    {
        if let Ok(mut pps) = self.post_processors.write()
        {
            pps.push(Box::new(processor));
        }
    }

    /// Apply all registered `BeanPostProcessor`s to a bean — before init.
    /// 将所有注册的 `BeanPostProcessor` 应用于bean —— 初始化之前。
    fn apply_post_processors_before(&self, bean: &mut dyn Any, bean_name: &str) -> Result<()>
    {
        if let Ok(pps) = self.post_processors.read()
        {
            for pp in pps.iter()
            {
                pp.post_process_before_initialization(bean, bean_name)
                    .map_err(|e| {
                        Error::internal(format!("BeanPostProcessor before error: {}", e))
                    })?;
            }
        }
        Ok(())
    }

    /// Apply all registered `BeanPostProcessor`s to a bean — after init.
    /// 将所有注册的 `BeanPostProcessor` 应用于bean —— 初始化之后。
    fn apply_post_processors_after(&self, bean: &mut dyn Any, bean_name: &str) -> Result<()>
    {
        if let Ok(pps) = self.post_processors.read()
        {
            for pp in pps.iter()
            {
                pp.post_process_after_initialization(bean, bean_name)
                    .map_err(|e| {
                        Error::internal(format!("BeanPostProcessor after error: {}", e))
                    })?;
            }
        }
        Ok(())
    }

    // ── Helper: resolve TypeId → bean name ──────────────────────────────

    /// Resolve a TypeId to its bean name via type_index, falling back to type_to_names.
    /// 通过 type_index 将 TypeId 解析为 bean 名称，回退到 type_to_names。
    fn resolve_type_to_name(&self, type_id: TypeId) -> Result<String>
    {
        let beans = self.read_beans()?;

        // Fast path: single default bean for this type
        if let Some(name) = beans.type_index.get(&type_id)
        {
            return Ok(name.clone());
        }

        // Multi-bean: check type_to_names
        let names = beans
            .type_to_names
            .get(&type_id)
            .map_or(&[] as &[String], |n| n.as_slice());

        match names.len()
        {
            0 => Err(Error::not_found(format!("Bean not found: {:?}", type_id))),
            1 =>
            {
                #[allow(clippy::unwrap_used)]
                Ok(names.first().unwrap().clone())
            },
            _ =>
            {
                // Multiple candidates: look for @Primary
                let primary_name = names
                    .iter()
                    .find(|name| beans.beans.get(*name).is_some_and(|e| e.primary));

                if let Some(name) = primary_name
                {
                    Ok(name.clone())
                }
                else
                {
                    Err(Error::internal(format!(
                        "Multiple beans of type {:?} found. Use get_qualified_bean() or \
                         get_bean_by_name() to specify. Candidates: {:?}",
                        type_id, names
                    )))
                }
            },
        }
    }

    // ── Registration ───────────────────────────────────────────────────

    /// Register a bean with a factory function.
    /// 使用工厂函数注册bean。
    ///
    /// Equivalent to Spring's `@Bean` method in `@Configuration` class.
    /// 等价于Spring中`@Configuration`类里的`@Bean`方法。
    pub fn register<T, F>(&mut self, factory: F) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let name = std::any::type_name::<T>().to_string();
        let registration = BeanRegistration::new(&name).factory(Arc::new(factory));

        let mut beans = self.write_beans()?;
        beans.type_index.insert(type_id, name.clone());
        beans
            .type_to_names
            .entry(type_id)
            .or_default()
            .push(name.clone());
        beans.beans.insert(name, BeanEntry {
            type_id,
            registration: Box::new(registration),
            instance: None,
            early_exposed: None,
            state: BeanState::Defined,
            lazy: false,
            primary: false,
            scope: Scope::Singleton,
            pre_destroy_hook: None,
            eager_init_fn: None,
            dependencies: Vec::new(),
        });

        Ok(())
    }

    /// Register a bean with full configuration.
    /// 使用完整配置注册bean。
    pub fn register_with<T>(&mut self, registration: BeanRegistration<T>) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let name = registration.definition.name.clone();

        let hook = registration.pre_destroy.as_ref().map(|pd| {
            Box::new(PreDestroyHookImpl {
                callback: pd.clone(),
            }) as Box<dyn PreDestroyHook>
        });

        let is_lazy = registration.definition.lazy;
        let entry_scope = registration.definition.scope;
        let entry_primary = registration.definition.primary;
        let name_clone = name.clone();

        let eager_fn = Arc::new(move |c: &Container| {
            c.get_bean_by_name::<T>(&name_clone)?;
            Ok(())
        }) as Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>;

        let mut beans = self.write_beans()?;
        beans.type_index.insert(type_id, name.clone());
        beans
            .type_to_names
            .entry(type_id)
            .or_default()
            .push(name.clone());
        beans.beans.insert(name, BeanEntry {
            type_id,
            registration: Box::new(registration),
            instance: None,
            early_exposed: None,
            state: BeanState::Defined,
            lazy: is_lazy,
            primary: entry_primary,
            scope: entry_scope,
            pre_destroy_hook: hook,
            eager_init_fn: Some(eager_fn),
            dependencies: Vec::new(),
        });

        Ok(())
    }

    /// Register a bean instance directly.
    /// 直接注册bean实例。
    pub fn register_bean<T: Bean + Send + Sync + 'static>(&mut self, bean: T) -> Result<()>
    {
        let type_id = TypeId::of::<T>();
        let name = std::any::type_name::<T>().to_string();
        let bean_arc: Arc<dyn Any + Send + Sync> = Arc::new(bean);

        // Check for post-construct callback
        let post_construct_callback = {
            let beans = self.read_beans()?;
            beans.beans.get(&name).and_then(|entry| {
                entry
                    .registration
                    .downcast_ref::<BeanRegistration<T>>()
                    .and_then(|reg_t| reg_t.post_construct.clone())
            })
        };

        if let Some(post_construct) = post_construct_callback
        {
            let bean_ref: &T = bean_arc
                .downcast_ref::<T>()
                .ok_or_else(|| Error::internal("Failed to downcast bean for post-construct"))?;
            if let Err(e) = post_construct(bean_ref)
            {
                return Err(Error::internal(format!(
                    "Post-construct callback failed for {}: {}",
                    name, e
                )));
            }
        }

        let mut beans = self.write_beans()?;
        // Update existing entry or create new one
        if let Some(entry) = beans.beans.get_mut(&name)
        {
            entry.instance = Some(bean_arc);
            entry.state = BeanState::Created;
        }
        else
        {
            beans.type_index.insert(type_id, name.clone());
            beans
                .type_to_names
                .entry(type_id)
                .or_default()
                .push(name.clone());
            let reg_name = name.clone();
            beans.beans.insert(name, BeanEntry {
                type_id,
                registration: Box::new(BeanRegistration::<T>::new(&reg_name)),
                instance: Some(bean_arc),
                early_exposed: None,
                state: BeanState::Created,
                lazy: false,
                primary: false,
                scope: Scope::Singleton,
                pre_destroy_hook: None,
                eager_init_fn: None,
                dependencies: Vec::new(),
            });
        }

        Ok(())
    }

    /// Register a bean factory for lazy initialization.
    /// 注册bean工厂以延迟初始化。
    pub fn register_factory<T, F>(&mut self, factory: F) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.register(move |_c| Ok(factory()))
    }

    /// Register a bean conditionally based on a [`Condition`].
    /// 根据条件 [`Condition`] 有条件地注册Bean。
    pub fn register_conditional<T, F, C>(&mut self, factory: F, condition: &C) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
        C: Condition + 'static,
    {
        let context = {
            let beans = self.read_beans()?;
            let registered_beans: Vec<TypeId> = beans.beans.values().map(|e| e.type_id).collect();
            let bean_names: HashMap<String, TypeId> = beans
                .type_index
                .iter()
                .map(|(tid, name)| (name.clone(), *tid))
                .collect();
            ConditionContext::new()
                .with_registered_beans(registered_beans)
                .with_bean_names(bean_names)
        };

        if condition.matches(&context)
        {
            self.register(factory)
        }
        else
        {
            Ok(())
        }
    }

    /// Register a bean with an explicit qualifier name.
    /// 使用显式限定符名称注册bean。
    pub fn register_named<T, F>(&mut self, name: &str, factory: F) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let registration = BeanRegistration::new(name).factory(Arc::new(factory));

        let mut beans = self.write_beans()?;
        beans
            .type_to_names
            .entry(type_id)
            .or_default()
            .push(name.to_string());
        beans.beans.insert(name.to_string(), BeanEntry {
            type_id,
            registration: Box::new(registration),
            instance: None,
            early_exposed: None,
            state: BeanState::Defined,
            lazy: false,
            primary: false,
            scope: Scope::Singleton,
            pre_destroy_hook: None,
            eager_init_fn: None,
            dependencies: Vec::new(),
        });

        Ok(())
    }

    /// Register a named bean with full lifecycle configuration.
    /// 使用完整生命周期配置注册命名bean。
    pub fn register_named_with<T>(&mut self, registration: BeanRegistration<T>) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let name = registration.definition.name.clone();
        let is_lazy = registration.definition.lazy;
        let entry_scope = registration.definition.scope;
        let entry_primary = registration.definition.primary;

        let hook = registration.pre_destroy.as_ref().map(|pd| {
            Box::new(PreDestroyHookImpl {
                callback: pd.clone(),
            }) as Box<dyn PreDestroyHook>
        });

        let name_clone = name.clone();
        let eager_fn = Arc::new(move |c: &Container| {
            c.get_qualified_bean::<T>(&name_clone)?;
            Ok(())
        }) as Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>;

        let mut beans = self.write_beans()?;
        beans
            .type_to_names
            .entry(type_id)
            .or_default()
            .push(name.clone());
        beans.beans.insert(name, BeanEntry {
            type_id,
            registration: Box::new(registration),
            instance: None,
            early_exposed: None,
            state: BeanState::Defined,
            lazy: is_lazy,
            primary: entry_primary,
            scope: entry_scope,
            pre_destroy_hook: hook,
            eager_init_fn: Some(eager_fn),
            dependencies: Vec::new(),
        });

        Ok(())
    }

    // ── Retrieval ─────────────────────────────────────────────────────

    /// Get a bean by type (resolving dependencies).
    /// 按类型获取bean（解析依赖）。
    pub fn get_bean<T: Bean + Send + Sync + 'static>(&self) -> Result<Arc<T>>
    {
        let type_id = TypeId::of::<T>();
        let name = self.resolve_type_to_name(type_id)?;
        self.get_bean_by_name::<T>(&name)
    }

    /// Get a bean by name.
    /// 按名称获取bean。
    pub fn get_bean_by_name<T: Bean + Send + Sync + 'static>(&self, name: &str) -> Result<Arc<T>>
    {
        // Check if destroyed
        {
            let beans = self.read_beans()?;
            if let Some(entry) = beans.beans.get(name)
            {
                if entry.state == BeanState::Destroyed
                {
                    return Err(Error::internal(format!("Bean {} has been destroyed", name)));
                }

                // Check cached singleton (skip for Prototype scope)
                // 检查缓存的单例（Prototype作用域跳过）
                if entry.scope == Scope::Singleton
                {
                    if let Some(ref instance) = entry.instance
                    {
                        if let Ok(typed) = Arc::clone(instance).downcast::<T>()
                        {
                            return Ok(typed);
                        }
                    }
                }

                // Check circular dependency via early exposed
                if beans.creating.borrow().contains(name)
                {
                    if let Some(entry) = beans.beans.get(name)
                    {
                        if let Some(ref weak) = entry.early_exposed
                        {
                            if let Some(arc) = weak.upgrade()
                            {
                                if let Ok(typed) = arc.downcast::<T>()
                                {
                                    return Ok(typed);
                                }
                            }
                        }
                    }
                    return Err(Error::internal(format!(
                        "Circular dependency detected while creating bean: {}",
                        name
                    )));
                }
            }
            else
            {
                return Err(Error::not_found(format!("Bean not found: {}", name)));
            }
        }

        // Look up factory and create
        let factory = {
            let beans = self.read_beans()?;
            beans
                .beans
                .get(name)
                .and_then(|entry| entry.registration.downcast_ref::<BeanRegistration<T>>())
                .and_then(|reg| reg.factory.clone())
        };

        if let Some(factory) = factory
        {
            // Mark as creating
            {
                let mut beans = self.write_beans()?;
                beans.creating.borrow_mut().insert(name.to_string());
                if let Some(entry) = beans.beans.get_mut(name)
                {
                    entry.state = BeanState::Creating;
                }
            }

            let mut bean: T = factory(self)?;

            // Phase 1: BeanPostProcessor before initialization
            // 第一阶段：初始化前的BeanPostProcessor
            {
                let dyn_bean: &mut dyn Any = &mut bean;
                self.apply_post_processors_before(&mut *dyn_bean, name)?;
            }

            // Phase 2: post-construct callback (@PostConstruct equivalent)
            // 第二阶段：post-construct 回调（等价于 @PostConstruct）
            {
                let beans = self.read_beans()?;
                if let Some(entry) = beans.beans.get(name)
                {
                    if let Some(reg_t) = entry.registration.downcast_ref::<BeanRegistration<T>>()
                    {
                        if let Some(post_construct) = &reg_t.post_construct
                        {
                            post_construct(&bean)?;
                        }
                    }
                }
            }

            // Phase 3: BeanPostProcessor after initialization
            // 第三阶段：初始化后的BeanPostProcessor
            {
                let dyn_bean: &mut dyn Any = &mut bean;
                self.apply_post_processors_after(&mut *dyn_bean, name)?;
            }

            let placeholder: Arc<T> = Arc::new(bean);

            // Update entry: store instance + early exposed (skip for Prototype)
            {
                let mut beans = self.write_beans()?;
                beans.creating.borrow_mut().remove(name);
                if let Some(entry) = beans.beans.get_mut(name)
                {
                    if entry.scope == Scope::Prototype
                    {
                        // Prototype: don't cache, just mark state
                        entry.state = BeanState::Created;
                    }
                    else
                    {
                        entry.early_exposed =
                            Some(Arc::downgrade(&placeholder)
                                as std::sync::Weak<dyn Any + Send + Sync>);
                        entry.instance = Some(placeholder.clone());
                        entry.state = BeanState::Created;
                    }
                }
            }

            Ok(placeholder)
        }
        else
        {
            Err(Error::not_found(format!("Bean not found: {}", name)))
        }
    }

    /// Get a bean by qualifier name.
    /// 通过限定符名称获取bean。
    pub fn get_qualified_bean<T: Bean + Send + Sync + 'static>(
        &self,
        qualifier: &str,
    ) -> Result<Arc<T>>
    {
        // Check cache first
        {
            let beans = self.read_beans()?;
            if let Some(entry) = beans.beans.get(qualifier)
            {
                if let Some(ref instance) = entry.instance
                {
                    if let Ok(typed) = Arc::clone(instance).downcast::<T>()
                    {
                        return Ok(typed);
                    }
                }
            }
        }

        // Delegate to get_bean_by_name which handles creation
        self.get_bean_by_name(qualifier)
    }

    /// Get all beans of a given type.
    /// 获取指定类型的所有bean。
    pub fn get_beans_of_type<T: Bean + Send + Sync + 'static>(&self) -> Vec<(String, Arc<T>)>
    {
        let type_id = TypeId::of::<T>();
        let mut results = Vec::new();

        if let Ok(beans) = self.read_beans()
        {
            for (name, entry) in &beans.beans
            {
                if entry.type_id == type_id
                {
                    if let Some(ref instance) = entry.instance
                    {
                        if let Ok(typed) = Arc::clone(instance).downcast::<T>()
                        {
                            results.push((name.clone(), typed));
                        }
                    }
                }
            }
        }

        results
    }

    /// Check if a bean is registered and available (not destroyed).
    /// 检查bean是否已注册且可用（未销毁）。
    pub fn has_bean<T: Bean + Send + Sync + 'static>(&self) -> bool
    {
        let type_id = TypeId::of::<T>();
        if let Ok(beans) = self.beans.try_read()
        {
            let name = beans
                .type_index
                .get(&type_id)
                .or_else(|| beans.type_to_names.get(&type_id).and_then(|n| n.first()));
            match name
            {
                Some(n) => beans
                    .beans
                    .get(n)
                    .is_some_and(|e| e.state != BeanState::Destroyed),
                None => false,
            }
        }
        else
        {
            false
        }
    }

    /// Get the extensions.
    /// 获取扩展。
    pub fn extensions(&self) -> &Extensions
    {
        &self.extensions
    }

    /// Get a mutable reference to extensions.
    /// 获取扩展的可变引用。
    pub fn extensions_mut(&mut self) -> &mut Extensions
    {
        &mut self.extensions
    }

    /// Get the reflection container.
    /// 获取反射容器。
    pub fn reflect(&self) -> &Arc<ReflectContainer>
    {
        &self.reflect
    }

    /// Get the lifecycle state of a bean.
    /// 获取bean的生命周期状态。
    pub fn bean_state<T: Bean + Send + Sync + 'static>(&self) -> Option<BeanState>
    {
        let type_id = TypeId::of::<T>();
        let beans = self.read_beans().ok()?;

        // Try default name first
        if let Some(name) = beans.type_index.get(&type_id)
        {
            return beans.beans.get(name).map(|e| e.state);
        }

        // Try first named bean
        beans
            .type_to_names
            .get(&type_id)
            .and_then(|names| names.first())
            .and_then(|name| beans.beans.get(name))
            .map(|e| e.state)
    }

    /// Initialize all registered beans (eager initialization).
    /// 初始化所有注册的bean（急切初始化）。
    pub fn initialize(&self) -> Result<()>
    {
        let to_init: Vec<(String, Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>)> = {
            let beans = self.read_beans()?;
            beans
                .beans
                .iter()
                .filter(|(_, entry)| !entry.lazy)
                .filter_map(|(name, entry)| {
                    entry
                        .eager_init_fn
                        .as_ref()
                        .map(|f| (name.clone(), f.clone()))
                })
                .collect()
        };

        for (_, init_fn) in to_init
        {
            init_fn(self)?;
        }

        Ok(())
    }

    /// Shutdown the container, calling pre-destroy callbacks.
    /// 关闭容器，调用销毁前回调。
    pub fn shutdown(&self) -> Result<()>
    {
        let mut beans = self.write_beans()?;

        // Transition all Created beans to Destroying and collect hooks
        let hooks: Vec<(String, Box<dyn PreDestroyHook>)> = beans
            .beans
            .iter_mut()
            .filter_map(|(name, entry)| {
                if entry.state == BeanState::Created
                {
                    entry.state = BeanState::Destroying;
                    entry.pre_destroy_hook.take().map(|h| (name.clone(), h))
                }
                else
                {
                    None
                }
            })
            .collect();

        // Invoke hooks
        for (name, hook) in hooks
        {
            if let Some(entry) = beans.beans.get(&name)
            {
                if let Some(ref instance) = entry.instance
                {
                    let _ = hook.invoke(instance.as_ref());
                }
            }
        }

        // Transition to Destroyed and release instances
        // 转为Destroyed状态并释放实例
        for entry in beans.beans.values_mut()
        {
            entry.instance = None;
            entry.early_exposed = None;
            entry.state = BeanState::Destroyed;
        }

        Ok(())
    }
}

impl Default for Container
{
    fn default() -> Self
    {
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
pub struct ApplicationContext
{
    container: Container,
    profile: String,
    active: bool,
}

impl ApplicationContext
{
    /// Create a new application context
    /// 创建新的应用上下文
    pub fn new() -> Self
    {
        Self {
            container: Container::new(),
            profile: std::env::var("SPRING_PROFILES_ACTIVE")
                .unwrap_or_else(|_| "default".to_string()),
            active: false,
        }
    }

    /// Get the active profile
    /// 获取活动配置文件
    pub fn profile(&self) -> &str
    {
        &self.profile
    }

    /// Set the active profile
    /// 设置活动配置文件
    pub fn set_profile(&mut self, profile: impl Into<String>)
    {
        self.profile = profile.into();
    }

    /// Check if a profile is active
    /// 检查配置文件是否活动
    pub fn accepts_profile(&self, profile: &str) -> bool
    {
        self.profile == profile || self.profile == "default" || profile == "default"
    }

    /// Get the underlying container
    /// 获取底层容器
    pub fn container(&self) -> &Container
    {
        &self.container
    }

    /// Get a mutable reference to the container
    /// 获取容器的可变引用
    pub fn container_mut(&mut self) -> &mut Container
    {
        &mut self.container
    }

    /// Register a bean
    /// 注册bean
    pub fn register<T: Bean + Send + Sync + 'static>(&mut self, bean: T) -> Result<()>
    {
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
    pub fn get_bean<T: Bean + Send + Sync + 'static>(&self) -> Result<Arc<T>>
    {
        self.container.get_bean()
    }

    /// Get a bean by name
    /// 按名称获取bean
    pub fn get_bean_by_name<T: Bean + Send + Sync + 'static>(&self, name: &str) -> Result<Arc<T>>
    {
        self.container.get_bean_by_name(name)
    }

    /// Register a named bean with qualifier
    /// 使用限定符注册命名bean
    pub fn register_named<T, F>(&mut self, name: &str, factory: F) -> Result<()>
    where
        T: Bean + Send + Sync + 'static,
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        self.container.register_named(name, factory)
    }

    /// Get a qualified bean by name
    /// 按名称获取限定bean
    pub fn get_qualified_bean<T: Bean + Send + Sync + 'static>(
        &self,
        qualifier: &str,
    ) -> Result<Arc<T>>
    {
        self.container.get_qualified_bean(qualifier)
    }

    /// Check if a bean exists
    /// 检查bean是否存在
    pub fn contains_bean<T: Bean + Send + Sync + 'static>(&self) -> bool
    {
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
    pub fn refresh(&mut self) -> Result<()>
    {
        // Step 1: Invoke pre-destroy callbacks on created beans
        // 步骤1：在已创建的bean上调用销毁前回调
        {
            let beans = self.container.read_beans()?;
            for entry in beans.beans.values()
            {
                if entry.state == BeanState::Created
                {
                    if let Some(ref hook) = entry.pre_destroy_hook
                    {
                        if let Some(ref instance) = entry.instance
                        {
                            let _ = hook.invoke(instance.as_ref());
                        }
                    }
                }
            }
        }

        // Step 2: Reset instances but keep registrations
        // 步骤2：重置实例但保留注册
        {
            let mut beans = self.container.write_beans()?;
            for entry in beans.beans.values_mut()
            {
                entry.instance = None;
                entry.early_exposed = None;
                entry.state = BeanState::Defined;
            }
        }

        // Step 3: Re-initialize eager beans
        // 步骤3：重新初始化急切bean
        self.active = false;
        self.start()?;

        Ok(())
    }

    /// Start the context (initialize all eager singletons)
    /// 启动上下文（初始化所有急切单例）
    pub fn start(&mut self) -> Result<()>
    {
        self.active = true;
        self.container.initialize()
    }

    /// Close the context and release resources
    /// 关闭上下文并释放资源
    pub fn close(self) -> Result<()>
    {
        self.container.shutdown()
    }

    /// Check if context is active
    /// 检查上下文是否活动
    pub fn is_active(&self) -> bool
    {
        self.active
    }
}

impl Default for ApplicationContext
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Component scanner (equivalent to @`ComponentScan`)
/// 组件扫描器（等价于 @`ComponentScan`）
pub struct ComponentScanner
{
    base_packages: Vec<String>,
}

impl ComponentScanner
{
    /// Create a new scanner
    /// 创建新扫描器
    pub fn new() -> Self
    {
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
    pub fn scan_package(mut self, package: impl Into<String>) -> Self
    {
        self.base_packages.push(package.into());
        self
    }

    /// Scan for components and register them
    /// 扫描组件并注册它们
    ///
    /// Note: In Rust, true runtime component scanning is not possible like in Java.
    /// Instead, this framework uses proc-macros for compile-time component registration.
    /// Use the `#[hiver_macros::component]` attribute to register components at compile time.
    ///
    /// 注意：在Rust中，像Java那样的真正运行时组件扫描是不可能的。
    /// 相反，此框架使用proc宏进行编译时组件注册。
    /// 使用 `#[hiver_macros::component]` 属性在编译时注册组件。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_core::container::ComponentScanner;
    /// use hiver_macros::component;
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
    pub fn scan(&self, _context: &mut ApplicationContext) -> Result<()>
    {
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
    ) -> Result<()>
    {
        // The proc-macro will generate a call to register_bean for each component
        // proc宏将为每个组件生成对register_bean的调用
        Ok(())
    }
}

impl Default for ComponentScanner
{
    fn default() -> Self
    {
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
/// use hiver_core::container::PostConstruct;
///
/// struct MyService {
///     initialized: bool,
/// }
///
/// impl PostConstruct for MyService {
///     fn post_construct(&self) -> Result<(), hiver_core::Error> {
///         println!("Service initialized!");
///         Ok(())
///     }
/// }
/// ```
pub trait PostConstruct
{
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
/// use hiver_core::container::PreDestroy;
///
/// struct MyService {
///     connection: Option<Database>,
/// }
///
/// impl PreDestroy for MyService {
///     fn pre_destroy(&self) -> Result<(), hiver_core::Error> {
///         if let Some(conn) = &self.connection {
///             conn.close();
///         }
///         println!("Service destroyed!");
///         Ok(())
///     }
/// }
/// ```
pub trait PreDestroy
{
    /// Called before the bean is destroyed
    /// 在bean销毁前调用
    fn pre_destroy(&self) -> Result<()>;
}

// ============================================================================
// BeanRegistrar — Builder for dependency-aware bean registration
// BeanRegistrar — 带依赖声明的 Bean 注册构建器
// ============================================================================

/// Builder for registering a bean with declared dependencies.
/// 带依赖声明的 Bean 注册构建器。
///
/// # Example / 示例
///
/// ```rust,ignore
/// container.register_bean::<UserService>()
///     .depends_on::<UserRepository>()
///     .depends_on::<EmailService>()
///     .factory(|c| {
///         Ok(UserService::new(
///             c.get_bean::<UserRepository>()?,
///             c.get_bean::<EmailService>()?,
///         ))
///     })
///     .build()?;
/// ```
pub struct BeanRegistrar<'a, T>
where
    T: Bean + Send + Sync + 'static,
{
    container: &'a mut Container,
    _marker: PhantomData<T>,
    deps: Vec<DependencyInfo>,
    factory: Option<Arc<dyn Fn(&Container) -> Result<T> + Send + Sync>>,
    scope: Scope,
    primary: bool,
    lazy: bool,
    name: String,
}

impl<'a, T: Bean + Send + Sync + 'static> BeanRegistrar<'a, T>
{
    fn new(container: &'a mut Container) -> Self
    {
        Self {
            container,
            _marker: PhantomData,
            deps: Vec::new(),
            factory: None,
            scope: Scope::Singleton,
            primary: false,
            lazy: false,
            name: std::any::type_name::<T>().to_string(),
        }
    }

    /// Declare a dependency on another bean type.
    /// 声明对另一个 bean 类型的依赖。
    pub fn depends_on<D: 'static>(mut self) -> Self
    {
        self.deps.push(DependencyInfo::of::<D>());
        self
    }

    /// Use dependencies declared via `BeanDependencies` trait.
    /// 使用通过 `BeanDependencies` trait 声明的依赖。
    ///
    /// The `#[derive(Bean)]` macro with `#[bean(depends(...))]` generates
    /// the `BeanDependencies` impl automatically.
    ///
    /// `#[derive(Bean)]` 宏配合 `#[bean(depends(...))]` 会自动生成
    /// `BeanDependencies` 实现。
    pub fn with_declared_deps(mut self) -> Self
    where
        T: super::bean::BeanDependencies,
    {
        self.deps = T::dependencies();
        self
    }

    /// Set the factory function.
    /// 设置工厂函数。
    pub fn factory<F>(mut self, f: F) -> Self
    where
        F: Fn(&Container) -> Result<T> + Send + Sync + 'static,
    {
        self.factory = Some(Arc::new(f));
        self
    }

    /// Set the scope.
    /// 设置作用域。
    pub fn scope(mut self, scope: Scope) -> Self
    {
        self.scope = scope;
        self
    }

    /// Mark this bean as the primary candidate for its type.
    /// 标记此 bean 为其类型的首选候选。
    pub fn primary(mut self) -> Self
    {
        self.primary = true;
        self
    }

    /// Set lazy initialization.
    /// 设置延迟初始化。
    pub fn lazy(mut self) -> Self
    {
        self.lazy = true;
        self
    }

    /// Set the bean name explicitly.
    /// 显式设置 bean 名称。
    pub fn named(mut self, name: impl Into<String>) -> Self
    {
        self.name = name.into();
        self
    }

    /// Build and register the bean with its declared dependencies.
    /// 构建并注册 bean 及其声明的依赖。
    pub fn build(self) -> Result<()>
    {
        let factory = self.factory.ok_or_else(|| {
            Error::internal(format!(
                "Bean '{}' registered without a factory function. Call .factory() before .build()",
                self.name
            ))
        })?;

        let type_id = TypeId::of::<T>();
        let registration = BeanRegistration::new(&self.name).factory(factory);

        let name_clone = self.name.clone();
        let eager_fn = if !self.lazy
        {
            Some(Arc::new(move |c: &Container| {
                c.get_bean_by_name::<T>(&name_clone)?;
                Ok(())
            }) as Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>)
        }
        else
        {
            None
        };

        let mut beans = self.container.write_beans()?;
        beans.type_index.insert(type_id, self.name.clone());
        beans
            .type_to_names
            .entry(type_id)
            .or_default()
            .push(self.name.clone());
        beans.beans.insert(self.name, BeanEntry {
            type_id,
            registration: Box::new(registration),
            instance: None,
            early_exposed: None,
            state: BeanState::Defined,
            lazy: self.lazy,
            primary: self.primary,
            scope: self.scope,
            pre_destroy_hook: None,
            eager_init_fn: eager_fn,
            dependencies: self.deps,
        });

        Ok(())
    }
}

// ============================================================================
// Container — Dependency Verification & Initialization
// Container — 依赖验证与初始化
// ============================================================================

impl Container
{
    /// Start registering a bean with the builder pattern.
    /// 使用构建器模式开始注册 bean。
    ///
    /// Returns a `BeanRegistrar` that allows declaring dependencies,
    /// setting scope, and configuring the factory function.
    ///
    /// 返回 `BeanRegistrar`，允许声明依赖、设置作用域和配置工厂函数。
    ///
    /// # Example / 示例
    ///
    /// ```rust,ignore
    /// container.bean_builder::<UserService>()
    ///     .depends_on::<UserRepository>()
    ///     .factory(|c| Ok(UserService::new(c.get_bean()?)))
    ///     .build()?;
    /// ```
    pub fn bean_builder<T: Bean + Send + Sync + 'static>(&mut self) -> BeanRegistrar<'_, T>
    {
        BeanRegistrar::new(self)
    }

    /// Verify that all declared bean dependencies are satisfied.
    /// 验证所有声明的 bean 依赖是否满足。
    ///
    /// Call this after all beans are registered but before creating them.
    /// Returns a list of warnings for missing dependencies.
    ///
    /// 在所有 bean 注册完成后、创建之前调用此方法。
    /// 返回缺失依赖的警告列表。
    ///
    /// Equivalent to Spring's `ApplicationContext.refresh()` validation step.
    /// 等价于 Spring 的 `ApplicationContext.refresh()` 验证步骤。
    pub fn verify_dependencies(&self) -> Result<Vec<String>>
    {
        let beans = self.read_beans()?;
        let registered: HashSet<TypeId> = beans.type_index.keys().copied().collect();

        let mut warnings = Vec::new();
        for (name, entry) in &beans.beans
        {
            for dep in &entry.dependencies
            {
                if !registered.contains(&dep.type_id)
                {
                    warnings.push(format!(
                        "Bean '{}' depends on '{}' which is not registered",
                        name, dep.type_name
                    ));
                }
            }
        }
        Ok(warnings)
    }
}

#[cfg(test)]
mod container_tests;
