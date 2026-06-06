//! Hiver Schedule - Spring Scheduling equivalent features
//! Hiver调度 - Spring Scheduling等价功能
//!
//! # Equivalent to Spring Scheduling / 等价于 Spring Scheduling
//!
//! - `@Scheduled` - Scheduled tasks
//! - `@EnableScheduling` - Enable scheduling
//! - `@Async` - Async method execution
//! - `@EnableAsync` - Enable async execution
//! - `TaskExecutor` - Task executor

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

pub mod scheduled;

pub use scheduled::{
    ScheduleType, ScheduledTask, TaskScheduler, schedule_fixed_delay, schedule_fixed_rate,
};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude
{
    pub use super::{ScheduleType, ScheduledTask, TaskScheduler};
}

/// Version of the schedule module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default scheduled task pool size
/// 默认定时任务线程池大小
pub const DEFAULT_SCHEDULED_POOL_SIZE: usize = 4;

/// Default fixed rate for scheduled tasks (milliseconds)
/// 默认固定速率（毫秒）
pub const DEFAULT_FIXED_RATE_MS: u64 = 5000;

/// Default initial delay for scheduled tasks (milliseconds)
/// 默认初始延迟（毫秒）
pub const DEFAULT_INITIAL_DELAY_MS: u64 = 0;
