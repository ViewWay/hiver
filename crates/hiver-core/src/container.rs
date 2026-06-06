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
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::{
    bean::{Bean, BeanDefinition, BeanState, Scope},
    conditional::{Condition, ConditionContext},
    error::{Error, Result},
    extension::Extensions,
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
            .map(|n| n.as_slice())
            .unwrap_or(&[]);

        match names.len()
        {
            0 => Err(Error::not_found(format!(
                "Bean not found: {:?}",
                type_id
            ))),
            1 => Ok(names[0].clone()),
            _ =>
            {
                // Multiple candidates: look for @Primary
                let primary_name = names.iter().find(|name| {
                    beans
                        .beans
                        .get(*name)
                        .map_or(false, |e| e.primary)
                });

                if let Some(name) = primary_name
                {
                    Ok(name.clone())
                }
                else
                {
                    Err(Error::internal(format!(
                        "Multiple beans of type {:?} found. \
                         Use get_qualified_bean() or get_bean_by_name() to specify. \
                         Candidates: {:?}",
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
        beans.type_to_names.entry(type_id).or_default().push(name.clone());
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
            Box::new(PreDestroyHookImpl { callback: pd.clone() }) as Box<dyn PreDestroyHook>
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
        beans.type_to_names.entry(type_id).or_default().push(name.clone());
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
                entry.registration
                    .downcast_ref::<BeanRegistration<T>>()
                    .and_then(|reg_t| reg_t.post_construct.clone())
            })
        };

        if let Some(post_construct) = post_construct_callback
        {
            let bean_ref: &T = bean_arc.downcast_ref::<T>().ok_or_else(|| {
                Error::internal("Failed to downcast bean for post-construct")
            })?;
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
            beans.type_to_names.entry(type_id).or_default().push(name.clone());
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
            let registered_beans: Vec<TypeId> = beans.beans.values()
                .map(|e| e.type_id)
                .collect();
            let bean_names: HashMap<String, TypeId> = beans.type_index.iter()
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
        beans.type_to_names.entry(type_id).or_default().push(name.to_string());
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
            Box::new(PreDestroyHookImpl { callback: pd.clone() }) as Box<dyn PreDestroyHook>
        });

        let name_clone = name.clone();
        let eager_fn = Arc::new(move |c: &Container| {
            c.get_qualified_bean::<T>(&name_clone)?;
            Ok(())
        }) as Arc<dyn Fn(&Container) -> Result<()> + Send + Sync>;

        let mut beans = self.write_beans()?;
        beans.type_to_names.entry(type_id).or_default().push(name.clone());
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
                    return Err(Error::internal(format!(
                        "Bean {} has been destroyed", name
                    )));
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
                        "Circular dependency detected while creating bean: {}", name
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
            beans.beans.get(name)
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

            let placeholder: Arc<T> = {
                let bean = factory(self)?;
                Arc::new(bean)
            };

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
                        entry.early_exposed = Some(
                            Arc::downgrade(&placeholder) as std::sync::Weak<dyn Any + Send + Sync>
                        );
                        entry.instance = Some(placeholder.clone());
                        entry.state = BeanState::Created;
                    }
                }
            }

            // Call post-construct if available
            {
                let beans = self.read_beans()?;
                if let Some(entry) = beans.beans.get(name)
                {
                    if let Some(reg_t) = entry.registration.downcast_ref::<BeanRegistration<T>>()
                    {
                        if let Some(post_construct) = &reg_t.post_construct
                        {
                            post_construct(&placeholder)?;
                        }
                    }
                }
            }

            Ok(placeholder)
        }
        else
        {
            Err(Error::not_found(format!(
                "Bean not found: {}", name
            )))
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
            let name = beans.type_index.get(&type_id)
                .or_else(|| beans.type_to_names.get(&type_id).and_then(|n| n.first()));
            match name
            {
                Some(n) => beans.beans.get(n).map_or(false, |e| e.state != BeanState::Destroyed),
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
        beans.type_to_names.get(&type_id)
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
            beans.beans.iter()
                .filter(|(_, entry)| !entry.lazy)
                .filter_map(|(name, entry)| {
                    entry.eager_init_fn.as_ref().map(|f| (name.clone(), f.clone()))
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
        let hooks: Vec<(String, Box<dyn PreDestroyHook>)> = beans.beans.iter_mut()
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
            for (_name, entry) in &beans.beans
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

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;
    use crate::conditional::{ConditionalOnBean, ConditionalOnMissingBean};

    // ── Test fixtures / 测试夹具 ────────────────────────────────────────

    #[derive(Debug, Default)]
    struct UserRepository
    {
        initialized: bool,
    }


    impl PostConstruct for UserRepository
    {
        fn post_construct(&self) -> Result<()>
        {
            Ok(())
        }
    }

    impl PreDestroy for UserRepository
    {
        fn pre_destroy(&self) -> Result<()>
        {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct UserService
    {
        user_count: u32,
    }

    #[derive(Debug, Default)]
    struct EmailService
    {
        sent_count: u32,
    }

    #[derive(Debug)]
    struct CacheService
    {
        hits: u64,
    }

    #[derive(Debug, Default)]
    struct AuditService;

    // ── Container::new / Container::default ────────────────────────────

    #[test]
    fn test_container_new()
    {
        let container = Container::new();
        assert!(!container.has_bean::<UserRepository>());
    }

    #[test]
    fn test_container_default()
    {
        let container = Container::default();
        assert!(!container.has_bean::<UserService>());
    }

    #[test]
    fn test_container_clone_independent()
    {
        let mut container = Container::new();
        container.register(|_| Ok(EmailService::default())).unwrap();
        // Clone shares underlying Arc<RwLock<>> so beans are shared / Clone共享底层Arc<RwLock<>>
        let cloned = container.clone();
        assert!(cloned.has_bean::<EmailService>());
    }

    // ── register / get_bean ────────────────────────────────────────────

    #[test]
    fn test_register_and_get_bean()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserRepository::default()))
            .unwrap();
        let bean = container.get_bean::<UserRepository>().unwrap();
        assert!(!bean.initialized);
    }

    #[test]
    fn test_register_factory_creates_instance()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 42 }))
            .unwrap();
        let bean = container.get_bean::<UserService>().unwrap();
        assert_eq!(bean.user_count, 42);
    }

    #[test]
    fn test_get_bean_missing_returns_error()
    {
        let container = Container::new();
        let result = container.get_bean::<UserService>();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_bean_singleton_identity()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 1 }))
            .unwrap();
        let first = container.get_bean::<UserService>().unwrap();
        let second = container.get_bean::<UserService>().unwrap();
        // Same Arc / 同一个Arc
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn test_register_factory_simple()
    {
        let mut container = Container::new();
        container
            .register_factory(|| EmailService { sent_count: 5 })
            .unwrap();
        let bean = container.get_bean::<EmailService>().unwrap();
        assert_eq!(bean.sent_count, 5);
    }

    #[test]
    fn test_register_factory_default()
    {
        let mut container = Container::new();
        container
            .register_factory(|| EmailService::default())
            .unwrap();
        let bean = container.get_bean::<EmailService>().unwrap();
        assert_eq!(bean.sent_count, 0);
    }

    // ── register_bean (direct instance) ────────────────────────────────

    #[test]
    fn test_register_bean_direct()
    {
        let mut container = Container::new();
        container
            .register_bean(UserRepository { initialized: true })
            .unwrap();
        let bean = container.get_bean::<UserRepository>().unwrap();
        assert!(bean.initialized);
    }

    // ── has_bean ───────────────────────────────────────────────────────

    #[test]
    fn test_has_bean_false_initially()
    {
        let container = Container::new();
        assert!(!container.has_bean::<UserService>());
    }

    #[test]
    fn test_has_bean_true_after_register()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        assert!(container.has_bean::<UserService>());
    }

    #[test]
    fn test_has_bean_true_after_register_bean()
    {
        let mut container = Container::new();
        container.register_bean(EmailService::default()).unwrap();
        assert!(container.has_bean::<EmailService>());
    }

    #[test]
    fn test_has_bean_false_for_unregistered_type()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        assert!(!container.has_bean::<EmailService>());
    }

    // ── get_bean_by_name ───────────────────────────────────────────────

    #[test]
    fn test_get_bean_by_name()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 7 }))
            .unwrap();
        let type_name = std::any::type_name::<UserService>();
        let bean = container
            .get_bean_by_name::<UserService>(type_name)
            .unwrap();
        assert_eq!(bean.user_count, 7);
    }

    #[test]
    fn test_get_bean_by_name_missing()
    {
        let container = Container::new();
        let result = container.get_bean_by_name::<UserService>("nonexistent");
        assert!(result.is_err());
    }

    // ── register_with (full configuration) ─────────────────────────────

    #[test]
    fn test_register_with_factory()
    {
        let mut container = Container::new();
        let reg = BeanRegistration::new("custom_service")
            .factory(Arc::new(|_| Ok(UserService { user_count: 99 })));
        container.register_with(reg).unwrap();
        let bean = container.get_bean::<UserService>().unwrap();
        assert_eq!(bean.user_count, 99);
    }

    #[test]
    fn test_register_with_post_construct()
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
            .post_construct(move |_bean| {
                called_clone.store(true, Ordering::SeqCst);
                Ok(())
            });
        container.register_with(reg).unwrap();
        container.get_bean::<UserService>().unwrap();
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_register_with_pre_destroy()
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        let destroyed = Arc::new(AtomicBool::new(false));
        let destroyed_clone = destroyed.clone();

        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
            .pre_destroy(move |_bean| {
                destroyed_clone.store(true, Ordering::SeqCst);
                Ok(())
            });
        container.register_with(reg).unwrap();
        container.get_bean::<UserService>().unwrap();
        container.shutdown().unwrap();
        assert!(destroyed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_register_with_scope()
    {
        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
            .scope(Scope::Prototype);
        container.register_with(reg).unwrap();
        assert!(container.has_bean::<UserService>());
    }

    #[test]
    fn test_register_with_primary()
    {
        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
            .primary(true);
        container.register_with(reg).unwrap();
        assert!(container.has_bean::<UserService>());
    }

    #[test]
    fn test_register_with_lazy()
    {
        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
            .lazy(true);
        container.register_with(reg).unwrap();
        assert!(container.has_bean::<UserService>());
    }

    // ── Dependency injection ───────────────────────────────────────────

    #[test]
    fn test_dependency_injection()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserRepository::default()))
            .unwrap();
        container
            .register(|c| {
                let _repo = c.get_bean::<UserRepository>()?;
                Ok(UserService { user_count: 0 })
            })
            .unwrap();
        let service = container.get_bean::<UserService>().unwrap();
        assert_eq!(service.user_count, 0);
    }

    // ── shutdown ───────────────────────────────────────────────────────

    #[test]
    fn test_shutdown_clears_beans()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        container.get_bean::<UserService>().unwrap();
        container.shutdown().unwrap();
        assert!(!container.has_bean::<UserService>());
    }

    #[test]
    fn test_shutdown_on_empty_container()
    {
        let container = Container::new();
        // Should not panic / 不应panic
        container.shutdown().unwrap();
    }

    // ── initialize ─────────────────────────────────────────────────────

    #[test]
    fn test_initialize_no_error()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        container.initialize().unwrap();
    }

    #[test]
    fn test_initialize_creates_eager_beans()
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        let created = Arc::new(AtomicBool::new(false));
        let created_clone = created.clone();

        let mut container = Container::new();
        let reg = BeanRegistration::new("svc").factory(Arc::new(move |_| {
            created_clone.store(true, Ordering::SeqCst);
            Ok(UserService { user_count: 42 })
        }));
        container.register_with(reg).unwrap();

        assert!(!created.load(Ordering::SeqCst));

        container.initialize().unwrap();

        assert!(created.load(Ordering::SeqCst));
        let bean = container.get_bean::<UserService>().unwrap();
        assert_eq!(bean.user_count, 42);
    }

    #[test]
    fn test_initialize_skips_lazy_beans()
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        let created = Arc::new(AtomicBool::new(false));
        let created_clone = created.clone();

        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(move |_| {
                created_clone.store(true, Ordering::SeqCst);
                Ok(UserService { user_count: 99 })
            }))
            .lazy(true);
        container.register_with(reg).unwrap();

        container.initialize().unwrap();

        // Factory should NOT have been called
        assert!(!created.load(Ordering::SeqCst));

        // But it works on first get_bean
        let bean = container.get_bean::<UserService>().unwrap();
        assert!(created.load(Ordering::SeqCst));
        assert_eq!(bean.user_count, 99);
    }

    #[test]
    fn test_initialize_mixed_lazy_and_eager()
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        let eager_created = Arc::new(AtomicBool::new(false));
        let eager_clone = eager_created.clone();

        let mut container = Container::new();
        let reg_eager = BeanRegistration::new("eager").factory(Arc::new(move |_| {
            eager_clone.store(true, Ordering::SeqCst);
            Ok(UserService { user_count: 1 })
        }));
        container.register_with(reg_eager).unwrap();

        container.initialize().unwrap();
        assert!(eager_created.load(Ordering::SeqCst));
    }

    // ── Extensions ─────────────────────────────────────────────────────

    #[test]
    fn test_container_extensions()
    {
        let mut container = Container::new();
        container.extensions_mut().insert("test".to_string());
        assert_eq!(container.extensions().get::<String>(), Some(&"test".to_string()));
    }

    #[test]
    fn test_container_extensions_mut()
    {
        let mut container = Container::new();
        container.extensions_mut().insert(42i32);
        if let Some(v) = container.extensions_mut().get_mut::<i32>()
        {
            *v = 100;
        }
        assert_eq!(container.extensions().get::<i32>(), Some(&100));
    }

    // ── Reflect ────────────────────────────────────────────────────────

    #[test]
    fn test_container_reflect()
    {
        let container = Container::new();
        let _reflect = container.reflect();
    }

    // ── register_conditional ───────────────────────────────────────────

    #[test]
    fn test_register_conditional_on_missing_bean_registers_when_absent()
    {
        let mut container = Container::new();
        let cond = ConditionalOnMissingBean::of::<CacheService>();
        container
            .register_conditional(|_| Ok(CacheService { hits: 0 }), &cond)
            .unwrap();
        assert!(container.has_bean::<CacheService>());
    }

    #[test]
    fn test_register_conditional_on_missing_bean_skips_when_present()
    {
        let mut container = Container::new();
        // First register / 先注册
        container
            .register(|_| Ok(CacheService { hits: 10 }))
            .unwrap();
        container.get_bean::<CacheService>().unwrap();
        // Second conditional should still register (registrations are independent)
        // 第二次条件注册仍会注册（注册是独立的）
        let cond = ConditionalOnMissingBean::of::<CacheService>();
        container
            .register_conditional(|_| Ok(CacheService { hits: 20 }), &cond)
            .unwrap();
    }

    #[test]
    fn test_register_conditional_on_bean_registers_when_present()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserRepository::default()))
            .unwrap();
        let cond = ConditionalOnBean::of::<UserRepository>();
        container
            .register_conditional(|_| Ok(UserService { user_count: 0 }), &cond)
            .unwrap();
        assert!(container.has_bean::<UserService>());
    }

    // ── ApplicationContext ─────────────────────────────────────────────

    #[test]
    fn test_application_context_new()
    {
        let ctx = ApplicationContext::new();
        assert_eq!(ctx.profile(), "default");
        assert!(!ctx.is_active());
    }

    #[test]
    fn test_application_context_default()
    {
        let ctx = ApplicationContext::default();
        assert_eq!(ctx.profile(), "default");
    }

    #[test]
    fn test_application_context_set_profile()
    {
        let mut ctx = ApplicationContext::new();
        ctx.set_profile("production");
        assert_eq!(ctx.profile(), "production");
    }

    #[test]
    fn test_application_context_accepts_profile_default()
    {
        let ctx = ApplicationContext::new();
        assert!(ctx.accepts_profile("default"));
        assert!(ctx.accepts_profile("anything")); // "default" context accepts all / "default"上下文接受所有
    }

    #[test]
    fn test_application_context_accepts_profile_specific()
    {
        let mut ctx = ApplicationContext::new();
        ctx.set_profile("staging");
        assert!(ctx.accepts_profile("staging"));
        assert!(!ctx.accepts_profile("production"));
        assert!(ctx.accepts_profile("default")); // "default" profile always accepted / "default"配置始终接受
    }

    #[test]
    fn test_application_context_start()
    {
        let mut ctx = ApplicationContext::new();
        ctx.start().unwrap();
        assert!(ctx.is_active());
    }

    #[test]
    fn test_application_context_register_and_get()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register(EmailService::default()).unwrap();
        let bean = ctx.get_bean::<EmailService>().unwrap();
        assert_eq!(bean.sent_count, 0);
    }

    #[test]
    fn test_application_context_register_with_factory()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register_with(|_| Ok(UserService { user_count: 5 }))
            .unwrap();
        let bean = ctx.get_bean::<UserService>().unwrap();
        assert_eq!(bean.user_count, 5);
    }

    #[test]
    fn test_application_context_contains_bean()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register(AuditService).unwrap();
        assert!(ctx.contains_bean::<AuditService>());
        assert!(!ctx.contains_bean::<UserService>());
    }

    #[test]
    fn test_application_context_get_bean_by_name()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register_with(|_| Ok(UserService { user_count: 3 }))
            .unwrap();
        let type_name = std::any::type_name::<UserService>();
        let bean = ctx.get_bean_by_name::<UserService>(type_name).unwrap();
        assert_eq!(bean.user_count, 3);
    }

    #[test]
    fn test_application_context_close()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register(EmailService::default()).unwrap();
        ctx.start().unwrap();
        ctx.close().unwrap();
    }

    #[test]
    fn test_application_context_refresh()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register_with(|_| Ok(UserService { user_count: 1 }))
            .unwrap();
        ctx.start().unwrap();
        ctx.refresh().unwrap();
        assert!(ctx.is_active());
    }

    #[test]
    fn test_application_context_container_access()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register_with(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        // Immutable access / 不可变访问
        assert!(ctx.container().has_bean::<UserService>());
        // Mutable access / 可变访问
        ctx.container_mut()
            .register(|_| Ok(EmailService::default()))
            .unwrap();
        assert!(ctx.container().has_bean::<EmailService>());
    }

    // ── ComponentScanner ───────────────────────────────────────────────

    #[test]
    fn test_component_scanner_new()
    {
        let scanner = ComponentScanner::new();
        let mut ctx = ApplicationContext::new();
        scanner.scan(&mut ctx).unwrap();
    }

    #[test]
    fn test_component_scanner_default()
    {
        let scanner = ComponentScanner::default();
        let mut ctx = ApplicationContext::new();
        scanner.scan(&mut ctx).unwrap();
    }

    #[test]
    fn test_component_scanner_scan_package_builder()
    {
        let scanner = ComponentScanner::new()
            .scan_package("com.example")
            .scan_package("com.other");
        let mut ctx = ApplicationContext::new();
        scanner.scan(&mut ctx).unwrap();
    }

    #[test]
    fn test_component_scanner_register_component()
    {
        let scanner = ComponentScanner::new();
        let mut ctx = ApplicationContext::new();
        scanner.register_component::<UserService>(&mut ctx).unwrap();
    }

    // ── PostConstruct / PreDestroy traits ──────────────────────────────

    #[test]
    fn test_post_construct_trait()
    {
        struct MySvc;
        impl PostConstruct for MySvc
        {
            fn post_construct(&self) -> Result<()>
            {
                Ok(())
            }
        }
        let svc = MySvc;
        assert!(svc.post_construct().is_ok());
    }

    #[test]
    fn test_pre_destroy_trait()
    {
        struct MySvc;
        impl PreDestroy for MySvc
        {
            fn pre_destroy(&self) -> Result<()>
            {
                Ok(())
            }
        }
        let svc = MySvc;
        assert!(svc.pre_destroy().is_ok());
    }

    // ── Edge cases / 边界情况 ──────────────────────────────────────────

    #[test]
    fn test_register_multiple_different_types()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 1 }))
            .unwrap();
        container
            .register(|_| Ok(EmailService { sent_count: 2 }))
            .unwrap();
        container
            .register(|_| Ok(CacheService { hits: 3 }))
            .unwrap();
        let user = container.get_bean::<UserService>().unwrap();
        let email = container.get_bean::<EmailService>().unwrap();
        let cache = container.get_bean::<CacheService>().unwrap();
        assert_eq!(user.user_count, 1);
        assert_eq!(email.sent_count, 2);
        assert_eq!(cache.hits, 3);
    }

    #[test]
    fn test_get_bean_after_shutdown_returns_error()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        container.get_bean::<UserService>().unwrap();
        container.shutdown().unwrap();
        let result = container.get_bean::<UserService>();
        assert!(result.is_err());
    }

    #[test]
    fn test_bean_registration_builder()
    {
        let reg: BeanRegistration<UserService> = BeanRegistration::new("test_svc")
            .scope(Scope::Prototype)
            .primary(true)
            .lazy(true);
        assert_eq!(reg.definition.name, "test_svc");
        assert_eq!(reg.definition.scope, Scope::Prototype);
        assert!(reg.definition.primary);
        assert!(reg.definition.lazy);
    }

    #[test]
    fn test_bean_registration_new_defaults()
    {
        let reg: BeanRegistration<UserService> = BeanRegistration::new("svc");
        assert_eq!(reg.definition.name, "svc");
        assert!(reg.factory.is_none());
        assert!(reg.post_construct.is_none());
        assert!(reg.pre_destroy.is_none());
        assert_eq!(reg.definition.scope, Scope::Singleton);
        assert!(!reg.definition.primary);
        assert!(!reg.definition.lazy);
    }

    // ── Additional container tests / 额外容器测试 ──────────────────────

    #[test]
    fn test_register_bean_overwrite()
    {
        let mut container = Container::new();
        container
            .register_bean(UserService { user_count: 1 })
            .unwrap();
        // Register again overwrites / 再次注册会覆盖
        container
            .register_bean(UserService { user_count: 99 })
            .unwrap();
        let bean = container.get_bean::<UserService>().unwrap();
        assert_eq!(bean.user_count, 99);
    }

    #[test]
    fn test_register_factory_with_container_access()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserRepository { initialized: true }))
            .unwrap();
        container
            .register(|c| {
                let repo = c.get_bean::<UserRepository>()?;
                Ok(UserService {
                    user_count: if repo.initialized { 100 } else { 0 },
                })
            })
            .unwrap();
        let svc = container.get_bean::<UserService>().unwrap();
        assert_eq!(svc.user_count, 100);
    }

    #[test]
    fn test_application_context_not_active_before_start()
    {
        let ctx = ApplicationContext::new();
        assert!(!ctx.is_active());
    }

    #[test]
    fn test_application_context_set_profile_multiple_times()
    {
        let mut ctx = ApplicationContext::new();
        ctx.set_profile("dev");
        assert_eq!(ctx.profile(), "dev");
        ctx.set_profile("prod");
        assert_eq!(ctx.profile(), "prod");
    }

    #[test]
    fn test_application_context_accepts_profile_default_always()
    {
        let mut ctx = ApplicationContext::new();
        ctx.set_profile("custom");
        // "default" profile is always accepted / "default"配置始终被接受
        assert!(ctx.accepts_profile("default"));
        assert!(ctx.accepts_profile("custom"));
        assert!(!ctx.accepts_profile("other"));
    }

    #[test]
    fn test_application_context_register_unit_type()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register(AuditService).unwrap();
        assert!(ctx.contains_bean::<AuditService>());
        let _bean = ctx.get_bean::<AuditService>().unwrap();
    }

    #[test]
    fn test_shutdown_twice()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        container.get_bean::<UserService>().unwrap();
        container.shutdown().unwrap();
        // Second shutdown on already-cleared container / 第二次关闭已清空的容器
        container.shutdown().unwrap();
    }

    #[test]
    fn test_get_bean_by_name_after_lazy_creation()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(CacheService { hits: 42 }))
            .unwrap();
        // First get by name triggers creation / 首次按名称获取触发创建
        let type_name = std::any::type_name::<CacheService>();
        let bean1 = container
            .get_bean_by_name::<CacheService>(type_name)
            .unwrap();
        assert_eq!(bean1.hits, 42);
        // Second get returns same singleton / 第二次获取返回同一单例
        let bean2 = container
            .get_bean_by_name::<CacheService>(type_name)
            .unwrap();
        assert!(Arc::ptr_eq(&bean1, &bean2));
    }

    #[test]
    fn test_container_extensions_isolation()
    {
        let mut container = Container::new();
        container.extensions_mut().insert(42i32);
        // Extensions are separate from beans / 扩展与bean分离
        assert!(!container.has_bean::<i32>());
        assert_eq!(container.extensions().get::<i32>(), Some(&42));
    }

    #[test]
    fn test_register_conditional_on_bean_skips_when_absent()
    {
        let mut container = Container::new();
        let cond = ConditionalOnBean::of::<UserService>();
        container
            .register_conditional(|_| Ok(EmailService { sent_count: 0 }), &cond)
            .unwrap();
        assert!(!container.has_bean::<EmailService>());
    }

    // ── BeanState lifecycle tests / BeanState 生命周期测试 ──────────────

    #[test]
    fn test_bean_state_defined_after_register()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        assert_eq!(container.bean_state::<UserService>(), Some(BeanState::Defined));
    }

    #[test]
    fn test_bean_state_created_after_get()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        container.get_bean::<UserService>().unwrap();
        assert_eq!(container.bean_state::<UserService>(), Some(BeanState::Created));
    }

    #[test]
    fn test_bean_state_destroyed_after_shutdown()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        container.get_bean::<UserService>().unwrap();
        container.shutdown().unwrap();
        assert_eq!(container.bean_state::<UserService>(), Some(BeanState::Destroyed));
    }

    // ── Prototype scope tests / Prototype 作用域测试 ──────────────────

    #[test]
    fn test_prototype_creates_new_instance_each_time()
    {
        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
            .scope(Scope::Prototype);
        container.register_with(reg).unwrap();

        let bean1 = container.get_bean::<UserService>().unwrap();
        let bean2 = container.get_bean::<UserService>().unwrap();
        // Prototype: each call creates a new instance
        // Prototype: 每次调用创建新实例
        assert!(!Arc::ptr_eq(&bean1, &bean2));
    }

    #[test]
    fn test_singleton_returns_same_instance()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserService { user_count: 0 }))
            .unwrap();
        let bean1 = container.get_bean::<UserService>().unwrap();
        let bean2 = container.get_bean::<UserService>().unwrap();
        assert!(Arc::ptr_eq(&bean1, &bean2));
    }

    #[test]
    fn test_prototype_not_cached_in_singletons()
    {
        let mut container = Container::new();
        let reg = BeanRegistration::new("svc")
            .factory(Arc::new(|_| Ok(UserService { user_count: 42 })))
            .scope(Scope::Prototype);
        container.register_with(reg).unwrap();

        let bean = container.get_bean::<UserService>().unwrap();
        assert_eq!(bean.user_count, 42);
        // Each call creates fresh instance
        // 每次调用创建新实例
        drop(bean);
        let bean2 = container.get_bean::<UserService>().unwrap();
        assert_eq!(bean2.user_count, 42);
    }

    #[test]
    fn test_multiple_beans_same_factory_pattern()
    {
        let mut container = Container::new();
        for i in 0..3
        {
            let count = i;
            match i % 3
            {
                0 => container
                    .register(move |_| Ok(UserService { user_count: count }))
                    .unwrap(),
                1 => container
                    .register(move |_| {
                        Ok(EmailService {
                            sent_count: count as u32,
                        })
                    })
                    .unwrap(),
                _ => container
                    .register(move |_| Ok(CacheService { hits: count as u64 }))
                    .unwrap(),
            }
        }
        assert!(container.has_bean::<UserService>());
        assert!(container.has_bean::<EmailService>());
        assert!(container.has_bean::<CacheService>());
    }

    // ── @Qualifier / Named bean tests / @Qualifier / 命名bean测试 ───────

    #[test]
    fn test_register_named_single_bean()
    {
        let mut container = Container::new();
        container
            .register_named("myService", |_| Ok(UserService { user_count: 42 }))
            .unwrap();
        let bean = container
            .get_qualified_bean::<UserService>("myService")
            .unwrap();
        assert_eq!(bean.user_count, 42);
    }

    #[test]
    fn test_register_named_multiple_same_type()
    {
        let mut container = Container::new();
        container
            .register_named("serviceA", |_| Ok(UserService { user_count: 1 }))
            .unwrap();
        container
            .register_named("serviceB", |_| Ok(UserService { user_count: 2 }))
            .unwrap();

        let a = container
            .get_qualified_bean::<UserService>("serviceA")
            .unwrap();
        let b = container
            .get_qualified_bean::<UserService>("serviceB")
            .unwrap();
        assert_eq!(a.user_count, 1);
        assert_eq!(b.user_count, 2);
    }

    #[test]
    fn test_qualified_bean_singleton_identity()
    {
        let mut container = Container::new();
        container
            .register_named("svc", |_| Ok(UserService { user_count: 10 }))
            .unwrap();

        let first = container.get_qualified_bean::<UserService>("svc").unwrap();
        let second = container.get_qualified_bean::<UserService>("svc").unwrap();
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn test_get_bean_falls_back_to_named_single()
    {
        let mut container = Container::new();
        container
            .register_named("onlyCache", |_| Ok(CacheService { hits: 99 }))
            .unwrap();

        // get_bean should find the single named bean
        // get_bean 应找到唯一的命名bean
        let bean = container.get_bean::<CacheService>().unwrap();
        assert_eq!(bean.hits, 99);
    }

    #[test]
    fn test_get_bean_multiple_named_without_primary_returns_error()
    {
        let mut container = Container::new();
        container
            .register_named("cacheA", |_| Ok(CacheService { hits: 1 }))
            .unwrap();
        container
            .register_named("cacheB", |_| Ok(CacheService { hits: 2 }))
            .unwrap();

        let result = container.get_bean::<CacheService>();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Multiple beans"));
    }

    #[test]
    fn test_has_bean_checks_named_storage()
    {
        let mut container = Container::new();
        assert!(!container.has_bean::<CacheService>());
        container
            .register_named("myCache", |_| Ok(CacheService { hits: 0 }))
            .unwrap();
        assert!(container.has_bean::<CacheService>());
    }

    #[test]
    fn test_get_qualified_bean_not_found()
    {
        let container = Container::new();
        let result = container.get_qualified_bean::<UserService>("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_beans_of_type()
    {
        let mut container = Container::new();
        container
            .register_named("alpha", |_| Ok(CacheService { hits: 10 }))
            .unwrap();
        container
            .register_named("beta", |_| Ok(CacheService { hits: 20 }))
            .unwrap();

        // Instantiate both named beans first
        // 先实例化两个命名bean
        container.get_qualified_bean::<CacheService>("alpha").unwrap();
        container.get_qualified_bean::<CacheService>("beta").unwrap();

        let beans = container.get_beans_of_type::<CacheService>();
        assert_eq!(beans.len(), 2);
    }

    #[test]
    fn test_qualified_bean_with_dependency_injection()
    {
        let mut container = Container::new();
        container
            .register(|_| Ok(UserRepository { initialized: true }))
            .unwrap();
        container
            .register_named("primaryService", |c| {
                let repo = c.get_bean::<UserRepository>()?;
                Ok(UserService {
                    user_count: if repo.initialized { 100 } else { 0 },
                })
            })
            .unwrap();

        let svc = container
            .get_qualified_bean::<UserService>("primaryService")
            .unwrap();
        assert_eq!(svc.user_count, 100);
    }

    // ── ApplicationContext @Qualifier tests ─────────────────────────────

    #[test]
    fn test_application_context_register_named()
    {
        let mut ctx = ApplicationContext::new();
        ctx.register_named("mySvc", |_| Ok(UserService { user_count: 7 }))
            .unwrap();
        let bean = ctx.get_qualified_bean::<UserService>("mySvc").unwrap();
        assert_eq!(bean.user_count, 7);
    }

}
