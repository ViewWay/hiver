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
