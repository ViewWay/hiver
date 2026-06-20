//! Runtime module
//! 运行时模块
//!
//! # Overview / 概述
//!
//! This module provides the main runtime implementation that brings together
//! the scheduler, driver, and timer components.
//!
//! 本模块提供了主要的运行时实现，将调度器、驱动和定时器组件组合在一起。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_runtime::Runtime;
//!
//! fn main() -> std::io::Result<()> {
//!     let runtime = Runtime::new()?;
//!     runtime.block_on(async {
//!         println!("Hello from Hiver!");
//!     });
//!     Ok(())
//! }
//! ```

use std::{future::Future, io, sync::RwLock};

use crate::time::Duration;

thread_local! {
    static CURRENT_HANDLE: std::cell::RefCell<Option<Handle>> = const { std::cell::RefCell::new(None) };
}

/// Global handle store so that tasks running on *worker threads* (which do not
/// inherit the main thread's thread-local) can still access the runtime's
/// driver and I/O registry to register FD interest.
///
/// 全局 handle 存储,使运行在 *worker 线程* 上的任务(不继承主线程的
/// thread-local)仍可访问运行时的 driver 与 I/O 注册表以注册 FD 兴趣。
///
/// This is a **resettable** global (RwLock<Option<Handle>>, not OnceLock) so
/// that `block_on` clears it on exit. With OnceLock the first test's handle
/// leaked into every subsequent test, breaking test isolation — `try_current()`
/// would return a stale handle from a torn-down runtime. Each `block_on` sets
/// it on entry and clears it on exit.
///
/// 这是**可重置**的全局存储(RwLock<Option<Handle>>,非 OnceLock),以便
/// `block_on` 在退出时清空它。若用 OnceLock,首个测试的 handle 会泄漏到后续
/// 每个测试,破坏测试隔离——`try_current()` 会返回已销毁 runtime 的过期
/// handle。每次 `block_on` 在进入时设置、退出时清空。
static GLOBAL_HANDLE: RwLock<Option<Handle>> = RwLock::new(None);

/// Runtime configuration / 运行时配置
///
/// Configuration values for the async runtime. In the async-executor/async-io
/// backend these are kept for API/Builder compatibility but most no longer drive
/// runtime internals (the executor and reactor manage their own queues and
/// driver). `enable_parking` / `park_timeout` are retained for compatibility
/// with existing call sites and tests.
///
/// 异步运行时的配置值。在 async-executor/async-io 后端中,这些为 API/Builder
/// 兼容性而保留,但多数已不再驱动 runtime 内部（执行器与 reactor 自行管理其队列
/// 与 driver）。`enable_parking` / `park_timeout` 为兼容既有调用点与测试而保留。
#[derive(Debug, Clone)]
pub struct RuntimeConfig
{
    /// Worker thread count hint (compat; async-executor is single-thread
    /// block_on-driven, so this is informational only).
    /// 工作线程数提示(兼容;async-executor 为单线程 block_on 驱动,故仅作信息用途)。
    pub worker_threads: usize,
    /// Scheduler queue size hint (compat).
    /// 调度器队列大小提示(兼容)。
    pub queue_size: usize,
    /// Thread name prefix (compat).
    /// 线程名前缀(兼容)。
    pub thread_name: String,
    /// Enable thread parking (compat).
    /// 启用线程休眠(兼容)。
    pub enable_parking: bool,
    /// Park timeout (compat).
    /// 休眠超时(兼容)。
    pub park_timeout: Duration,
}

impl Default for RuntimeConfig
{
    fn default() -> Self
    {
        Self {
            worker_threads: 1,
            queue_size: 256,
            thread_name: "hiver-worker".to_string(),
            enable_parking: true,
            park_timeout: Duration::from_millis(100),
        }
    }
}

/// Runtime builder / 运行时构建器
///
/// Provides a fluent API for configuring and building a runtime.
/// 提供用于配置和构建运行时的流畅API。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::Runtime;
///
/// let runtime = Runtime::builder()
///     .worker_threads(4)
///     .queue_size(512)
///     .build()
///     .unwrap();
/// ```
pub struct RuntimeBuilder
{
    config: RuntimeConfig,
}

impl RuntimeBuilder
{
    /// Create a new runtime builder with default configuration
    /// 使用默认配置创建新的运行时构建器
    pub fn new() -> Self
    {
        Self {
            config: RuntimeConfig::default(),
        }
    }

