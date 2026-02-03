//! Compression middleware module
//! 压缩中间件模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - server.compression.enabled
//! - GzipFilter, DeflateFilter
//! - Content-Encoding
//!
//! # Features / 特性
//!
//! - Gzip compression (default)
//! - Deflate compression
//! - Brotli compression (optional)
//! - Configurable minimum response size
//! - MIME type filtering
//! - User agent exclusion
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_router::Router;
//! use nexus_middleware::CompressionMiddleware;
//! use std::sync::Arc;
//!
//! let compression = Arc::new(CompressionMiddleware::new());
//! let router = Router::new()
//!     .middleware(compression)
//!     .get("/", handler);
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use nexus_http::{Body, Response, Result};
use nexus_router::{Middleware, Next};

/// Compression type
/// 压缩类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// Gzip compression
    /// Gzip压缩
    Gzip,

    /// Deflate compression
    /// Deflate压缩
    Deflate,

    /// Brotli compression
    /// Brotli压缩
    Brotli,

    /// No compression
    /// 不压缩
    None,
}

impl CompressionType {
    /// Get the content encoding value
    /// 获取内容编码值
    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionType::Gzip => "gzip",
            CompressionType::Deflate => "deflate",
            CompressionType::Brotli => "br",
            CompressionType::None => "identity",
        }
    }

    /// Parse from Accept-Encoding header value
    /// 从 Accept-Encoding 头部值解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "gzip" => Some(CompressionType::Gzip),
            "deflate" => Some(CompressionType::Deflate),
            "br" => Some(CompressionType::Brotli),
            "identity" | "" => Some(CompressionType::None),
            _ => None,
        }
    }

    /// Parse quality value from Accept-Encoding (e.g., "gzip; q=0.8")
    /// 从 Accept-Encoding 解析质量值（例如 "gzip; q=0.8"）
    pub fn parse_with_quality(s: &str) -> (Option<Self>, f32) {
        let parts: Vec<&str> = s.split(';').collect();
        let compression = Self::from_str(parts.first().unwrap_or(&""));

        let quality = if parts.len() > 1 {
            parts[1]
                .trim()
                .strip_prefix("q=")
                .and_then(|q| q.parse::<f32>().ok())
                .unwrap_or(1.0)
        } else {
            1.0
        };

        (compression, quality)
    }
}

/// Compression middleware
/// 压缩中间件
///
/// Equivalent to Spring Boot's:
/// - `server.compression.enabled=true`
/// - `GzipFilter`, `DeflateFilter`
/// - `ContentEncodingFilter`
///
/// 这等价于Spring Boot的：
/// - `server.compression.enabled=true`
/// - `GzipFilter`, `DeflateFilter`
/// - `ContentEncodingFilter`
#[derive(Clone)]
pub struct CompressionMiddleware {
    /// Minimum response size to compress (bytes)
    /// 压缩的最小响应大小（字节）
    pub min_response_size: usize,

    /// Compression types to offer
    /// 提供的压缩类型
    pub compression_types: Vec<CompressionType>,

    /// MIME types to compress
    /// 要压缩的MIME类型
    pub mime_types: Vec<String>,

    /// Excluded user agents
    /// 排除的用户代理
    pub excluded_agents: Vec<String>,

    /// Compression level (0-9 for gzip/deflate)
    /// 压缩级别（gzip/deflate 为 0-9）
    pub compression_level: u8,
}

impl CompressionMiddleware {
    /// Create a new compression middleware
    /// 创建新的压缩中间件
    pub fn new() -> Self {
        Self {
            min_response_size: 1024,
            compression_types: vec![CompressionType::Gzip, CompressionType::Deflate],
            mime_types: vec![
                "text/html".to_string(),
                "text/plain".to_string(),
                "text/css".to_string(),
                "text/javascript".to_string(),
                "application/javascript".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
                "text/xml".to_string(),
            ],
            excluded_agents: Vec::new(),
            compression_level: 6, // Default compression level / 默认压缩级别
        }
    }

    /// Set minimum response size
    /// 设置最小响应大小
    pub fn min_size(mut self, size: usize) -> Self {
        self.min_response_size = size;
        self
    }

    /// Add a compression type
    /// 添加压缩类型
    pub fn add_compression(mut self, compression: CompressionType) -> Self {
        self.compression_types.push(compression);
        self
    }

