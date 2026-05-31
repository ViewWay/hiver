//! Extension traits for HTTP types / HTTP 类型的扩展 trait
//!
//! # Overview / 概述
//!
//! This module provides extension traits that add type-safe key-value storage
//! to [`Request`](crate::Request) and [`Response`](crate::Response).
//!
//! 本模块提供扩展 trait，为 [`Request`](crate::Request) 和
//! [`Response`](crate::Response) 添加类型安全的键值存储。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `HttpServletRequest.setAttribute()` / `getAttribute()`
//! - `RequestAttributes` / `ServletContext` attributes
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::ext::RequestExt;
//!
//! // Store request-scoped data
//! req.set_ext("user_id".to_string());
//!
//! // Retrieve it later in middleware or handlers
//! if let Some(user_id) = req.get_ext::<String>() {
//!     println!("User ID: {}", user_id);
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

/// Extension trait for [`Request`](crate::Request).
/// [`Request`](crate::Request) 的扩展 trait。
///
/// Provides typed get/set methods for storing request-scoped data,
/// similar to `HttpServletRequest.setAttribute()` in Spring.
///
/// 提供类型化的 get/set 方法用于存储请求范围的数据，
/// 类似于 Spring 中的 `HttpServletRequest.setAttribute()`。
pub trait RequestExt {
    /// Get a value from the request extensions.
    /// 从请求扩展中获取值。
    ///
    /// Returns `None` if no value of type `T` has been set.
    /// 如果未设置类型为 `T` 的值，返回 `None`。
    fn get_ext<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;

    /// Set a value in the request extensions.
    /// 在请求扩展中设置值。
    ///
    /// Returns the old value if one of the same type already existed.
    /// 如果已存在相同类型的旧值，则返回旧值。
    fn set_ext<T: Clone + Send + Sync + 'static>(&mut self, value: T) -> Option<T>;
}

/// Extension trait for [`Response`](crate::Response).
/// [`Response`](crate::Response) 的扩展 trait。
///
/// Provides typed get/set methods for storing response-scoped data,
/// such as tracing IDs, custom metadata, or middleware state.
///
/// 提供类型化的 get/set 方法用于存储响应范围的数据，
/// 例如追踪 ID、自定义元数据或中间件状态。
pub trait ResponseExt {
    /// Get a value from the response extensions.
    /// 从响应扩展中获取值。
    ///
    /// Returns `None` if no value of type `T` has been set.
    /// 如果未设置类型为 `T` 的值，返回 `None`。
    fn get_ext<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;

    /// Set a value in the response extensions.
    /// 在响应扩展中设置值。
    ///
    /// Returns the old value if one of the same type already existed.
    /// 如果已存在相同类型的旧值，则返回旧值。
    fn set_ext<T: Clone + Send + Sync + 'static>(&mut self, value: T) -> Option<T>;
}