    /// Set the number of worker threads (compat hint).
    /// 设置工作线程数量(兼容提示)。
    pub fn worker_threads(mut self, count: usize) -> Self
    {
        self.config.worker_threads = count;
        self.config.queue_size = count * 256;
        self
    }

    /// Set the queue size (compat hint).
    /// 设置队列大小(兼容提示)。
    pub fn queue_size(mut self, size: usize) -> Self
    {
        self.config.queue_size = size;
        self
    }

    /// Set the thread name pattern (compat).
    /// 设置线程名称模式(兼容)。
    pub fn thread_name(mut self, name: impl Into<String>) -> Self
    {
        self.config.thread_name = name.into();
        self
    }

    /// Set the driver type (removed: the reactor is always async-io).
    /// 设置 driver 类型(已移除:reactor 始终为 async-io)。
    #[deprecated(
        note = "driver selection is no longer configurable; the reactor is always async-io"
    )]
    pub fn driver_type(self, _driver_type: ()) -> Self
    {
        self
    }

    /// Set the I/O driver queue depth (removed: async-io manages its own sizing).
    /// 设置 I/O driver 队列深度(已移除:async-io 自行管理其容量)。
    #[deprecated(note = "io_entries is no longer configurable; async-io manages its own sizing")]
    pub fn io_entries(self, _entries: u32) -> Self
    {
        self
    }

    /// Enable or disable thread parking (compat).
    /// 启用或禁用线程休眠(兼容)。
    pub fn enable_parking(mut self, enable: bool) -> Self
    {
        self.config.enable_parking = enable;
        self
    }

    /// Set the park timeout (compat).
    /// 设置休眠超时(兼容)。
    pub fn park_timeout(mut self, timeout: Duration) -> Self
    {
        self.config.park_timeout = timeout;
        self
    }

    /// Build the runtime
    /// 构建运行时
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if runtime initialization fails.
    /// 如果运行时初始化失败则返回错误。
    pub fn build(self) -> io::Result<Runtime>
    {
        Runtime::with_config(self.config)
    }
}

impl Default for RuntimeBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// The async runtime / 异步运行时
///
/// Main entry point for the async runtime. Manages scheduler, driver, and timers.
/// 异步运行时的主入口点。管理调度器、驱动和定时器。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::Runtime;
///
/// fn main() -> std::io::Result<()> {
///     let runtime = Runtime::new()?;
///     runtime.block_on(async {
///         println!("Hello, world!");
///     });
///     Ok(())
/// }
/// ```
pub struct Runtime
{
    /// Runtime configuration (mostly compat; the executor/reactor manage
    /// their own internals). Retained so `Runtime` can be reconstructed /
    /// inspected by tooling, and to keep the builder chain coherent.
    /// 运行时配置(多数为兼容;执行器/reactor 自行管理其内部)。保留以便
    /// tooling 重建/检视 `Runtime`,并保持 builder 链一致。
    #[allow(dead_code)]
    config: RuntimeConfig,
    /// Async executor that drives spawned tasks. Stored as a `'static`
    /// reference (the executor is leaked from the heap) so that `spawn()`
    /// can reach it via the `Handle` without lifetime gymnastics, and so the
    /// executor outlives any `Handle` clone handed to user code.
    ///
    /// 异步执行器,驱动被 spawn 的任务。以 `'static` 引用存储（执行器从堆上
    /// leak）,使 `spawn()` 能经由 `Handle` 访问它而无需生命周期 gymnastics,
    /// 且执行器的生命周期长于任何交给用户代码的 `Handle` 克隆。
    executor: &'static async_executor::Executor<'static>,
}

impl Runtime
{
    /// Create a new runtime with default configuration
    /// 使用默认配置创建新的运行时
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if runtime initialization fails.
    /// 如果运行时初始化失败则返回错误。
    pub fn new() -> io::Result<Self>
    {
        Self::with_config(RuntimeConfig::default())
    }

    /// Create a runtime builder
    /// 创建运行时构建器
    pub fn builder() -> RuntimeBuilder
    {
        RuntimeBuilder::new()
    }

