//! Content negotiation module / 内容协商模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `MediaType` — MIME type representation
//! - `Accept` header parsing — determine the best response format
//! - `ContentNegotiationManager` — Spring's content negotiation strategy
//!
//! Content negotiation allows a server to serve different representations
//! of the same resource based on the client's `Accept` header.
//! 内容协商允许服务器根据客户端的 `Accept` 头提供同一资源的不同表示形式。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_http::negotiation::ContentNegotiationManager;
//!
//! let manager = ContentNegotiationManager::default();
//! let accept = "application/json, text/html;q=0.9";
//! let best = manager.negotiate(accept);
//! assert_eq!(best, Some("application/json".to_string()));
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt;

/// Represents a media type (MIME type) with optional quality factor.
/// 表示带可选质量因子的媒体类型（MIME类型）。
///
/// Equivalent to Spring's `org.springframework.http.MediaType`.
/// 等价于 Spring 的 `org.springframework.http.MediaType`。
#[derive(Debug, Clone, PartialEq)]
pub struct MediaType
{
    /// Primary type (e.g., "application"). / 主类型。
    pub primary_type: String,
    /// Sub type (e.g., "json"). / 子类型。
    pub sub_type: String,
    /// Quality factor (0.0–1.0, default 1.0). / 质量因子。
    pub quality: f64,
}

impl MediaType
{
    /// Parse a media type string like `"application/json"` or `"text/html;q=0.9"`.
    /// 解析媒体类型字符串，如 `"application/json"` 或 `"text/html;q=0.9"`。
    pub fn parse(input: &str) -> Option<Self>
    {
        let input = input.trim();
        if input.is_empty() || input == "*/*"
        {
            return Some(Self {
                primary_type: "*".to_string(),
                sub_type: "*".to_string(),
                quality: 0.01, // wildcard gets lowest priority
            });
        }

        let (type_part, quality) = if let Some(idx) = input.find(";q=")
        {
            let q: f64 = input[idx + 3..].trim().parse().unwrap_or(1.0);
            (&input[..idx], q)
        }
        else
        {
            (input, 1.0)
        };

        let mut parts = type_part.splitn(2, '/');
        let primary = parts.next()?.to_string();
        let sub = parts.next().unwrap_or("*").to_string();

        Some(Self {
            primary_type: primary,
            sub_type: sub,
            quality,
        })
    }

    /// Returns `true` if this type is a wildcard (matches anything).
    /// 如果是通配符（匹配任何类型），返回 `true`。
    pub fn is_wildcard(&self) -> bool
    {
        self.primary_type == "*"
    }

    /// Returns `true` if the sub type is a wildcard.
    /// 如果子类型是通配符，返回 `true`。
    pub fn is_wildcard_sub(&self) -> bool
    {
        self.sub_type == "*"
    }

    /// Check compatibility with the given content-type.
    /// 检查与给定 content-type 的兼容性。
    pub fn matches(&self, content_type: &str) -> bool
    {
        if self.is_wildcard()
        {
            return true;
        }
        let Some(other) = MediaType::parse(content_type)
        else
        {
            return false;
        };
        self.primary_type == other.primary_type
            && (self.sub_type == other.sub_type || self.is_wildcard_sub())
    }
}

impl fmt::Display for MediaType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}/{}", self.primary_type, self.sub_type)?;
        if (self.quality - 1.0).abs() > f64::EPSILON
        {
            write!(f, ";q={:.3}", self.quality)?;
        }
        Ok(())
    }
}

/// Content negotiation manager.
/// 内容协商管理器。
///
/// Resolves the best media type from a client `Accept` header against
/// a set of supported media types.
/// 从客户端 `Accept` 头解析最佳媒体类型，与支持的媒体类型集合匹配。
///
/// Equivalent to Spring's `ContentNegotiationManager`.
/// 等价于 Spring 的 `ContentNegotiationManager`。
#[derive(Debug, Clone)]
pub struct ContentNegotiationManager
{
    /// Media types the server can produce. / 服务器可生产的媒体类型。
    supported: Vec<MediaType>,
    /// Default type when no match is found. / 未找到匹配时的默认类型。
    default: MediaType,
}

#[allow(clippy::unwrap_used)]
impl Default for ContentNegotiationManager
{
    fn default() -> Self
    {
        Self {
            supported: vec![
                MediaType::parse("application/json").unwrap(),
                MediaType::parse("text/html").unwrap(),
                MediaType::parse("text/plain").unwrap(),
                MediaType::parse("application/xml").unwrap(),
            ],
            default: MediaType::parse("application/json").unwrap(),
        }
    }
}

