//! Multipart form data extractor module
//! Multipart 表单数据提取器模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `UploadedFile` - `MultipartFile` representation
//! - `Multipart` - `@RequestParam` / `@RequestPart` extractor
//! - `UploadConfig` - `MultipartResolver` configuration
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_extractors::Multipart;
//!
//! async fn upload_files(multipart: Multipart) -> String {
//!     for (name, files) in multipart.files() {
//!         for file in files {
//!             let ext = file.extension().unwrap_or("unknown");
//!             println!("{}: {} ({} bytes, ext: {})", name, file.original_name, file.size, ext);
//!             file.save_to("./uploads").ok();
//!         }
//!     }
//!     format!("Uploaded {} files", multipart.file_count())
//! }
//! ```

use crate::{ExtractorError, ExtractorFuture, FromRequest, Request};
use hiver_http::HttpBody;
use std::collections::HashMap;
use std::io;
use std::path::Path;

// ============================================================================
// UploadedFile - Uploaded File Representation
// UploadedFile - 上传文件表示
// ============================================================================

/// Represents an uploaded file received in a multipart request.
/// 表示在 multipart 请求中接收的上传文件。
///
/// Equivalent to Spring's `MultipartFile`.
/// 等价于 Spring 的 `MultipartFile`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::multipart::UploadedFile;
///
/// // Save uploaded file to disk
/// file.save_to("./uploads/photo.jpg")?;
///
/// // Check file type
/// if file.is_image() {
///     println!("Image: {} ({} bytes)", file.original_name, file.size);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// Form field name.
    /// 表单字段名。
    pub name: String,

    /// Original filename from the client.
    /// 客户端提交的原始文件名。
    pub original_name: String,

    /// Content type (MIME type) of the file, if provided.
    /// 文件的内容类型（MIME 类型），如果提供的话。
    pub content_type: Option<String>,

    /// File size in bytes.
    /// 文件大小（字节）。
    pub size: usize,

    /// Raw file data.
    /// 原始文件数据。
    pub data: Vec<u8>,
}

impl UploadedFile {
    /// Create a new `UploadedFile`.
    /// 创建新的 `UploadedFile`。
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        content_type: Option<String>,
        data: Vec<u8>,
    ) -> Self {
        let size = data.len();
        Self {
            name: name.into(),
            original_name: original_name.into(),
            content_type,
            size,
            data,
        }
    }

    /// Save the file to the specified path.
    /// 将文件保存到指定路径。
    ///
    /// Creates parent directories if they do not exist.
    /// 如果父目录不存在则创建。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// uploaded_file.save_to("./uploads/photo.jpg")?;
    /// ```
    pub fn save_to(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &self.data)
    }

    /// Get the file extension (lowercase), if present.
    /// 获取文件扩展名（小写），如果存在的话。
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// if let Some(ext) = file.extension() {
    ///     println!("Extension: {}", ext);
    /// }
    /// ```
    pub fn extension(&self) -> Option<&str> {
        // Find the last dot and ensure there is text both before and after it.
        // 找到最后一个点，并确保其前后都有文本。
        let pos = self.original_name.rfind('.')?;
        if pos == 0 || pos + 1 >= self.original_name.len() {
            return None;
        }
        Some(&self.original_name[pos + 1..])
    }

    /// Check if the file is an image based on content type or extension.
    /// 根据内容类型或扩展名检查文件是否为图片。
    pub fn is_image(&self) -> bool {
        const IMAGE_EXTENSIONS: &[&str] = &[
            "jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico", "tiff", "avif",
        ];
        const IMAGE_TYPES: &[&str] = &["image/"];

        if let Some(ct) = &self.content_type {
            if IMAGE_TYPES.iter().any(|t| ct.starts_with(t)) {
                return true;
            }
        }
        self.extension().is_some_and(|ext| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|&e| ext.eq_ignore_ascii_case(e))
        })
    }

    /// Check if the file is a document based on content type or extension.
    /// 根据内容类型或扩展名检查文件是否为文档。
    pub fn is_document(&self) -> bool {
        const DOC_EXTENSIONS: &[&str] = &[
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf", "odt", "ods", "csv",
        ];
        const DOC_TYPES: &[&str] = &[
            "application/pdf",
            "application/msword",
            "application/vnd.openxmlformats",
            "application/vnd.oasis",
            "text/plain",
            "text/csv",
        ];

        if let Some(ct) = &self.content_type {
            if DOC_TYPES.iter().any(|t| ct.starts_with(t)) {
                return true;
            }
        }
        self.extension()
            .is_some_and(|ext| DOC_EXTENSIONS.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
    }

    /// Get the file data as a byte slice.
    /// 获取文件数据的字节切片。
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// Check if the file has a specific content type prefix.
    /// 检查文件是否具有特定的内容类型前缀。
    pub fn has_content_type(&self, prefix: &str) -> bool {
        self.content_type
            .as_ref()
            .is_some_and(|ct| ct.starts_with(prefix))
    }

    /// Check if the file has a specific extension (case-insensitive).
    /// 检查文件是否具有特定扩展名（不区分大小写）。
    pub fn has_extension(&self, ext: &str) -> bool {
        self.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    }
}

