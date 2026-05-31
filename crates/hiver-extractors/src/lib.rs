//! Hiver Extractors - Request data extractors
//! Hiver提取器 - 请求数据提取器
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `Path<T>` - `@PathVariable`
//! - `Query<T>` - `@RequestParam` / `@RequestParam`
//! - `Json<T>` - `@RequestBody`
//! - `Form<T>` - Form data only
//! - `ModelAttribute<T>` - `@ModelAttribute` (query + form)
//! - `QueryParams<T>` - Query parameters only
//! - `State<T>` - `@Autowired` / Application state
//! - `RequestAttribute<T>` - `@RequestAttribute`
//! - `Header<T>` - `@RequestHeader`
//! - `Cookie<T>` - `@CookieValue`
//! - `MatrixVariable<T>` - `@MatrixVariable`

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow indexing_slicing: path variable names are guaranteed-length
// by prior length checks. Using .get() adds unnecessary complexity.
// 允许索引/切片：路径变量名由先前的长度检查保证长度。
// 使用 .get() 会增加不必要的复杂性。
#![allow(clippy::indexing_slicing)]

#[cfg(test)]
mod tests;

pub mod attribute;
pub mod cookie;
pub mod form;
#[cfg(feature = "multipart")]
pub mod multipart;
pub mod header;
pub mod json;
pub mod matrix;
pub mod model;
pub mod path;
pub mod query;
pub mod state;

pub use attribute::{NamedRequestAttribute, RequestAttribute};
pub use cookie::{Cookie, CookieOption, NamedCookie};
pub use form::Form;
#[cfg(feature = "multipart")]
pub use multipart::{Multipart, MultipartParser, UploadedFile, UploadConfig, UploadError};
pub use header::{Header, HeaderOption, NamedHeader};
pub use json::Json;
pub use matrix::{MatrixPath, MatrixVariable, MatrixVariables};
pub use model::{ModelAttribute, QueryParams};
pub use path::Path;
pub use query::Query;
pub use state::State;

use std::future::Future;
use std::pin::Pin;

// Re-export Request from hiver-http
// 从 hiver-http 重新导出 Request
pub use hiver_http::Request;

/// Extractor trait
/// 提取器trait
///
/// Equivalent to Spring's method parameter resolution for:
/// - `@PathVariable`
/// - `@RequestParam`
/// - `@RequestBody`
/// - `@RequestHeader`
/// - `@CookieValue`
/// - `@ModelAttribute`
/// - `@ModelAttribute` (with `QueryParams` variant)
/// - `@RequestAttribute`
/// - `@MatrixVariable`
///
/// 这等价于Spring的以下方法参数解析：
/// - `@PathVariable`
/// - `@RequestParam`
/// - `@RequestBody`
/// - `@RequestHeader`
/// - `@CookieValue`
/// - `@ModelAttribute`
/// - `@ModelAttribute` (带 `QueryParams` 变体)
/// - `@RequestAttribute`
/// - `@MatrixVariable`
pub trait FromRequest: Sized {
    /// Extract from request
    /// 从请求提取
    fn from_request(req: &Request) -> ExtractorFuture<Self>;
}

/// Future type returned by `FromRequest`
/// `FromRequest` 返回的 Future 类型
pub type ExtractorFuture<T> = Pin<Box<dyn Future<Output = Result<T, ExtractorError>> + Send>>;

/// Extractor error
/// 提取器错误
#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    /// Missing parameter
    /// 缺少参数
    #[error("Missing parameter: {0}")]
    Missing(String),

    /// Invalid parameter format
    /// 无效参数格式
    #[error("Invalid parameter format: {0}")]
    Invalid(String),

    /// IO error
    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    /// JSON错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Other error
    /// 其他错误
    #[error("Error: {0}")]
    Other(String),
}
