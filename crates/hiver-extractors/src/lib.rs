//! Hiver Extractors - Request data extractors
//! Hiver提取器 - 请求数据提取器
//!
//! # Overview / 概述
//!
//! This crate provides type-safe extractors for pulling data from HTTP requests.
//! Each extractor corresponds to a Spring Boot annotation for easy migration.
//!
//! 本crate提供类型安全的提取器，用于从HTTP请求中提取数据。
//! 每个提取器对应一个Spring Boot注解，便于迁移。
//!
//! # Features / 功能特性
//!
//! - Path variable extraction (`@PathVariable`)
//! - Query parameter extraction (`@RequestParam`)
//! - JSON body extraction (`@RequestBody`)
//! - Form data extraction (`@ModelAttribute`)
//! - Header extraction (`@RequestHeader`)
//! - Cookie extraction (`@CookieValue`)
//! - Matrix variable extraction (`@MatrixVariable`)
//! - Application state injection (`@Autowired`)
//! - Request attribute access (`@RequestAttribute`)
//! - Multipart file upload (`MultipartFile`)
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! | Hiver Extractor | Spring Annotation | Description / 描述 |
//! |---|---|---|
//! | `Path<T>` | `@PathVariable` | 路径变量 / Path variables |
//! | `Query<T>` | `@RequestParam` | 查询参数 / Query parameters |
//! | `Json<T>` | `@RequestBody` | JSON请求体 / JSON request body |
//! | `Form<T>` | `@ModelAttribute` (form) | 表单数据 / Form data |
//! | `ModelAttribute<T>` | `@ModelAttribute` | 查询+表单 / Query + form |
//! | `QueryParams<T>` | `@RequestParam` (query only) | 仅查询参数 / Query parameters only |
//! | `State<T>` | `@Autowired` | 应用状态 / Application state |
//! | `RequestAttribute<T>` | `@RequestAttribute` | 请求属性 / Request attributes |
//! | `Header<T>` | `@RequestHeader` | 请求头 / Request headers |
//! | `Cookie<T>` | `@CookieValue` | Cookie值 / Cookie values |
//! | `MatrixVariable<T>` | `@MatrixVariable` | 矩阵变量 / Matrix variables |
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserQuery {
//!     name: String,
//!     page: Option<u32>,
//! }
//!
//! // GET /search?name=Alice&page=1
//! async fn search(Query(params): Query<UserQuery>) -> String {
//!     format!("Searching for: {}", params.name)
//! }
//! ```

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
pub mod header;
pub mod json;
pub mod matrix;
pub mod model;
#[cfg(feature = "multipart")]
pub mod multipart;
pub mod path;
pub mod query;
pub mod state;

pub use attribute::{NamedRequestAttribute, RequestAttribute};
pub use cookie::{Cookie, CookieOption, NamedCookie};
pub use form::Form;
pub use header::{Header, HeaderOption, NamedHeader};
pub use json::Json;
pub use matrix::{MatrixPath, MatrixVariable, MatrixVariables};
pub use model::{ModelAttribute, QueryParams};
#[cfg(feature = "multipart")]
pub use multipart::{Multipart, MultipartParser, UploadConfig, UploadError, UploadedFile};
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
/// # Description / 描述
///
/// Core trait implemented by all request data extractors.
/// Each extractor pulls a specific piece of data from the HTTP request
/// and produces a typed result, returning an [`ExtractorError`] on failure.
///
/// 所有请求数据提取器实现的核心trait。
/// 每个提取器从HTTP请求中提取特定类型的数据并产生类型化结果，
/// 失败时返回 [`ExtractorError`]。
///
/// # Equivalent to Spring Boot / 等价于 Spring Boot
///
/// This trait corresponds to Spring's method parameter resolution mechanism
/// used by `@PathVariable`, `@RequestParam`, `@RequestBody`, `@RequestHeader`,
/// `@CookieValue`, `@ModelAttribute`, `@RequestAttribute`, and `@MatrixVariable`.
///
/// 该trait对应Spring的方法参数解析机制，用于 `@PathVariable`、`@RequestParam`、
/// `@RequestBody`、`@RequestHeader`、`@CookieValue`、`@ModelAttribute`、
/// `@RequestAttribute` 和 `@MatrixVariable`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::{FromRequest, Query};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Params { id: u64 }
///
/// async fn handler(Query(params): Query<Params>) -> String {
///     format!("ID: {}", params.id)
/// }
/// ```
pub trait FromRequest: Sized {
    /// Extract from request
    /// 从请求提取
    ///
    /// Attempts to extract a value of type `Self` from the given request.
    /// Returns a future that resolves to `Result<Self, ExtractorError>`.
    ///
    /// 尝试从给定请求中提取 `Self` 类型的值。
    /// 返回一个future，解析为 `Result<Self, ExtractorError>`。
    fn from_request(req: &Request) -> ExtractorFuture<Self>;
}

/// Future type returned by `FromRequest`.
/// `FromRequest` 返回的 Future 类型。
///
/// A pinned, boxed, Send-safe future that resolves to either the extracted
/// value `T` or an [`ExtractorError`].
///
/// 一个固定化的、装箱的、Send安全的 future，解析为提取的值 `T`
/// 或 [`ExtractorError`]。
pub type ExtractorFuture<T> = Pin<Box<dyn Future<Output = Result<T, ExtractorError>> + Send>>;

/// Extractor error
/// 提取器错误
///
/// # Description / 描述
///
/// Errors that can occur during request data extraction.
/// Each variant represents a different category of extraction failure.
///
/// 请求数据提取过程中可能发生的错误。
/// 每个变体代表不同类别的提取失败。
#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    /// Missing parameter
    /// 缺少参数
    ///
    /// A required parameter was not found in the request.
    /// 请求中未找到所需参数。
    #[error("Missing parameter: {0}")]
    Missing(String),

    /// Invalid parameter format
    /// 无效参数格式
    ///
    /// A parameter was found but could not be parsed or converted
    /// to the target type.
    /// 找到了参数，但无法解析或转换为目标类型。
    #[error("Invalid parameter format: {0}")]
    Invalid(String),

    /// IO error
    /// IO错误
    ///
    /// An I/O error occurred while reading the request body.
    /// 读取请求主体时发生I/O错误。
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    /// JSON错误
    ///
    /// A JSON deserialization error occurred while parsing the
    /// request body.
    /// 解析请求主体时发生JSON反序列化错误。
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Other error
    /// 其他错误
    ///
    /// A generic error that does not fit into the other categories.
    /// 不适合其他类别的通用错误。
    #[error("Error: {0}")]
    Other(String),
}
