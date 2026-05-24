//! Kafka topic and partition
//! Kafka主题和分区

use serde::{Deserialize, Serialize};

/// Topic partition
/// 主题分区
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// TopicPartition topicPartition = new TopicPartition("my_topic", 0);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicPartition {
    /// Topic name
    /// 主题名称
    pub topic: String,

    /// Partition number
    /// 分区号
    pub partition: i32,
}

impl TopicPartition {
    /// Create new topic partition
    /// 创建新的主题分区
    pub fn new(topic: impl Into<String>, partition: i32) -> Self {
        Self {
            topic: topic.into(),
            partition,
        }
    }
}

/// Offset position
/// 偏移位置
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Offset {
    /// Beginning of the partition
    /// 分区开始位置
    Beginning,

    /// End of the partition
    /// 分区结束位置
    End,

    /// Specific offset
    /// 特定偏移
    Specific(i64),
}

impl Offset {
    /// Get offset value or return default
    /// 获取偏移值或返回默认值
    pub fn value_or(self, default: i64) -> i64 {
        match self {
            Self::Beginning => 0,
            Self::End => default,
            Self::Specific(offset) => offset,
        }
    }
}

/// Topic partition builder
/// 主题分区构建器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// TopicPartitionInfo info = TopicPartitionInfo.builder()
///     .name("my_topic")
///     .partitions(3)
///     .replicationFactor((short) 3)
///     .build();
/// ```
#[derive(Clone, Debug)]
pub struct TopicPartitionBuilder {
    /// Topic name
    /// 主题名称
    pub topic: String,

    /// Number of partitions
    /// 分区数量
    pub partitions: i32,

    /// Replication factor
    /// 复制因子
    pub replication_factor: i32,

    /// Config entries
    /// 配置条目
    pub config: std::collections::HashMap<String, String>,
}

impl TopicPartitionBuilder {
    /// Create new topic partition builder
    /// 创建新的主题分区构建器
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            partitions: 1,
            replication_factor: 1,
            config: std::collections::HashMap::new(),
        }
    }

    /// Set partitions
    /// 设置分区
    pub fn with_partitions(mut self, partitions: i32) -> Self {
        self.partitions = partitions;
        self
    }

    /// Set replication factor
    /// 设置复制因子
    pub fn with_replication_factor(mut self, factor: i32) -> Self {
        self.replication_factor = factor;
        self
    }

    /// Add config
    /// 添加配置
    pub fn with_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }

    /// Build topic partition
    /// 构建主题分区
    pub fn build(self) -> TopicPartition {
        TopicPartition::new(self.topic, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test TopicPartition construction
    /// 测试 TopicPartition 构造
    #[test]
    fn test_topic_partition_new() {
        let tp = TopicPartition::new("my-topic", 3);
        assert_eq!(tp.topic, "my-topic");
        assert_eq!(tp.partition, 3);
    }

    /// Test TopicPartition equality and hashing
    /// 测试 TopicPartition 相等性和哈希
    #[test]
    fn test_topic_partition_equality() {
        let tp1 = TopicPartition::new("topic", 0);
        let tp2 = TopicPartition::new("topic", 0);
        let tp3 = TopicPartition::new("topic", 1);
        assert_eq!(tp1, tp2);
        assert_ne!(tp1, tp3);
    }

    /// Test TopicPartition serde round-trip
    /// 测试 TopicPartition 序列化往返
    #[test]
    fn test_topic_partition_serde_roundtrip() {
        let tp = TopicPartition::new("serde-topic", 5);
        let json = serde_json::to_string(&tp).unwrap();
        let restored: TopicPartition = serde_json::from_str(&json).unwrap();
        assert_eq!(tp, restored);
    }

    // ── Offset tests ──────────────────────────────────────────────────

    /// Test Offset::value_or for all variants
    /// 测试所有变体的 Offset::value_or
    #[test]
    fn test_offset_value_or() {
        assert_eq!(Offset::Beginning.value_or(999), 0);
        assert_eq!(Offset::End.value_or(999), 999);
        assert_eq!(Offset::Specific(42).value_or(999), 42);
    }

    /// Test Offset equality and copy
    /// 测试 Offset 相等性和拷贝
    #[test]
    fn test_offset_equality() {
        assert_eq!(Offset::Beginning, Offset::Beginning);
        assert_eq!(Offset::End, Offset::End);
        assert_eq!(Offset::Specific(10), Offset::Specific(10));
        assert_ne!(Offset::Beginning, Offset::End);
    }

    /// Test Offset serde round-trip
    /// 测试 Offset 序列化往返
    #[test]
    fn test_offset_serde_roundtrip() {
        let offsets = vec![Offset::Beginning, Offset::End, Offset::Specific(123)];
        for off in &offsets {
            let json = serde_json::to_string(off).unwrap();
            let restored: Offset = serde_json::from_str(&json).unwrap();
            assert_eq!(*off, restored);
        }
    }

    // ── TopicPartitionBuilder tests ───────────────────────────────────

    /// Test TopicPartitionBuilder defaults
    /// 测试 TopicPartitionBuilder 默认值
    #[test]
    fn test_topic_partition_builder_defaults() {
        let builder = TopicPartitionBuilder::new("my-topic");
        assert_eq!(builder.topic, "my-topic");
        assert_eq!(builder.partitions, 1);
        assert_eq!(builder.replication_factor, 1);
        assert!(builder.config.is_empty());
    }

    /// Test TopicPartitionBuilder with all options
    /// 测试带所有选项的 TopicPartitionBuilder
    #[test]
    fn test_topic_partition_builder_full() {
        let tp = TopicPartitionBuilder::new("full-topic")
            .with_partitions(6)
            .with_replication_factor(3)
            .with_config("retention.ms", "604800000")
            .with_config("cleanup.policy", "compact")
            .build();

        assert_eq!(tp.topic, "full-topic");
        // build() always returns partition 0
        assert_eq!(tp.partition, 0);
    }

    /// Test TopicPartitionBuilder clone
    /// 测试 TopicPartitionBuilder 克隆
    #[test]
    fn test_topic_partition_builder_clone() {
        let builder = TopicPartitionBuilder::new("clone-topic")
            .with_partitions(3)
            .with_replication_factor(2);
        let cloned = builder.clone();
        assert_eq!(cloned.topic, "clone-topic");
        assert_eq!(cloned.partitions, 3);
        assert_eq!(cloned.replication_factor, 2);
    }
}
