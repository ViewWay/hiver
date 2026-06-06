//! Connection module
//! 连接模块
//!
//! # Overview / 概述
//!
//! This module provides connection tracking and management for HTTP server connections.
//! 本模块提供HTTP服务器连接的跟踪和管理。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `ServerConnection`, client connection management
//!
//! # Features / 功能
//!
//! - Connection state tracking / 连接状态跟踪
//! - Unique connection IDs / 唯一连接ID
//! - Activity monitoring for keep-alive / 保活活动监控
//! - Graceful connection closure / 优雅连接关闭

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};

use super::error::{Error, Result};

/// Global connection ID counter
/// 全局连接ID计数器
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Connection state
/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState
{
    /// Connection is active and can process requests
    /// 连接活动，可以处理请求
    Active,

    /// Connection is draining (finishing existing requests)
    /// 连接正在排空（完成现有请求）
    Draining,

    /// Connection is closed
    /// 连接已关闭
    Closed,
}

/// Represents an active HTTP connection
/// 表示活动的HTTP连接
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_http::conn::Connection;
/// use std::net::SocketAddr;
///
/// let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
/// let conn = Connection::with_address(addr);
///
/// assert_eq!(conn.remote_addr(), Some("127.0.0.1:8080"));
/// assert!(conn.is_alive());
/// assert!(conn.state() == ConnectionState::Active);
/// ```
#[derive(Debug)]
pub struct Connection
{
    /// Unique connection identifier
    /// 唯一连接标识符
    id: u64,

    /// Remote socket address
    /// 远程套接字地址
    remote_addr: Option<String>,

    /// Current connection state
    /// 当前连接状态
    state: AtomicBool, // false=Active/Draining, true=Closed

    /// Last activity timestamp
    /// 最后活动时间戳
    last_activity: Instant,

    /// Connection creation timestamp
    /// 连接创建时间戳
    created_at: Instant,

    /// Maximum idle duration before connection is considered stale
    /// 连接被视为陈旧前的最大空闲时长
    max_idle: Duration,
}

impl Connection
{
    /// Create a new connection without a remote address
    /// 创建没有远程地址的新连接
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::conn::Connection;
    ///
    /// let conn = Connection::new();
    /// assert!(conn.id() > 0);
    /// ```
    pub fn new() -> Self
    {
        Self::with_remote_addr_opt(None::<String>)
    }

    /// Create a connection with a remote address string
    /// 创建带远程地址字符串的连接
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::conn::Connection;
    ///
    /// let conn = Connection::with_remote_addr("192.168.1.100:12345");
    /// assert_eq!(conn.remote_addr(), Some("192.168.1.100:12345"));
    /// ```
    pub fn with_remote_addr(addr: impl Into<String>) -> Self
    {
        Self::with_remote_addr_opt(Some(addr.into()))
    }

    /// Create a connection with a `SocketAddr`
    /// `创建带SocketAddr的连接`
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::conn::Connection;
    /// use std::net::SocketAddr;
    ///
    /// let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    /// let conn = Connection::with_address(addr);
    /// ```
    pub fn with_address(addr: SocketAddr) -> Self
    {
        Self::with_remote_addr_opt(Some(addr.to_string()))
    }

    /// Internal constructor with optional remote address
    /// 带可选远程地址的内部构造函数
    fn with_remote_addr_opt(addr: Option<String>) -> Self
    {
        let now = Instant::now();
        Self {
            id: NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
            remote_addr: addr,
            state: AtomicBool::new(false),
            last_activity: now,
            created_at: now,
            max_idle: Duration::from_secs(60), /* Default 60 second keep-alive
                                                * 默认 60 秒保活 */
        }
    }

    /// Get the unique connection ID
    /// 获取唯一连接ID
    ///
    /// Connection IDs are monotonically increasing from 1.
    /// 连接ID从1开始单调递增。
    pub fn id(&self) -> u64
    {
        self.id
    }

    /// Get the remote address
    /// 获取远程地址
    pub fn remote_addr(&self) -> Option<&str>
    {
        self.remote_addr.as_deref()
    }

    /// Get the current connection state
    /// 获取当前连接状态
    pub fn state(&self) -> ConnectionState
    {
        if self.state.load(Ordering::Acquire)
        {
            ConnectionState::Closed
        }
        else
        {
            ConnectionState::Active
        }
    }

    /// Check if the connection is still alive
    /// 检查连接是否仍然活动
    ///
    /// A connection is considered alive if:
    /// 连接被视为活动需要满足：
    /// - It has not been closed
    /// - 它未被关闭
    /// - It has not exceeded the maximum idle time
    /// - 它未超过最大空闲时间
    pub fn is_alive(&self) -> bool
    {
        if self.state.load(Ordering::Acquire)
        {
            return false;
        }
        self.idle_duration() < self.max_idle
    }

    /// Get the duration since last activity
    /// 获取自上次活动以来的时长
    pub fn idle_duration(&self) -> Duration
    {
        self.last_activity.elapsed()
    }

    /// Update the last activity timestamp
    /// 更新最后活动时间戳
    ///
    /// This should be called whenever data is sent or received on the connection.
    /// 每当连接上发送或接收数据时应调用此方法。
    pub fn record_activity(&mut self)
    {
        self.last_activity = Instant::now();
    }

    /// Get a reference to the last activity timestamp
    /// 获取最后活动时间戳的引用
    pub fn last_activity(&self) -> Instant
    {
        self.last_activity
    }

