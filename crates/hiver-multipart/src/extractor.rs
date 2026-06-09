//! Multipart extractors for Hiver framework
//! Hiver框架的 Multipart 提取器
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - `@RequestPart` - `Part<T>` extractor
//! - `MultipartFile` - Direct extraction as file
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_multipart::MultipartFile;
//! use hiver_macros::post;
//!
//! #[post("/upload")]
//! async fn upload_file(file: MultipartFile) -> Result<String, MultipartError> {
//!     let filename = file.filename().unwrap_or("upload.bin");
//!     file.save_to(format!("/tmp/{}", filename)).await?;
//!     Ok("Upload successful".to_string())
//! }
//! ```

use std::collections::HashSet;

// Use hiver_http for Body type instead of http_body crate
// 使用 hiver_http 的 Body 类型而不是 http_body crate
use hiver_http::Request;

use crate::{Multipart, MultipartFile, MultipartResult, error::MultipartError};

/// File type validator
/// 文件类型验证器
#[derive(Debug, Clone)]
pub struct FileValidator
{
    /// Allowed MIME types
    /// 允许的 MIME 类型
    allowed_types: Option<HashSet<String>>,

    /// Allowed extensions
    /// 允许的扩展名
    allowed_extensions: Option<HashSet<String>>,

    /// Maximum file size in bytes
    /// 最大文件大小（字节）
    max_size: Option<usize>,
}

impl FileValidator
{
    /// Create a new file validator
    /// 创建新的文件验证器
    pub fn new() -> Self
    {
        Self {
            allowed_types: None,
            allowed_extensions: None,
            max_size: None,
        }
    }

    /// Set allowed MIME types
    /// 设置允许的 MIME 类型
    pub fn allowed_types(mut self, types: impl IntoIterator<Item = impl Into<String>>) -> Self
    {
        self.allowed_types = Some(types.into_iter().map(Into::into).collect());
        self
    }

    /// Set allowed extensions
    /// 设置允许的扩展名
    pub fn allowed_extensions(
        mut self,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self
    {
        self.allowed_extensions = Some(
            extensions
                .into_iter()
                .map(|s| s.into().to_lowercase())
                .collect(),
        );
        self
    }

    /// Set maximum file size
    /// 设置最大文件大小
    pub fn max_size(mut self, size: usize) -> Self
    {
        self.max_size = Some(size);
        self
    }

    /// Validate a multipart file
    /// 验证 multipart 文件
    pub fn validate(&self, file: &MultipartFile) -> MultipartResult<()>
    {
        // Check file size
        if let Some(max) = self.max_size
            && file.size() > max
        {
            return Err(MultipartError::FileTooLarge {
                size: file.size(),
                max,
            });
        }

        // Check MIME type
        if let Some(allowed) = &self.allowed_types
            && let Some(content_type) = file.content_type()
        {
            // Parse MIME type (handle parameters like charset)
            let mime = content_type
                .split(';')
                .next()
                .unwrap_or(content_type)
                .trim();

            if !allowed.contains(mime)
            {
                return Err(MultipartError::InvalidType {
                    found: mime.to_string(),
                    allowed: allowed.iter().cloned().collect::<Vec<_>>().join(", "),
                });
            }
        }

        // Check extension
        if let Some(allowed) = &self.allowed_extensions
            && let Some(ext) = file.extension()
            && !allowed.contains(&ext.to_lowercase())
        {
            return Err(MultipartError::InvalidExtension {
                found: ext.to_string(),
                allowed: allowed.iter().cloned().collect::<Vec<_>>().join(", "),
            });
        }

        Ok(())
    }
}

impl Default for FileValidator
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Common MIME type constants
/// 常用 MIME 类型常量
pub mod mime_types
{
    /// Image MIME types / 图片类型

    /// JPEG image format
    /// JPEG 图片格式
    pub const IMAGE_JPEG: &str = "image/jpeg";

    /// PNG image format
    /// PNG 图片格式
    pub const IMAGE_PNG: &str = "image/png";

    /// GIF image format
    /// GIF 图片格式
    pub const IMAGE_GIF: &str = "image/gif";

    /// `WebP` image format
    /// `WebP` 图片格式
    pub const IMAGE_WEBP: &str = "image/webp";

    /// SVG vector image format
    /// SVG 矢量图片格式
    pub const IMAGE_SVG: &str = "image/svg+xml";

    /// Document MIME types / 文档类型

    /// PDF document format
    /// PDF 文档格式
    pub const APPLICATION_PDF: &str = "application/pdf";

    /// JSON data format
    /// JSON 数据格式
    pub const APPLICATION_JSON: &str = "application/json";

    /// XML data format
    /// XML 数据格式
    pub const APPLICATION_XML: &str = "application/xml";

    /// Plain text format
    /// 纯文本格式
    pub const TEXT_PLAIN: &str = "text/plain";

