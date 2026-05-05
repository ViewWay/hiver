//! Multipart Form Data Support / Multipart 表单数据支持
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `MultipartFile` - Uploaded file representation
//! - `@RequestParam` / `@RequestPart` - Multipart request parts
//! - `MultipartResolver` - Multipart request handling
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_http::MultipartFile;
//! use nexus_http::multipart::MultipartForm;
//!
//! #[derive(Deserialize)]
//! struct UploadRequest {
//!     description: String,
//! }
//!
//! #[post("/upload")]
//! async fn upload_file(
//!     #[request_part] file: MultipartFile,
//!     #[request_part] data: MultipartForm<UploadRequest>,
//! ) -> Result<ApiResponse<String>, Error> {
//!     let filename = file.filename().unwrap_or("unknown");
//!     let contents = file.bytes().await?;
//!
//!     // Process file...
//!     // 处理文件...
//!
//!     Ok(ApiResponse::success_data(format!("Uploaded: {}", filename)))
//! }
//! ```

use crate::body::HttpBody;
use crate::error::{Error, Result};
use crate::request::Request;
use serde::Deserialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

// ============================================================================
// MultipartFile - Uploaded File Representation
// MultipartFile - 上传文件表示
// ============================================================================

/// Represents an uploaded file received in a multipart request
/// 表示在 multipart 请求中接收的上传文件
///
/// Equivalent to Spring's `MultipartFile`.
/// 等价于 Spring 的 `MultipartFile`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_http::MultipartFile;
///
/// async fn handle_upload(file: MultipartFile) -> Result<String, Error> {
///     let filename = file.filename().unwrap_or("unknown");
///     let content_type = file.content_type().unwrap_or("application/octet-stream");
///     let bytes = file.bytes().await?;
///
///     Ok(format!("Received {} ({}), {} bytes", filename, content_type, bytes.len()))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MultipartFile {
    /// Original filename
    /// 原始文件名
    filename: Option<String>,

    /// Content type
    /// 内容类型
    content_type: Option<String>,

    /// File data
    /// 文件数据
    data: Vec<u8>,

    /// Form field name
    /// 表单字段名
    name: String,
}

impl MultipartFile {
    /// Create a new `MultipartFile`
    /// 创建新的 `MultipartFile`
    pub fn new(
        name: impl Into<String>,
        filename: Option<String>,
        content_type: Option<String>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            name: name.into(),
            filename,
            content_type,
            data,
        }
    }

    /// Get the original filename
    /// 获取原始文件名
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// Get the content type
    /// 获取内容类型
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Get the form field name
    /// 获取表单字段名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the file size in bytes
    /// 获取文件大小（字节）
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if the file is empty
    /// 检查文件是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the file bytes as a slice
    /// 获取文件字节切片
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// Convert bytes to a String
    /// 将字节转换为字符串
    pub fn bytes_as_string(&self) -> Result<String> {
        String::from_utf8(self.data.clone())
            .map_err(|_| Error::bad_request("File is not valid UTF-8".to_string()))
    }

    /// Get file extension
    /// 获取文件扩展名
    pub fn extension(&self) -> Option<&str> {
        self.filename.as_ref()?
            .rsplit('.')
            .next()
            .filter(|ext| !ext.is_empty())
    }

    /// Check if the file has a specific content type
    /// 检查文件是否有特定内容类型
    pub fn has_content_type(&self, content_type: &str) -> bool {
        self.content_type
            .as_ref()
            .is_some_and(|ct| ct.starts_with(content_type))
    }

    /// Check if the file has a specific extension
    /// 检查文件是否有特定扩展名
    pub fn has_extension(&self, extension: &str) -> bool {
        self.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
    }
}

// ============================================================================
// MultipartForm - Multipart Form Data
// MultipartForm - Multipart 表单数据
// ============================================================================

/// Wrapper for multipart form data
/// multipart 表单数据包装器
///
/// This type wraps deserializable form data from a multipart request.
/// 此类型包装 multipart 请求中可反序列化的表单数据。
///
/// Equivalent to Spring's `@ModelAttribute` with multipart requests.
/// 等价于 Spring 的 `@ModelAttribute` 配合 multipart 请求。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_http::multipart::MultipartForm;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UploadData {
///     title: String,
///     description: String,
///     category: String,
/// }
///
/// #[post("/upload")]
/// async fn upload_with_metadata(
///     #[request_part] file: MultipartFile,
///     #[request_part] data: MultipartForm<UploadData>,
/// ) -> Result<ApiResponse<String>, Error> {
///     Ok(ApiResponse::success_data(format!(
///         "{}: {}",
///         data.title,
///         file.filename().unwrap_or("unknown")
///     )))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MultipartForm<T> {
    /// The deserialized form data
    /// 反序列化的表单数据
    pub inner: T,
}

