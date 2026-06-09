//! Content negotiation based on Accept header.
//! 基于 Accept 头部的内容协商。
//!
//! Equivalent to Spring's `ContentNegotiationManager`.
//! 等价于 Spring 的 `ContentNegotiationManager`。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

/// Supported content types for negotiation.
/// 内容协商支持的媒体类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType
{
    /// application/json
    Json,
    /// application/xml
    Xml,
    /// text/plain
    PlainText,
    /// text/html
    Html,
    /// application/octet-stream
    Binary,
    /// application/protobuf
    Protobuf,
    /// Any type (*/*)
    Any,
}

impl MediaType
{
    /// Get the MIME type string.
    /// 获取 MIME 类型字符串。
    pub fn as_str(&self) -> &'static str
    {
        match self
        {
            Self::Json => "application/json",
            Self::Xml => "application/xml",
            Self::PlainText => "text/plain",
            Self::Html => "text/html",
            Self::Binary => "application/octet-stream",
            Self::Protobuf => "application/protobuf",
            Self::Any => "*/*",
        }
    }

    /// Parse from a MIME type string.
    /// 从 MIME 类型字符串解析。
    pub fn from_mime(mime: &str) -> Option<Self>
    {
        match mime.trim().to_lowercase().as_str()
        {
            "application/json" | "text/json" => Some(Self::Json),
            "application/xml" | "text/xml" => Some(Self::Xml),
            "text/plain" => Some(Self::PlainText),
            "text/html" => Some(Self::Html),
            "application/octet-stream" => Some(Self::Binary),
            "application/protobuf" | "application/x-protobuf" => Some(Self::Protobuf),
            "*/*" | "application/*" | "text/*" => Some(Self::Any),
            _ => None,
        }
    }
}

/// Content negotiator that resolves the best media type.
/// 内容协商器，解析最佳媒体类型。
pub struct ContentNegotiator
{
    /// Ordered list of supported media types (first = highest priority).
    supported_types: Vec<MediaType>,
    /// Default media type when no Accept header is present.
    default_type: MediaType,
}

impl Default for ContentNegotiator
{
    fn default() -> Self
    {
        Self {
            supported_types: vec![MediaType::Json, MediaType::Xml, MediaType::PlainText],
            default_type: MediaType::Json,
        }
    }
}

impl ContentNegotiator
{
    /// Create a new content negotiator with defaults.
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set the supported media types in priority order.
    pub fn supported_types(mut self, types: Vec<MediaType>) -> Self
    {
        self.supported_types = types;
        self
    }

    /// Set the default media type.
    pub fn default_type(mut self, media_type: MediaType) -> Self
    {
        self.default_type = media_type;
        self
    }

    /// Resolve the best media type from an Accept header value.
    /// 从 Accept 头部值解析最佳媒体类型。
    pub fn resolve(&self, accept_header: Option<&str>) -> MediaType
    {
        let Some(header) = accept_header else
        {
            return self.default_type;
        };

        let mut candidates: Vec<(MediaType, f32)> = Vec::new();

        for part in header.split(',')
        {
            let part = part.trim();
            let mut quality = 1.0f32;

            for param in part.split(';').skip(1)
            {
                let param = param.trim();
                if let Some(q_val) = param.strip_prefix("q=")
                {
                    quality = q_val.parse::<f32>().unwrap_or(0.5);
                }
            }

            let mime = part.split(';').next().unwrap_or("").trim();
            if let Some(media_type) = MediaType::from_mime(mime)
            {
                candidates.push((media_type, quality));
            }
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for supported in &self.supported_types
        {
            for (candidate, quality) in &candidates
            {
                if *quality <= 0.0
                {
                    continue;
                }
                if candidate == supported || *candidate == MediaType::Any
                {
                    return *supported;
                }
            }
        }

        self.default_type
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_media_type_from_mime()
    {
        assert_eq!(MediaType::from_mime("application/json"), Some(MediaType::Json));
        assert_eq!(MediaType::from_mime("*/*"), Some(MediaType::Any));
    }

    #[test]
    fn test_resolve_json()
    {
        let cn = ContentNegotiator::new();
        assert_eq!(cn.resolve(Some("application/json")), MediaType::Json);
    }

    #[test]
    fn test_resolve_with_quality()
    {
        let cn = ContentNegotiator::new();
        assert_eq!(cn.resolve(Some("text/html;q=0.9, application/json;q=1.0")), MediaType::Json);
    }

    #[test]
    fn test_resolve_wildcard()
    {
        let cn = ContentNegotiator::new();
        assert_eq!(cn.resolve(Some("*/*")), MediaType::Json);
    }

    #[test]
    fn test_resolve_no_header()
    {
        let cn = ContentNegotiator::new();
        assert_eq!(cn.resolve(None), MediaType::Json);
    }
}