// ============================================================================
// Multipart - Multipart Form Data Extractor
// Multipart - Multipart 表单数据提取器
// ============================================================================

/// Multipart form data extractor.
/// Multipart 表单数据提取器。
///
/// Extracts both text fields and uploaded files from a `multipart/form-data` request.
/// 从 `multipart/form-data` 请求中提取文本字段和上传文件。
///
/// Equivalent to Spring's `MultipartHttpServletRequest` or `@RequestPart`.
/// 等价于 Spring 的 `MultipartHttpServletRequest` 或 `@RequestPart`。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::Multipart;
///
/// async fn handle_upload(multipart: Multipart) -> String {
///     // Access a single file
///     if let Some(files) = multipart.get_file("avatar") {
///         if let Some(file) = files.first() {
///             file.save_to("./uploads/avatar.png").ok();
///         }
///     }
///
///     // Access a text field
///     if let Some(desc) = multipart.get_field("description") {
///         println!("Description: {}", desc);
///     }
///
///     format!("Received {} files, {} fields",
///         multipart.file_count(), multipart.field_count())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Multipart {
    /// Text fields from the form.
    /// 表单中的文本字段。
    fields: HashMap<String, String>,

    /// Uploaded files grouped by form field name.
    /// 按表单字段名分组的上传文件。
    files: HashMap<String, Vec<UploadedFile>>,
}

impl Multipart {
    /// Create a new empty `Multipart`.
    /// 创建新的空 `Multipart`。
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            files: HashMap::new(),
        }
    }

    /// Add a text field.
    /// 添加文本字段。
    pub fn add_field(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.fields.insert(name.into(), value.into());
    }

    /// Add an uploaded file.
    /// 添加上传文件。
    pub fn add_file(&mut self, name: impl Into<String>, file: UploadedFile) {
        self.files.entry(name.into()).or_default().push(file);
    }

    /// Get a text field value by name.
    /// 按名称获取文本字段值。
    pub fn get_field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// Get all text fields.
    /// 获取所有文本字段。
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Get files by field name. Returns `None` if no files were uploaded under that name.
    /// 按字段名获取文件。如果该字段名下没有文件则返回 `None`。
    pub fn get_file(&self, name: &str) -> Option<&Vec<UploadedFile>> {
        self.files.get(name)
    }

    /// Get the first file for a given field name, if any.
    /// 获取给定字段名的第一个文件（如果有）。
    pub fn first_file(&self, name: &str) -> Option<&UploadedFile> {
        self.files.get(name).and_then(|v| v.first())
    }

    /// Get all files.
    /// 获取所有文件。
    pub fn files(&self) -> &HashMap<String, Vec<UploadedFile>> {
        &self.files
    }

    /// Get the total number of uploaded files across all fields.
    /// 获取所有字段中上传文件的总数量。
    pub fn file_count(&self) -> usize {
        self.files.values().map(Vec::len).sum()
    }

    /// Get the number of text fields.
    /// 获取文本字段的数量。
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Check if a file field exists.
    /// 检查文件字段是否存在。
    pub fn has_file(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    /// Check if a text field exists.
    /// 检查文本字段是否存在。
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }
}

impl Default for Multipart {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// UploadConfig - File Upload Configuration
// UploadConfig - 文件上传配置
// ============================================================================

/// Configuration for file upload limits and constraints.
/// 文件上传限制和约束的配置。
///
/// Equivalent to Spring's `MultipartResolver` configuration properties.
/// 等价于 Spring 的 `MultipartResolver` 配置属性。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::multipart::UploadConfig;
///
/// let config = UploadConfig::builder()
///     .max_file_size(5 * 1024 * 1024)   // 5MB per file
///     .max_total_size(50 * 1024 * 1024)  // 50MB total
///     .max_files_per_field(5)
///     .allowed_extensions(vec!["jpg", "png", "gif".to_string()])
///     .allowed_mime_types(vec!["image/".to_string()])
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Maximum size of a single file in bytes. Default: 10MB.
    /// 单个文件的最大大小（字节）。默认：10MB。
    pub max_file_size: usize,

