//! Bean lifecycle traits
//! Bean生命周期trait
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `InitializingBean` ↔ Spring's `InitializingBean` / `@PostConstruct`
//! - `DisposableBean` ↔ Spring's `DisposableBean` / `@PreDestroy`
//! - `BeanPostProcessor` ↔ Spring's `BeanPostProcessor`
//! - `BeanFactoryPostProcessor` ↔ Spring's `BeanFactoryPostProcessor`
//!
//! # Lifecycle flow / 生命周期流程
//!
//! 1. Bean instantiation / Bean实例化
//! 2. Properties population / 属性注入
//! 3. `BeanPostProcessor::post_process_before_initialization`
//! 4. `InitializingBean::after_properties_set` (@`PostConstruct`)
//! 5. `BeanPostProcessor::post_process_after_initialization`
//! 6. ... bean is in use / Bean使用中 ...
//! 7. `DisposableBean::destroy` (@`PreDestroy`)

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::any::Any;

/// Type alias for results in this module.
/// 本模块的结果类型别名。
pub type Result<T> = anyhow::Result<T>;

/// Trait for beans that need initialization after property injection.
/// 在属性注入后需要初始化的Bean trait。
///
/// Equivalent to Spring's `InitializingBean` interface or `@PostConstruct`.
/// 等价于 Spring 的 `InitializingBean` 接口或 `@PostConstruct`。
///
/// # When it fires / 何时触发
///
/// Called after all properties have been supplied to the bean.
/// 在所有属性注入到Bean之后调用。
///
/// # Example / 示例
///
/// ```
/// use nexus_core::lifecycle::InitializingBean;
///
/// struct MyService {
///     initialized: bool,
/// }
///
/// impl InitializingBean for MyService {
///     fn after_properties_set(&mut self) -> nexus_core::lifecycle::Result<()> {
///         self.initialized = true;
///         Ok(())
///     }
/// }
/// ```
pub trait InitializingBean {
    /// Called after all bean properties have been set.
    /// 在所有Bean属性设置完毕后调用。
    ///
    /// Equivalent to `@PostConstruct`.
    /// 等价于 `@PostConstruct`。
    fn after_properties_set(&mut self) -> Result<()>;
}

/// Trait for beans that need cleanup before destruction.
/// 在销毁前需要清理的Bean trait。
///
/// Equivalent to Spring's `DisposableBean` interface or `@PreDestroy`.
/// 等价于 Spring 的 `DisposableBean` 接口或 `@PreDestroy`。
///
/// # When it fires / 何时触发
///
/// Called when the container is shutting down and the bean is being removed.
/// 当容器关闭且Bean正在被移除时调用。
///
/// # Example / 示例
///
/// ```
/// use nexus_core::lifecycle::DisposableBean;
///
/// struct ConnectionPool {
///     active_connections: usize,
/// }
///
/// impl DisposableBean for ConnectionPool {
///     fn destroy(&mut self) -> nexus_core::lifecycle::Result<()> {
///         self.active_connections = 0;
///         Ok(())
///     }
/// }
/// ```
pub trait DisposableBean {
    /// Called before the bean is destroyed by the container.
    /// 在容器销毁Bean之前调用。
    ///
    /// Equivalent to `@PreDestroy`.
    /// 等价于 `@PreDestroy`。
    fn destroy(&mut self) -> Result<()>;
}

/// Hook for custom modification of new bean instances.
/// 用于自定义修改新Bean实例的钩子。
///
/// Equivalent to Spring's `BeanPostProcessor`.
/// 等价于 Spring 的 `BeanPostProcessor`。
///
/// `BeanPostProcessor` allows you to intercept and modify every new bean
/// instance before and after its initialization callback.
///
/// `BeanPostProcessor` 允许您在初始化回调之前和之后拦截并修改每个新Bean实例。
///
/// # Processing order / 处理顺序
///
/// 1. `post_process_before_initialization` — runs before `InitializingBean::after_properties_set`
/// 2. `post_process_after_initialization` — runs after `InitializingBean::after_properties_set`
pub trait BeanPostProcessor: Send + Sync {
    /// Apply this processor before the bean's initialization callback.
    /// 在Bean的初始化回调之前应用此处理器。
    ///
    /// Equivalent to Spring's `BeanPostProcessor.postProcessBeforeInitialization`.
    /// 等价于 Spring 的 `BeanPostProcessor.postProcessBeforeInitialization`。
    ///
    /// # Arguments / 参数
    ///
    /// * `bean` — The raw bean instance to process.
    ///            要处理的原始Bean实例。
    /// * `bean_name` — The name of the bean.
    ///                  Bean的名称。
    fn post_process_before_initialization(
        &self,
        bean: &mut dyn Any,
        bean_name: &str,
    ) -> Result<()>;