    /// CSV spreadsheet format
    /// CSV 电子表格格式
    pub const TEXT_CSV: &str = "text/csv";

    /// Video MIME types / 视频类型

    /// MP4 video format
    /// MP4 视频格式
    pub const VIDEO_MP4: &str = "video/mp4";

    /// `WebM` video format
    /// `WebM` 视频格式
    pub const VIDEO_WEBM: &str = "video/webm";

    /// Audio MIME types / 音频类型

    /// MP3 audio format
    /// MP3 音频格式
    pub const AUDIO_MP3: &str = "audio/mpeg";

    /// WAV audio format
    /// WAV 音频格式
    pub const AUDIO_WAV: &str = "audio/wav";

    /// Common image types set / 常用图片类型集合
    pub fn image_types() -> Vec<&'static str>
    {
        vec![IMAGE_JPEG, IMAGE_PNG, IMAGE_GIF, IMAGE_WEBP, IMAGE_SVG]
    }

    /// Common document types set / 常用文档类型集合
    pub fn document_types() -> Vec<&'static str>
    {
        vec![
            APPLICATION_PDF,
            APPLICATION_JSON,
            APPLICATION_XML,
            TEXT_PLAIN,
            TEXT_CSV,
        ]
    }
}

/// Multipart extractor configuration
/// Multipart 提取器配置
#[derive(Debug, Clone)]
pub struct MultipartConfig
{
    /// Maximum file size
    /// 最大文件大小
    pub max_file_size: usize,

    /// Maximum buffer size
    /// 最大缓冲区大小
    pub max_buffer_size: usize,

    /// File validator
    /// 文件验证器
    pub file_validator: Option<FileValidator>,
}

impl Default for MultipartConfig
{
    fn default() -> Self
    {
        Self {
            max_file_size: crate::DEFAULT_MAX_FILE_SIZE,
            max_buffer_size: crate::DEFAULT_MAX_BUFFER_SIZE,
            file_validator: None,
        }
    }
}

impl MultipartConfig
{
    /// Create a new multipart config
    /// 创建新的 multipart 配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set maximum file size
    /// 设置最大文件大小
    pub fn max_file_size(mut self, size: usize) -> Self
    {
        self.max_file_size = size;
        self
    }

    /// Set maximum buffer size
    /// 设置最大缓冲区大小
    pub fn max_buffer_size(mut self, size: usize) -> Self
    {
        self.max_buffer_size = size;
        self
    }

    /// Set file validator
    /// 设置文件验证器
    pub fn file_validator(mut self, validator: FileValidator) -> Self
    {
        self.file_validator = Some(validator);
        self
    }
}

/// Part extractor - extracts a single part from multipart form
/// Part 提取器 - 从 multipart 表单提取单个部分
///
/// Equivalent to Spring's `@RequestPart`.
/// 等价于 Spring 的 `@RequestPart`。
#[derive(Debug)]
pub struct Part<T>
{
    /// The extracted value
    /// 提取的值
    pub inner: T,

    /// The field name
    /// 字段名
    pub name: String,

    /// The filename (if file upload)
    /// 文件名（如果是文件上传）
    pub filename: Option<String>,

    /// Content type
    /// 内容类型
    pub content_type: Option<String>,
}

impl<T> Part<T>
{
    /// Create a new part
    /// 创建新的 part
    pub fn new(
        inner: T,
        name: String,
        filename: Option<String>,
        content_type: Option<String>,
    ) -> Self
    {
        Self {
            inner,
            name,
            filename,
            content_type,
        }
    }

    /// Map the inner value
    /// 映射内部值
    pub fn map<U, F>(self, f: F) -> Part<U>
    where
        F: FnOnce(T) -> U,
    {
        Part {
            inner: f(self.inner),
            name: self.name,
            filename: self.filename,
            content_type: self.content_type,
        }
    }

    /// Get a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T
    {
        &self.inner
    }

    /// Get the inner value
    /// 获取内部值
    pub fn into_inner(self) -> T
    {
        self.inner
    }
}

/// Extract Multipart from request
/// 从请求提取 Multipart
///
/// This is typically used internally by the framework.
/// 这通常由框架内部使用。
///
/// # Parameters / 参数
///
/// * `req` - The HTTP request to extract multipart data from 从中提取多部分数据的 HTTP 请求
/// * `config` - Configuration for multipart processing 多部分处理的配置
///
/// # Returns / 返回
///
/// Returns `Ok(Multipart)` if the request contains valid multipart/form-data,
/// or an error if the content type is missing/invalid or the body cannot be read.
///
/// 如果请求包含有效的 multipart/form-data，则返回 `Ok(Multipart)`，
/// 如果内容类型缺失/无效或无法读取主体，则返回错误。
#[allow(clippy::unused_async)]
pub async fn extract_multipart(
    req: &Request,
    config: &MultipartConfig,
) -> MultipartResult<Multipart>
{
    // Get content type header
    // 获取 content-type 头部
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| MultipartError::InvalidRequest("Missing Content-Type header".to_string()))?;

