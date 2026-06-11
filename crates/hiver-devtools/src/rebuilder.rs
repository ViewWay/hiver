//! Auto-rebuilder — watches source files and triggers cargo build.
//! 自动重构建器 — 监控源文件变化并触发 cargo build。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring Boot DevTools automatic restart (ClassLoader-based).
//! Hiver uses `notify` for file watching + `cargo build` for recompilation.
//!
//! # Rust Advantage / Rust优势
//!
//! - Native OS file watching (inotify/kqueue) vs Spring's polling
//! - Compile-time validation of changes before restart
//! - No classloader overhead — Rust compiles to native code

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use notify::Watcher;
use tokio::sync::watch;
use tokio::time::{timeout, Duration};

use crate::error::DevResult;

/// Auto-rebuilder configuration.
/// 自动重构建配置。
#[derive(Debug, Clone)]
pub struct RebuilderConfig
{
    /// Source directories to watch.
    /// 要监控的源代码目录。
    pub src_dirs: Vec<PathBuf>,

    /// Cargo command (default: "cargo").
    /// Cargo 命令（默认："cargo"）。
    pub build_command: String,

    /// Build arguments (default: `[\"build\"]`).
    /// 构建参数（默认：`[\"build\"]`）。
    pub build_args: Vec<String>,

    /// Debounce interval in milliseconds (default: 300ms).
    /// 去抖间隔（默认：300毫秒）。
    pub debounce_ms: u64,

    /// Working directory for the build command.
    /// 构建命令的工作目录。
    pub working_dir: Option<PathBuf>,
}

impl Default for RebuilderConfig
{
    fn default() -> Self
    {
        Self {
            src_dirs: vec![PathBuf::from("src")],
            build_command: "cargo".to_string(),
            build_args: vec!["build".to_string()],
            debounce_ms: 300,
            working_dir: None,
        }
    }
}

impl RebuilderConfig
{
    /// Create a new config with default values.
    /// 创建带默认值的配置。
    pub fn new() -> Self { Self::default() }

    /// Add a source directory to watch.
    /// 添加要监控的源代码目录。
    pub fn watch_dir(mut self, dir: impl Into<PathBuf>) -> Self
    {
        self.src_dirs.push(dir.into());
        self
    }

    /// Set the build command.
    /// 设置构建命令。
    pub fn build_command(mut self, cmd: impl Into<String>) -> Self
    {
        self.build_command = cmd.into();
        self
    }

    /// Add a build argument.
    /// 添加构建参数。
    pub fn build_arg(mut self, arg: impl Into<String>) -> Self
    {
        self.build_args.push(arg.into());
        self
    }

    /// Set debounce interval in milliseconds.
    /// 设置去抖间隔（毫秒）。
    pub fn debounce_ms(mut self, ms: u64) -> Self
    {
        self.debounce_ms = ms;
        self
    }

    /// Set working directory.
    /// 设置工作目录。
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self
    {
        self.working_dir = Some(dir.into());
        self
    }
}

/// Build status reported by the auto-rebuilder.
/// 自动重构建器报告的构建状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStatus
{
    /// Idle — no changes detected.
    /// 空闲 — 未检测到变化。
    Idle,

    /// Building — compilation in progress.
    /// 构建中 — 编译进行中。
    Building,

    /// Build succeeded.
    /// 构建成功。
    Success,

    /// Build failed.
    /// 构建失败。
    Failed,
}

/// Auto-rebuilder — watches source files and triggers cargo build.
/// 自动重构建器 — 监控源文件变化并触发 cargo build。
///
/// # Example / 示例
///
/// ```rust,no_run
/// use hiver_devtools::{AutoRebuilder, RebuilderConfig};
///
/// #[tokio::main]
/// async fn main() {
///     let config = RebuilderConfig::new()
///         .watch_dir("src")
///         .watch_dir("crates");
///
///     let mut rebuilder = AutoRebuilder::start(config).unwrap();
///
///     // In your app loop:
///     loop {
///         if rebuilder.rebuild_rx.changed().await.is_ok() {
///             println!("Rebuild completed, restarting...");
///         }
///     }
/// }
/// ```
pub struct AutoRebuilder
{
    config: RebuilderConfig,
    running: Arc<AtomicBool>,
    status: Arc<std::sync::Mutex<BuildStatus>>,
    /// Receiver that gets notified when a rebuild completes successfully.
    /// 当重构建成功完成时收到通知的接收器。
    pub rebuild_rx: watch::Receiver<bool>,
    rebuild_tx: watch::Sender<bool>,
}

impl AutoRebuilder
{
    /// Create and start the auto-rebuilder.
    /// 创建并启动自动重构建器。
    pub fn start(config: RebuilderConfig) -> DevResult<Self>
    {
        let running = Arc::new(AtomicBool::new(true));
        let status = Arc::new(std::sync::Mutex::new(BuildStatus::Idle));
        let (rebuild_tx, rebuild_rx) = watch::channel(false);

        let rebuilder = Self {
            config,
            running,
            status,
            rebuild_rx,
            rebuild_tx,
        };

        rebuilder.spawn_watcher();
        Ok(rebuilder)
    }

