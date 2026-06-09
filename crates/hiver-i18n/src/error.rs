//! I18n errors
//! 国际化错误

use std::fmt;

/// I18n error
/// 国际化错误
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring throws NoSuchMessageException
/// throw new NoSuchMessageException(code, locale);
/// ```
#[derive(Debug)]
pub enum I18nError {
    /// Message not found for code
    /// 未找到对应代码的消息
    MessageNotFound {
        /// Message code
        /// 消息代码
        code: String,
        /// Locale
        /// 语言环境
        locale: String,
    },

    /// Locale not found or invalid
    /// 语言环境未找到或无效
    InvalidLocale(String),

    /// Resource bundle not found
    /// 资源包未找到
    ResourceBundleNotFound {
        /// Basename
        /// 基础名称
        basename: String,
        /// Locale
        /// 语言环境
        locale: String,
    },

    /// IO error reading resources
    /// 读取资源的IO错误
    IoError(String),

    /// Parse error in properties file
    /// 属性文件解析错误
    ParseError {
        /// File path
        /// 文件路径
        file: String,
        /// Error message
        /// 错误消息
        message: String,
    },

    /// Encoding error
    /// 编码错误
    EncodingError(String),

    /// Other error
    /// 其他错误
    Other(String),
}

impl fmt::Display for I18nError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            I18nError::MessageNotFound { code, locale } => {
                write!(f, "Message not found: code='{}', locale='{}'", code, locale)
            },
            I18nError::InvalidLocale(msg) => write!(f, "Invalid locale: {}", msg),
            I18nError::ResourceBundleNotFound { basename, locale } => {
                write!(f, "Resource bundle not found: basename='{}', locale='{}'", basename, locale)
            },
            I18nError::IoError(msg) => write!(f, "IO error: {}", msg),
            I18nError::ParseError { file, message } => {
                write!(f, "Parse error in file '{}': {}", file, message)
            },
            I18nError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            I18nError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for I18nError {}

impl From<std::io::Error> for I18nError {
    fn from(err: std::io::Error) -> Self {
        I18nError::IoError(err.to_string())
    }
}

/// I18n result type
/// 国际化结果类型
pub type I18nResult<T> = Result<T, I18nError>;

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = I18nError::MessageNotFound {
            code: "test.code".to_string(),
            locale: "en_US".to_string(),
        };
        assert_eq!(err.to_string(), "Message not found: code='test.code', locale='en_US'");
    }

    #[test]
    fn test_i18n_result() {
        let result: I18nResult<()> = Ok(());
        assert!(result.is_ok());

        let result: I18nResult<()> = Err(I18nError::InvalidLocale("bad".to_string()));
        assert!(result.is_err());
    }
}
