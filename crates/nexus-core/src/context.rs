//! Context module
//! 上下文模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `RequestContext`
//! - `ServletRequest`, `HttpServletRequest`
//! - Model attributes

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use super::extension::Extensions;
use std::sync::Arc;

/// Request context
/// 请求上下文
///
/// This holds all the contextual information for a single HTTP request.
/// 这保存单个HTTP请求的所有上下文信息。
///
/// Equivalent to Spring's `HttpServletRequest`.
/// 等价于Spring的`HttpServletRequest`。
#[derive(Clone)]
pub struct RequestContext {
    /// Request ID
    /// 请求ID
    pub request_id: String,

    /// Extensions for custom data
    /// 自定义数据的扩展
    pub extensions: Extensions,

    /// State shared across the request
    /// 跨请求共享的状态
    pub state: Arc<State>,
}

/// Shared state
/// 共享状态
///
/// This can be used to share data across the request lifecycle.
/// 这可以用于在请求生命周期中共享数据。
#[derive(Clone, Default)]
pub struct State {
    /// Inner state data
    /// 内部状态数据
    #[allow(dead_code)] // public API scaffolding, field access not yet implemented
    inner: Arc<state::Inner>,
}

mod state {
    /// Inner state representation
    /// 内部状态表示
    #[derive(Default)]
    pub(crate) struct Inner {
        /// Shared data storage
        /// 共享数据存储
        #[allow(dead_code)] // storage for future shared state accessors
        pub(crate) data: std::sync::RwLock<super::Extensions>,
    }
}

impl RequestContext {
    /// Create a new request context
    /// 创建新请求上下文
    pub fn new() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            extensions: Extensions::new(),
            state: Arc::new(State::default()),
        }
    }

    /// Get a value from extensions
    /// 从扩展中获取值
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get()
    }

    /// Insert a value into extensions
    /// 向扩展中插入值
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.extensions.insert(val);
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}
