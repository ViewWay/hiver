//! Hiver Schedule — async-native task scheduling.
//! Hiver 调度 — async 原生任务调度。
//!
//! # Equivalent to Spring Scheduling / 等价于 Spring Scheduling
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `ScheduledTask` | `@Scheduled` |
//! | `TaskScheduler` | `@EnableScheduling` |
//! | `ScheduleType::FixedRate` | `fixedRate` |
//! | `ScheduleType::FixedDelay` | `fixedDelay` |
//! | `ScheduleType::Cron` | `cron` |
//! | `CronExpression` | `CronTrigger` |
//! | `TaskStateTracker` | `ScheduledTaskRegistrar` |
//! | `ScheduleStatistics` | `Spring Boot Actuator metrics` |
//!
//! # Rust Advantage / Rust 优势
//!
//! - async/await = zero thread pool overhead (Spring uses `ThreadPoolTaskScheduler`)
//! - `JoinHandle::abort()` = clean cancellation (Spring needs `Future.cancel()`)
//! - Compile-time cron validation via proc-macro (Spring can only validate at runtime)
//! - No GC pressure from task scheduling

#![warn(missing_docs)]
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

pub mod cron;
mod scheduled;
mod task_state;

pub use cron::{CronError, CronExpression, CronField, FieldError, FieldKind};
pub use scheduled::{
    AsyncTaskFn, ScheduleType, ScheduledTask, TaskFn, TaskScheduler, schedule_fixed_delay,
    schedule_fixed_delay_sync, schedule_fixed_rate, schedule_fixed_rate_sync,
};
pub use task_state::{ScheduleStatistics, TaskState, TaskStateTracker};

/// Re-exports of commonly used types.
/// 常用类型的重新导出。
pub mod prelude
{
    pub use crate::{
        CronError, CronExpression, ScheduleStatistics, ScheduleType, ScheduledTask, TaskScheduler,
        TaskState, TaskStateTracker, schedule_fixed_delay, schedule_fixed_rate,
    };
}

/// Default scheduled task pool size.
/// 默认定时任务线程池大小。
pub const DEFAULT_SCHEDULED_POOL_SIZE: usize = 4;

/// Default fixed rate in milliseconds.
/// 默认固定速率（毫秒）。
pub const DEFAULT_FIXED_RATE_MS: u64 = 5000;

/// Default initial delay in milliseconds.
/// 默认初始延迟（毫秒）。
pub const DEFAULT_INITIAL_DELAY_MS: u64 = 0;

/// Version of the schedule module.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