    // Check if it's multipart
    // 检查是否为 multipart
    if !content_type.starts_with("multipart/form-data")
    {
        return Err(MultipartError::InvalidRequest(
            "Invalid Content-Type, expected multipart/form-data".to_string(),
        ));
    }

    // Get body bytes directly from the request
    // 直接从请求获取主体字节
    //
    // In hiver-http, Request::body() returns &Body (which is FullBody).
    // FullBody has a data() method that returns &Bytes.
    // We clone the bytes since Bytes is reference-counted (efficient clone).
    // 在 hiver-http 中，Request::body() 返回 &Body（即 FullBody）。
    // FullBody 有一个 data() 方法返回 &Bytes。
    // 我们克隆字节，因为 Bytes 是引用计数的（高效克隆）。
    let body_bytes = req.body().data().clone();

    // Create multipart with the extracted data
    // 使用提取的数据创建 multipart
    Multipart::new(content_type, body_bytes, config.max_file_size)
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_file_validator_new()
    {
        let validator = FileValidator::new();
        assert!(validator.allowed_types.is_none());
        assert!(validator.allowed_extensions.is_none());
        assert!(validator.max_size.is_none());
    }

    #[test]
    fn test_file_validator_allowed_types()
    {
        let types = vec!["image/jpeg", "image/png"];
        let validator = FileValidator::new().allowed_types(types);
        assert!(validator.allowed_types.is_some());
        let allowed = validator.allowed_types.unwrap();
        assert!(allowed.contains("image/jpeg"));
        assert!(allowed.contains("image/png"));
    }

    #[test]
    fn test_file_validator_allowed_extensions()
    {
        let extensions = vec!["jpg", "png", "gif"];
        let validator = FileValidator::new().allowed_extensions(extensions);
        assert!(validator.allowed_extensions.is_some());
        let allowed = validator.allowed_extensions.unwrap();
        assert!(allowed.contains("jpg"));
        assert!(allowed.contains("png"));
    }

    #[test]
    fn test_file_validator_max_size()
    {
        let validator = FileValidator::new().max_size(1024 * 1024);
        assert_eq!(validator.max_size, Some(1024 * 1024));
    }

    #[test]
    fn test_mime_types_constants()
    {
        assert_eq!(mime_types::IMAGE_JPEG, "image/jpeg");
        assert_eq!(mime_types::IMAGE_PNG, "image/png");
        assert_eq!(mime_types::APPLICATION_PDF, "application/pdf");
    }

    #[test]
    fn test_mime_types_helpers()
    {
        let image_types = mime_types::image_types();
        assert!(image_types.contains(&mime_types::IMAGE_JPEG));
        assert!(image_types.contains(&mime_types::IMAGE_PNG));

        let doc_types = mime_types::document_types();
        assert!(doc_types.contains(&mime_types::APPLICATION_PDF));
    }

    #[test]
    fn test_multipart_config_default()
    {
        let config = MultipartConfig::default();
        assert_eq!(config.max_file_size, crate::DEFAULT_MAX_FILE_SIZE);
        assert_eq!(config.max_buffer_size, crate::DEFAULT_MAX_BUFFER_SIZE);
        assert!(config.file_validator.is_none());
    }

    #[test]
    fn test_multipart_config_builder()
    {
        let validator = FileValidator::new().max_size(1024);
        let config = MultipartConfig::new()
            .max_file_size(2048)
            .max_buffer_size(4096)
            .file_validator(validator);

        assert_eq!(config.max_file_size, 2048);
        assert_eq!(config.max_buffer_size, 4096);
        assert!(config.file_validator.is_some());
    }

    #[test]
    fn test_part_new()
    {
        let part = Part::new(
            "test data".to_string(),
            "field".to_string(),
            Some("file.txt".to_string()),
            Some("text/plain".to_string()),
        );

        assert_eq!(part.inner, "test data");
        assert_eq!(part.name, "field");
        assert_eq!(part.filename, Some("file.txt".to_string()));
        assert_eq!(part.content_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_part_map()
    {
        let part = Part::new(42, "num".to_string(), None, None);
        let mapped = part.map(|x| x * 2);

        assert_eq!(mapped.inner, 84);
        assert_eq!(mapped.name, "num");
    }

    #[test]
    fn test_part_get()
    {
        let part = Part::new("value".to_string(), "key".to_string(), None, None);
        assert_eq!(part.get(), &"value".to_string());
    }

    #[test]
    fn test_part_into_inner()
    {
        let part = Part::new("value".to_string(), "key".to_string(), None, None);
        assert_eq!(part.into_inner(), "value".to_string());
    }
}