    /// Add MIME types to compress
    /// 添加要压缩的MIME类型
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_types.push(mime.into());
        self
    }

    /// Add excluded user agent
    /// 添加排除的用户代理
    pub fn exclude_agent(mut self, agent: impl Into<String>) -> Self {
        self.excluded_agents.push(agent.into());
        self
    }

    /// Set compression level (0-9)
    /// 设置压缩级别（0-9）
    pub fn level(mut self, level: u8) -> Self {
        self.compression_level = level.min(9);
        self
    }

    /// Select the best compression type based on Accept-Encoding header
    /// 根据 Accept-Encoding 头部选择最佳压缩类型
    fn select_compression(&self, accept_encoding: &str) -> Option<CompressionType> {
        if accept_encoding.is_empty() {
            return None;
        }

        let mut best_type: Option<CompressionType> = None;
        let mut best_quality = 0.0;

        for encoding in accept_encoding.split(',') {
            let (compression, quality) = CompressionType::parse_with_quality(encoding);

            if let Some(comp) = compression {
                // Check if we support this compression type
                // Note: CompressionType::None is not in compression_types, but may be selected
                // 检查我们是否支持这种压缩类型
                // 注意：CompressionType::None 不在 compression_types 中，但可能会被选择
                let is_supported = comp == CompressionType::None || self.compression_types.contains(&comp);

                if is_supported && quality > best_quality {
                    best_type = Some(comp);
                    best_quality = quality;
                }
            }
        }

        best_type
    }

    /// Check if response should be compressed based on MIME type
    /// 根据 MIME 类型检查是否应该压缩响应
    fn should_compress_mime(&self, content_type: Option<&str>) -> bool {
        if let Some(ct) = content_type {
            // Extract MIME type without charset (e.g., "text/html; charset=utf-8")
            // 提取不带字符集的 MIME 类型（例如 "text/html; charset=utf-8"）
            let mime = ct.split(';').next().unwrap_or(ct).trim();
            self.mime_types.iter().any(|m| m == mime)
        } else {
            true // No content type header, attempt compression / 没有内容类型头部，尝试压缩
        }
    }

    /// Check if user agent is excluded
    /// 检查用户代理是否被排除
    fn is_agent_excluded(&self, user_agent: Option<&str>) -> bool {
        if let Some(ua) = user_agent {
            self.excluded_agents.iter().any(|excluded| {
                ua.contains(excluded.as_str())
            })
        } else {
            false
        }
    }

    /// Check if response body size meets minimum size requirement
    /// 检查响应体大小是否满足最小大小要求
    fn meets_min_size(&self, body: &Body) -> bool {
        body.data().len() >= self.min_response_size
    }

    /// Compress response body using the specified compression type
    /// 使用指定的压缩类型压缩响应体
    #[cfg(feature = "compression")]
    fn compress_body(
        &self,
        body: Body,
        compression: CompressionType,
    ) -> Result<Body> {
        let body_bytes = body.data();

        // Check if body is large enough to compress
        // 检查正文是否足够大以进行压缩
        if body_bytes.len() < self.min_response_size {
            return Ok(Body::from_bytes(body_bytes.clone()));
        }

        // Compress based on type / 根据类型压缩
        let compressed = match compression {
            #[cfg(all(feature = "compression", feature = "gzip"))]
            CompressionType::Gzip => {
                self.compress_gzip(body_bytes)?
            }
            #[cfg(all(feature = "compression", not(feature = "gzip")))]
            CompressionType::Gzip => {
                tracing::warn!("Gzip compression requested but feature not enabled");
                body_bytes.clone()
            }
            #[cfg(all(feature = "compression", feature = "deflate"))]
            CompressionType::Deflate => {
                self.compress_deflate(body_bytes)?
            }
            #[cfg(all(feature = "compression", not(feature = "deflate")))]
            CompressionType::Deflate => {
                tracing::warn!("Deflate compression requested but feature not enabled");
                body_bytes.clone()
            }
            #[cfg(all(feature = "compression", feature = "brotli"))]
            CompressionType::Brotli => {
                self.compress_brotli(body_bytes)?
            }
            #[cfg(all(feature = "compression", not(feature = "brotli")))]
            CompressionType::Brotli => {
                tracing::warn!("Brotli compression requested but feature not enabled");
                body_bytes.clone()
            }
            CompressionType::None => body_bytes.clone(),
        };

        Ok(Body::from_bytes(compressed))
    }

    /// Compress data using gzip
    /// 使用 gzip 压缩数据
    #[cfg(all(feature = "compression", feature = "gzip"))]
    fn compress_gzip(&self, data: &Bytes) -> Result<Bytes> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.compression_level.into()));
        encoder
            .write_all(data.as_ref())
            .map_err(|e| nexus_http::Error::internal(format!("Gzip compression failed: {}", e)))?;
        let compressed = encoder
            .finish()
            .map_err(|e| nexus_http::Error::internal(format!("Gzip finalization failed: {}", e)))?;

        Ok(Bytes::from(compressed))
    }

    /// Compress data using deflate
    /// 使用 deflate 压缩数据
    #[cfg(all(feature = "compression", feature = "deflate"))]
    fn compress_deflate(&self, data: &Bytes) -> Result<Bytes> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.compression_level.into()));
        encoder
            .write_all(data.as_ref())
            .map_err(|e| nexus_http::Error::internal(format!("Deflate compression failed: {}", e)))?;
        let compressed = encoder
            .finish()
            .map_err(|e| nexus_http::Error::internal(format!("Deflate finalization failed: {}", e)))?;

        Ok(Bytes::from(compressed))
    }

    /// Compress data using brotli
    /// 使用 brotli 压缩数据
    #[cfg(all(feature = "compression", feature = "brotli"))]
    fn compress_brotli(&self, data: &Bytes) -> Result<Bytes> {
        use brotli::CompressorWriter;
        use std::io::Write;

        let mut compressed = Vec::new();
        {
            // CompressorWriter::new takes: writer, buffer_size, quality, lgwin
            // CompressorWriter::new 接受: writer, buffer_size, quality, lgwin
            let mut writer = CompressorWriter::new(
                &mut compressed,
                4096,  // buffer size / 缓冲区大小
                self.compression_level as u32,  // quality / 质量
                22,  // lgwin (log window size) / lgwin（窗口大小对数）
            );
            writer
                .write_all(data.as_ref())
                .map_err(|e| nexus_http::Error::internal(format!("Brotli compression failed: {}", e)))?;
        }
        Ok(Bytes::from(compressed))
    }

    /// No-op compression when feature is disabled
    /// 功能禁用时不压缩
    #[cfg(not(feature = "compression"))]
    fn compress_body(&self, body: Body, _compression: CompressionType) -> Result<Body> {
        Ok(body)
    }
}

