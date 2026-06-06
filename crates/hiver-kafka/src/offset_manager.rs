//! Offset management for Kafka consumers.
//! Kafka消费者的偏移管理。
//!
//! Provides fine-grained control over consumer offsets including
//! manual commits, seeks, and position queries.
//!
//! 提供对消费者偏移的精细控制，包括手动提交、跳转和位置查询。

use std::collections::HashMap;

/// Tracks the offset state for a topic-partition.
/// 跟踪主题分区的偏移状态。
#[derive(Clone, Debug)]
struct PartitionOffsetState
{
    /// Last committed offset.
    /// 最后提交的偏移。
    committed: i64,

    /// Current consumer position (next offset to be fetched).
    /// 当前消费者位置（下一个要拉取的偏移）。
    position: i64,

    /// End offset (high watermark) of the partition.
    /// 分区的结束偏移（高水位标记）。
    end_offset: i64,
}

/// Key for topic-partition lookup.
/// 主题分区查找的键。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct TopicPartitionKey
{
    topic: String,
    partition: i32,
}

/// Offset manager for Kafka consumers.
/// Kafka消费者的偏移管理器。
///
/// Provides methods to commit, query, and seek consumer offsets
/// on a per-partition basis.
///
/// 提供按分区提交、查询和跳转消费者偏移的方法。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// KafkaConsumer<String, String> consumer = ...;
/// consumer.commitSync(Map.of(new TopicPartition("topic", 0), new OffsetAndMetadata(42)));
/// consumer.committed(new TopicPartition("topic", 0));
/// consumer.seek(new TopicPartition("topic", 0), 0);
/// consumer.position(new TopicPartition("topic", 0));
/// ```
#[derive(Clone)]
pub struct OffsetManager
{
    /// Per-partition offset state.
    /// 每个分区的偏移状态。
    offsets: HashMap<TopicPartitionKey, PartitionOffsetState>,
}

impl OffsetManager
{
    /// Create a new offset manager.
    /// 创建新的偏移管理器。
    pub fn new() -> Self
    {
        Self {
            offsets: HashMap::new(),
        }
    }

    /// Manually commit an offset for a topic-partition.
    /// 手动提交主题分区的偏移。
    ///
    /// The `offset` should be the offset of the next message to be consumed
    /// (i.e. last consumed offset + 1).
    ///
    /// `offset` 应该是下一个要消费的消息的偏移（即最后消费的偏移 + 1）。
    pub fn commit_offset(&mut self, topic: &str, partition: i32, offset: i64)
    -> Result<(), String>
    {
        tracing::debug!(
            "Committing offset: topic={}, partition={}, offset={}",
            topic,
            partition,
            offset
        );
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        let state = self
            .offsets
            .entry(key)
            .or_insert_with(|| PartitionOffsetState {
                committed: 0,
                position: 0,
                end_offset: 0,
            });
        state.committed = offset;
        Ok(())
    }

    /// Get the last committed offset for a topic-partition.
    /// 获取主题分区的最后提交偏移。
    ///
    /// Returns `None` if no offset has been committed for this partition.
    /// Returns the committed offset value otherwise.
    ///
    /// 如果该分区没有提交过偏移则返回 `None`。
    /// 否则返回已提交的偏移值。
    pub fn committed_offset(&self, topic: &str, partition: i32) -> Result<Option<i64>, String>
    {
        tracing::debug!("Querying committed offset: topic={}, partition={}", topic, partition);
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        Ok(self.offsets.get(&key).map(|s| s.committed))
    }

    /// Seek the consumer to the beginning of a topic-partition.
    /// 将消费者跳转到主题分区的开头。
    ///
    /// After this call, the next `poll` will start from offset 0.
    ///
    /// 调用后，下一次 `poll` 将从偏移 0 开始。
    pub fn seek_to_beginning(&mut self, topic: &str, partition: i32) -> Result<(), String>
    {
        tracing::debug!("Seeking to beginning: topic={}, partition={}", topic, partition);
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        let state = self
            .offsets
            .entry(key)
            .or_insert_with(|| PartitionOffsetState {
                committed: 0,
                position: 0,
                end_offset: 0,
            });
        state.position = 0;
        Ok(())
    }

