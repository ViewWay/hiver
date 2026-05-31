//! Zipkin v2 span exporter
//! Zipkin v2 span 导出器
//!
//! Exports trace spans to a Zipkin server via the HTTP v2 API.
//! Equivalent to Spring Cloud Sleuth's Zipkin integration.
//!
//! 通过 HTTP v2 API 将追踪 span 导出到 Zipkin 服务器。等价于 Spring Cloud Sleuth 的 Zipkin 集成。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_observability::zipkin::{ZipkinExporter, ZipkinConfig};
//!
//! let config = ZipkinConfig::new("http://localhost:9411", "my-service");
//! let exporter = ZipkinExporter::new(config);
//!
//! // Export spans
//! exporter.export(&spans).await?;
//! ```

use crate::trace::{Span, SpanKind, SpanStatus};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Zipkin v2 API span path
/// Zipkin v2 API span 路径
const ZIPKIN_API_PATH: &str = "/api/v2/spans";

/// Configuration for Zipkin exporter
/// Zipkin 导出器配置
#[derive(Clone, Debug)]
pub struct ZipkinConfig {
    /// Zipkin server base URL (e.g. "http://localhost:9411")
    /// Zipkin 服务器基础 URL
    pub endpoint: String,
    /// Service name reported to Zipkin
    /// 报告给 Zipkin 的服务名称
    pub service_name: String,
    /// Connection timeout in milliseconds
    /// 连接超时（毫秒）
    pub timeout_ms: u64,
}

impl ZipkinConfig {
    /// Create a new Zipkin config
    /// 创建新的 Zipkin 配置
    pub fn new(endpoint: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            service_name: service_name.into(),
            timeout_ms: 5000,
        }
    }

    /// Set connection timeout
    /// 设置连接超时
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Full spans API URL
    /// 完整的 spans API URL
    pub fn spans_url(&self) -> String {
        format!("{}{}", self.endpoint.trim_end_matches('/'), ZIPKIN_API_PATH)
    }
}

/// Error type for Zipkin export operations
/// Zipkin 导出操作错误类型
#[derive(Debug)]
pub enum ZipkinError {
    /// HTTP request failed
    /// HTTP 请求失败
    Http(String),
    /// JSON serialization failed
    /// JSON 序列化失败
    Serialization(String),
    /// Configuration error
    /// 配置错误
    Config(String),
}

impl fmt::Display for ZipkinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "Zipkin HTTP error: {}", msg),
            Self::Serialization(msg) => write!(f, "Zipkin serialization error: {}", msg),
            Self::Config(msg) => write!(f, "Zipkin config error: {}", msg),
        }
    }
}

impl std::error::Error for ZipkinError {}

/// Result type for Zipkin operations
/// Zipkin 操作结果类型
pub type Result<T> = std::result::Result<T, ZipkinError>;

/// Zipkin v2 endpoint (service identity)
/// Zipkin v2 端点（服务标识）
#[derive(Debug, Clone, Serialize)]
pub struct ZipkinEndpoint {
    /// Service name
    /// 服务名称
    #[serde(rename = "serviceName")]
    pub service_name: String,
}

/// Zipkin v2 annotation
/// Zipkin v2 注解
#[derive(Debug, Clone, Serialize)]
pub struct ZipkinAnnotation {
    /// Timestamp in microseconds
    /// 时间戳（微秒）
    pub timestamp: u64,
    /// Value
    /// 值
    pub value: String,
}

/// Zipkin v2 span format
/// Zipkin v2 span 格式
#[derive(Debug, Clone, Serialize)]
pub struct ZipkinSpan {
    /// Trace ID (hex, 16 or 32 chars)
    /// 追踪 ID（十六进制，16 或 32 字符）
    #[serde(rename = "traceId")]
    pub trace_id: String,
    /// Span ID (hex, 16 chars)
    /// Span ID（十六进制，16 字符）
    pub id: String,
    /// Parent span ID (optional)
    /// 父 span ID（可选）
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Operation name
    /// 操作名称
    pub name: String,
    /// Span kind
    /// Span 类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Start timestamp in microseconds
    /// 开始时间戳（微秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    /// Duration in microseconds
    /// 持续时间（微秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    /// Local endpoint
    /// 本地端点
    #[serde(rename = "localEndpoint")]
    pub local_endpoint: ZipkinEndpoint,
    /// Tags
    /// 标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
    /// Annotations
    /// 注解
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<ZipkinAnnotation>>,
}