impl ContentNegotiationManager
{
    /// Create a new manager with custom supported types.
    /// 使用自定义支持的类型创建新管理器。
    #[allow(clippy::expect_used)]
    pub fn new(supported: &[&str]) -> Self
    {
        let types: Vec<MediaType> = supported
            .iter()
            .filter_map(|s| MediaType::parse(s))
            .collect();
        let default = types
            .first()
            .cloned()
            .unwrap_or_else(|| MediaType::parse("application/json").expect("valid media type"));
        Self {
            supported: types,
            default,
        }
    }

    /// Negotiate the best matching media type from an `Accept` header.
    /// 从 `Accept` 头协商最佳匹配的媒体类型。
    ///
    /// Parses the Accept header, sorts by quality, and returns the
    /// best match from the server's supported list.
    ///
    /// 解析 Accept 头，按质量排序，返回服务器支持列表中的最佳匹配。
    pub fn negotiate(&self, accept_header: &str) -> Option<String>
    {
        let mut accepted: Vec<MediaType> = accept_header
            .split(',')
            .filter_map(MediaType::parse)
            .collect();

        if accepted.is_empty()
        {
            return Some(self.default.to_string());
        }

        // Sort by quality descending / 按质量降序排序
        accepted.sort_by(|a, b| {
            b.quality
                .partial_cmp(&a.quality)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Find first match / 查找第一个匹配
        for accept in &accepted
        {
            for supported in &self.supported
            {
                if accept.matches(&supported.to_string())
                {
                    return Some(supported.to_string());
                }
            }
        }

        // No match — return the default / 无匹配 — 返回默认值
        Some(self.default.to_string())
    }

    /// Set the default media type.
    /// 设置默认媒体类型。
    pub fn set_default(&mut self, media_type: &str)
    {
        if let Some(mt) = MediaType::parse(media_type)
        {
            self.default = mt;
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

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

    // ── MediaType tests ─────────────────────────────────────────────────────

    #[test]
    fn test_media_type_parse_simple()
    {
        let mt = MediaType::parse("application/json").unwrap();
        assert_eq!(mt.primary_type, "application");
        assert_eq!(mt.sub_type, "json");
        assert!((mt.quality - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_media_type_parse_with_quality()
    {
        let mt = MediaType::parse("text/html;q=0.9").unwrap();
        assert_eq!(mt.primary_type, "text");
        assert_eq!(mt.sub_type, "html");
        assert!((mt.quality - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_media_type_parse_wildcard()
    {
        let mt = MediaType::parse("*/*").unwrap();
        assert!(mt.is_wildcard());
    }

    #[test]
    fn test_media_type_matches_exact()
    {
        let mt = MediaType::parse("application/json").unwrap();
        assert!(mt.matches("application/json"));
        assert!(!mt.matches("text/html"));
    }

    #[test]
    fn test_media_type_wildcard_matches_all()
    {
        let mt = MediaType::parse("*/*").unwrap();
        assert!(mt.matches("application/json"));
        assert!(mt.matches("text/html"));
        assert!(mt.matches("image/png"));
    }

    #[test]
    fn test_media_type_display()
    {
        let mt = MediaType::parse("application/json").unwrap();
        assert_eq!(mt.to_string(), "application/json");
    }

    // ── ContentNegotiationManager tests ─────────────────────────────────────

    #[test]
    fn test_negotiation_manager_default()
    {
        let manager = ContentNegotiationManager::default();
        let result = manager.negotiate("application/json");
        assert_eq!(result, Some("application/json".to_string()));
    }

    #[test]
    fn test_negotiation_picks_highest_quality()
    {
        let manager = ContentNegotiationManager::default();
        let result = manager.negotiate("text/html;q=0.5, application/json;q=0.9");
        assert_eq!(result, Some("application/json".to_string()));
    }

    #[test]
    fn test_negotiation_empty_header_returns_default()
    {
        let manager = ContentNegotiationManager::default();
        assert_eq!(manager.negotiate(""), Some("application/json".to_string()));
    }

    #[test]
    fn test_negotiation_wildcard_falls_back_to_default()
    {
        let manager = ContentNegotiationManager::default();
        let result = manager.negotiate("*/*");
        assert!(result.is_some());
    }

    #[test]
    fn test_negotiation_unsupported_type()
    {
        let manager = ContentNegotiationManager::default();
        let result = manager.negotiate("image/png");
        assert_eq!(result, Some("application/json".to_string()));
    }

    #[test]
    fn test_custom_supported_types()
    {
        let manager = ContentNegotiationManager::new(&["text/xml", "application/xml"]);
        let result = manager.negotiate("text/xml, application/json;q=0.5");
        assert_eq!(result, Some("text/xml".to_string()));
    }
}