    /// Get current build status.
    /// 获取当前构建状态。
    #[allow(clippy::unwrap_used)]
    pub fn status(&self) -> BuildStatus
    {
        *self.status.lock().unwrap()
    }

    /// Stop the rebuilder.
    /// 停止重构建器。
    pub fn stop(&self)
    {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Check if the rebuilder is running.
    /// 检查重构建器是否正在运行。
    pub fn is_running(&self) -> bool { self.running.load(Ordering::Relaxed) }

    #[allow(clippy::unwrap_used)]
    fn spawn_watcher(&self)
    {
        let running = self.running.clone();
        let status = self.status.clone();
        let rebuild_tx = self.rebuild_tx.clone();
        let build_cmd = self.config.build_command.clone();
        let build_args = self.config.build_args.clone();
        let debounce = Duration::from_millis(self.config.debounce_ms);
        let working_dir = self.config.working_dir.clone();
        let src_dirs = self.config.src_dirs.clone();

        let (change_tx, mut change_rx) = tokio::sync::mpsc::channel::<()>(100);
        let change_tx2 = change_tx.clone();
        let running2 = running.clone();
        let src_dirs2 = src_dirs.clone();

        // notify watcher thread — uses native OS file watching.
        // notify 监控线程 — 使用原生 OS 文件监控。
        std::thread::spawn(move || {
            let mut watcher = match notify::recommended_watcher(
                move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res
                    {
                        if event.kind.is_modify() || event.kind.is_create()
                        {
                            let has_rs = event
                                .paths
                                .iter()
                                .any(|p| p.extension().is_some_and(|e| e == "rs"));
                            if has_rs
                            {
                                let _ = change_tx2.blocking_send(());
                            }
                        }
                    }
                },
            ) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("AutoRebuilder watcher failed: {}", e);
                    return;
                }
            };

            for dir in &src_dirs2
            {
                if let Err(e) = watcher.watch(dir, notify::RecursiveMode::Recursive)
                {
                    tracing::warn!("Cannot watch {:?}: {}", dir, e);
                }
            }

            while running2.load(Ordering::Relaxed)
            {
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        // Debounce + build task.
        // 去抖 + 构建任务。
        tokio::spawn(async move {
            let mut last_change: Option<tokio::time::Instant> = None;

            while running.load(Ordering::Relaxed)
            {
                match timeout(Duration::from_millis(50), change_rx.recv()).await
                {
                    Ok(Some(())) => {
                        last_change = Some(tokio::time::Instant::now());
                    }
                    Ok(None) => break,
                    Err(_) => {}
                }

                if let Some(t) = last_change
                {
                    if t.elapsed() >= debounce
                    {
                        last_change = None;
                        *status.lock().unwrap() = BuildStatus::Building;

                        let mut cmd = tokio::process::Command::new(&build_cmd);
                        cmd.args(&build_args)
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped());

                        if let Some(ref dir) = working_dir
                        {
                            cmd.current_dir(dir);
                        }

                        match cmd.status().await
                        {
                            Ok(s) if s.success() => {
                                *status.lock().unwrap() = BuildStatus::Success;
                                let _ = rebuild_tx.send(true);
                                tracing::info!("AutoRebuilder: build succeeded");
                            }
                            Ok(s) => {
                                *status.lock().unwrap() = BuildStatus::Failed;
                                tracing::warn!("AutoRebuilder: build failed ({})", s);
                            }
                            Err(e) => {
                                *status.lock().unwrap() = BuildStatus::Failed;
                                tracing::error!("AutoRebuilder: build error: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }
}

impl Drop for AutoRebuilder
{
    fn drop(&mut self) { self.stop(); }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_config_default()
    {
        let config = RebuilderConfig::default();
        assert_eq!(config.build_command, "cargo");
        assert_eq!(config.build_args, vec!["build"]);
        assert_eq!(config.debounce_ms, 300);
    }

    #[test]
    fn test_config_builder()
    {
        let config = RebuilderConfig::new()
            .watch_dir("crates")
            .build_arg("--release")
            .debounce_ms(500)
            .working_dir("/tmp/project");

        assert_eq!(config.src_dirs.len(), 2);
        assert!(config.build_args.contains(&"--release".to_string()));
        assert_eq!(config.debounce_ms, 500);
        assert_eq!(config.working_dir, Some(PathBuf::from("/tmp/project")));
    }

    #[test]
    fn test_build_status()
    {
        assert_ne!(BuildStatus::Idle, BuildStatus::Building);
        assert_ne!(BuildStatus::Success, BuildStatus::Failed);
    }

    #[tokio::test]
    async fn test_rebuilder_drop_stops()
    {
        let config = RebuilderConfig::new().debounce_ms(100);
        let rebuilder = AutoRebuilder::start(config).unwrap();
        assert!(rebuilder.is_running());
        drop(rebuilder);
    }
}