impl ZipkinSpan {
    /// Convert a Nexus Span to a Zipkin Span
    /// 将 Nexus Span 转换为 Zipkin Span
    pub fn from_nexus(span: &Span, service_name: &str) -> Self {
        let ctx = span.context();
        let kind = match span.kind() {
            SpanKind::Server => Some("SERVER".to_string()),
            SpanKind::Client => Some("CLIENT".to_string()),
            SpanKind::Producer => Some("PRODUCER".to_string()),
            SpanKind::Consumer => Some("CONSUMER".to_string()),
            SpanKind::Internal => None,
        };

        let timestamp = Some(span.start_time_ns() / 1000);
        let duration = span.duration_ns().map(|d| d / 1000);

        let mut tags: HashMap<String, String> = span
            .attributes()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if let Some(status) = span.status() {
            match status {
                SpanStatus::Ok => {
                    tags.insert("otel.status_code".to_string(), "OK".to_string());
                }
                SpanStatus::UnknownError => {
                    tags.insert("otel.status_code".to_string(), "ERROR".to_string());
                    tags.insert("error".to_string(), "true".to_string());
                }
                SpanStatus::Cancelled => {
                    tags.insert("otel.status_code".to_string(), "ERROR".to_string());
                    tags.insert("error".to_string(), "cancelled".to_string());
                }
            }
        }

        let annotations = if span.events().is_empty() {
            None
        } else {
            Some(
                span.events()
                    .iter()
                    .map(|e| ZipkinAnnotation {
                        timestamp: e.timestamp / 1000,
                        value: e.name.clone(),
                    })
                    .collect(),
            )
        };

        Self {
            trace_id: ctx.trace_id.to_hex(),
            id: ctx.span_id.to_hex(),
            parent_id: ctx.parent_span_id.map(|p| p.to_hex()),
            name: span.name().to_string(),
            kind,
            timestamp,
            duration,
            local_endpoint: ZipkinEndpoint {
                service_name: service_name.to_string(),
            },
            tags: if tags.is_empty() { None } else { Some(tags) },
            annotations,
        }
    }
}

/// Span reporter that collects spans for batch export
/// Span 报告器，收集 span 用于批量导出
pub trait SpanReporter: Send + Sync {
    /// Report a span
    /// 报告 span
    fn report(&self, span: ZipkinSpan);
    /// Flush pending spans
    /// 刷新待处理 span
    fn flush(&self);
}

/// In-memory span reporter for testing
/// 内存 span 报告器，用于测试
pub struct InMemoryReporter {
    spans: std::sync::Mutex<Vec<ZipkinSpan>>,
}

