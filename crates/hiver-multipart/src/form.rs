//! Multipart form handling
//! Multipart 表单处理

use bytes::Bytes;

use crate::{MultipartError, MultipartFile, MultipartResult};

/// Multipart form data
/// Multipart 表单数据
///
/// Equivalent to Spring's `MultipartHttpServletRequest`.
/// `等价于Spring的MultipartHttpServletRequest`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PostMapping("/upload")
/// public String handleUpload(MultipartHttpServletRequest request) {
///     Iterator<MultipartFile> files = request.getFileNames();
///     while (files.hasNext()) {
///         MultipartFile file = files.next();
///         // Process file
///     }
///     return "success";
/// }
/// ```
pub struct Multipart
{
    /// Inner multer multipart
    /// 内部 multer multipart
    inner: multer::Multipart<'static>,
}

impl Multipart
{
    /// Create a new multipart handler from a request body
    /// 从请求体创建新的 multipart 处理器
    ///
    /// # Arguments / 参数
    ///
    /// * `content_type` - The Content-Type header value
    /// * `body` - The request body as bytes
    /// * `max_file_size` - Maximum file size in bytes
    pub fn new(content_type: &str, body: Bytes, max_file_size: usize) -> MultipartResult<Self>
    {
        let boundary = Self::extract_boundary(content_type)?;

        // Create a stream from the body bytes
        let stream = futures::stream::once(async move { Ok::<Bytes, std::io::Error>(body) });

        let multipart = multer::Multipart::with_constraints(
            stream,
            boundary,
            multer::Constraints::new()
                .size_limit(multer::SizeLimit::new().whole_stream(max_file_size as u64)),
        );

        Ok(Self { inner: multipart })
    }

    /// Extract boundary from Content-Type header
    /// 从 Content-Type 头提取 boundary
    fn extract_boundary(content_type: &str) -> MultipartResult<String>
    {
        content_type
            .split("boundary=")
            .nth(1)
            .map(|s| s.trim().to_string())
            .ok_or_else(|| MultipartError::InvalidRequest("Missing boundary".to_string()))
    }

    /// Get the next field in the multipart form
    /// 获取 multipart 表单中的下一个字段
    ///
    /// Returns `Ok(None)` when there are no more fields.
    /// 当没有更多字段时返回 `Ok(None)`。
    pub async fn next_field(&mut self) -> MultipartResult<Option<MultipartField>>
    {
        match self.inner.next_field().await
        {
            Ok(Some(field)) =>
            {
                let name = field.name().unwrap_or("").to_string();
                let filename = field.file_name().map(ToString::to_string);
                let content_type = field.content_type().map(ToString::to_string);

                let data = match field.bytes().await
                {
                    Ok(bytes) => bytes,
                    Err(e) => return Err(MultipartError::from(e)),
                };

                Ok(Some(MultipartField {
                    name,
                    filename,
                    content_type,
                    data,
                }))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(MultipartError::from(e)),
        }
    }

    /// Get a field by name
    /// 按名称获取字段
    ///
    /// This will consume the multipart iterator until the field is found.
    /// 这将消耗 multipart 迭代器直到找到字段。
    pub async fn field(&mut self, name: &str) -> MultipartResult<Option<MultipartField>>
    {
        while let Some(field) = self.next_field().await?
        {
            if field.name() == name
            {
                return Ok(Some(field));
            }
        }
        Ok(None)
    }

    /// Get all fields with the given name
    /// 获取具有给定名称的所有字段
    pub async fn fields(&mut self, name: &str) -> MultipartResult<Vec<MultipartField>>
    {
        let mut fields = Vec::new();
        while let Some(field) = self.next_field().await?
        {
            if field.name() == name
            {
                fields.push(field);
            }
        }
        Ok(fields)
    }

    /// Get all file fields
    /// 获取所有文件字段
    pub async fn files(&mut self) -> MultipartResult<Vec<MultipartFile>>
    {
        let mut files = Vec::new();
        while let Some(field) = self.next_field().await?
        {
            if field.is_file()
            {
                files.push(MultipartFile::new(
                    field.name().to_string(),
                    field.filename().map(ToString::to_string),
                    field.content_type().map(ToString::to_string),
                    field.data,
                ));
            }
        }
        Ok(files)
    }
}

/// A single field in a multipart form
/// Multipart 表单中的单个字段
pub struct MultipartField
{
    /// Field name
    /// 字段名
    name: String,

    /// Filename (if file upload)
    /// 文件名（如果是文件上传）
    filename: Option<String>,

    /// Content type
    /// 内容类型
    content_type: Option<String>,

    /// Field data
    /// 字段数据
    data: Bytes,
}

impl MultipartField
{
    /// Get the field name
    /// 获取字段名
    pub fn name(&self) -> &str
    {
        &self.name
    }

    /// Get the filename (if file upload)
    /// 获取文件名（如果是文件上传）
    pub fn filename(&self) -> Option<&str>
    {
        self.filename.as_deref()
    }

    /// Get the content type
    /// 获取内容类型
    pub fn content_type(&self) -> Option<&str>
    {
        self.content_type.as_deref()
    }

    /// Check if this is a file upload
    /// 检查是否是文件上传
    pub fn is_file(&self) -> bool
    {
        self.filename.is_some()
    }

    /// Get the field data as bytes
    /// 获取字段字节
    pub fn bytes(&self) -> &[u8]
    {
        &self.data
    }

    /// Get the field data as Bytes
    /// 获取字段 Bytes
    pub fn data(&self) -> Bytes
    {
        self.data.clone()
    }

    /// Get the field data as a UTF-8 string
    /// 获取字段数据为 UTF-8 字符串
    pub fn text(&self) -> MultipartResult<String>
    {
        std::str::from_utf8(&self.data)
            .map(ToString::to_string)
            .map_err(|_| MultipartError::Decode("Invalid UTF-8".to_string()))
    }

    /// Get the field data as JSON
    /// 获取字段数据为 JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> MultipartResult<T>
    {
        serde_json::from_slice(&self.data).map_err(|e| MultipartError::Decode(e.to_string()))
    }

    /// Get the size of the field data
    /// 获取字段数据大小
    pub fn size(&self) -> usize
    {
        self.data.len()
    }

    /// Check if the field data is empty
    /// 检查字段数据是否为空
    pub fn is_empty(&self) -> bool
    {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_extract_boundary()
    {
        let ct = "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let boundary = Multipart::extract_boundary(ct).unwrap();
        assert_eq!(boundary, "----WebKitFormBoundary7MA4YWxkTrZu0gW");
    }

    #[test]
    fn test_extract_boundary_missing()
    {
        let ct = "multipart/form-data";
        assert!(Multipart::extract_boundary(ct).is_err());
    }

    #[tokio::test]
    async fn test_multipart_field()
    {
        let field = MultipartField {
            name: "test".to_string(),
            filename: None,
            content_type: Some("text/plain".to_string()),
            data: Bytes::from("hello"),
        };

        assert_eq!(field.name(), "test");
        assert_eq!(field.text().unwrap(), "hello");
        assert_eq!(field.size(), 5);
        assert!(!field.is_empty());
        assert!(!field.is_file());
    }
}
