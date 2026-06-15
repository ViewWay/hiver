//! Configuration hot-reloader — watches files and reloads on change.
//! 配置热重载器 — 监控文件变化并自动重载。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring Boot DevTools automatic config restart.
//! Spring Boot DevTools 自动配置重启。
//!
//! # Rust Advantage / Rust优势
//!
//! `notify` crate uses native OS APIs (inotify/kqueue) — zero overhead vs Spring's polling.
//! 类型安全的配置重载 — 无反射。

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use notify::Watcher;
use tokio::sync::RwLock;

use crate::error::{DevResult, DevToolsError};

/// A hot-reloadable configuration value.
/// 可热重载的配置值。
#[derive(Debug, Clone)]
pub struct ConfigEntry
{
    /// Key path (e.g. "server.port").
    pub key: String,

    /// Current value.
    pub value: serde_json::Value,

    /// Source file.
    pub source: PathBuf,

    /// Last modified timestamp.
    pub modified: std::time::Instant,
}

/// Configuration hot-reloader — watches files and reloads on change.
/// 配置热重载器 — 监控文件变化并自动重载。
///
/// # Example / 示例
///
/// ```rust,no_run
/// use hiver_devtools::ConfigReloader;
///
/// #[tokio::main]
/// async fn main()
/// {
///     let reloader = ConfigReloader::new().watch_file("config.json");
///
///     reloader.start().await.unwrap();
///
///     // Later, get config values:
///     if let Some(port) = reloader.get("server.port").await
///     {
///         println!("Port: {}", port);
///     }
/// }
/// ```
pub struct ConfigReloader
{
    files: Vec<PathBuf>,
    config: Arc<RwLock<HashMap<String, ConfigEntry>>>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl ConfigReloader
{
    /// Create a new config reloader.
    /// 创建新的配置重载器。
    pub fn new() -> Self
    {
        Self {
            files: Vec::new(),
            config: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Add a file to watch.
    /// 添加要监控的文件。
    pub fn watch_file(mut self, path: impl Into<PathBuf>) -> Self
    {
        self.files.push(path.into());
        self
    }

    /// Load all config files initially.
    /// 初始加载所有配置文件。
    pub async fn load_initial(&self) -> DevResult<()>
    {
        for path in &self.files
        {
            self.reload_file(path).await?;
        }
        Ok(())
    }

    /// Start watching for file changes in the background.
    /// 启动后台文件变化监控。
    pub async fn start(&self) -> DevResult<()>
    {
        if self.files.is_empty()
        {
            return Err(DevToolsError::Watch("no files to watch".into()));
        }
        self.running
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.load_initial().await?;

        let files = self.files.clone();
        #[allow(clippy::redundant_clone)]
        let config = self.config.clone();
        #[allow(clippy::redundant_clone)]
        let running = self.running.clone();

        tokio::spawn(async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<PathBuf>(100);
            let tx2 = tx.clone();
            let files2 = files.clone();
            let running2 = running.clone();

            std::thread::spawn(move || {
                let mut watcher = match notify::recommended_watcher(
                    move |res: Result<notify::Event, notify::Error>| {
                        if let Ok(event) = res
                        {
                            if event.kind.is_modify() || event.kind.is_create()
                            {
                                for p in event.paths
                                {
                                    let _ = tx2.blocking_send(p);
                                }
                            }
                        }
                    },
                )
                {
                    Ok(w) => w,
                    Err(e) =>
                    {
                        tracing::error!("Watcher failed: {}", e);
                        return;
                    },
                };

                for f in &files2
                {
                    let _ = watcher.watch(f, notify::RecursiveMode::NonRecursive);
                }
                if let Some(parent) = files2.first().and_then(|f| f.parent())
                {
                    let _ = watcher.watch(parent, notify::RecursiveMode::Recursive);
                }
                while running2.load(std::sync::atomic::Ordering::Relaxed)
                {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            while running.load(std::sync::atomic::Ordering::Relaxed)
            {
                match tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv()).await
                {
                    Ok(Some(path)) if files.contains(&path) =>
                    {
                        let mut cfg = config.write().await;
                        cfg.retain(|_, v| v.source != path);
                        drop(cfg);
                        match Self::reload_static(&config, &path).await
                        {
                            Ok(()) => tracing::info!("Config reloaded: {:?}", path),
                            Err(e) => tracing::warn!("Reload failed {:?}: {}", path, e),
                        }
                    },
                    Ok(None) => break,
                    Err(_) | Ok(Some(_)) =>
                    {},
                }
            }
        });
        Ok(())
    }

    /// Stop watching.
    /// 停止监控。
    pub fn stop(&self)
    {
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get a config value by key.
    /// 根据键获取配置值。
    pub async fn get(&self, key: &str) -> Option<serde_json::Value>
    {
        self.config.read().await.get(key).map(|e| e.value.clone())
    }

    /// Number of loaded config entries.
    /// 已加载的配置项数量。
    pub async fn len(&self) -> usize
    {
        self.config.read().await.len()
    }

    /// Whether there are no loaded config entries.
    /// 是否没有已加载的配置项。
    pub async fn is_empty(&self) -> bool
    {
        self.config.read().await.is_empty()
    }

    async fn reload_file(&self, path: &Path) -> DevResult<()>
    {
        Self::reload_static(&self.config, path).await
    }

    async fn reload_static(
        config: &Arc<RwLock<HashMap<String, ConfigEntry>>>,
        path: &Path,
    ) -> DevResult<()>
    {
        let content = tokio::fs::read_to_string(path).await?;
        let parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| DevToolsError::Config(format!("parse error in {:?}: {}", path, e)))?;
        let modified = std::time::Instant::now();
        let mut cfg = config.write().await;
        for (key, value) in parsed
        {
            cfg.insert(key.clone(), ConfigEntry {
                key,
                value,
                source: path.to_path_buf(),
                modified,
            });
        }
        Ok(())
    }
}

impl Default for ConfigReloader
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_load_json()
    {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.json");
        tokio::fs::write(&file, r#"{"server.port":8080,"debug":true}"#)
            .await
            .unwrap();
        let r = ConfigReloader::new().watch_file(&file);
        r.load_initial().await.unwrap();
        assert_eq!(r.len().await, 2);
        assert_eq!(r.get("server.port").await, Some(serde_json::json!(8080)));
    }

    #[tokio::test]
    async fn test_missing_file()
    {
        let r = ConfigReloader::new().watch_file("/nonexistent.json");
        assert!(r.load_initial().await.is_err());
    }

    #[tokio::test]
    async fn test_no_files()
    {
        assert!(ConfigReloader::new().start().await.is_err());
    }

    #[tokio::test]
    async fn test_reload_updates()
    {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("r.json");
        tokio::fs::write(&file, r#"{"count":1}"#).await.unwrap();
        let r = ConfigReloader::new().watch_file(&file);
        r.load_initial().await.unwrap();
        assert_eq!(r.get("count").await, Some(serde_json::json!(1)));
        tokio::fs::write(&file, r#"{"count":42}"#).await.unwrap();
        r.load_initial().await.unwrap();
        assert_eq!(r.get("count").await, Some(serde_json::json!(42)));
    }

    #[tokio::test]
    async fn test_empty_json()
    {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("e.json");
        tokio::fs::write(&file, "{}").await.unwrap();
        let r = ConfigReloader::new().watch_file(&file);
        r.load_initial().await.unwrap();
        assert_eq!(r.len().await, 0);
    }
}
