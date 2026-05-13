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
/// use nexus_grpc::metadata::MetadataBuilder;
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
