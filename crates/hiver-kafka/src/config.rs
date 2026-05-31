//! Kafka configuration
//! Kafka配置

use serde::{Deserialize, Serialize};

/// Producer configuration
/// 生产者配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public ProducerFactory<String, String> producerFactory() {
///     Map<String, Object> props = new HashMap<>();
///     props.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");
///     props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
///     props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
///     return new DefaultKafkaProducerFactory<>(props);
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProducerConfig {
    /// Bootstrap servers
    /// 引导服务器
    pub bootstrap_servers: String,

    /// Client ID
    /// 客户端ID
    #[serde(default = "default_client_id")]
    pub client_id: String,

    /// Acknowledgment level (0, 1, or all)
    /// 确认级别（0、1或all）
    #[serde(default = "default_acks")]
    pub acks: String,

    /// Enable idempotence
    /// 启用幂等性
    #[serde(default = "default_idempotent")]
    pub idempotent: bool,

    /// Compression type (none, gzip, snappy, lz4, zstd)
    /// 压缩类型
    #[serde(default)]
    pub compression_type: CompressionType,

    /// Linger time (milliseconds)
    /// 延迟时间（毫秒）
    #[serde(default = "default_linger")]
    pub linger_ms: u32,

    /// Batch size
    /// 批次大小
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,

    /// Request timeout (milliseconds)
    /// 请求超时时间（毫秒）
    #[serde(default = "default_request_timeout")]
    pub request_timeout_ms: u32,

    /// Max in-flight requests
    /// 最大飞行中请求数
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight_requests_per_connection: i32,

    /// Enable SSL
    /// 启用SSL
    #[serde(default)]
    pub ssl: bool,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            client_id: default_client_id(),
            acks: default_acks(),
            idempotent: default_idempotent(),
            compression_type: CompressionType::default(),
            linger_ms: default_linger(),
            batch_size: default_batch_size(),
            request_timeout_ms: default_request_timeout(),
            max_in_flight_requests_per_connection: default_max_in_flight(),
            ssl: false,
        }
    }
}

impl ProducerConfig {
    /// Create new producer config
    /// 创建新的生产者配置
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bootstrap servers
    /// 设置引导服务器
    pub fn with_bootstrap_servers(mut self, servers: impl Into<String>) -> Self {
        self.bootstrap_servers = servers.into();
        self
    }

    /// Set client ID
    /// 设置客户端ID
    pub fn with_client_id(mut self, id: impl Into<String>) -> Self {
        self.client_id = id.into();
        self
    }

    /// Set acks
    /// 设置确认级别
    pub fn with_acks(mut self, acks: impl Into<String>) -> Self {
        self.acks = acks.into();
        self
    }

    /// Set compression type
    /// 设置压缩类型
    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression_type = compression;
        self
    }
}

/// Compression type
/// 压缩类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CompressionType {
    #[default]
    None,
    Gzip,
    Snappy,
    Lz4,
    Zstd,
}