impl Default for CompressionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Middleware<S> for CompressionMiddleware
where
    S: Send + Sync + 'static,
{
    fn call(
        &self,
        req: nexus_http::Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        let min_size = self.min_response_size;
        let compression_types = self.compression_types.clone();
        let mime_types = self.mime_types.clone();
        let excluded_agents = self.excluded_agents.clone();
        let compression_level = self.compression_level;
        let is_feature_enabled = cfg!(feature = "compression");

        Box::pin(async move {
            // Get Accept-Encoding header / 获取 Accept-Encoding 头部
            let accept_encoding = req.header("Accept-Encoding").unwrap_or("").to_string();

            // Check if user agent is excluded (e.g., old browsers)
            // 检查用户代理是否被排除（例如旧浏览器）
            let middleware_check = CompressionMiddleware {
                min_response_size: min_size,
                compression_types: vec![],
                mime_types: mime_types.clone(),
                excluded_agents: excluded_agents.clone(),
                compression_level,
            };
            let user_agent = req.header("User-Agent");
            if middleware_check.is_agent_excluded(user_agent) {
                tracing::debug!("User agent excluded from compression");
                return next.call(req, state).await;
            }

            // Select compression type based on Accept-Encoding
            // 根据 Accept-Encoding 选择压缩类型
            let compression_type = if !compression_types.is_empty() {
                Self {
                    min_response_size: min_size,
                    compression_types,
                    mime_types: mime_types.clone(),
                    excluded_agents: excluded_agents.clone(),
                    compression_level,
                }
                .select_compression(&accept_encoding)
            } else {
                None
            };

            if compression_type.is_none() {
                return next.call(req, state).await;
            }

            let compression = compression_type.unwrap();

            // If client explicitly requested identity (no compression), skip
            // 如果客户端明确请求 identity（不压缩），则跳过
            if compression == CompressionType::None {
                return next.call(req, state).await;
            }

            // Call next middleware/handler
            // 调用下一个中间件/处理程序
            let mut response = next.call(req, state).await?;

            // Check if response should be compressed
            // 检查响应是否应该被压缩
            if !is_feature_enabled {
                return Ok(response);
            }

            // Don't compress if already has Content-Encoding
            // 如果已经有 Content-Encoding 则不压缩
            if response.header("Content-Encoding").is_some() {
                tracing::debug!("Response already has Content-Encoding, skipping compression");
                return Ok(response);
            }

            // Check content type
            // 检查内容类型
            let content_type = response.header("Content-Type").map(|s| s.to_string());
            let should_compress = middleware_check.should_compress_mime(content_type.as_deref());

            if !should_compress {
                tracing::debug!("Content type not in compressible list: {:?}", content_type);
                return Ok(response);
            }

            // Check response size / 检查响应大小
            let body = response.body();
            if !middleware_check.meets_min_size(body) {
                tracing::debug!("Response too small to compress (min: {} bytes)", min_size);
                return Ok(response);
            }

            // Compress the body / 压缩正文
            let compression_middleware = CompressionMiddleware {
                min_response_size: min_size,
                compression_types: vec![compression],
                mime_types,
                excluded_agents,
                compression_level,
            };

            match compression_middleware.compress_body(response.take_body(), compression) {
                Ok(compressed_body) => {
                    // Add Content-Encoding header
                    // 添加 Content-Encoding 头部
                    response.set_body(compressed_body);
                    response.insert_header("Content-Encoding", compression.as_str());

                    // Remove Content-Length header (body size changed)
                    // 移除 Content-Length 头部（正文大小已更改）
                    response.remove_header("Content-Length");

                    tracing::debug!(
                        "Response compressed with {} (original size: >= {} bytes)",
                        compression.as_str(),
                        min_size
                    );

                    Ok(response)
                }
                Err(e) => {
                    tracing::warn!("Compression failed, returning uncompressed: {}", e);
                    Ok(response)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_creation() {
        let compression = CompressionMiddleware::new();
        assert_eq!(compression.min_response_size, 1024);
        assert_eq!(compression.compression_types.len(), 2);
        assert_eq!(compression.compression_level, 6);
    }

    #[test]
    fn test_compression_builder() {
        let compression = CompressionMiddleware::new()
            .min_size(2048)
            .mime_type("application/xml")
            .level(9);

        assert_eq!(compression.min_response_size, 2048);
        assert!(compression.mime_types.contains(&"application/xml".to_string()));
        assert_eq!(compression.compression_level, 9);
    }

    #[test]
    fn test_compression_type_from_str() {
        assert_eq!(CompressionType::from_str("gzip"), Some(CompressionType::Gzip));
        assert_eq!(CompressionType::from_str("deflate"), Some(CompressionType::Deflate));
        assert_eq!(CompressionType::from_str("br"), Some(CompressionType::Brotli));
        assert_eq!(CompressionType::from_str("identity"), Some(CompressionType::None));
        assert_eq!(CompressionType::from_str("unknown"), None);
    }

    #[test]
    fn test_compression_type_as_str() {
        assert_eq!(CompressionType::Gzip.as_str(), "gzip");
        assert_eq!(CompressionType::Deflate.as_str(), "deflate");
        assert_eq!(CompressionType::Brotli.as_str(), "br");
        assert_eq!(CompressionType::None.as_str(), "identity");
    }

    #[test]
    fn test_parse_with_quality() {
        let (comp, quality) = CompressionType::parse_with_quality("gzip");
        assert_eq!(comp, Some(CompressionType::Gzip));
        assert_eq!(quality, 1.0);

        let (comp, quality) = CompressionType::parse_with_quality("gzip; q=0.5");
        assert_eq!(comp, Some(CompressionType::Gzip));
        assert_eq!(quality, 0.5);

        let (comp, quality) = CompressionType::parse_with_quality("unknown; q=0.8");
        assert_eq!(comp, None);
        assert_eq!(quality, 0.8);
    }

    #[test]
    fn test_select_compression() {
        let middleware = CompressionMiddleware::new();

        // Basic gzip
        assert_eq!(
            middleware.select_compression("gzip"),
            Some(CompressionType::Gzip)
        );

        // Multiple encodings
        assert_eq!(
            middleware.select_compression("gzip, deflate, br"),
            Some(CompressionType::Gzip) // First match
        );

        // Quality preference
        assert_eq!(
            middleware.select_compression("deflate; q=0.8, gzip; q=0.5"),
            Some(CompressionType::Deflate) // Higher quality
        );

        // Identity (no compression)
        assert_eq!(
            middleware.select_compression("identity"),
            Some(CompressionType::None)
        );

        // Empty header
        assert_eq!(middleware.select_compression(""), None);
    }

    #[test]
    fn test_should_compress_mime() {
        let middleware = CompressionMiddleware::new();

        assert!(middleware.should_compress_mime(Some("text/html")));
        assert!(middleware.should_compress_mime(Some("application/json")));
        assert!(middleware.should_compress_mime(Some("text/html; charset=utf-8")));
        assert!(!middleware.should_compress_mime(Some("image/png")));
        assert!(!middleware.should_compress_mime(Some("video/mp4")));

        // No content type - should attempt compression
        assert!(middleware.should_compress_mime(None));
    }

    #[test]
    fn test_is_agent_excluded() {
        let middleware = CompressionMiddleware::new()
            .exclude_agent("MSIE 6.0")
            .exclude_agent("old-browser");

        assert!(middleware.is_agent_excluded(Some("Mozilla/4.0 (MSIE 6.0)")));
        assert!(middleware.is_agent_excluded(Some("old-browser/1.0")));
        assert!(!middleware.is_agent_excluded(Some("Mozilla/5.0")));
        assert!(!middleware.is_agent_excluded(None));
    }
}
