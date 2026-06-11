//! Cron expression parser and next-fire-time calculator.
//! Cron表达式解析器和下次触发时间计算器。
//!
//! Pure-Rust cron implementation — no external scheduler dependency.
//! Uses sorted `Vec<u32>` per field for O(log n) matching via binary search.
//!
//! 纯 Rust cron 实现 — 不依赖外部调度器。
//! 每个字段使用排序的 `Vec<u32>` 通过二分搜索实现 O(log n) 匹配。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring's `CronExpression` + `CronTrigger` — but Hiver adds compile-time
//! validation via `#[scheduled(cron = "...")]`.
//!
//! Spring 的 `CronExpression` + `CronTrigger` — 但 Hiver 通过
//! `#[scheduled(cron = "...")]` 增加了编译时验证。

mod expression;
mod field;

pub use expression::{CronError, CronExpression};
pub use field::{CronField, FieldError, FieldKind};