    /// Maximum total size of all files in a request in bytes. Default: 100MB.
    /// 请求中所有文件的最大总大小（字节）。默认：100MB。
    pub max_total_size: usize,

    /// Maximum number of files per form field. Default: 10.
    /// 每个表单字段的最大文件数量。默认：10。
    pub max_files_per_field: usize,

    /// Allowed file extensions (lowercase, without dot). Empty = allow all.
    /// 允许的文件扩展名（小写，不带点）。空 = 允许所有。
    pub allowed_extensions: Vec<String>,

    /// Allowed MIME type prefixes. Empty = allow all.
    /// 允许的 MIME 类型前缀。空 = 允许所有。
    pub allowed_mime_types: Vec<String>,

    /// Directory for saving uploaded files. Default: "./uploads".
    /// 保存上传文件的目录。默认："./uploads"。
    pub upload_dir: String,
}

impl UploadConfig {
    /// Default file size limit: 10MB.
    /// 默认文件大小限制：10MB。
    pub const DEFAULT_MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

    /// Default total size limit: 100MB.
    /// 默认总大小限制：100MB。
    pub const DEFAULT_MAX_TOTAL_SIZE: usize = 100 * 1024 * 1024;

    /// Default max files per field: 10.
    /// 默认每个字段最大文件数：10。
    pub const DEFAULT_MAX_FILES_PER_FIELD: usize = 10;

    /// Default upload directory.
    /// 默认上传目录。
    pub const DEFAULT_UPLOAD_DIR: &'static str = "./uploads";

    /// Create a new `UploadConfig` with default settings.
    /// 使用默认设置创建新的 `UploadConfig`。
    pub fn new() -> Self {
        Self {
            max_file_size: Self::DEFAULT_MAX_FILE_SIZE,
            max_total_size: Self::DEFAULT_MAX_TOTAL_SIZE,
            max_files_per_field: Self::DEFAULT_MAX_FILES_PER_FIELD,
            allowed_extensions: Vec::new(),
            allowed_mime_types: Vec::new(),
            upload_dir: Self::DEFAULT_UPLOAD_DIR.to_string(),
        }
    }

    /// Create a builder for `UploadConfig`.
    /// 创建 `UploadConfig` 的构建器。
    pub fn builder() -> UploadConfigBuilder {
        UploadConfigBuilder::new()
    }

    /// Validate a single file against this configuration.
    /// 根据此配置验证单个文件。
    ///
    /// Returns `Ok(())` if the file passes all checks.
    /// 如果文件通过所有检查则返回 `Ok(())`。
    pub fn validate_file(&self, file: &UploadedFile) -> Result<(), UploadError> {
        // Check file size
        // 检查文件大小
        if file.size > self.max_file_size {
            return Err(UploadError::FileSizeExceeded {
                name: file.original_name.clone(),
                size: file.size,
                max: self.max_file_size,
            });
        }

        // Check extension
        // 检查扩展名
        if !self.allowed_extensions.is_empty() {
            if let Some(ext) = file.extension() {
                if !self
                    .allowed_extensions
                    .iter()
                    .any(|allowed| ext.eq_ignore_ascii_case(allowed))
                {
                    return Err(UploadError::ExtensionNotAllowed {
                        name: file.original_name.clone(),
                        extension: ext.to_string(),
                    });
                }
            } else {
                return Err(UploadError::ExtensionRequired {
                    name: file.original_name.clone(),
                });
            }
        }

        // Check MIME type
        // 检查 MIME 类型
        if !self.allowed_mime_types.is_empty() {
            if let Some(ref ct) = file.content_type {
                if !self
                    .allowed_mime_types
                    .iter()
                    .any(|allowed| ct.starts_with(allowed))
                {
                    return Err(UploadError::MimeTypeNotAllowed {
                        name: file.original_name.clone(),
                        content_type: ct.clone(),
                    });
                }
            } else {
                return Err(UploadError::MimeTypeRequired {
                    name: file.original_name.clone(),
                });
            }
        }

        Ok(())
    }
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// UploadConfigBuilder
// UploadConfigBuilder - 上传配置构建器
// ============================================================================

/// Builder for `UploadConfig`.
/// `UploadConfig` 的构建器。
#[derive(Debug, Clone)]
pub struct UploadConfigBuilder {
    max_file_size: usize,
    max_total_size: usize,
    max_files_per_field: usize,
    allowed_extensions: Vec<String>,
    allowed_mime_types: Vec<String>,
    upload_dir: String,
}

impl UploadConfigBuilder {
    /// Create a new builder with default values.
    /// 使用默认值创建新的构建器。
    pub fn new() -> Self {
        Self {
            max_file_size: UploadConfig::DEFAULT_MAX_FILE_SIZE,
            max_total_size: UploadConfig::DEFAULT_MAX_TOTAL_SIZE,
            max_files_per_field: UploadConfig::DEFAULT_MAX_FILES_PER_FIELD,
            allowed_extensions: Vec::new(),
            allowed_mime_types: Vec::new(),
            upload_dir: UploadConfig::DEFAULT_UPLOAD_DIR.to_string(),
        }
    }