    /// Create a new runtime with the specified configuration
    /// 使用指定配置创建新的运行时
    ///
    /// In the async-executor/async-io backend most configuration is
    /// informational; the executor and reactor manage their own internals.
    ///
    /// 在 async-executor/async-io 后端,多数配置仅作信息用途;执行器与 reactor
    /// 自行管理其内部。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if executor creation fails (extremely unlikely).
    /// 若执行器创建失败则返回错误(极不可能)。
    pub fn with_config(config: RuntimeConfig) -> io::Result<Self>
    {
        // Create the executor. Leaked to obtain a `'static` reference so that
        // `spawn()` can capture it through the `Handle` (which needs `'static`
        // to be safely stored in a thread-local / global). The executor lives
        // for the process lifetime; runtimes are not torn down repeatedly.
        // 创建执行器。leak 以获得 `'static` 引用,使 `spawn()` 能经由 `Handle`
        //（需 `'static` 才能安全存于 thread-local / 全局）捕获它。执行器存活于
        // 进程生命周期;runtime 不会被反复销毁。
        let executor: &'static async_executor::Executor<'static> =
            Box::leak(Box::new(async_executor::Executor::new()));

        Ok(Self { config, executor })
    }

    /// Run a future to completion on this runtime
    /// 在此运行时上运行future到完成
    ///
    /// This is the main entry point for executing async code.
    /// 这是执行异步代码的主入口点。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if the future returns an error.
    /// 如果future返回错误则返回错误。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_runtime::Runtime;
    ///
    /// let runtime = Runtime::new().unwrap();
    /// runtime.block_on(async {
    ///     println!("Hello, world!");
    /// });
    /// ```
    pub fn block_on<F: Future>(&mut self, future: F) -> io::Result<F::Output>
    where
        F::Output: Send,
    {
        // Set the current runtime handle for this thread
        // 为当前线程设置运行时句柄
        let handle = Handle {
            executor: Some(self.executor),
        };
        Handle::set_current(Some(handle));
        // NOTE: we deliberately do NOT write the process-global `GLOBAL_HANDLE`
        // here. In the new single-thread-executor design, every spawned task
        // runs on THIS thread (the one driving `executor.run`), so it inherits
        // the thread-local `CURRENT_HANDLE` directly — no worker threads, no
        // global fallback needed. Writing the global caused parallel tests'
        // `block_on` calls to race on the same `RwLock` (one test's exit
        // cleared the other's live handle), hanging the suite.
        // 注意:此处故意不写入进程全局 `GLOBAL_HANDLE`。在新的单线程执行器设计中,
        // 每个 spawn 出的任务都在当前线程（驱动 `executor.run` 的那个）上运行,
        // 故直接继承 thread-local `CURRENT_HANDLE`——无需 worker 线程、无需全局
        // 回退。写入全局会导致并行测试的 `block_on` 在同一 `RwLock` 上竞争（一个
        // 测试退出清空了另一个仍活着的 handle）,使测试套件挂起。

        // Drive the main future to completion on THIS thread, together with
        // any tasks spawned onto the executor. `executor.run(future)` polls the
        // main future and drains the executor's ready queue in the same loop.
        //
        // CRITICAL: we drive this with `async_io::block_on` (not
        // `futures_lite::future::block_on`). `async_io::block_on` is the
        // reactor-aware driver that smol itself uses — it locks the async-io
        // reactor, calls `react()` to process I/O events, and uses a custom
        // waker (`BlockOnWaker`) that notifies the reactor when woken from
        // another thread. `futures_lite::block_on` is a plain parker that does
        // NOT drive the reactor — under it, a future blocked on `accept()`
        // would never make progress, and fire-and-forget spawned tasks (which
        // rely on the reactor to wake their read/timer wakers) would hang. This
        // was the root cause of the HTTP server's "connection reset" failure.
        //
        // 在当前线程上把主 future 驱动至完成,同时驱动任何被 spawn 到执行器上的
        // 任务。`executor.run(future)` 在同一循环里轮询主 future 并排空执行器的
        // 就绪队列。
        //
        // 关键:此处用 `async_io::block_on`（而非 `futures_lite::future::block_on`）
        // 驱动。`async_io::block_on` 是 smol 自身使用的 reactor 感知驱动器 —— 它
        // 锁定 async-io reactor、调用 `react()` 处理 I/O 事件,并使用自定义 waker
        //（`BlockOnWaker`）在被其它线程唤醒时通知 reactor。`futures_lite::block_on`
        // 是普通 parker,不驱动 reactor —— 在其下,阻塞于 `accept()` 的 future 永远
        // 不会推进,依赖 reactor 唤醒其 read/timer waker 的 fire-and-forget 任务会
        // 挂起。这正是 HTTP 服务端 "connection reset" 失败的根因。
        let result = async_io::block_on(self.executor.run(future));

        // Clear the thread-local handle.
        // 清除线程本地句柄。
        Handle::set_current(None);
        // The global is never written by `block_on` in the new design (see
        // entry comment), but clear it defensively in case some other path set
        // it, so `try_current()` outside a runtime returns `None`.
        // 新设计中 `block_on` 从不写入全局（见入口注释),但防御性地清空它,
        // 以防其它路径写入,使 runtime 之外的 `try_current()` 返回 `None`。
        if let Ok(mut g) = GLOBAL_HANDLE.write()
        {
            *g = None;
        }

        Ok(result)
    }
}

