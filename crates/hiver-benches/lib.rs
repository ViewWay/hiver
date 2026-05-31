//! Nexus Benchmarks Library
//! Nexus 基准测试库
//!
//! Internal library for benchmark infrastructure.
//! 基准测试基础设施的内部库。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

/// Benchmark utilities
/// 基准测试工具
pub mod util {
    /// Re-export criterion for benchmark use
    /// 重新导出 criterion 供基准测试使用
    pub use criterion;
}
