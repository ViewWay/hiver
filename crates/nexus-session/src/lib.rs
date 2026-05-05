//! Nexus Session - Distributed session management
//! Nexus 会话 - 分布式会话管理
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Nexus | Spring |
//! |-------|--------|
//! | `SessionStore` | `SessionRepository` |
//! | `Session` | `Session` |
//! | `RedisSessionStore` | `RedisIndexedSessionRepository` |
//! | `MongoSessionStore` | `MongoSessionRepository` |
//! | `SessionMiddleware` | `SessionRepositoryFilter` |
//!
//! # Features / 功能
//!
//! - In-memory session storage / 内存会话存储
//! - Redis session storage / Redis会话存储
//! - `MongoDB` session storage / `MongoDB会话存储`
//! - Session expiration / 会话过期
//! - Session attributes / 会话属性
//! - Spring Boot compatible API / Spring Boot 兼容 API
//!
//! # Quick Start / 快速开始
//!
//! ```rust,ignore
//! use nexus_session::{Session, MemorySessionStore, SessionConfig};
//! use nexus_http::{Request, Response};
//!
//! // Create session store
//! // 创建会话存储
//! let store = MemorySessionStore::new();
//!
//! // Create session from request
//! // 从请求创建会话
//! let session = Session::new(&store);
//! session.set("user_id", 123);
//! session.set("username", "john_doe");
//!
//! // Save session
//! // 保存会话
//! session.save().await?;
//! ```
//!
//! # Modules / 模块
//!
//! - [`session`] - Session types / 会话类型
//! - [`store`] - Session stores / 会话存储
//! - [`config`] - Configuration / 配置
//! - [`middleware`] - HTTP middleware / HTTP中间件
//!
//! # Examples / 示例
//!
//! More examples are available in the [Session documentation](https://nexus.viewway.io/session).
//! 更多示例请参考 [会话文档](https://nexus.viewway.io/session)。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod session;
pub mod store;
pub mod config;
pub mod middleware;

pub use session::{Session, SessionId, SessionAttribute};
pub use store::{SessionStore, MemorySessionStore};

#[cfg(feature = "redis")]
pub use store::RedisSessionStore;

#[cfg(feature = "mongodb")]
pub use store::MongoSessionStore;

pub use config::{SessionConfig, CookieConfig, SessionStrategy};

/// Version of the session module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default session timeout (seconds)
/// 默认会话超时时间（秒）
pub const DEFAULT_SESSION_TIMEOUT_SECS: u64 = 1800;

/// Default cookie name
/// 默认cookie名称
pub const DEFAULT_COOKIE_NAME: &str = "SESSION";

/// Default session ID length
/// 默认会话ID长度
pub const DEFAULT_SESSION_ID_LENGTH: usize = 32;

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        Session, SessionId, SessionAttribute,
        SessionStore, MemorySessionStore, SessionConfig,
        DEFAULT_SESSION_TIMEOUT_SECS, DEFAULT_COOKIE_NAME,
    };

    #[cfg(feature = "redis")]
    pub use super::RedisSessionStore;

    #[cfg(feature = "mongodb")]
    pub use super::MongoSessionStore;
}