/// Consumer configuration
/// 消费者配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public ConsumerFactory<String, String> consumerFactory() {
///     Map<String, Object> props = new HashMap<>();
///     props.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, "localhost:9092");
///     props.put(ConsumerConfig.GROUP_ID_CONFIG, "my-group");
///     props.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class);
///     props.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class);
///     return new DefaultKafkaConsumerFactory<>(props);
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsumerConfig {
    /// Bootstrap servers
    /// 引导服务器
    pub bootstrap_servers: String,

    /// Group ID
    /// 组ID
    #[serde(default = "default_group_id")]
    pub group_id: String,

    /// Client ID
    /// 客户端ID
    #[serde(default = "default_client_id")]
    pub client_id: String,

    /// Enable auto commit
    /// 启用自动提交
    #[serde(default = "default_auto_commit")]
    pub enable_auto_commit: bool,

    /// Auto commit interval (milliseconds)
    /// 自动提交间隔（毫秒）
    #[serde(default = "default_auto_commit_interval")]
    pub auto_commit_interval_ms: u32,

    /// Session timeout (milliseconds)
    /// 会话超时（毫秒）
    #[serde(default = "default_session_timeout")]
    pub session_timeout_ms: u32,

    /// Max poll records
    /// 最大轮询记录数
    #[serde(default = "default_max_poll_records")]
    pub max_poll_records: i32,

    /// Max poll interval (milliseconds)
    /// 最大轮询间隔（毫秒）
    #[serde(default = "default_max_poll_interval")]
    pub max_poll_interval_ms: u32,

    /// Auto offset reset
    /// 自动偏移重置
    #[serde(default = "default_auto_offset_reset")]
    pub auto_offset_reset: AutoOffsetReset,

    /// Fetch min bytes
    /// 最小拉取字节数
    #[serde(default = "default_fetch_min_bytes")]
    pub fetch_min_bytes: i32,

    /// Fetch max bytes
    /// 最大拉取字节数
    #[serde(default = "default_fetch_max_bytes")]
    pub fetch_max_bytes: i32,

    /// Fetch max wait (milliseconds)
    /// 最大拉取等待时间（毫秒）
    #[serde(default = "default_fetch_max_wait")]
    pub fetch_max_wait_ms: u32,

    /// Enable SSL
    /// 启用SSL
    #[serde(default)]
    pub ssl: bool,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            group_id: default_group_id(),
            client_id: default_client_id(),
            enable_auto_commit: default_auto_commit(),
            auto_commit_interval_ms: default_auto_commit_interval(),
            session_timeout_ms: default_session_timeout(),
            max_poll_records: default_max_poll_records(),
            max_poll_interval_ms: default_max_poll_interval(),
            auto_offset_reset: AutoOffsetReset::default(),
            fetch_min_bytes: default_fetch_min_bytes(),
            fetch_max_bytes: default_fetch_max_bytes(),
            fetch_max_wait_ms: default_fetch_max_wait(),
            ssl: false,
        }
    }
}

impl ConsumerConfig {
    /// Create new consumer config
    /// 创建新的消费者配置
    pub fn new(group_id: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            ..Self::default()
        }
    }

    /// Set bootstrap servers
    /// 设置引导服务器
    pub fn with_bootstrap_servers(mut self, servers: impl Into<String>) -> Self {
        self.bootstrap_servers = servers.into();
        self
    }
}

/// Auto offset reset policy
/// 自动偏移重置策略
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AutoOffsetReset {
    Earliest,
    #[default]
    Latest,
    None,
}

/// Consumer offset
/// 消费者偏移
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsumerOffset {
    /// Topic
    /// 主题
    pub topic: String,

    /// Partition
    /// 分区
    pub partition: i32,

    /// Offset
    /// 偏移
    pub offset: i64,
}

impl ConsumerOffset {
    /// Create new consumer offset
    /// 创建新的消费者偏移
    pub fn new(topic: impl Into<String>, partition: i32, offset: i64) -> Self {
        Self {
            topic: topic.into(),
            partition,
            offset,
        }
    }
}

fn default_client_id() -> String {
    "hiver-kafka".to_string()
}

fn default_acks() -> String {
    "all".to_string()
}

fn default_idempotent() -> bool {
    true
}

fn default_linger() -> u32 {
    0
}

fn default_batch_size() -> u32 {
    16384
}

fn default_request_timeout() -> u32 {
    30000
}

fn default_max_in_flight() -> i32 {
    5
}

fn default_group_id() -> String {
    "hiver-consumer-group".to_string()
}

fn default_auto_commit() -> bool {
    true
}

fn default_auto_commit_interval() -> u32 {
    5000
}

fn default_session_timeout() -> u32 {
    30000
}

fn default_max_poll_records() -> i32 {
    500
}

fn default_max_poll_interval() -> u32 {
    300_000
}

fn default_auto_offset_reset() -> AutoOffsetReset {
    AutoOffsetReset::Latest
}

fn default_fetch_min_bytes() -> i32 {
    1
}

fn default_fetch_max_bytes() -> i32 {
    52_428_800
}