/// Spawning handle for the runtime
/// 运行时的生成句柄
///
/// Provides access to the runtime's executor so that tasks running inside
/// `block_on` can call `spawn()` to schedule more tasks.
/// 提供对运行时执行器的访问,使运行在 `block_on` 内部的任务可调用 `spawn()`
/// 以调度更多任务。
#[derive(Clone)]
pub struct Handle
{
    /// The async executor, used by `spawn()` to schedule tasks. `'static` so
    /// the handle can be cloned into spawned futures and stored in the
    /// thread-local / global handle slots.
    /// 异步执行器,`spawn()` 用它调度任务。`'static` 使句柄可被克隆进被 spawn
    /// 的 future,并存于 thread-local / 全局句柄槽。
    executor: Option<&'static async_executor::Executor<'static>>,
}

impl Handle
{
    /// Get a handle to the current runtime
    /// 获取当前运行时的句柄
    ///
    /// # Panics / 恐慌
    ///
    /// Panics if called outside of a runtime context.
    /// 如果在运行时上下文之外调用则恐慌。
    #[allow(clippy::expect_used)]
    pub fn current() -> Self
    {
        Self::try_current().expect("Handle::current() called outside of a runtime context")
    }

    /// Try to get a handle to the current runtime. Returns None if outside a runtime.
    /// 尝试获取当前运行时的句柄。如果在运行时外部则返回None。
    ///
    /// In the single-thread-executor design, tasks spawned inside `block_on`
    /// run on the same thread, so they inherit the thread-local `CURRENT_HANDLE`
    /// directly. The process-global `GLOBAL_HANDLE` is a defensive fallback.
    /// 在单线程执行器设计中,在 `block_on` 内 spawn 的任务运行于同一线程,
    /// 故直接继承 thread-local `CURRENT_HANDLE`。进程全局 `GLOBAL_HANDLE`
    /// 为防御性回退。
    pub fn try_current() -> Option<Self>
    {
        let local = CURRENT_HANDLE.with(|h| h.borrow().clone());
        if local.is_some()
        {
            local
        }
        else
        {
            GLOBAL_HANDLE.read().ok()?.clone()
        }
    }

    /// Set the current runtime handle for this thread
    /// 为当前线程设置运行时句柄
    fn set_current(handle: Option<Handle>)
    {
        CURRENT_HANDLE.with(|h| *h.borrow_mut() = handle);
    }

