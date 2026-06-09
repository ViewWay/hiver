//! RefreshScope — configuration refresh support
//! RefreshScope — 配置刷新支持
//!
//! Equivalent to Spring Cloud's `@RefreshScope` annotation.
//! Provides dynamic configuration refresh without restarting the application.
//! 等价于 Spring Cloud 的 `@RefreshScope` 注解。
//! 提供无需重启应用的动态配置刷新。
//!
//! # Spring Equivalent / Spring等价物
//!
//! ```java
//! @RefreshScope
//! @Bean
//! public DataSource dataSource() {
//!     return DataSourceBuilder.create().url(config.get("db.url")).build();
//! }
//! ```
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_config::refresh::{RefreshScope, ConfigChangeEvent, Refreshable};
//!
//! let mut scope = RefreshScope::new();
//! let refreshable = Refreshable::new("db.url", "localhost:5432".to_string());
//! scope.register("db.url", refreshable);
//!
//! // Simulate config change
//! let event = ConfigChangeEvent::new("db.url", Some("localhost:5432"), "db.example.com:5432");
//! scope.fire_event(&event);
//! ```

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Event fired when a configuration property changes.
/// 当配置属性发生更改时触发的事件。
///
/// Equivalent to Spring Cloud's `EnvironmentChangeEvent`.
/// 等价于 Spring Cloud 的 `EnvironmentChangeEvent`。
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent
{
    /// The key of the changed property / 已更改属性的键
    pub key: String,
    /// The old value (None if the property is new) / 旧值（如果属性是新的则为 None）
    pub old_value: Option<String>,
    /// The new value / 新值
    pub new_value: String,
}

impl ConfigChangeEvent
{
    /// Create a new config change event / 创建新的配置变更事件
    pub fn new(
        key: impl Into<String>,
        old_value: Option<impl Into<String>>,
        new_value: impl Into<String>,
    ) -> Self
    {
        Self {
            key: key.into(),
            old_value: old_value.map(Into::into),
            new_value: new_value.into(),
        }
    }

    /// Whether this event represents a new property / 此事件是否代表新属性
    pub fn is_new(&self) -> bool
    {
        self.old_value.is_none()
    }

    /// Whether the value was removed (empty new value) / 值是否被移除（新值为空）
    pub fn is_removed(&self) -> bool
    {
        self.new_value.is_empty()
    }
}

/// Callback type for config change listeners / 配置变更监听器的回调类型
pub(crate) type ChangeListener = Box<dyn Fn(&ConfigChangeEvent) + Send + Sync>;

/// RefreshScope — marks beans that should be refreshed when config changes.
/// RefreshScope — 标记在配置更改时应刷新的 bean。
///
/// Equivalent to Spring Cloud's `@RefreshScope`.
/// 等价于 Spring Cloud 的 `@RefreshScope`。
///
/// Manages refreshable values and fires events when configuration changes.
/// 管理可刷新值，并在配置更改时触发事件。
pub struct RefreshScope
{
    /// Registered refreshables by key / 按键注册的可刷新值
    refreshables: Arc<RwLock<HashMap<String, Arc<RwLock<RefreshableValue>>>>>,

    /// Change listeners / 变更监听器
    listeners: Arc<RwLock<Vec<ChangeListener>>>,
}

impl std::fmt::Debug for RefreshScope
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let count = self.refreshables.read().map_or(0, |m| m.len());
        let listener_count = self.listeners.read().map_or(0, |v| v.len());
        f.debug_struct("RefreshScope")
            .field("refreshable_count", &count)
            .field("listener_count", &listener_count)
            .finish()
    }
}

/// Internal storage for a refreshable value / 可刷新值的内部存储
#[derive(Debug)]
struct RefreshableValue
{
    /// Current value / 当前值
    value: String,
}

