#![allow(clippy::empty_line_after_doc_comments, clippy::doc_overindented_list_items)]
//! Nexus Multipart - Spring `MultipartFile` equivalent features
//! Nexus Multipart - Spring `MultipartFile` 等价功能
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `MultipartFile` - `MultipartFile`
//! - `@RequestPart` - `Part<T>` extractor
//! - `Multipart` - `Multipart` extractor
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_multipart::Multipart;
//! use hiver_http::Request;
//!
//! async fn upload_file(mut multipart: Multipart) -> Result<String, MultipartError> {
//!     while let Some(mut field) = multipart.next_field().await? {
//!         let name = field.name().unwrap_or("").to_string();
//!         let filename = field.filename().map(|s| s.to_string());
//!         let data = field.bytes().await?;
//!
//!         // Process file / 处理文件
//!         save_file(&name, &filename, &data).await?;
//!     }
//!     Ok("Upload successful".to_string())
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
mod tests;

pub mod error;
pub mod extractor;
pub mod field;
pub mod form;

pub use error::{MultipartError, MultipartResult};
pub use extractor::{FileValidator, MultipartConfig, Part, mime_types};
pub use field::MultipartFile;
pub use form::Multipart;

/// Version of the multipart module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default max file size (10MB)
/// 默认最大文件大小（10MB）
pub const DEFAULT_MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Default max buffer size
/// 默认最大缓冲区大小
pub const DEFAULT_MAX_BUFFER_SIZE: usize = 8 * 1024;

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::mime_types;
    pub use super::{DEFAULT_MAX_BUFFER_SIZE, DEFAULT_MAX_FILE_SIZE};
    pub use super::{FileValidator, MultipartConfig, Part};
    pub use super::{Multipart, MultipartError, MultipartFile, MultipartResult};
}
