//! gRPC metadata utilities.
//! gRPC 元数据工具。
//!
//! Equivalent to Spring's gRPC metadata / header propagation.
//! 等价于 Spring 的 gRPC metadata / header 传播。

use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};

/// Helper for building and reading gRPC metadata.
/// 构建和读取 gRPC 元数据的助手。
///
/// # Example / 示例
/// ```rust
/// use hiver_grpc::metadata::MetadataBuilder;
/// let meta = MetadataBuilder::new()
///     .insert("authorization", "Bearer token123")
///     .insert("x-request-id", "abc-123")
///     .build();
/// ```
#[derive(Default)]
pub struct MetadataBuilder {
    entries: Vec<(String, String)>,
}

impl MetadataBuilder {
    /// Create a new builder.
    /// 创建新的构建器。
    pub fn new() -> Self { Self::default() }

    /// Insert a key-value pair.
    /// 插入键值对。
    pub fn insert(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }

    /// Build the `MetadataMap`.
    /// 构建 MetadataMap。
    pub fn build(self) -> MetadataMap {
        let mut map = MetadataMap::new();
        for (k, v) in self.entries {
            if let (Ok(key), Ok(val)) = (
                k.parse::<MetadataKey<tonic::metadata::Ascii>>(),
                v.parse::<MetadataValue<tonic::metadata::Ascii>>(),
            ) {
                map.insert(key, val);
            }
        }
        map
    }
}

/// Extension trait for reading values out of a `MetadataMap`.
/// 从 MetadataMap 读取值的扩展 trait。
pub trait MetadataMapExt {
    /// Get a string value by key.
    /// 按键获取字符串值。
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Get the `authorization` header (Bearer token).
    /// 获取 authorization 头（Bearer token）。
    fn bearer_token(&self) -> Option<&str> {
        self.get_str("authorization").and_then(|v| v.strip_prefix("Bearer "))
    }

    /// Get the `x-request-id` header.
    /// 获取 x-request-id 头。
    fn request_id(&self) -> Option<&str> {
        self.get_str("x-request-id")
    }
}

impl MetadataMapExt for MetadataMap {
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.to_str().ok())
    }
}

use std::time::Duration;

/// Deadline propagation via gRPC metadata.
/// 通过 gRPC 元数据传播 Deadline。
///
/// Equivalent to Spring Cloud's deadline propagation / gRPC deadline handling.
/// 等价于 Spring Cloud 的 deadline 传播 / gRPC deadline 处理。
///
/// Stores the absolute deadline as a Unix timestamp (milliseconds) in the
/// `x-deadline-ms` metadata header, so downstream services know how much
/// time remains.
///
/// 将绝对 deadline 以 Unix 时间戳（毫秒）存储在 `x-deadline-ms` 元数据头中，
/// 以便下游服务知道剩余时间。
pub struct DeadlinePropagator;

/// Metadata key for deadline propagation.
/// deadline 传播的元数据键。
pub const DEADLINE_KEY: &str = "x-deadline-ms";

impl DeadlinePropagator {
    /// Inject a deadline into request metadata.
    /// 将 deadline 注入请求元数据。
    ///
    /// The `remaining` duration is converted to an absolute timestamp.
    /// `remaining` 持续时间被转换为绝对时间戳。
    pub fn inject<T>(request: &mut tonic::Request<T>, remaining: Duration) {
        let deadline_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
            + remaining.as_millis() as u64;
        let value: MetadataValue<tonic::metadata::Ascii> =
            deadline_ms.to_string().parse().unwrap_or_else(|_| MetadataValue::from_static("0"));
        request.metadata_mut().insert(DEADLINE_KEY, value);
    }

    /// Extract the remaining duration from request metadata.
    /// 从请求元数据中提取剩余持续时间。
    ///
    /// Returns `None` if no deadline header is present or parsing fails.
    /// 若无 deadline 头或解析失败则返回 None。
    pub fn extract<T>(request: &tonic::Request<T>) -> Option<Duration> {
        let raw = request.metadata().get_str(DEADLINE_KEY)?;
        let deadline_ms: u64 = raw.parse().ok()?;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        if deadline_ms > now_ms {
            Some(Duration::from_millis(deadline_ms - now_ms))
        } else {
            Some(Duration::ZERO)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let meta = MetadataBuilder::new()
            .insert("x-request-id", "req-001")
            .insert("authorization", "Bearer secret")
            .build();
        assert_eq!(meta.get_str("x-request-id"), Some("req-001"));
        assert_eq!(meta.bearer_token(), Some("secret"));
    }

    #[test]
    fn test_empty() {
        let meta = MetadataBuilder::new().build();
        assert!(meta.is_empty());
    }
}
