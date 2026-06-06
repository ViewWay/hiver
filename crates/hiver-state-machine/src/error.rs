//! State machine errors
//! 状态机错误

use thiserror::Error;

/// State machine error type
/// 状态机错误类型
#[derive(Error, Debug)]
pub enum StateMachineError
{
    /// No valid transition for the given event
    /// 没有针对给定事件的有效转换
    #[error("No valid transition from state '{from}' on event '{event}'")]
    NoValidTransition
    {
        from: String, event: String
    },

    /// Guard evaluation failed
    /// 守卫评估失败
    #[error("Guard evaluation failed: {0}")]
    GuardFailed(String),

    /// Action execution failed
    /// 动作执行失败
    #[error("Action execution failed: {0}")]
    ActionFailed(String),

    /// State not found
    /// 状态未找到
    #[error("State not found: {0}")]
    StateNotFound(String),

    /// Invalid configuration
    /// 无效配置
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// State machine result type
/// 状态机结果类型
pub type StateMachineResult<T> = Result<T, StateMachineError>;