impl<T> MultipartForm<T> {
    /// Create a new `MultipartForm`
    /// 创建新的 `MultipartForm`
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Get the inner value
    /// 获取内部值
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get a reference to the inner value
    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner value
    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> From<T> for MultipartForm<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T> std::ops::Deref for MultipartForm<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for MultipartForm<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// ============================================================================
// MultipartData - Raw Multipart Data
// MultipartData - 原始 Multipart 数据
// ============================================================================

/// Raw multipart form data
/// 原始 multipart 表单数据
///
/// Contains all parts of a multipart request including files and form fields.
/// 包含 multipart 请求的所有部分，包括文件和表单字段。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_http::multipart::MultipartData;
///
/// #[post("/upload")]
/// async fn upload_multiple(data: MultipartData) -> Result<ApiResponse<String>, Error> {
///     // Access files
///     for (name, file) in data.files() {
///         println!("File {}: {} ({} bytes)", name, file.filename().unwrap(), file.size());
///     }
///
///     // Access form fields
///     for (name, value) in data.fields() {
///         println!("Field {}: {}", name, value);
///     }
///
///     Ok(ApiResponse::success())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MultipartData {
    /// Uploaded files
    /// 上传的文件
    files: HashMap<String, MultipartFile>,

    /// Form fields
    /// 表单字段
    fields: HashMap<String, String>,
}

impl MultipartData {
    /// Create a new `MultipartData`
    /// 创建新的 `MultipartData`
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            fields: HashMap::new(),
        }
    }

    /// Add a file
    /// 添加文件
    pub fn add_file(&mut self, name: impl Into<String>, file: MultipartFile) {
        self.files.insert(name.into(), file);
    }

    /// Add a field
    /// 添加字段
    pub fn add_field(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.fields.insert(name.into(), value.into());
    }

    /// Get a file by name
    /// 按名称获取文件
    pub fn file(&self, name: &str) -> Option<&MultipartFile> {
        self.files.get(name)
    }

    /// Get all files
    /// 获取所有文件
    pub fn files(&self) -> &HashMap<String, MultipartFile> {
        &self.files
    }

    /// Get a field value by name
    /// 按名称获取字段值
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// Get all fields
    /// 获取所有字段
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Check if a file exists
    /// 检查文件是否存在
    pub fn has_file(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    /// Check if a field exists
    /// 检查字段是否存在
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// Get the number of files
    /// 获取文件数量
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Get the number of fields
    /// 获取字段数量
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

impl Default for MultipartData {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Multipart Extractor Trait
// Multipart 提取器 Trait
// ============================================================================

/// Trait for extracting multipart data from requests
/// 从请求中提取 multipart 数据的 Trait
pub trait FromMultipart: Sized {
    /// Extract multipart data from the request
    /// 从请求中提取 multipart 数据
    fn from_multipart(req: &Request) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + '_>>;
}

impl FromMultipart for MultipartData {
    fn from_multipart(req: &Request) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + '_>> {
        Box::pin(async move {
            let content_type = req.header("content-type").unwrap_or("");

            if !content_type.starts_with("multipart/") {
                return Err(Error::bad_request("Expected multipart request".to_string()));
            }

            // Parse multipart boundary
            // 解析 multipart 边界
            let _boundary = content_type
                .split("boundary=")
                .nth(1)
                .ok_or_else(|| Error::bad_request("Missing boundary in Content-Type".to_string()))?;

            // For now, return empty data - full multipart parsing would be implemented here
            // 现在返回空数据 - 完整的 multipart 解析将在这里实现
            Ok(MultipartData::new())
        })
    }
}

impl<T: for<'de> Deserialize<'de> + Send> FromMultipart for MultipartForm<T> {
    fn from_multipart(req: &Request) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + '_>> {
        Box::pin(async move {
            let body = req
                .body()
                .as_bytes()
                .ok_or_else(|| Error::bad_request("Request body is not available".to_string()))?;

            // For now, try to deserialize as JSON (form-data deserialization would be implemented here)
            // 现在尝试作为 JSON 反序列化（form-data 反序列化将在这里实现）
            let inner: T = serde_json::from_slice(body)
                .map_err(|e| Error::bad_request(format!("Failed to parse form data: {}", e)))?;

            Ok(MultipartForm::new(inner))
        })
    }
}

