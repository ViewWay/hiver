//! Multipart error types
//! Multipart 错误类型

use std::fmt;

/// Multipart result type
/// Multipart 结果类型
pub type MultipartResult<T> = Result<T, MultipartError>;

/// Multipart error
/// Multipart 错误
///
/// Equivalent to Spring's `MultipartException`.
/// `等价于Spring的MultipartException`。
#[derive(Debug)]
pub enum MultipartError
{
    /// Invalid multipart request
    /// 无效的 multipart 请求
    InvalidRequest(String),

    /// Field not found
    /// 字段未找到
    FieldNotFound(String),

    /// File too large
    /// 文件过大
    FileTooLarge
    {
        /// File size / 文件大小
        size: usize,
        /// Max allowed size / 最大允许大小
        max: usize,
    },

    /// Invalid file type / MIME type
    /// 无效的文件类型 / MIME 类型
    InvalidType
    {
        /// Found type / 发现的类型
        found: String,
        /// Allowed types / 允许的类型
        allowed: String,
    },

    /// Invalid file extension
    /// 无效的文件扩展名
    InvalidExtension
    {
        /// Found extension / 发现的扩展名
        found: String,
        /// Allowed extensions / 允许的扩展名
        allowed: String,
    },

    /// IO error
    /// IO 错误
    Io(std::io::Error),

    /// Decode error
    /// 解码错误
    Decode(String),

    /// Unknown field
    /// 未知字段
    UnknownField(String),
}

impl fmt::Display for MultipartError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::InvalidRequest(msg) => write!(f, "Invalid multipart request: {}", msg),
            Self::FieldNotFound(name) => write!(f, "Field not found: {}", name),
            Self::FileTooLarge { size, max } =>
            {
                write!(f, "File too large: size={} (max: {})", size, max)
            },
            Self::InvalidType { found, allowed } =>
            {
                write!(f, "Invalid file type: found='{}', allowed=[{}]", found, allowed)
            },
            Self::InvalidExtension { found, allowed } =>
            {
                write!(f, "Invalid file extension: found='{}', allowed=[{}]", found, allowed)
            },
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Decode(msg) => write!(f, "Decode error: {}", msg),
            Self::UnknownField(name) => write!(f, "Unknown field: {}", name),
        }
    }
}

impl std::error::Error for MultipartError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self
        {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<multer::Error> for MultipartError
{
    fn from(err: multer::Error) -> Self
    {
        match err
        {
            multer::Error::UnknownField { field_name } =>
            {
                Self::UnknownField(field_name.unwrap_or_else(|| "unknown".to_string()))
            },
            multer::Error::IncompleteFieldData { .. } =>
            {
                Self::InvalidRequest("Incomplete field data".to_string())
            },
            _ => Self::InvalidRequest(err.to_string()),
        }
    }
}

impl From<std::io::Error> for MultipartError
{
    fn from(err: std::io::Error) -> Self
    {
        Self::Io(err)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_error_display()
    {
        let err = MultipartError::FieldNotFound("file".to_string());
        assert_eq!(err.to_string(), "Field not found: file");

        let err = MultipartError::FileTooLarge {
            size: 20 * 1024 * 1024,
            max: 10 * 1024 * 1024,
        };
        assert!(err.to_string().contains("File too large"));

        let err = MultipartError::InvalidType {
            found: "text/html".to_string(),
            allowed: "image/jpeg, image/png".to_string(),
        };
        assert!(err.to_string().contains("Invalid file type"));

        let err = MultipartError::InvalidExtension {
            found: "exe".to_string(),
            allowed: "jpg, png, gif".to_string(),
        };
        assert!(err.to_string().contains("Invalid file extension"));
    }
}