    /// Set the maximum file size in bytes.
    /// 设置最大文件大小（字节）。
    pub fn max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set the maximum total size in bytes.
    /// 设置最大总大小（字节）。
    pub fn max_total_size(mut self, size: usize) -> Self {
        self.max_total_size = size;
        self
    }

    /// Set the maximum number of files per field.
    /// 设置每个字段的最大文件数量。
    pub fn max_files_per_field(mut self, count: usize) -> Self {
        self.max_files_per_field = count;
        self
    }

    /// Set the allowed file extensions.
    /// 设置允许的文件扩展名。
    pub fn allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = extensions;
        self
    }

    /// Set the allowed MIME type prefixes.
    /// 设置允许的 MIME 类型前缀。
    pub fn allowed_mime_types(mut self, types: Vec<String>) -> Self {
        self.allowed_mime_types = types;
        self
    }

    /// Set the upload directory.
    /// 设置上传目录。
    pub fn upload_dir(mut self, dir: impl Into<String>) -> Self {
        self.upload_dir = dir.into();
        self
    }

    /// Build the `UploadConfig`.
    /// 构建 `UploadConfig`。
    pub fn build(self) -> UploadConfig {
        UploadConfig {
            max_file_size: self.max_file_size,
            max_total_size: self.max_total_size,
            max_files_per_field: self.max_files_per_field,
            allowed_extensions: self.allowed_extensions,
            allowed_mime_types: self.allowed_mime_types,
            upload_dir: self.upload_dir,
        }
    }
}

impl Default for UploadConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// UploadError
// UploadError - 上传错误
// ============================================================================

/// Errors that can occur during file upload validation.
/// 文件上传验证过程中可能发生的错误。
#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    /// A single file exceeds the maximum allowed size.
    /// 单个文件超过最大允许大小。
    #[error("File '{name}' size ({size} bytes) exceeds maximum ({max} bytes)")]
    FileSizeExceeded {
        /// File name.
        /// 文件名。
        name: String,
        /// Actual file size in bytes.
        /// 实际文件大小（字节）。
        size: usize,
        /// Maximum allowed size in bytes.
        /// 最大允许大小（字节）。
        max: usize,
    },

    /// Total upload size exceeds the maximum allowed.
    /// 总上传大小超过最大允许值。
    #[error("Total upload size ({size} bytes) exceeds maximum ({max} bytes)")]
    TotalSizeExceeded {
        /// Total size in bytes.
        /// 总大小（字节）。
        size: usize,
        /// Maximum allowed total size in bytes.
        /// 最大允许总大小（字节）。
        max: usize,
    },

    /// Too many files for a single field.
    /// 单个字段的文件过多。
    #[error("Field '{name}' has {count} files, maximum is {max}")]
    TooManyFiles {
        /// Field name.
        /// 字段名。
        name: String,
        /// Number of files in the field.
        /// 字段中的文件数量。
        count: usize,
        /// Maximum allowed files per field.
        /// 每个字段最大允许文件数。
        max: usize,
    },

    /// File extension is not allowed.
    /// 文件扩展名不被允许。
    #[error("File '{name}' has disallowed extension: .{extension}")]
    ExtensionNotAllowed {
        /// File name.
        /// 文件名。
        name: String,
        /// File extension.
        /// 文件扩展名。
        extension: String,
    },

    /// File has no extension but one is required.
    /// 文件没有扩展名但需要一个。
    #[error("File '{name}' has no extension, but one is required")]
    ExtensionRequired {
        /// File name.
        /// 文件名。
        name: String,
    },

    /// File MIME type is not allowed.
    /// 文件 MIME 类型不被允许。
    #[error("File '{name}' has disallowed content type: {content_type}")]
    MimeTypeNotAllowed {
        /// File name.
        /// 文件名。
        name: String,
        /// File content type.
        /// 文件内容类型。
        content_type: String,
    },

    /// File has no content type but one is required.
    /// 文件没有内容类型但需要一个。
    #[error("File '{name}' has no content type, but one is required")]
    MimeTypeRequired {
        /// File name.
        /// 文件名。
        name: String,
    },

    /// Missing boundary in Content-Type header.
    /// Content-Type 头中缺少 boundary。
    #[error("Missing boundary in multipart Content-Type")]
    MissingBoundary,

    /// Invalid Content-Type (not multipart).
    /// 无效的 Content-Type（不是 multipart）。
    #[error("Expected multipart/form-data content type")]
    InvalidContentType,

    /// Multipart parsing error.
    /// Multipart 解析错误。
    #[error("Multipart parsing error: {0}")]
    ParseError(String),
}