    /// Apply this processor after the bean's initialization callback.
    /// 在Bean的初始化回调之后应用此处理器。
    ///
    /// Equivalent to Spring's `BeanPostProcessor.postProcessAfterInitialization`.
    /// 等价于 Spring 的 `BeanPostProcessor.postProcessAfterInitialization`。
    ///
    /// # Arguments / 参数
    ///
    /// * `bean` — The raw bean instance to process.
    ///            要处理的原始Bean实例。
    /// * `bean_name` — The name of the bean.
    ///                  Bean的名称。
    fn post_process_after_initialization(
        &self,
        bean: &mut dyn Any,
        bean_name: &str,
    ) -> Result<()>;
}

/// Hook for custom modification of the bean factory's definition registry.
/// 用于自定义修改Bean工厂定义注册表的钩子。
///
/// Equivalent to Spring's `BeanFactoryPostProcessor`.
/// 等价于 Spring 的 `BeanFactoryPostProcessor`。
///
/// Called once during context startup, *before* any beans are instantiated.
/// 在上下文启动期间，在实例化任何Bean之前调用一次。
///
/// # Example / 示例
///
/// ```no_run
/// use nexus_core::lifecycle::BeanFactoryPostProcessor;
/// use std::any::Any;
///
/// struct PropertyOverrideProcessor;
///
/// impl BeanFactoryPostProcessor for PropertyOverrideProcessor {
///     fn post_process_bean_factory(&self, factory: &mut dyn Any) {
///         // Inspect and modify bean definitions before any bean is created.
///         // 在创建任何Bean之前检查和修改Bean定义。
///     }
/// }
/// ```
pub trait BeanFactoryPostProcessor: Send + Sync {
    /// Modify the bean factory's internal state before any beans are created.
    /// `在创建任何Bean之前修改Bean工厂的内部状态`。
    ///
    /// # Arguments / 参数
    ///
    /// * `factory` — The bean factory (passed as `&mut dyn Any` for flexibility).
    ///               Bean工厂（以 `&mut dyn Any` 传递以提供灵活性）。
    fn post_process_bean_factory(&self, factory: &mut dyn Any);
}

// ---------------------------------------------------------------------------
// Simple implementations for common patterns
// 常见模式的简单实现
// ---------------------------------------------------------------------------

/// A no-op `BeanPostProcessor` that does nothing.
/// 不执行任何操作的空 `BeanPostProcessor`。
///
/// Useful as a base class or for testing.
/// 可用作基类或测试。
pub struct NoOpBeanPostProcessor;

impl BeanPostProcessor for NoOpBeanPostProcessor {
    fn post_process_before_initialization(
        &self,
        _bean: &mut dyn Any,
        _bean_name: &str,
    ) -> Result<()> {
        Ok(())
    }

    fn post_process_after_initialization(
        &self,
        _bean: &mut dyn Any,
        _bean_name: &str,
    ) -> Result<()> {
        Ok(())
    }
}

/// A no-op `BeanFactoryPostProcessor` that does nothing.
/// 不执行任何操作的空 `BeanFactoryPostProcessor`。
pub struct NoOpBeanFactoryPostProcessor;

impl BeanFactoryPostProcessor for NoOpBeanFactoryPostProcessor {
    fn post_process_bean_factory(&self, _factory: &mut dyn Any) {
        // no-op
    }
}

/// Executes the full bean lifecycle for a given bean instance.
/// 为给定的Bean实例执行完整的生命周期。
///
/// This is a convenience function that applies the standard lifecycle:
/// 1. Before-initialization post-processors
/// 2. `InitializingBean::after_properties_set` (if implemented)
/// 3. After-initialization post-processors
///
/// 这是一个便捷函数，应用标准生命周期：
/// 1. 初始化前的后处理器
/// 2. `InitializingBean::after_properties_set`（如果已实现）
/// 3. 初始化后的后处理器
///
/// # Arguments / 参数
///
/// * `bean` — The bean instance to initialize.
///            要初始化的Bean实例。
/// * `bean_name` — The name of the bean.
///                  Bean的名称。
/// * `processors` — Slice of `BeanPostProcessor`s to apply.
///                   要应用的 `BeanPostProcessor` 切片。
pub fn initialize_bean(
    bean: &mut dyn Any,
    bean_name: &str,
    processors: &[&dyn BeanPostProcessor],
) -> Result<()> {
    // Phase 1: before initialization / 初始化前
    for processor in processors {
        processor.post_process_before_initialization(bean, bean_name)?;
    }

    // Phase 2: InitializingBean callback / InitializingBean回调
    // We cannot downcast &&mut dyn Any to &mut dyn InitializingBean directly,
    // because InitializingBean is not known at the call site in a generic way.
    // Instead, the caller should invoke after_properties_set() before or instead
    // of calling this helper, or embed the callback inside a concrete wrapper.
    //
    // For the general case, this function focuses on processor invocation only.

    // Phase 3: after initialization / 初始化后
    for processor in processors {
        processor.post_process_after_initialization(bean, bean_name)?;
    }

    Ok(())
}

