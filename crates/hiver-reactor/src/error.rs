//! Error types for the reactor.
//! 响应式的错误类型。

use thiserror::Error;

/// A specialized [`Result`] for reactor operations.
/// 响应式操作的专用 [`Result`]。
pub type ReactorResult<T> = Result<T, ReactorError>;

/// Errors that can occur in reactive pipelines.
/// 响应式管线中可能出现的错误。
#[derive(Debug, Error)]
pub enum ReactorError
{
    /// The pipeline produced an error value (e.g. via `Flux::error` or a failing
    /// operator). Carries the original error as a source.
    /// 管线产生了错误值（例如通过 `Flux::error` 或失败的算子）。携带原始错误作为来源。
    #[error("reactive pipeline error / 响应式管线错误: {0}")]
    Pipeline(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// A sink rejected a value due to overflow under the configured
    /// [`BackpressureStrategy`](crate::BackpressureStrategy).
    /// 在配置的 [`BackpressureStrategy`](crate::BackpressureStrategy) 下，sink 因溢出而拒绝值。
    #[error("backpressure overflow / 背压溢出: {0}")]
    Overflow(String),

    /// The sink is closed and cannot accept more values.
    /// sink 已关闭，不能再接受更多值。
    #[error("sink is closed / sink 已关闭")]
    SinkClosed,

    /// A timeout elapsed before the pipeline completed.
    /// 超时在管线完成前到期。
    #[error("timeout elapsed / 超时已到")]
    Timeout,
}

impl ReactorError
{
    /// Wrap an arbitrary error as a pipeline error.
    /// 将任意错误包装为管线错误。
    pub fn pipeline<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Pipeline(err.into())
    }
}