impl RefreshScope
{
    /// Create a new empty RefreshScope / 创建新的空 RefreshScope
    pub fn new() -> Self
    {
        Self {
            refreshables: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a refreshable value / 注册可刷新值
    pub fn register(&self, key: impl Into<String>, initial_value: impl Into<String>)
    {
        let mut map = self
            .refreshables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.insert(
            key.into(),
            Arc::new(RwLock::new(RefreshableValue {
                value: initial_value.into(),
            })),
        );
    }

    /// Get the current value of a refreshable / 获取可刷新值的当前值
    pub fn get(&self, key: &str) -> Option<String>
    {
        let map = self
            .refreshables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.get(key).map(|v| {
            let guard = v.read().unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.value.clone()
        })
    }

    /// Add a change listener / 添加变更监听器
    pub fn add_listener(&self, listener: impl Fn(&ConfigChangeEvent) + Send + Sync + 'static)
    {
        let mut listeners = self.listeners.write().unwrap_or_else(|e| e.into_inner());
        listeners.push(Box::new(listener));
    }

    /// Fire a config change event and update the registered refreshables.
    /// 触发配置变更事件并更新已注册的可刷新值。
    ///
    /// This method updates the refreshable value if registered, and notifies
    /// all registered listeners.
    /// 此方法更新已注册的可刷新值，并通知所有已注册的监听器。
    pub fn fire_event(&self, event: &ConfigChangeEvent)
    {
        // Update the refreshable value if registered
        // 如果已注册则更新可刷新值
        let map = self
            .refreshables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(refreshable) = map.get(&event.key)
        {
            let mut guard = refreshable
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.value.clone_from(&event.new_value);
        }

        // Notify all listeners
        // 通知所有监听器
        let listeners = self.listeners.read().unwrap_or_else(|e| e.into_inner());
        for listener in listeners.iter()
        {
            listener(event);
        }
    }

    /// Refresh all registered values with a provided getter function.
    /// 使用提供的 getter 函数刷新所有已注册的值。
    ///
    /// The getter is called for each registered key to obtain the new value.
    /// 对每个已注册的键调用 getter 以获取新值。
    pub fn refresh_all(&self, getter: impl Fn(&str) -> Option<String>)
    {
        let map = self
            .refreshables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (key, refreshable) in map.iter()
        {
            if let Some(new_value) = getter(key)
            {
                let old_value = {
                    let guard = refreshable
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    Some(guard.value.clone())
                };

                let event = ConfigChangeEvent::new(key.as_str(), old_value, &new_value);

                {
                    let mut guard = refreshable
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.value = new_value;
                }

                // Notify listeners
                let listeners = self.listeners.read().unwrap_or_else(|e| e.into_inner());
                for listener in listeners.iter()
                {
                    listener(&event);
                }
            }
        }
    }

    /// Get the number of registered refreshables / 获取已注册的可刷新值数量
    pub fn len(&self) -> usize
    {
        let map = self
            .refreshables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.len()
    }

    /// Check if there are no refreshables / 检查是否没有可刷新值
    pub fn is_empty(&self) -> bool
    {
        self.len() == 0
    }
}

impl Default for RefreshScope
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// ConfigWatcher — watches config sources for changes.
/// ConfigWatcher — 监视配置源的更改。
///
/// Supports file modification watching and HTTP polling.
/// Equivalent to Spring Cloud Config's watch mechanism.
/// 支持文件修改监视和 HTTP 轮询。
/// 等价于 Spring Cloud Config 的监视机制。
#[derive(Debug)]
pub struct ConfigWatcher
{
    /// File paths being watched / 正在监视的文件路径
    watched_files: Vec<std::path::PathBuf>,

    /// Last known modification times / 上次已知的修改时间
    last_modified: HashMap<std::path::PathBuf, std::time::SystemTime>,

    /// Refresh scope to notify on changes / 配置更改时通知的刷新作用域
    scope: RefreshScope,
}

impl ConfigWatcher
{
    /// Create a new config watcher / 创建新配置监视器
    pub fn new(scope: RefreshScope) -> Self
    {
        Self {
            watched_files: Vec::new(),
            last_modified: HashMap::new(),
            scope,
        }
    }

    /// Add a file to watch / 添加要监视的文件
    pub fn watch_file(&mut self, path: impl Into<std::path::PathBuf>)
    {
        let path = path.into();
        if let Ok(metadata) = std::fs::metadata(&path)
            && let Ok(modified) = metadata.modified()
        {
            self.last_modified.insert(path.clone(), modified);
        }
        self.watched_files.push(path);
    }

    /// Check for file changes and fire events / 检查文件更改并触发事件
    ///
    /// Returns a list of keys that changed (empty if no changes detected).
    /// 返回已更改的键列表（如果未检测到更改则为空）。
    pub fn check_changes(&mut self) -> Vec<String>
    {
        let mut changed = Vec::new();

        for path in &self.watched_files
        {
            if let Ok(metadata) = std::fs::metadata(path)
                && let Ok(modified) = metadata.modified()
            {
                let prev = self.last_modified.get(path).copied();
                if prev != Some(modified)
                {
                    let path_str = path.to_string_lossy().to_string();
                    let event = ConfigChangeEvent::new(&path_str, prev.map(|_| "old"), "updated");
                    self.scope.fire_event(&event);
                    self.last_modified.insert(path.clone(), modified);
                    changed.push(path_str);
                }
            }
        }

        changed
    }

    /// Get the underlying RefreshScope / 获取底层 RefreshScope
    pub fn scope(&self) -> &RefreshScope
    {
        &self.scope
    }

    /// Get the number of watched files / 获取被监视文件的数量
    pub fn watched_count(&self) -> usize
    {
        self.watched_files.len()
    }
}

/// A wrapper that auto-refreshes its value when configuration changes.
/// 在配置更改时自动刷新其值的包装器。
///
/// Equivalent to Spring Cloud's `@RefreshScope` bean behavior.
/// 等价于 Spring Cloud 的 `@RefreshScope` bean 行为。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_config::refresh::Refreshable;
///
/// let mut port = Refreshable::new("server.port", 8080);
/// assert_eq!(*port.get(), 8080);
///
/// port.update("9090".to_string());
/// assert_eq!(*port.get(), 9090);
/// ```
#[derive(Debug)]
pub struct Refreshable<T>
{
    /// Configuration key / 配置键
    key: String,
    /// Current value / 当前值
    value: Arc<RwLock<T>>,
}

impl<T: Clone> Refreshable<T>
{
    /// Create a new refreshable / 创建新可刷新值
    pub fn new(key: impl Into<String>, value: T) -> Self
    {
        Self {
            key: key.into(),
            value: Arc::new(RwLock::new(value)),
        }
    }

    /// Get the current value / 获取当前值
    pub fn get(&self) -> std::sync::RwLockReadGuard<'_, T>
    {
        self.value
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Update the value / 更新值
    pub fn update(&self, new_value: T)
    {
        let mut guard = self
            .value
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = new_value;
    }

    /// Get the configuration key / 获取配置键
    pub fn key(&self) -> &str
    {
        &self.key
    }

    /// Create a cloned copy of the current value / 创建当前值的克隆副本
    pub fn value(&self) -> T
    {
        self.get().clone()
    }
}

impl<T: Clone> Clone for Refreshable<T>
{
    fn clone(&self) -> Self
    {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

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
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn test_config_change_event()
    {
        let event = ConfigChangeEvent::new("db.url", Some("old_host"), "new_host");
        assert_eq!(event.key, "db.url");
        assert_eq!(event.old_value, Some("old_host".to_string()));
        assert_eq!(event.new_value, "new_host");
        assert!(!event.is_new());
        assert!(!event.is_removed());
    }

    #[test]
    fn test_config_change_event_new_property()
    {
        let event = ConfigChangeEvent::new("new.key", None::<String>, "value");
        assert!(event.is_new());
        assert!(!event.is_removed());
    }

    #[test]
    fn test_config_change_event_removed()
    {
        let event = ConfigChangeEvent::new("old.key", Some("value"), "");
        assert!(event.is_removed());
    }

    #[test]
    fn test_refresh_scope_register_and_get()
    {
        let scope = RefreshScope::new();
        scope.register("db.url", "localhost:5432");
        scope.register("server.port", "8080");

        assert_eq!(scope.get("db.url"), Some("localhost:5432".to_string()));
        assert_eq!(scope.get("server.port"), Some("8080".to_string()));
        assert_eq!(scope.get("missing"), None);
        assert_eq!(scope.len(), 2);
    }

    #[test]
    fn test_refresh_scope_fire_event()
    {
        let scope = RefreshScope::new();
        scope.register("db.url", "localhost:5432");

        let event = ConfigChangeEvent::new("db.url", Some("localhost:5432"), "db.example.com:5432");
        scope.fire_event(&event);

        assert_eq!(scope.get("db.url"), Some("db.example.com:5432".to_string()));
    }

    #[test]
    fn test_refresh_scope_listener()
    {
        let scope = RefreshScope::new();
        scope.register("db.url", "old");

        let call_count = Arc::new(AtomicUsize::new(0));
        let count_clone = call_count.clone();
        scope.add_listener(move |_event| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = ConfigChangeEvent::new("db.url", Some("old"), "new");
        scope.fire_event(&event);

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_refresh_scope_refresh_all()
    {
        let scope = RefreshScope::new();
        scope.register("a", "1");
        scope.register("b", "2");

        let new_values: HashMap<String, String> = {
            let mut m = HashMap::new();
            m.insert("a".to_string(), "100".to_string());
            m.insert("b".to_string(), "200".to_string());
            m
        };

        scope.refresh_all(|key| new_values.get(key).cloned());

        assert_eq!(scope.get("a"), Some("100".to_string()));
        assert_eq!(scope.get("b"), Some("200".to_string()));
    }

    #[test]
    fn test_refresh_scope_default()
    {
        let scope = RefreshScope::default();
        assert!(scope.is_empty());
    }

    #[test]
    fn test_config_watcher()
    {
        let scope = RefreshScope::new();
        let mut watcher = ConfigWatcher::new(scope);
        assert_eq!(watcher.watched_count(), 0);

        // Watching a nonexistent file should not panic
        watcher.watch_file("/nonexistent/config.yaml");
        assert_eq!(watcher.watched_count(), 1);

        // Checking changes on nonexistent files should return empty
        let changes = watcher.check_changes();
        assert!(changes.is_empty());
    }

    #[test]
    fn test_refreshable()
    {
        let refreshable = Refreshable::new("server.port", 8080);
        assert_eq!(refreshable.key(), "server.port");
        assert_eq!(*refreshable.get(), 8080);
        assert_eq!(refreshable.value(), 8080);

        refreshable.update(9090);
        assert_eq!(*refreshable.get(), 9090);
    }

    #[test]
    fn test_refreshable_clone()
    {
        let r1 = Refreshable::new("key", "value");
        let r2 = r1.clone();
        r1.update("new_value");
        // Both point to the same underlying value
        assert_eq!(*r2.get(), "new_value");
    }
}