    /// Get the connection creation timestamp
    /// 获取连接创建时间戳
    pub fn created_at(&self) -> Instant
    {
        self.created_at
    }

    /// Get the connection age
    /// 获取连接时长
    pub fn age(&self) -> Duration
    {
        self.created_at.elapsed()
    }

    /// Set the maximum idle duration
    /// 设置最大空闲时长
    ///
    /// Connections idle longer than this duration will be considered not alive.
    /// 空闲时间超过此时长的连接将被视为不活动。
    pub fn set_max_idle(&mut self, duration: Duration)
    {
        self.max_idle = duration;
    }

    /// Get the maximum idle duration
    /// 获取最大空闲时长
    pub fn max_idle(&self) -> Duration
    {
        self.max_idle
    }

    /// Close the connection
    /// 关闭连接
    ///
    /// Marks the connection as closed. Returns an error if already closed.
    /// 将连接标记为已关闭。如果已关闭则返回错误。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_http::conn::Connection;
    ///
    /// let mut conn = Connection::new();
    /// assert!(conn.is_alive());
    ///
    /// conn.close()?;
    /// assert!(!conn.is_alive());
    /// assert!(matches!(conn.state(), ConnectionState::Closed));
    /// ```
    pub fn close(&mut self) -> Result<()>
    {
        // Use compare_exchange to ensure we only close once
        // 使用compare_exchange确保只关闭一次
        if self
            .state
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(Error::connection("Connection already closed"));
        }
        Ok(())
    }

    /// Force close the connection without checking current state
    /// 强制关闭连接而不检查当前状态
    ///
    /// This will set the connection to closed state regardless of current state.
    /// 这将把连接设置为关闭状态，无论当前状态如何。
    pub fn force_close(&mut self)
    {
        self.state.store(true, Ordering::Release);
    }
}

impl Default for Connection
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Clone for Connection
{
    /// Clone creates a new connection view that shares state but with a new ID
    /// 克隆创建共享状态的新连接视图，但具有新ID
    fn clone(&self) -> Self
    {
        let now = Instant::now();
        Self {
            id: NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
            remote_addr: self.remote_addr.clone(),
            state: AtomicBool::new(self.state.load(Ordering::Acquire)),
            last_activity: now,
            created_at: now,
            max_idle: self.max_idle,
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_connection_new()
    {
        let conn = Connection::new();
        assert!(conn.id() > 0);
        assert!(conn.remote_addr().is_none());
        assert!(conn.is_alive());
        assert!(matches!(conn.state(), ConnectionState::Active));
    }

    #[test]
    fn test_connection_with_remote_addr()
    {
        let conn = Connection::with_remote_addr("127.0.0.1:8080");
        assert_eq!(conn.remote_addr(), Some("127.0.0.1:8080"));
        assert!(conn.is_alive());
    }

    #[test]
    fn test_connection_with_socket_addr()
    {
        let addr: SocketAddr = "192.168.1.1:9000".parse().unwrap();
        let conn = Connection::with_address(addr);
        assert_eq!(conn.remote_addr(), Some("192.168.1.1:9000"));
    }

    #[test]
    fn test_connection_close()
    {
        let mut conn = Connection::new();
        assert!(conn.is_alive());

        conn.close().unwrap();
        assert!(!conn.is_alive());
        assert!(matches!(conn.state(), ConnectionState::Closed));
    }

    #[test]
    fn test_connection_double_close()
    {
        let mut conn = Connection::new();
        conn.close().unwrap();

        let result = conn.close();
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_idle_duration()
    {
        let conn = Connection::new();
        let idle = conn.idle_duration();
        assert!(idle.as_millis() < 100); // Should be very recent
    }

    #[test]
    fn test_connection_age()
    {
        let conn = Connection::new();
        let age = conn.age();
        assert!(age.as_millis() < 100); // Should be very recent
    }

    #[test]
    fn test_connection_max_idle()
    {
        let mut conn = Connection::new();
        assert_eq!(conn.max_idle(), Duration::from_secs(60));

        conn.set_max_idle(Duration::from_secs(30));
        assert_eq!(conn.max_idle(), Duration::from_secs(30));
    }

    #[test]
    fn test_connection_stale_after_max_idle()
    {
        let mut conn = Connection::new();
        conn.set_max_idle(Duration::from_millis(10));

        // Simulate time passing by creating a connection with old activity
        // 通过创建活动陈旧的连接来模拟时间流逝
        conn.last_activity = Instant::now().checked_sub(Duration::from_millis(20)).unwrap();

        assert!(!conn.is_alive());
    }

    #[test]
    fn test_connection_record_activity()
    {
        let mut conn = Connection::new();
        conn.set_max_idle(Duration::from_millis(10));

        // Make connection stale
        // 使连接陈旧
        conn.last_activity = Instant::now().checked_sub(Duration::from_millis(20)).unwrap();
        assert!(!conn.is_alive());

        // Record activity
        // 记录活动
        conn.record_activity();
        assert!(conn.is_alive());
    }

    #[test]
    fn test_connection_force_close()
    {
        let mut conn = Connection::new();
        assert!(conn.is_alive());

        conn.force_close();
        assert!(!conn.is_alive());
    }

    #[test]
    fn test_connection_ids_are_unique()
    {
        let conn1 = Connection::new();
        let conn2 = Connection::new();
        let conn3 = Connection::new();

        assert!(conn1.id() < conn2.id());
        assert!(conn2.id() < conn3.id());
    }
}
