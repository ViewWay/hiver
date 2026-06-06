//! Hiver Exceptions - Spring @`ControllerAdvice` equivalent features
//! Hiver Exceptions - Spring @`ControllerAdvice` 等价功能
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `@ControllerAdvice` - `ControllerAdvice` trait
//! - `@ExceptionHandler` - `ExceptionHandler` trait
//! - `ResponseEntityExceptionHandler` - `DefaultExceptionHandler`
//! - `@ResponseStatus` - `status_code()` method
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_exceptions::{ControllerAdvice, ExceptionHandler};
//! use hiver_http::{Request, Response, StatusCode};
//!
//! struct MyExceptionHandler;
//!
//! impl ControllerAdvice for MyExceptionHandler {
//!     // Auto-generated from @ExceptionHandler methods
//! }
//!
//! impl ExceptionHandler<ValidationError> for MyExceptionHandler {
//!     fn handle(&self, err: ValidationError, _req: &Request) -> Response {
//!         Response::bad_request().json(serde_json::json!({
//!             "error": err.message
//!         }))
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

pub mod advice;
pub mod error_body;
pub mod handler;
pub mod response;

pub use advice::ControllerAdvice;
pub use error_body::ErrorBody;
pub use handler::{ExceptionHandler, HandlerResult};
pub use response::ErrorResponse;

/// Version of the exceptions module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude
{
    pub use super::{ControllerAdvice, ErrorBody, ErrorResponse, ExceptionHandler};
}