    /// Seek the consumer to the end of a topic-partition.
    /// 将消费者跳转到主题分区的末尾。
    ///
    /// After this call, the next `poll` will only receive newly produced messages.
    ///
    /// 调用后，下一次 `poll` 只会接收新产生的消息。
    pub fn seek_to_end(&mut self, topic: &str, partition: i32) -> Result<(), String>
    {
        tracing::debug!("Seeking to end: topic={}, partition={}", topic, partition);
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        let state = self
            .offsets
            .entry(key)
            .or_insert_with(|| PartitionOffsetState {
                committed: 0,
                position: 0,
                end_offset: 0,
            });
        state.position = state.end_offset;
        Ok(())
    }

    /// Seek the consumer to the offset at a given timestamp.
    /// 将消费者跳转到给定时间戳对应的偏移。
    ///
    /// The `timestamp` is milliseconds since epoch. The consumer will seek
    /// to the first message whose timestamp is greater than or equal to
    /// the given timestamp.
    ///
    /// `timestamp` 是自纪元以来的毫秒数。消费者将跳转到时间戳
    /// 大于等于给定时间戳的第一条消息。
    pub fn seek_to_timestamp(
        &mut self,
        topic: &str,
        partition: i32,
        timestamp: i64,
    ) -> Result<(), String>
    {
        tracing::debug!(
            "Seeking to timestamp: topic={}, partition={}, timestamp={}",
            topic,
            partition,
            timestamp
        );
        // Mock implementation: position at end_offset as a placeholder.
        // In a real implementation, this would use the offsetsForTimes API.
        // 模拟实现：将位置设为 end_offset 作为占位。
        // 真实实现将使用 offsetsForTimes API。
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        let state = self
            .offsets
            .entry(key)
            .or_insert_with(|| PartitionOffsetState {
                committed: 0,
                position: 0,
                end_offset: 0,
            });
        state.position = state.end_offset;
        Ok(())
    }

    /// Get the current consumer position for a topic-partition.
    /// 获取主题分区的当前消费者位置。
    ///
    /// The position is the offset of the next message to be fetched.
    /// Returns `None` if no state exists for this partition.
    ///
    /// 位置是下一个要拉取的消息的偏移。
    /// 如果该分区不存在状态则返回 `None`。
    pub fn position(&self, topic: &str, partition: i32) -> Result<Option<i64>, String>
    {
        tracing::debug!("Querying position: topic={}, partition={}", topic, partition);
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        Ok(self.offsets.get(&key).map(|s| s.position))
    }

    /// Set the end offset for a topic-partition (for testing / initialization).
    /// 设置主题分区的结束偏移（用于测试/初始化）。
    pub fn set_end_offset(&mut self, topic: &str, partition: i32, end_offset: i64)
    {
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        let state = self
            .offsets
            .entry(key)
            .or_insert_with(|| PartitionOffsetState {
                committed: 0,
                position: 0,
                end_offset: 0,
            });
        state.end_offset = end_offset;
    }

    /// Initialize a partition with committed offset, position, and end offset.
    /// 用提交偏移、位置和结束偏移初始化分区。
    pub fn init_partition(
        &mut self,
        topic: &str,
        partition: i32,
        committed: i64,
        position: i64,
        end_offset: i64,
    )
    {
        let key = TopicPartitionKey {
            topic: topic.to_string(),
            partition,
        };
        self.offsets.insert(key, PartitionOffsetState {
            committed,
            position,
            end_offset,
        });
    }
}