// ============================================================================
// MultipartParser - Multipart Boundary Parser
// MultipartParser - Multipart 边界解析器
// ============================================================================

/// Parser for `multipart/form-data` request bodies.
/// `multipart/form-data` 请求体的解析器。
///
/// Parses the raw multipart body using the boundary from the Content-Type header,
/// extracting text fields and file parts.
///
/// 使用 Content-Type 头中的 boundary 解析原始 multipart body，
/// 提取文本字段和文件部分。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_extractors::multipart::{MultipartParser, UploadConfig};
///
/// let config = UploadConfig::new();
/// let content_type = "multipart/form-data; boundary=----WebKitFormBoundary";
/// let body = b"------WebKitFormBoundary\r\n...";
///
/// let multipart = MultipartParser::parse(content_type, body, &config)?;
/// ```
pub struct MultipartParser;

impl MultipartParser {
    /// Parse a multipart request body into a `Multipart` structure.
    /// 将 multipart 请求体解析为 `Multipart` 结构。
    ///
    /// # Arguments / 参数
    ///
    /// - `content_type` - The value of the Content-Type header.
    ///   Content-Type 头的值。
    /// - `body` - The raw request body bytes.
    ///   原始请求体字节。
    /// - `config` - Upload configuration for validation.
    ///   用于验证的上传配置。
    pub fn parse(
        content_type: &str,
        body: &[u8],
        config: &UploadConfig,
    ) -> Result<Multipart, UploadError> {
        // Validate content type
        // 验证 content type
        if !content_type.starts_with("multipart/form-data") {
            return Err(UploadError::InvalidContentType);
        }

        // Extract boundary
        // 提取 boundary
        let boundary = content_type
            .split("boundary=")
            .nth(1)
            .ok_or(UploadError::MissingBoundary)?
            .trim();

        // Remove any trailing semicolon or whitespace from boundary
        // 删除 boundary 末尾的分号或空格
        let boundary = boundary.split(';').next().unwrap_or(boundary).trim();

        if boundary.is_empty() {
            return Err(UploadError::MissingBoundary);
        }

        let boundary_bytes = format!("--{}", boundary);
        let _end_boundary_bytes = format!("--{}--", boundary);

        let mut multipart = Multipart::new();
        let mut total_size: usize = 0;

        // Split the body into parts using the boundary
        // 使用 boundary 将 body 分割为各个部分
        let body_str = String::from_utf8_lossy(body);
        let parts: Vec<&str> = body_str.split(&boundary_bytes).collect();

        for part in parts.iter().skip(1) {
            // Check for end boundary
            // 检查结束 boundary
            if part.starts_with("--") {
                break;
            }

            // Remove leading \r\n
            // 删除前导 \r\n
            let part = part.strip_prefix("\r\n").unwrap_or(part);

            // Split headers from body (separated by \r\n\r\n)
            // 分离头部和主体（以 \r\n\r\n 分隔）
            let (header_section, body_content) = match part.split_once("\r\n\r\n") {
                Some((h, b)) => (h, b),
                None => continue,
            };

            // Remove trailing \r\n from body content (before next boundary)
            // 删除 body content 末尾的 \r\n（在下一个 boundary 之前）
            let body_content = body_content.strip_suffix("\r\n").unwrap_or(body_content);

            // Parse Content-Disposition header to get field name and filename
            // 解析 Content-Disposition 头以获取字段名和文件名
            let (field_name, filename) = parse_content_disposition(header_section);

            // Check if this is a file (has filename) or a text field
            // 检查这是文件（有文件名）还是文本字段
            if let Some(filename) = filename {
                // Extract content type from headers
                // 从头部提取 content type
                let content_type_part = extract_header_value(header_section, "content-type");

                let file_data = body_content.as_bytes().to_vec();
                total_size += file_data.len();

                // Check total size
                // 检查总大小
                if total_size > config.max_total_size {
                    return Err(UploadError::TotalSizeExceeded {
                        size: total_size,
                        max: config.max_total_size,
                    });
                }

                let uploaded_file =
                    UploadedFile::new(&field_name, filename, content_type_part, file_data);

                // Validate the file
                // 验证文件
                config.validate_file(&uploaded_file)?;

                // Check per-field file count
                // 检查每个字段的文件数量
                let field_count = multipart
                    .files()
                    .get(&field_name)
                    .map(Vec::len)
                    .unwrap_or(0);
                if field_count >= config.max_files_per_field {
                    return Err(UploadError::TooManyFiles {
                        name: field_name,
                        count: field_count + 1,
                        max: config.max_files_per_field,
                    });
                }

                multipart.add_file(&field_name, uploaded_file);
            } else {
                // Text field
                // 文本字段
                let value = body_content.to_string();
                multipart.add_field(&field_name, value);
            }
        }

        Ok(multipart)
    }
}