    /// Get the async executor backing this runtime, if available.
    /// `spawn()` uses this to schedule tasks. Returns `None` for the fallback
    /// handle created outside a runtime.
    ///
    /// 获取支撑本 runtime 的异步执行器（若可用）。`spawn()` 用它调度任务。
    /// 在 runtime 之外创建的回退句柄返回 `None`。
    #[must_use]
    pub fn executor(&self) -> Option<&'static async_executor::Executor<'static>>
    {
        self.executor
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
    use super::*;

    #[test]
    fn test_runtime_config_default()
    {
        let config = RuntimeConfig::default();
        assert_eq!(config.queue_size, 256);
        assert!(config.enable_parking);
        assert_eq!(config.park_timeout.as_millis(), 100);
    }

    #[test]
    fn test_runtime_builder()
    {
        let builder = RuntimeBuilder::new()
            .worker_threads(4)
            .queue_size(512)
            .thread_name("test-worker")
            .enable_parking(false);

        assert_eq!(builder.config.queue_size, 512);
        assert_eq!(builder.config.thread_name, "test-worker");
        assert!(!builder.config.enable_parking);
    }

    #[test]
    fn test_runtime_builder_driver_config()
    {
        // driver_type/io_entries are now no-ops (reactor is always async-io);
        // verify park_timeout still propagates.
        // driver_type/io_entries 现为 no-op（reactor 始终为 async-io);
        // 验证 park_timeout 仍可传播。
        #[allow(deprecated)]
        let builder = RuntimeBuilder::new()
            .driver_type(())
            .io_entries(512)
            .park_timeout(Duration::from_millis(50));

        assert_eq!(builder.config.park_timeout.as_millis(), 50);
    }

    #[test]
    fn test_runtime_creation()
    {
        let runtime = Runtime::new();
        #[cfg(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        ))]
        {
            assert!(runtime.is_ok());
        }
    }

    #[test]
    fn test_block_on_simple()
    {
        let mut runtime = Runtime::new().unwrap();
        let result = runtime.block_on(async {});
        assert!(result.is_ok());
    }

    #[test]
    fn test_spawn_executes_through_scheduler()
    {
        use std::sync::{
            Arc,
            atomic::{AtomicI32, Ordering},
        };

        let mut runtime = Runtime::new().unwrap();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = counter.clone();

        runtime
            .block_on(async move {
                let handle = crate::task::spawn(async move {
                    counter_clone.store(42, Ordering::SeqCst);
                });
                let _ = handle.wait().await;
            })
            .unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_spawn_returns_value()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let handle = crate::task::spawn(async { 42i32 });
                let result = handle.wait().await.unwrap();
                assert_eq!(result, 42);
            })
            .unwrap();
    }

    #[test]
    fn test_multiple_spawns()
    {
        use std::sync::{
            Arc,
            atomic::{AtomicI32, Ordering},
        };

        let mut runtime = Runtime::new().unwrap();
        let counter = Arc::new(AtomicI32::new(0));

        runtime
            .block_on(async {
                let mut handles = vec![];
                for _ in 0..10
                {
                    let c = counter.clone();
                    handles.push(crate::task::spawn(async move {
                        c.fetch_add(1, Ordering::SeqCst);
                    }));
                }
                for h in handles
                {
                    let _ = h.wait().await;
                }
            })
            .unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_spawn_with_async_computation()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let h1 = crate::task::spawn(async { 1i32 });
                let h2 = crate::task::spawn(async { 2i32 });
                let h3 = crate::task::spawn(async { 3i32 });

                let sum =
                    h1.wait().await.unwrap() + h2.wait().await.unwrap() + h3.wait().await.unwrap();

                assert_eq!(sum, 6);
            })
            .unwrap();
    }

    #[test]
    fn test_spawn_join_handle_id()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let h1 = crate::task::spawn(async { 1i32 });
                let h2 = crate::task::spawn(async { 2i32 });
                assert_ne!(h1.id(), crate::task::TaskId::UNKNOWN);
                assert_ne!(h2.id(), crate::task::TaskId::UNKNOWN);
                assert_ne!(h1.id(), h2.id());
                let _ = h1.wait().await;
                let _ = h2.wait().await;
            })
            .unwrap();
    }

    #[test]
    fn test_spawn_join_handle_is_finished()
    {
        let mut runtime = Runtime::new().unwrap();
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        runtime
            .block_on(async move {
                let handle = crate::task::spawn(async move {
                    flag_clone.store(true, Ordering::SeqCst);
                });
                let _ = handle.wait().await;
                // After wait completes, the task must be finished
                assert!(flag.load(Ordering::SeqCst));
            })
            .unwrap();
    }

    #[test]
    fn test_spawn_string_return()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let handle = crate::task::spawn(async { String::from("hello") });
                let result = handle.wait().await.unwrap();
                assert_eq!(result, "hello");
            })
            .unwrap();
    }

    #[test]
    fn test_spawn_vec_return()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let handle = crate::task::spawn(async { vec![1, 2, 3] });
                let result = handle.wait().await.unwrap();
                assert_eq!(result, vec![1, 2, 3]);
            })
            .unwrap();
    }

    #[test]
    fn test_spawn_tuple_return()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let handle = crate::task::spawn(async { (42i32, true, "test".to_string()) });
                let result = handle.wait().await.unwrap();
                assert_eq!(result, (42, true, "test".to_string()));
            })
            .unwrap();
    }

    #[test]
    fn test_spawn_unit_return()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let handle: crate::task::JoinHandle<()> = crate::task::spawn(async {});
                let result = handle.wait().await;
                assert!(result.is_ok());
            })
            .unwrap();
    }

    #[test]
    fn test_spawn_option_return()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let handle = crate::task::spawn(async { Some(42i32) });
                let result = handle.wait().await.unwrap();
                assert_eq!(result, Some(42));
            })
            .unwrap();
    }

    #[test]
    fn test_nested_spawn()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                let handle = crate::task::spawn(async {
                    let inner = crate::task::spawn(async { 10i32 });
                    inner.wait().await.unwrap()
                });
                let result = handle.wait().await.unwrap();
                assert_eq!(result, 10);
            })
            .unwrap();
    }

    #[test]
    fn test_handle_current_and_try_current()
    {
        let mut runtime = Runtime::new().unwrap();

        runtime
            .block_on(async {
                // Inside runtime context, both should succeed
                let handle = Handle::current();
                assert!(Handle::try_current().is_some());

                // Verify the executor is reachable via the handle.
                // 验证执行器可经由 handle 访问。
                assert!(handle.executor().is_some());
            })
            .unwrap();

        // Outside runtime context
        assert!(Handle::try_current().is_none());
    }

    #[test]
    #[should_panic(expected = "outside of a runtime context")]
    fn test_handle_current_panics_outside_runtime()
    {
        let _ = Handle::current();
    }

    #[test]
    fn test_block_on_with_config()
    {
        let config = RuntimeConfig {
            park_timeout: Duration::from_millis(10),
            ..RuntimeConfig::default()
        };
        let mut runtime = Runtime::with_config(config).unwrap();
        let result = runtime.block_on(async {});
        assert!(result.is_ok());
    }

    /// Regression guard for the io_uring_enter ETIME-vs-ETIMEDOUT bug.
    /// io_uring_enter ETIME 与 ETIMEDOUT 混淆的回归守护。
    ///
    /// `wait_timeout` must return `Ok((n, true))` — NOT `Err` — when the
    /// kernel reports the timeout via -ETIME (errno 62). A previous fix
    /// compared against `libc::ETIMEDOUT` (errno 110), which never matched,
    /// so `block_on` aborted with `Os { code: 62, "Timer expired" }` on
    /// every pure-computation spawn. This test forces that code path with
    /// a 1ms park_timeout and many spawns so a timeout is near-certain;
    /// if the errno check regresses, `block_on().unwrap()` panics here.
    /// `wait_timeout` 在内核以 -ETIME（errno 62）报告超时时必须返回
    /// `Ok((n, true))`，而非 `Err`。之前的修复错误地与 `libc::ETIMEDOUT`
    /// （errno 110）比较，永不匹配，导致每次纯计算 spawn 都因
    /// `Os { code: 62, "Timer expired" }` 中止 block_on。本测试用 1ms
    /// park_timeout 和大量 spawn 强制触发该路径；若 errno 判断回退，
    /// `block_on().unwrap()` 会在此 panic。
    #[test]
    fn test_block_on_survives_driver_wait_timeout()
    {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        // 1ms park: run_once's driver.wait_timeout will frequently time out
        // while the scheduler worker completes spawned tasks, exercising the
        // timeout-as-normal-idle path on every iteration.
        // 1ms 休眠：run_once 的 driver.wait_timeout 会频繁超时，同时调度器
        // worker 完成已 spawn 的任务，从而在每次迭代都触发"超时即正常空闲"路径。
        let config = RuntimeConfig {
            park_timeout: Duration::from_millis(1),
            ..RuntimeConfig::default()
        };
        let mut runtime = Runtime::with_config(config).unwrap();

        let counter = Arc::new(AtomicUsize::new(0));

        runtime
            .block_on(async {
                // 50 spawns spread across many run_once iterations make a
                // driver timeout almost certain; if ETIME is mishandled as
                // an error, block_on returns Err and unwrap() fails here.
                // 50 次 spawn 分散在多次 run_once 迭代中，几乎必然触发一次
                // driver 超时；若 ETIME 被误判为错误，block_on 返回 Err，
                // 此处 unwrap() 失败。
                let mut handles = vec![];
                for _ in 0..50
                {
                    let c = counter.clone();
                    handles.push(crate::task::spawn(async move {
                        c.fetch_add(1, Ordering::Relaxed);
                    }));
                }
                for h in handles
                {
                    let _ = h.wait().await;
                }
            })
            .unwrap();

        assert_eq!(counter.load(Ordering::Relaxed), 50);
    }
}