// ============================================================================
// File Size Validation
// 文件大小验证
// ============================================================================

/// File size limits for uploads
/// 上传文件大小限制
#[derive(Debug, Clone, Copy)]
pub struct FileSizeLimits {
    /// Maximum file size in bytes
    /// 最大文件大小（字节）
    pub max_file_size: usize,

    /// Maximum total request size in bytes
    /// 最大请求总大小（字节）
    pub max_request_size: usize,
}

impl FileSizeLimits {
    /// Create new file size limits
    /// 创建新的文件大小限制
    pub fn new(max_file_size: usize, max_request_size: usize) -> Self {
        Self {
            max_file_size,
            max_request_size,
        }
    }

    /// Default limits: 10MB per file, 100MB total
    /// 默认限制：每个文件 10MB，总共 100MB
    pub fn default_limits() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_request_size: 100 * 1024 * 1024, // 100MB
        }
    }

    /// Create custom max file size
    /// 创建自定义最大文件大小
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Create custom max request size
    /// 创建自定义最大请求大小
    pub fn with_max_request_size(mut self, size: usize) -> Self {
        self.max_request_size = size;
        self
    }
}

impl Default for FileSizeLimits {
    fn default() -> Self {
        Self::default_limits()
    }
}

// ============================================================================
// Utility Functions
// 工具函数
// ============================================================================

/// Validate file extension
/// 验证文件扩展名
pub fn validate_extension(filename: &str, allowed: &[&str]) -> bool {
    filename
        .rsplit('.')
        .next()
        .is_some_and(|ext| allowed.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)))
}

/// Validate content type
/// 验证内容类型
pub fn validate_content_type(content_type: &str, allowed: &[&str]) -> bool {
    allowed
        .iter()
        .any(|allowed| content_type.starts_with(allowed))
}

/// Get common media type by extension
/// 根据扩展名获取常见媒体类型
pub fn media_type_for_extension(extension: &str) -> Option<&'static str> {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "svg" => Some("image/svg+xml"),
        "pdf" => Some("application/pdf"),
        "txt" => Some("text/plain"),
        "html" => Some("text/html"),
        "css" => Some("text/css"),
        "js" => Some("application/javascript"),
        "json" => Some("application/json"),
        "xml" => Some("application/xml"),
        "zip" => Some("application/zip"),
        "mp3" => Some("audio/mpeg"),
        "mp4" => Some("video/mp4"),
        "wav" => Some("audio/wav"),
        "avi" => Some("video/x-msvideo"),
        _ => Some("application/octet-stream"),
    }
}

// ============================================================================
// Tests
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart_file() {
        let file = MultipartFile::new(
            "file",
            Some("test.txt".to_string()),
            Some("text/plain".to_string()),
            b"Hello, World!".to_vec(),
        );

        assert_eq!(file.name(), "file");
        assert_eq!(file.filename(), Some("test.txt"));
        assert_eq!(file.content_type(), Some("text/plain"));
        assert_eq!(file.size(), 13);
        assert!(!file.is_empty());
        assert_eq!(file.extension(), Some("txt"));
        assert!(file.has_extension("txt"));
        assert!(file.has_content_type("text/"));
    }

    #[test]
    fn test_multipart_form() {
        #[derive(Debug, Clone)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let form = MultipartForm::new(data);
        assert_eq!(form.name, "test");
        assert_eq!(form.value, 42);
    }

    #[test]
    fn test_multipart_data() {
        let mut data = MultipartData::new();
        data.add_field("title", "Test Title");
        data.add_field("description", "Test Description");

        assert_eq!(data.field("title"), Some("Test Title"));
        assert_eq!(data.field_count(), 2);
        assert!(data.has_field("title"));
        assert!(!data.has_field("missing"));
    }

    #[test]
    fn test_file_size_limits() {
        let limits = FileSizeLimits::default_limits();
        assert_eq!(limits.max_file_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_request_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_validate_extension() {
        let allowed = vec!["jpg", "png", "gif"];

        assert!(validate_extension("image.jpg", &allowed));
        assert!(validate_extension("image.PNG", &allowed));
        assert!(!validate_extension("image.pdf", &allowed));
        assert!(!validate_extension("noext", &allowed));
    }

    #[test]
    fn test_media_type_for_extension() {
        assert_eq!(media_type_for_extension("jpg"), Some("image/jpeg"));
        assert_eq!(media_type_for_extension("png"), Some("image/png"));
        assert_eq!(media_type_for_extension("pdf"), Some("application/pdf"));
        assert_eq!(media_type_for_extension("unknown"), Some("application/octet-stream"));
    }
}