/// Parse Content-Disposition header to extract field name and optional filename.
/// 解析 Content-Disposition 头以提取字段名和可选的文件名。
fn parse_content_disposition(header_section: &str) -> (String, Option<String>) {
    let mut field_name = String::new();
    let mut filename: Option<String> = None;

    for line in header_section.lines() {
        let line = line.trim();
        let lower = line.to_lowercase();
        if !lower.starts_with("content-disposition:") {
            continue;
        }
        // Extract value from the original line (preserves case in quoted strings)
        // 从原始行提取值（保留引号内字符串的大小写）
        let value = line.splitn(2, ':').nth(1).unwrap_or("").trim();

        for part in value.split(';') {
            let part = part.trim();
            if let Some(name) = part.strip_prefix("name=") {
                field_name = unquote(name).to_string();
            } else if let Some(name) = part.strip_prefix("filename=") {
                filename = Some(unquote(name).to_string());
            } else if let Some(name) = part.strip_prefix("filename*=") {
                // RFC 5987 encoded filename
                // RFC 5987 编码的文件名
                filename = Some(unquote(name).to_string());
            }
        }
    }

    (field_name, filename)
}

/// Extract a header value by name (case-insensitive).
/// 按名称提取头部值（不区分大小写）。
fn extract_header_value(header_section: &str, header_name: &str) -> Option<String> {
    for line in header_section.lines() {
        let line = line.trim();
        if let Some(value) = line.split_once(':') {
            if value.0.trim().eq_ignore_ascii_case(header_name) {
                return Some(value.1.trim().to_string());
            }
        }
    }
    None
}

/// Remove surrounding quotes from a string.
/// 删除字符串周围的引号。
fn unquote(s: &str) -> &str {
    s.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s)
}

// ============================================================================
// FromRequest implementation for Multipart
// Multipart 的 FromRequest 实现
// ============================================================================

impl FromRequest for Multipart {
    fn from_request(req: &Request) -> ExtractorFuture<Self> {
        let content_type = req.header("content-type").unwrap_or("").to_string();
        let body_bytes = req.body().as_bytes().map(<[u8]>::to_vec);

        Box::pin(async move {
            let config = UploadConfig::new();
            let body = body_bytes.ok_or_else(|| {
                ExtractorError::Invalid("Request body is not available".to_string())
            })?;

            MultipartParser::parse(&content_type, &body, &config)
                .map_err(|e| ExtractorError::Other(e.to_string()))
        })
    }
}

// ============================================================================
// Utility Functions
// 工具函数
// ============================================================================