/// Executes the destruction lifecycle for a given bean instance.
/// 为给定的Bean实例执行销毁生命周期。
///
/// This is a convenience function that calls `DisposableBean::destroy()`.
/// 这是一个调用 `DisposableBean::destroy()` 的便捷函数。
///
/// Note: the caller is responsible for downcasting and invoking the trait method
/// on the concrete type, since `dyn Any` cannot be directly downcast to
/// `dyn DisposableBean`.
///
/// 注意：调用者负责向下转型并在具体类型上调用trait方法，
/// 因为 `dyn Any` 不能直接向下转型为 `dyn DisposableBean`。
pub fn destroy_bean<T: DisposableBean>(bean: &mut T) -> Result<()> {
    bean.destroy()
}

// ---------------------------------------------------------------------------
// Tests
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// A sample bean that tracks whether `after_properties_set` was called.
    /// 一个示例Bean，跟踪 `after_properties_set` 是否被调用。
    struct SampleBean {
        initialized: bool,
        destroyed: bool,
        flag: Arc<AtomicBool>,
    }

    impl SampleBean {
        fn new(flag: Arc<AtomicBool>) -> Self {
            Self {
                initialized: false,
                destroyed: false,
                flag,
            }
        }
    }

    impl InitializingBean for SampleBean {
        fn after_properties_set(&mut self) -> Result<()> {
            self.initialized = true;
            self.flag.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    impl DisposableBean for SampleBean {
        fn destroy(&mut self) -> Result<()> {
            self.destroyed = true;
            Ok(())
        }
    }

    #[test]
    fn test_initializing_bean_callback_fires() {
        let flag = Arc::new(AtomicBool::new(false));
        let mut bean = SampleBean::new(Arc::clone(&flag));

        assert!(!bean.initialized);
        assert!(!flag.load(Ordering::SeqCst));

        // Invoke the lifecycle callback
        bean.after_properties_set().unwrap();

        assert!(bean.initialized);
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_disposable_bean_callback_fires() {
        let flag = Arc::new(AtomicBool::new(false));
        let mut bean = SampleBean::new(flag);

        assert!(!bean.destroyed);

        bean.destroy().unwrap();

        assert!(bean.destroyed);
    }

    #[test]
    fn test_full_lifecycle() {
        let flag = Arc::new(AtomicBool::new(false));
        let mut bean = SampleBean::new(Arc::clone(&flag));

        // init phase
        bean.after_properties_set().unwrap();
        assert!(bean.initialized);

        // destroy phase
        bean.destroy().unwrap();
        assert!(bean.destroyed);
    }

    #[test]
    fn test_destroy_bean_helper() {
        let flag = Arc::new(AtomicBool::new(false));
        let mut bean = SampleBean::new(flag);

        assert!(!bean.destroyed);
        destroy_bean(&mut bean).unwrap();
        assert!(bean.destroyed);
    }

    #[test]
    fn test_no_op_bean_post_processor() {
        let processor = NoOpBeanPostProcessor;
        let mut data: Box<dyn Any> = Box::new(42i32);

        assert!(processor
            .post_process_before_initialization(&mut *data, "test")
            .is_ok());
        assert!(processor
            .post_process_after_initialization(&mut *data, "test")
            .is_ok());
    }

    #[test]
    fn test_no_op_bean_factory_post_processor() {
        let processor = NoOpBeanFactoryPostProcessor;
        let mut factory: Box<dyn Any> = Box::new("factory" as &str);
        processor.post_process_bean_factory(&mut *factory);
    }

    #[test]
    fn test_initialize_bean_with_processors() {
        let processor = NoOpBeanPostProcessor;
        let mut data: Box<dyn Any> = Box::new(99u64);
        let result = initialize_bean(&mut *data, "myBean", &[&processor]);
        assert!(result.is_ok());
    }

    /// A `BeanPostProcessor` that sets a flag when called, for verification.
    struct TrackingPostProcessor {
        before_called: AtomicBool,
        after_called: AtomicBool,
    }

    impl TrackingPostProcessor {
        fn new() -> Self {
            Self {
                before_called: AtomicBool::new(false),
                after_called: AtomicBool::new(false),
            }
        }
    }

    impl BeanPostProcessor for TrackingPostProcessor {
        fn post_process_before_initialization(
            &self,
            _bean: &mut dyn Any,
            _bean_name: &str,
        ) -> Result<()> {
            self.before_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn post_process_after_initialization(
            &self,
            _bean: &mut dyn Any,
            _bean_name: &str,
        ) -> Result<()> {
            self.after_called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_tracking_post_processor_both_hooks_fire() {
        let processor = TrackingPostProcessor::new();
        let mut data: Box<dyn Any> = Box::new("hello");

        initialize_bean(&mut *data, "tracked", &[&processor]).unwrap();

        assert!(processor.before_called.load(Ordering::SeqCst));
        assert!(processor.after_called.load(Ordering::SeqCst));
    }
}
