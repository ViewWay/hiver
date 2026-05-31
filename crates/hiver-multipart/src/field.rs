//! Multipart file field
//! Multipart 文件字段

use crate::{MultipartError, MultipartResult};
use bytes::Bytes;
use std::path::Path;

/// Multipart file
/// Multipart 文件
///
/// Equivalent to Spring's `MultipartFile`.
/// `等价于Spring的MultipartFile`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PostMapping("/upload")
/// public String handleUpload(@RequestParam("file") MultipartFile file) {
///     if (!file.isEmpty()) {
///         byte[] bytes = file.getBytes();
///         // Process file
///     }
///     return "redirect:uploadSuccess";
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MultipartFile {
    /// Field name
    /// 字段名
    field_name: String,

    /// Filename
    /// 文件名
    filename: Option<String>,

    /// Content type
    /// 内容类型
    content_type: Option<String>,

    /// File data
    /// 文件数据
    data: Bytes,
}

impl MultipartFile {
    /// Create a new multipart file
    /// 创建新的 multipart 文件
    pub fn new(
        field_name: String,
        filename: Option<String>,
        content_type: Option<String>,
        data: Bytes,
    ) -> Self {
        Self {
            field_name,
            filename,
            content_type,
            data,
        }
    }

    /// Get the field name
    /// 获取字段名
    pub fn field_name(&self) -> &str {
        &self.field_name
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

    /// Get the file data as bytes
    /// 获取文件字节
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the file data as Bytes
    /// 获取文件 Bytes
    pub fn data(&self) -> Bytes {
        self.data.clone()
    }

    /// Get the file size
    /// 获取文件大小
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if the file is empty
    /// 检查文件是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the file extension
    /// 获取文件扩展名
    pub fn extension(&self) -> Option<&str> {
        self.filename
            .as_ref()
            .and_then(|name| Path::new(name).extension().and_then(|ext| ext.to_str()))
    }

    /// Save the file to a path
    /// 保存文件到路径
    pub async fn save_to<P: AsRef<Path>>(&self, path: P) -> MultipartResult<()> {
        tokio::fs::write(path, &self.data).await?;
        Ok(())
    }

    /// Get the file data as a UTF-8 string
    /// 获取文件数据为 UTF-8 字符串
    pub fn text(&self) -> MultipartResult<String> {
        std::str::from_utf8(&self.data)
            .map(ToString::to_string)
            .map_err(|_| MultipartError::Decode("Invalid UTF-8".to_string()))
    }

    /// Get the file data as JSON
    /// 获取文件数据为 JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> MultipartResult<T> {
        serde_json::from_slice(&self.data).map_err(|e| MultipartError::Decode(e.to_string()))
    }
}

/// A single field in a multipart form
/// Multipart 表单中的单个字段
#[derive(Debug)]
pub struct MultipartField {
    /// Field name
    /// 字段名
    name: String,

    /// Field filename (if file upload)
    /// 字段文件名（如果是文件上传）
    filename: Option<String>,

    /// Content type
    /// 内容类型
    content_type: Option<String>,
}

impl MultipartField {
    /// Create a new multipart field
    /// 创建新的 multipart 字段
    pub fn new(name: String, filename: Option<String>, content_type: Option<String>) -> Self {
        Self {
            name,
            filename,
            content_type,
        }
    }

    /// Get the field name
    /// 获取字段名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the filename (if this is a file upload)
    /// 获取文件名（如果这是文件上传）
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// Get the content type
    /// 获取内容类型
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Check if this is a file upload
    /// 检查是否是文件上传
    pub fn is_file(&self) -> bool {
        self.filename.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart_file() {
        let file = MultipartFile::new(
            "upload".to_string(),
            Some("test.txt".to_string()),
            Some("text/plain".to_string()),
            Bytes::from("Hello, World!"),
        );

        assert_eq!(file.field_name(), "upload");
        assert_eq!(file.filename(), Some("test.txt"));
        assert_eq!(file.content_type(), Some("text/plain"));
        assert_eq!(file.size(), 13);
        assert!(!file.is_empty());
        assert_eq!(file.extension(), Some("txt"));
    }

    #[test]
    fn test_multipart_file_text() {
        let file = MultipartFile::new(
            "upload".to_string(),
            Some("test.txt".to_string()),
            Some("text/plain".to_string()),
            Bytes::from("Hello, World!"),
        );

        assert_eq!(file.text().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_multipart_field() {
        let field = MultipartField::new("name".to_string(), None, Some("text/plain".to_string()));

        assert_eq!(field.name(), "name");
        assert!(field.filename().is_none());
        assert!(!field.is_file());
    }

    #[test]
    fn test_multipart_field_file() {
        let field = MultipartField::new(
            "file".to_string(),
            Some("upload.jpg".to_string()),
            Some("image/jpeg".to_string()),
        );

        assert_eq!(field.filename(), Some("upload.jpg"));
        assert!(field.is_file());
    }
}