/// Get common media type by extension.
/// 根据扩展名获取常见媒体类型。
pub fn media_type_for_extension(extension: &str) -> Option<&'static str> {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "svg" => Some("image/svg+xml"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "tiff" | "tif" => Some("image/tiff"),
        "avif" => Some("image/avif"),
        "pdf" => Some("application/pdf"),
        "doc" => Some("application/msword"),
        "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        "xls" => Some("application/vnd.ms-excel"),
        "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        "txt" => Some("text/plain"),
        "html" | "htm" => Some("text/html"),
        "css" => Some("text/css"),
        "js" => Some("application/javascript"),
        "json" => Some("application/json"),
        "xml" => Some("application/xml"),
        "zip" => Some("application/zip"),
        "mp3" => Some("audio/mpeg"),
        "mp4" => Some("video/mp4"),
        "wav" => Some("audio/wav"),
        "avi" => Some("video/x-msvideo"),
        "csv" => Some("text/csv"),
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

    // -- UploadedFile tests --

    #[test]
    fn test_uploaded_file_creation() {
        let file = UploadedFile::new(
            "photo",
            "landscape.jpg",
            Some("image/jpeg".to_string()),
            b"fake-jpeg-data".to_vec(),
        );
        assert_eq!(file.name, "photo");
        assert_eq!(file.original_name, "landscape.jpg");
        assert_eq!(file.content_type, Some("image/jpeg".to_string()));
        assert_eq!(file.size, 14);
        assert_eq!(file.bytes(), b"fake-jpeg-data");
    }

    #[test]
    fn test_uploaded_file_extension() {
        let file = UploadedFile::new("f", "photo.jpg", None, vec![]);
        assert_eq!(file.extension(), Some("jpg"));

        let no_ext = UploadedFile::new("f", "README", None, vec![]);
        assert_eq!(no_ext.extension(), None);

        let dot_only = UploadedFile::new("f", "dotfile.", None, vec![]);
        assert_eq!(dot_only.extension(), None);
    }

    #[test]
    fn test_uploaded_file_is_image() {
        let by_ext = UploadedFile::new("f", "photo.png", None, vec![]);
        assert!(by_ext.is_image());

        let by_ct = UploadedFile::new("f", "file.dat", Some("image/webp".to_string()), vec![]);
        assert!(by_ct.is_image());

        let not_image =
            UploadedFile::new("f", "doc.pdf", Some("application/pdf".to_string()), vec![]);
        assert!(!not_image.is_image());
    }

    #[test]
    fn test_uploaded_file_is_document() {
        let by_ext = UploadedFile::new("f", "report.pdf", None, vec![]);
        assert!(by_ext.is_document());

        let by_ct = UploadedFile::new("f", "file.dat", Some("text/plain".to_string()), vec![]);
        assert!(by_ct.is_document());

        let not_doc = UploadedFile::new("f", "photo.jpg", Some("image/jpeg".to_string()), vec![]);
        assert!(!not_doc.is_document());
    }

    #[test]
    fn test_uploaded_file_save_to() {
        let dir = std::env::temp_dir().join("hiver_test_upload");
        let file = UploadedFile::new("f", "test.txt", None, b"hello".to_vec());
        let path = dir.join("subdir").join("test.txt");

        file.save_to(&path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "hello");

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- Multipart tests --

    #[test]
    fn test_multipart_empty() {
        let m = Multipart::new();
        assert_eq!(m.file_count(), 0);
        assert_eq!(m.field_count(), 0);
        assert!(!m.has_file("a"));
        assert!(!m.has_field("b"));
    }

    #[test]
    fn test_multipart_fields_and_files() {
        let mut m = Multipart::new();
        m.add_field("title", "Hello");
        m.add_file("avatar", UploadedFile::new("avatar", "a.jpg", None, vec![]));
        m.add_file("avatar", UploadedFile::new("avatar", "b.jpg", None, vec![]));
        m.add_file("doc", UploadedFile::new("doc", "readme.txt", None, vec![]));

        assert_eq!(m.get_field("title"), Some("Hello"));
        assert_eq!(m.field_count(), 1);
        assert_eq!(m.get_file("avatar").unwrap().len(), 2);
        assert_eq!(m.file_count(), 3);
        assert!(m.has_file("avatar"));
        assert!(m.has_file("doc"));
        assert!(!m.has_file("other"));
    }

    // -- UploadConfig tests --

    #[test]
    fn test_upload_config_defaults() {
        let config = UploadConfig::new();
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
        assert_eq!(config.max_total_size, 100 * 1024 * 1024);
        assert_eq!(config.max_files_per_field, 10);
        assert!(config.allowed_extensions.is_empty());
        assert!(config.allowed_mime_types.is_empty());
        assert_eq!(config.upload_dir, "./uploads");
    }

    #[test]
    fn test_upload_config_builder() {
        let config = UploadConfig::builder()
            .max_file_size(1024)
            .max_total_size(2048)
            .max_files_per_field(3)
            .allowed_extensions(vec!["jpg".to_string(), "png".to_string()])
            .allowed_mime_types(vec!["image/".to_string()])
            .upload_dir("/tmp/uploads")
            .build();

        assert_eq!(config.max_file_size, 1024);
        assert_eq!(config.max_total_size, 2048);
        assert_eq!(config.max_files_per_field, 3);
        assert_eq!(config.allowed_extensions.len(), 2);
        assert_eq!(config.allowed_mime_types.len(), 1);
        assert_eq!(config.upload_dir, "/tmp/uploads");
    }

    #[test]
    fn test_upload_config_validate_file_size() {
        let config = UploadConfig::builder().max_file_size(10).build();
        let file = UploadedFile::new("f", "big.txt", None, vec![0u8; 20]);
        let result = config.validate_file(&file);
        assert!(matches!(result, Err(UploadError::FileSizeExceeded { .. })));
    }

    #[test]
    fn test_upload_config_validate_extension() {
        let config = UploadConfig::builder()
            .allowed_extensions(vec!["jpg".to_string(), "png".to_string()])
            .build();

        let ok = UploadedFile::new("f", "photo.jpg", None, vec![]);
        assert!(config.validate_file(&ok).is_ok());

        let bad = UploadedFile::new("f", "script.exe", None, vec![]);
        assert!(matches!(
            config.validate_file(&bad),
            Err(UploadError::ExtensionNotAllowed { .. })
        ));
    }

    #[test]
    fn test_upload_config_validate_mime_type() {
        let config = UploadConfig::builder()
            .allowed_mime_types(vec!["image/".to_string()])
            .build();

        let ok = UploadedFile::new("f", "photo.jpg", Some("image/jpeg".to_string()), vec![]);
        assert!(config.validate_file(&ok).is_ok());

        let bad = UploadedFile::new(
            "f",
            "data.bin",
            Some("application/octet-stream".to_string()),
            vec![],
        );
        assert!(matches!(
            config.validate_file(&bad),
            Err(UploadError::MimeTypeNotAllowed { .. })
        ));
    }

    // -- MultipartParser tests --

    #[test]
    fn test_parser_invalid_content_type() {
        let result = MultipartParser::parse("application/json", &[], &UploadConfig::new());
        assert!(matches!(result, Err(UploadError::InvalidContentType)));
    }

    #[test]
    fn test_parser_missing_boundary() {
        let result = MultipartParser::parse("multipart/form-data", &[], &UploadConfig::new());
        assert!(matches!(result, Err(UploadError::MissingBoundary)));
    }

    #[test]
    fn test_parser_text_fields_only() {
        let body = concat!(
            "------Boundary\r\n",
            "Content-Disposition: form-data; name=\"title\"\r\n",
            "\r\n",
            "Hello World\r\n",
            "------Boundary\r\n",
            "Content-Disposition: form-data; name=\"description\"\r\n",
            "\r\n",
            "A description\r\n",
            "------Boundary--\r\n"
        );

        let result = MultipartParser::parse(
            "multipart/form-data; boundary=----Boundary",
            body.as_bytes(),
            &UploadConfig::new(),
        );

        let m = result.unwrap();
        assert_eq!(m.get_field("title"), Some("Hello World"));
        assert_eq!(m.get_field("description"), Some("A description"));
        assert_eq!(m.file_count(), 0);
    }

    #[test]
    fn test_parser_file_upload() {
        let body = concat!(
            "------Boundary\r\n",
            "Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "file contents here\r\n",
            "------Boundary\r\n",
            "Content-Disposition: form-data; name=\"label\"\r\n",
            "\r\n",
            "my-file\r\n",
            "------Boundary--\r\n"
        );

        let result = MultipartParser::parse(
            "multipart/form-data; boundary=----Boundary",
            body.as_bytes(),
            &UploadConfig::new(),
        );

        let m = result.unwrap();
        assert_eq!(m.file_count(), 1);
        assert_eq!(m.get_field("label"), Some("my-file"));

        let files = m.get_file("file").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].original_name, "test.txt");
        assert_eq!(files[0].content_type, Some("text/plain".to_string()));
        assert_eq!(files[0].bytes(), b"file contents here");
    }

    #[test]
    fn test_parser_total_size_limit() {
        let config = UploadConfig::builder().max_total_size(20).build();

        let big_data = "x".repeat(50);
        let body = format!(
            "------Boundary\r\n\
Content-Disposition: form-data; name=\"file\"; filename=\"big.txt\"\r\n\
Content-Type: text/plain\r\n\
\r\n\
{big_data}\r\n\
------Boundary--\r\n"
        );

        let result = MultipartParser::parse(
            "multipart/form-data; boundary=----Boundary",
            body.as_bytes(),
            &config,
        );

        assert!(matches!(result, Err(UploadError::TotalSizeExceeded { .. })));
    }

    // -- Utility tests --

    #[test]
    fn test_media_type_for_extension() {
        assert_eq!(media_type_for_extension("jpg"), Some("image/jpeg"));
        assert_eq!(media_type_for_extension("png"), Some("image/png"));
        assert_eq!(media_type_for_extension("pdf"), Some("application/pdf"));
        assert_eq!(media_type_for_extension("unknown"), Some("application/octet-stream"));
    }

    #[test]
    fn test_unquote() {
        assert_eq!(unquote("\"hello.txt\""), "hello.txt");
        assert_eq!(unquote("hello.txt"), "hello.txt");
    }
}
