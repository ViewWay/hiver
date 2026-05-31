//! Test execution listeners for test lifecycle hooks
//! 测试执行监听器，用于测试生命周期钩子
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `TestExecutionListener`
//! - `@BeforeTestClass` / `@AfterTestClass`
//! - `@BeforeTestMethod` / `@AfterTestMethod`

/// Hook for test class lifecycle events.
/// 测试类生命周期事件钩子。
pub trait TestExecutionListener: Send + Sync {
    /// Called once before all tests in the class.
    /// 在类中所有测试之前调用一次。
    fn before_class(&self, _context: &TestLifecycleContext) {}

    /// Called once after all tests in the class.
    /// 在类中所有测试之后调用一次。
    fn after_class(&self, _context: &TestLifecycleContext) {}

    /// Called before each test method.
    /// 在每个测试方法之前调用。
    fn before_method(&self, _context: &TestLifecycleContext, _method: &str) {}

    /// Called after each test method.
    /// 在每个测试方法之后调用。
    fn after_method(&self, _context: &TestLifecycleContext, _method: &str) {}
}

/// Context provided to test lifecycle hooks.
/// 提供给测试生命周期钩子的上下文。
#[derive(Debug, Clone)]
pub struct TestLifecycleContext {
    /// Test class name.
    /// 测试类名。
    pub class_name: String,
    /// Custom attributes for sharing state across hooks.
    /// 跨钩子共享状态的自定义属性。
    pub attributes: std::collections::HashMap<String, String>,
}

impl TestLifecycleContext {
    /// Create a new context for the given test class.
    /// 为给定测试类创建新上下文。
    pub fn new(class_name: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
            attributes: std::collections::HashMap::new(),
        }
    }

    /// Set an attribute.
    /// 设置属性。
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Get an attribute.
    /// 获取属性。
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }
}

/// Registry for test execution listeners.
/// 测试执行监听器注册表。
#[derive(Default)]
pub struct TestListenerRegistry {
    listeners: Vec<Box<dyn TestExecutionListener>>,
}

impl TestListenerRegistry {
    /// Create a new empty registry.
    /// 创建新的空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a listener.
    /// 注册监听器。
    pub fn register(&mut self, listener: Box<dyn TestExecutionListener>) {
        self.listeners.push(listener);
    }

    /// Notify all listeners: before class.
    /// 通知所有监听器：测试类之前。
    pub fn fire_before_class(&self, context: &TestLifecycleContext) {
        for l in &self.listeners {
            l.before_class(context);
        }
    }

    /// Notify all listeners: after class.
    /// 通知所有监听器：测试类之后。
    pub fn fire_after_class(&self, context: &TestLifecycleContext) {
        for l in &self.listeners {
            l.after_class(context);
        }
    }

    /// Notify all listeners: before method.
    /// 通知所有监听器：测试方法之前。
    pub fn fire_before_method(&self, context: &TestLifecycleContext, method: &str) {
        for l in &self.listeners {
            l.before_method(context, method);
        }
    }

    /// Notify all listeners: after method.
    /// 通知所有监听器：测试方法之后。
    pub fn fire_after_method(&self, context: &TestLifecycleContext, method: &str) {
        for l in &self.listeners {
            l.after_method(context, method);
        }
    }

    /// Number of registered listeners.
    /// 已注册监听器数量。
    pub fn len(&self) -> usize {
        self.listeners.len()
    }

    /// Check if empty.
    /// 检查是否为空。
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }
}

/// A listener that logs test lifecycle events.
/// 记录测试生命周期事件的监听器。
pub struct LoggingTestListener;

impl TestExecutionListener for LoggingTestListener {
    fn before_class(&self, ctx: &TestLifecycleContext) {
        eprintln!("[TEST] Before class: {}", ctx.class_name);
    }

    fn after_class(&self, ctx: &TestLifecycleContext) {
        eprintln!("[TEST] After class: {}", ctx.class_name);
    }

    fn before_method(&self, ctx: &TestLifecycleContext, method: &str) {
        eprintln!("[TEST] Before method: {}::{}", ctx.class_name, method);
    }

    fn after_method(&self, ctx: &TestLifecycleContext, method: &str) {
        eprintln!("[TEST] After method: {}::{}", ctx.class_name, method);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_lifecycle_context() {
        let mut ctx = TestLifecycleContext::new("MyTests");
        ctx.set_attribute("key", "value");
        assert_eq!(ctx.get_attribute("key"), Some("value"));
        assert_eq!(ctx.class_name, "MyTests");
    }

    #[test]
    fn test_listener_registry_fire() {
        #[derive(Default)]
        struct Counter {
            before_class: Arc<Mutex<usize>>,
            after_class: Arc<Mutex<usize>>,
        }

        let counter = Counter::default();
        let bc = counter.before_class.clone();
        let ac = counter.after_class.clone();

        struct Inner {
            before_class: Arc<Mutex<usize>>,
            after_class: Arc<Mutex<usize>>,
        }

        impl TestExecutionListener for Inner {
            fn before_class(&self, _ctx: &TestLifecycleContext) {
                *self.before_class.lock().unwrap() += 1;
            }
            fn after_class(&self, _ctx: &TestLifecycleContext) {
                *self.after_class.lock().unwrap() += 1;
            }
        }

        let mut reg = TestListenerRegistry::new();
        reg.register(Box::new(Inner { before_class: bc, after_class: ac }));

        let ctx = TestLifecycleContext::new("Test");
        reg.fire_before_class(&ctx);
        reg.fire_after_class(&ctx);
        reg.fire_before_method(&ctx, "test_a");
        reg.fire_after_method(&ctx, "test_a");

        assert_eq!(*counter.before_class.lock().unwrap(), 1);
        assert_eq!(*counter.after_class.lock().unwrap(), 1);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_logging_listener_no_panic() {
        let listener = LoggingTestListener;
        let ctx = TestLifecycleContext::new("DemoTest");
        listener.before_class(&ctx);
        listener.before_method(&ctx, "test_demo");
        listener.after_method(&ctx, "test_demo");
        listener.after_class(&ctx);
    }
}