impl Default for OffsetManager
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Test committing and reading back offset.
    /// 测试提交并读回偏移。
    #[test]
    fn test_commit_and_committed_offset()
    {
        let mut mgr = OffsetManager::new();
        mgr.init_partition("topic-a", 0, 10, 15, 100);

        mgr.commit_offset("topic-a", 0, 20).unwrap();
        let committed = mgr.committed_offset("topic-a", 0).unwrap();
        assert_eq!(committed, Some(20));
    }

    /// Test committed_offset returns None for unknown partition.
    /// 测试未知分区 committed_offset 返回 None。
    #[test]
    fn test_committed_offset_unknown_partition()
    {
        let mgr = OffsetManager::new();
        let result = mgr.committed_offset("unknown", 0).unwrap();
        assert!(result.is_none());
    }

    /// Test seek_to_beginning resets position.
    /// 测试 seek_to_beginning 重置位置。
    #[test]
    fn test_seek_to_beginning()
    {
        let mut mgr = OffsetManager::new();
        mgr.init_partition("topic-a", 0, 50, 75, 100);

        mgr.seek_to_beginning("topic-a", 0).unwrap();
        let pos = mgr.position("topic-a", 0).unwrap();
        assert_eq!(pos, Some(0));
    }

    /// Test seek_to_end sets position to end_offset.
    /// 测试 seek_to_end 设置位置到 end_offset。
    #[test]
    fn test_seek_to_end()
    {
        let mut mgr = OffsetManager::new();
        mgr.init_partition("topic-a", 0, 50, 75, 200);

        mgr.seek_to_end("topic-a", 0).unwrap();
        let pos = mgr.position("topic-a", 0).unwrap();
        assert_eq!(pos, Some(200));
    }

    /// Test seek_to_timestamp sets position to end_offset (mock).
    /// 测试 seek_to_timestamp 设置位置到 end_offset（模拟）。
    #[test]
    fn test_seek_to_timestamp()
    {
        let mut mgr = OffsetManager::new();
        mgr.init_partition("topic-a", 0, 50, 75, 300);

        mgr.seek_to_timestamp("topic-a", 0, 1700000000000).unwrap();
        let pos = mgr.position("topic-a", 0).unwrap();
        assert_eq!(pos, Some(300));
    }

    /// Test position returns None for unknown partition.
    /// 测试未知分区 position 返回 None。
    #[test]
    fn test_position_unknown_partition()
    {
        let mgr = OffsetManager::new();
        let result = mgr.position("no-topic", 5).unwrap();
        assert!(result.is_none());
    }

    /// Test position returns initialized value.
    /// 测试 position 返回初始化值。
    #[test]
    fn test_position_initialized()
    {
        let mut mgr = OffsetManager::new();
        mgr.init_partition("topic-x", 3, 100, 120, 500);
        let pos = mgr.position("topic-x", 3).unwrap();
        assert_eq!(pos, Some(120));
    }

    /// Test set_end_offset updates existing state.
    /// 测试 set_end_offset 更新现有状态。
    #[test]
    fn test_set_end_offset()
    {
        let mut mgr = OffsetManager::new();
        mgr.init_partition("topic-a", 0, 10, 20, 100);
        mgr.set_end_offset("topic-a", 0, 150);

        mgr.seek_to_end("topic-a", 0).unwrap();
        let pos = mgr.position("topic-a", 0).unwrap();
        assert_eq!(pos, Some(150));
    }

    /// Test committing offset creates state for new partition.
    /// 测试提交偏移为新分区创建状态。
    #[test]
    fn test_commit_creates_state()
    {
        let mut mgr = OffsetManager::new();
        mgr.commit_offset("new-topic", 2, 42).unwrap();
        assert_eq!(mgr.committed_offset("new-topic", 2).unwrap(), Some(42));
    }

    /// Test Default trait implementation.
    /// 测试 Default trait 实现。
    #[test]
    fn test_default()
    {
        let mgr = OffsetManager::default();
        assert!(mgr.committed_offset("any", 0).unwrap().is_none());
    }

    /// Test multiple partitions independently.
    /// 测试多个分区独立操作。
    #[test]
    fn test_multiple_partitions()
    {
        let mut mgr = OffsetManager::new();
        mgr.init_partition("topic-a", 0, 10, 15, 100);
        mgr.init_partition("topic-a", 1, 20, 25, 200);
        mgr.init_partition("topic-b", 0, 30, 35, 300);

        mgr.commit_offset("topic-a", 0, 50).unwrap();
        mgr.seek_to_beginning("topic-a", 1).unwrap();

        assert_eq!(mgr.committed_offset("topic-a", 0).unwrap(), Some(50));
        assert_eq!(mgr.position("topic-a", 1).unwrap(), Some(0));
        assert_eq!(mgr.committed_offset("topic-b", 0).unwrap(), Some(30));
        assert_eq!(mgr.position("topic-b", 0).unwrap(), Some(35));
    }
}