fn default_fetch_max_wait() -> u32 {
    500
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ProducerConfig tests ──────────────────────────────────────────

    /// Test default producer config values
    /// 测试默认生产者配置值
    #[test]
    fn test_producer_config_default_values() {
        let config = ProducerConfig::default();
        assert_eq!(config.bootstrap_servers, "localhost:9092");
        assert_eq!(config.client_id, "hiver-kafka");
        assert_eq!(config.acks, "all");
        assert!(config.idempotent);
        assert_eq!(config.compression_type, CompressionType::None);
        assert_eq!(config.linger_ms, 0);
        assert_eq!(config.batch_size, 16384);
        assert_eq!(config.request_timeout_ms, 30000);
        assert_eq!(config.max_in_flight_requests_per_connection, 5);
        assert!(!config.ssl);
    }

    /// Test producer config builder methods
    /// 测试生产者配置构建器方法
    #[test]
    fn test_producer_config_builder() {
        let config = ProducerConfig::new()
            .with_bootstrap_servers("kafka://broker1:9092,broker2:9092")
            .with_client_id("test-client")
            .with_acks("1")
            .with_compression(CompressionType::Gzip);

        assert_eq!(config.bootstrap_servers, "kafka://broker1:9092,broker2:9092");
        assert_eq!(config.client_id, "test-client");
        assert_eq!(config.acks, "1");
        assert_eq!(config.compression_type, CompressionType::Gzip);
    }

    /// Test producer config serialization round-trip
    /// 测试生产者配置序列化往返
    #[test]
    fn test_producer_config_serde_roundtrip() {
        let config = ProducerConfig::new()
            .with_bootstrap_servers("localhost:9092")
            .with_compression(CompressionType::Lz4);
        let json = serde_json::to_string(&config).expect("serialize failed");
        let deserialized: ProducerConfig = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(config.bootstrap_servers, deserialized.bootstrap_servers);
        assert_eq!(config.compression_type, deserialized.compression_type);
        assert_eq!(config.acks, deserialized.acks);
    }

    // ── ConsumerConfig tests ──────────────────────────────────────────

    /// Test consumer config with custom group id
    /// 测试带自定义组ID的消费者配置
    #[test]
    fn test_consumer_config_custom_group() {
        let config = ConsumerConfig::new("my-group");
        assert_eq!(config.group_id, "my-group");
        assert_eq!(config.bootstrap_servers, "localhost:9092");
        assert!(config.enable_auto_commit);
        assert_eq!(config.auto_commit_interval_ms, 5000);
        assert_eq!(config.session_timeout_ms, 30000);
        assert_eq!(config.max_poll_records, 500);
        assert_eq!(config.max_poll_interval_ms, 300_000);
        assert_eq!(config.auto_offset_reset, AutoOffsetReset::Latest);
        assert!(!config.ssl);
    }

    /// Test consumer config builder methods
    /// 测试消费者配置构建器方法
    #[test]
    fn test_consumer_config_builder() {
        let config = ConsumerConfig::new("test-group")
            .with_bootstrap_servers("broker.example.com:9093");
        assert_eq!(config.group_id, "test-group");
        assert_eq!(config.bootstrap_servers, "broker.example.com:9093");
    }

    /// Test consumer config serialization round-trip
    /// 测试消费者配置序列化往返
    #[test]
    fn test_consumer_config_serde_roundtrip() {
        let config = ConsumerConfig::new("serde-group");
        let json = serde_json::to_string(&config).expect("serialize failed");
        let deserialized: ConsumerConfig = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(config.group_id, deserialized.group_id);
        assert_eq!(config.bootstrap_servers, deserialized.bootstrap_servers);
        assert_eq!(config.enable_auto_commit, deserialized.enable_auto_commit);
    }

    // ── CompressionType tests ─────────────────────────────────────────

    /// Test all compression type variants
    /// 测试所有压缩类型变体
    #[test]
    fn test_compression_type_variants() {
        assert_eq!(CompressionType::default(), CompressionType::None);

        let variants = vec![
            CompressionType::None,
            CompressionType::Gzip,
            CompressionType::Snappy,
            CompressionType::Lz4,
            CompressionType::Zstd,
        ];
        // Verify all are distinct
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }

    // ── AutoOffsetReset tests ─────────────────────────────────────────

    /// Test auto offset reset default and variants
    /// 测试自动偏移重置默认值和变体
    #[test]
    fn test_auto_offset_reset_default() {
        assert_eq!(AutoOffsetReset::default(), AutoOffsetReset::Latest);
    }

    // ── ConsumerOffset tests ──────────────────────────────────────────

    /// Test consumer offset construction
    /// 测试消费者偏移构造
    #[test]
    fn test_consumer_offset_new() {
        let offset = ConsumerOffset::new("my-topic", 2, 42);
        assert_eq!(offset.topic, "my-topic");
        assert_eq!(offset.partition, 2);
        assert_eq!(offset.offset, 42);
    }
}
