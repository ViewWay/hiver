//! Extension module for HTTP types
//! HTTP类型的扩展模块

#![warn(missing_docs)]
#![warn(unreachable_pub)]

/// Extension trait for Request
/// Request的扩展trait
pub trait RequestExt {
    /// Get a value from the request extensions
    /// 从请求扩展中获取值
    fn get_ext<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;

    /// Set a value in the request extensions
    /// 在请求扩展中设置值
    fn set_ext<T: Clone + Send + Sync + 'static>(&mut self, value: T) -> Option<T>;
}

/// Extension trait for Response
/// Response的扩展trait
pub trait ResponseExt {
    /// Get a value from the response extensions
    /// 从响应扩展中获取值
    fn get_ext<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;

    /// Set a value in the response extensions
    /// 在响应扩展中设置值
    fn set_ext<T: Clone + Send + Sync + 'static>(&mut self, value: T) -> Option<T>;
}
