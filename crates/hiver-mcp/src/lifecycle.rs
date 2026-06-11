//! Server lifecycle state machine (State Pattern).
//! 服务器生命周期状态机（状态模式）。
//!
//! ```text
//!  ┌──────────────┐   initialize    ┌───────────────┐
//!  │  Uninitialized ├──────────────►│  Initializing  │
//!  └──────────────┘                 └───────┬───────┘
//!                                            │ notifications/initialized
//!                                            ▼
//!                                    ┌───────────────┐
//!                                    │     Ready      │◄─── ping / normal ops
//!                                    └───────┬───────┘
//!                                            │ transport close / error
//!                                            ▼
//!                                    ┌───────────────┐
//!                                    │   Shutdown     │
//!                                    └───────┬───────┘
//!                                            ▼
//!                                    ┌───────────────┐
//!                                    │    Closed      │
//!                                    └───────────────┘
//! ```

use std::fmt;

/// Server lifecycle states.
/// 服务器生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerLifecycle
{
    /// Waiting for `initialize` request.
    /// 等待 `initialize` 请求。
    Uninitialized,
    /// Received `initialize`, awaiting `notifications/initialized`.
    /// 收到 `initialize`，等待 `notifications/initialized`。
    Initializing,
    /// Fully initialized — normal operation.
    /// 完全初始化 — 正常操作。
    Ready,
    /// Shutting down — no new requests accepted.
    /// 关闭中 — 不接受新请求。
    Shutdown,
    /// Connection closed.
    /// 连接已关闭。
    Closed,
}

impl fmt::Display for ServerLifecycle
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Uninitialized => write!(f, "uninitialized"),
            Self::Initializing => write!(f, "initializing"),
            Self::Ready => write!(f, "ready"),
            Self::Shutdown => write!(f, "shutdown"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

impl ServerLifecycle
{
    /// Whether the server is ready to handle normal requests.
    /// 服务器是否就绪以处理正常请求。
    #[must_use]
    pub fn is_ready(self) -> bool
    {
        self == Self::Ready
    }

    /// Whether the server can still receive the `initialize` request.
    /// 服务器是否仍可接收 `initialize` 请求。
    #[must_use]
    pub fn can_initialize(self) -> bool
    {
        self == Self::Uninitialized
    }

    /// Transition: Uninitialized → Initializing (on `initialize` request).
    /// 转换：Uninitialized → Initializing（收到 `initialize` 请求时）。
    ///
    /// # Errors
    /// Returns `McpError::ProtocolError` if the current state is not `Uninitialized`.
    pub fn begin_initialize(self) -> Result<Self, crate::McpError>
    {
        if self == Self::Uninitialized
        {
            Ok(Self::Initializing)
        }
        else
        {
            Err(crate::McpError::ProtocolError(format!(
                "Cannot initialize from state {self}"
            )))
        }
    }

    /// Transition: Initializing → Ready (on `notifications/initialized`).
    /// 转换：Initializing → Ready（收到 `notifications/initialized` 时）。
    ///
    /// # Errors
    /// Returns `McpError::ProtocolError` if the current state is not `Initializing`.
    pub fn complete_initialize(self) -> Result<Self, crate::McpError>
    {
        if self == Self::Initializing
        {
            Ok(Self::Ready)
        }
        else
        {
            Err(crate::McpError::ProtocolError(format!(
                "Cannot complete initialization from state {self}"
            )))
        }
    }

    /// Transition: any → Shutdown.
    /// 转换：任意 → Shutdown。
    pub fn shutdown(self) -> Self
    {
        Self::Shutdown
    }

    /// Transition: Shutdown → Closed.
    /// 转换：Shutdown → Closed。
    pub fn close(self) -> Self
    {
        Self::Closed
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_lifecycle_happy_path()
    {
        let state = ServerLifecycle::Uninitialized;
        assert!(state.can_initialize());
        assert!(!state.is_ready());

        let state = state.begin_initialize().unwrap();
        assert_eq!(state, ServerLifecycle::Initializing);

        let state = state.complete_initialize().unwrap();
        assert!(state.is_ready());
        assert_eq!(state, ServerLifecycle::Ready);
    }

    #[test]
    fn test_cannot_reinitialize()
    {
        let state = ServerLifecycle::Initializing;
        assert!(state.begin_initialize().is_err());
    }

    #[test]
    fn test_cannot_complete_without_begin()
    {
        let state = ServerLifecycle::Uninitialized;
        assert!(state.complete_initialize().is_err());
    }

    #[test]
    fn test_shutdown_flow()
    {
        let state = ServerLifecycle::Ready.shutdown();
        assert_eq!(state, ServerLifecycle::Shutdown);
        let state = state.close();
        assert_eq!(state, ServerLifecycle::Closed);
    }

    #[test]
    fn test_display()
    {
        assert_eq!(ServerLifecycle::Ready.to_string(), "ready");
        assert_eq!(ServerLifecycle::Uninitialized.to_string(), "uninitialized");
    }
}