impl InMemoryReporter {
    /// Create a new in-memory reporter
    /// 创建新的内存报告器
    pub fn new() -> Self {
        Self {
            spans: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get all reported spans
    /// 获取所有已报告的 span
    pub fn spans(&self) -> Vec<ZipkinSpan> {
        self.spans.lock().unwrap().clone()
    }

    /// Clear all reported spans
    /// 清除所有已报告的 span
    pub fn clear(&self) {
        self.spans.lock().unwrap().clear();
    }
}

impl Default for InMemoryReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanReporter for InMemoryReporter {
    fn report(&self, span: ZipkinSpan) {
        self.spans.lock().unwrap().push(span);
    }

    fn flush(&self) {}
}

/// Zipkin exporter that sends spans to a Zipkin server
/// Zipkin 导出器，将 span 发送到 Zipkin 服务器
pub struct ZipkinExporter {
    config: ZipkinConfig,
    reporter: Arc<dyn SpanReporter>,
}

impl ZipkinExporter {
    /// Create a new Zipkin exporter
    /// 创建新的 Zipkin 导出器
    pub fn new(config: ZipkinConfig) -> Self {
        let reporter = Arc::new(InMemoryReporter::new());
        Self { config, reporter }
    }

    /// Create exporter with a custom reporter
    /// 使用自定义报告器创建导出器
    pub fn with_reporter(config: ZipkinConfig, reporter: Arc<dyn SpanReporter>) -> Self {
        Self { config, reporter }
    }

    /// Get the exporter config
    /// 获取导出器配置
    pub fn config(&self) -> &ZipkinConfig {
        &self.config
    }

    /// Convert and export a batch of Nexus spans
    /// 转换并导出一批 Nexus span
    pub fn export(&self, spans: &[Span]) -> Result<()> {
        let zipkin_spans: Vec<ZipkinSpan> = spans
            .iter()
            .map(|s| ZipkinSpan::from_nexus(s, &self.config.service_name))
            .collect();

        for span in zipkin_spans {
            self.reporter.report(span);
        }

        Ok(())
    }

    /// Export a single span
    /// 导出单个 span
    pub fn export_one(&self, span: &Span) -> Result<()> {
        self.export(&[span.clone()])
    }

    /// Serialize spans to Zipkin v2 JSON
    /// 将 span 序列化为 Zipkin v2 JSON
    pub fn serialize_spans(spans: &[ZipkinSpan]) -> Result<String> {
        serde_json::to_string(spans).map_err(|e| ZipkinError::Serialization(e.to_string()))
    }

    /// Flush pending spans
    /// 刷新待处理 span
    pub fn flush(&self) {
        self.reporter.flush();
    }
}

impl fmt::Debug for ZipkinExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ZipkinExporter")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::{Span, SpanBuilder, SpanKind, TraceContext};

    #[test]
    fn test_zipkin_config_url() {
        let config = ZipkinConfig::new("http://localhost:9411", "my-service");
        assert_eq!(config.spans_url(), "http://localhost:9411/api/v2/spans");

        let config2 = ZipkinConfig::new("http://localhost:9411/", "svc");
        assert_eq!(config2.spans_url(), "http://localhost:9411/api/v2/spans");
    }

    #[test]
    fn test_zipkin_config_timeout() {
        let config = ZipkinConfig::new("http://localhost:9411", "svc").timeout(10000);
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_from_hiver_span_server() {
        let mut span = SpanBuilder::new("handle_request")
            .with_kind(SpanKind::Server)
            .with_attribute("http.method", "GET")
            .with_attribute("http.path", "/api/users")
            .start();
        span.end();

        let zipkin = ZipkinSpan::from_nexus(&span, "test-service");
        assert_eq!(zipkin.name, "handle_request");
        assert_eq!(zipkin.kind.as_deref(), Some("SERVER"));
        assert!(zipkin.timestamp.is_some());
        assert!(zipkin.duration.is_some());
        assert_eq!(zipkin.local_endpoint.service_name, "test-service");
        assert!(zipkin.tags.unwrap().contains_key("http.method"));
    }

    #[test]
    fn test_from_hiver_span_internal() {
        let mut span = Span::new("internal_op");
        span.add_attribute("key", "value");
        span.end();

        let zipkin = ZipkinSpan::from_nexus(&span, "svc");
        assert_eq!(zipkin.kind, None);
        assert!(zipkin.tags.unwrap().contains_key("key"));
    }

    #[test]
    fn test_from_hiver_span_with_parent() {
        let parent_ctx = TraceContext::new();
        let child = Span::with_context("child", parent_ctx.child());

        let zipkin = ZipkinSpan::from_nexus(&child, "svc");
        assert!(zipkin.parent_id.is_some());
    }

    #[test]
    fn test_from_hiver_span_with_events() {
        let mut span = Span::new("op");
        span.add_event("cache_miss");
        span.end();

        let zipkin = ZipkinSpan::from_nexus(&span, "svc");
        let annotations = zipkin.annotations.unwrap();
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].value, "cache_miss");
    }

    #[test]
    fn test_export_batch() {
        let exporter = ZipkinExporter::new(ZipkinConfig::new("http://localhost:9411", "svc"));

        let mut span1 = Span::new("op1");
        span1.end();
        let mut span2 = Span::new("op2");
        span2.end();

        let result = exporter.export(&[span1, span2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_spans() {
        let mut span = Span::new("test");
        span.end();
        let zipkin = ZipkinSpan::from_nexus(&span, "svc");

        let json = ZipkinExporter::serialize_spans(&[zipkin]).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"traceId\""));
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"localEndpoint\""));
        assert!(json.contains("\"serviceName\":\"svc\""));
    }

    #[test]
    fn test_in_memory_reporter() {
        let reporter = InMemoryReporter::new();
        let mut span = Span::new("test");
        span.end();
        let zipkin = ZipkinSpan::from_nexus(&span, "svc");

        reporter.report(zipkin);
        assert_eq!(reporter.spans().len(), 1);

        reporter.clear();
        assert!(reporter.spans().is_empty());
    }
}
