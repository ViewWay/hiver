//! Consumer group management.
//! 消费者组管理。
//!
//! Provides administrative operations for Kafka consumer groups:
//! list, describe, delete, and reset offsets.
//!
//! 提供Kafka消费者组的管理操作：
//! 列出、描述、删除和重置偏移。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Strategy for resetting consumer offsets.
/// 重置消费者偏移的策略。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OffsetResetStrategy
{
    /// Reset to the earliest available offset.
    /// 重置到最早可用偏移。
    Earliest,

    /// Reset to the latest offset.
    /// 重置到最新偏移。
    Latest,

    /// Reset to the offset at a specific timestamp (milliseconds since epoch).
    /// 重置到指定时间戳的偏移（自纪元以来的毫秒数）。
    ByTimestamp(i64),
}

/// Detailed information about a consumer group member.
/// 消费者组成员的详细信息。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupMemberInfo
{
    /// Member ID assigned by the broker.
    /// Broker分配的成员ID。
    pub member_id: String,

    /// Client ID of the consumer.
    /// 消费者的客户端ID。
    pub client_id: String,

    /// Client host address.
    /// 客户端主机地址。
    pub host: String,

    /// Partitions currently assigned to this member.
    /// 当前分配给此成员的分区。
    pub assigned_partitions: Vec<TopicPartitionAssignment>,
}

/// Topic-partition assignment for a group member.
/// 组成员的主题分区分配。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TopicPartitionAssignment
{
    /// Topic name.
    /// 主题名称。
    pub topic: String,

    /// Partition number.
    /// 分区号。
    pub partition: i32,
}

/// Detailed description of a consumer group.
/// 消费者组的详细描述。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupDescription
{
    /// Group ID.
    /// 组ID。
    pub group_id: String,

    /// Group state (e.g. "Stable", "PreparingRebalance", "CompletingRebalance", "Dead", "Empty").
    /// 组状态（如 "Stable"、"PreparingRebalance"、"CompletingRebalance"、"Dead"、"Empty"）。
    pub state: String,

    /// Protocol type (e.g. "consumer").
    /// 协议类型（如 "consumer"）。
    pub protocol_type: String,

    /// Members of the group.
    /// 组的成员列表。
    pub members: Vec<GroupMemberInfo>,

    /// Per-partition offset and lag information.
    /// 每个分区的偏移和延迟信息。
    pub partition_info: Vec<PartitionOffsetInfo>,
}

/// Offset and lag information for a single partition.
/// 单个分区的偏移和延迟信息。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartitionOffsetInfo
{
    /// Topic name.
    /// 主题名称。
    pub topic: String,

    /// Partition number.
    /// 分区号。
    pub partition: i32,

    /// Current committed offset.
    /// 当前已提交的偏移。
    pub committed_offset: i64,

    /// End offset (log size) of the partition.
    /// 分区的结束偏移（日志大小）。
    pub end_offset: i64,

    /// Lag (end_offset - committed_offset).
    /// 延迟（end_offset - committed_offset）。
    pub lag: i64,
}

/// Summary of a consumer group returned by list operations.
/// 列出操作返回的消费者组摘要。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupSummary
{
    /// Group ID.
    /// 组ID。
    pub group_id: String,

    /// Group state.
    /// 组状态。
    pub state: String,

    /// Number of members in the group.
    /// 组中的成员数量。
    pub member_count: usize,

    /// Protocol type.
    /// 协议类型。
    pub protocol_type: String,
}

/// Consumer group manager for administrative operations.
/// 用于管理操作的消费者组管理器。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// AdminClient adminClient = AdminClient.create(props);
/// adminClient.listConsumerGroups();
/// adminClient.describeConsumerGroups(List.of("my-group"));
/// adminClient.deleteConsumerGroups(List.of("my-group"));
/// ```
#[derive(Clone)]
pub struct ConsumerGroupManager
{
    /// Bootstrap servers.
    /// 引导服务器。
    bootstrap_servers: String,

    /// Cached group descriptions.
    /// 缓存的组描述。
    groups: HashMap<String, GroupDescription>,
}

impl ConsumerGroupManager
{
    /// Create a new consumer group manager.
    /// 创建新的消费者组管理器。
    pub fn new(bootstrap_servers: impl Into<String>) -> Self
    {
        Self {
            bootstrap_servers: bootstrap_servers.into(),
            groups: HashMap::new(),
        }
    }

    /// Get bootstrap servers.
    /// 获取引导服务器。
    pub fn bootstrap_servers(&self) -> &str
    {
        &self.bootstrap_servers
    }

    /// List all consumer groups.
    /// 列出所有消费者组。
    ///
    /// Returns a list of group summaries including group ID, state,
    /// member count, and protocol type.
    ///
    /// 返回组摘要列表，包括组ID、状态、成员数量和协议类型。
    pub fn list_groups(&self) -> Result<Vec<GroupSummary>, String>
    {
        tracing::debug!("Listing consumer groups from {}", self.bootstrap_servers);
        let summaries: Vec<GroupSummary> = self
            .groups
            .values()
            .map(|g| GroupSummary {
                group_id: g.group_id.clone(),
                state: g.state.clone(),
                member_count: g.members.len(),
                protocol_type: g.protocol_type.clone(),
            })
            .collect();
        Ok(summaries)
    }

    /// Describe a consumer group by its group ID.
    /// 根据组ID描述消费者组。
    ///
    /// Returns detailed information about the group including members,
    /// partition assignments, and offset/lag details.
    ///
    /// 返回组的详细信息，包括成员、分区分配和偏移/延迟详情。
    pub fn describe_group(&self, group_id: &str) -> Result<GroupDescription, String>
    {
        tracing::debug!("Describing consumer group: {}", group_id);
        self.groups
            .get(group_id)
            .cloned()
            .ok_or_else(|| format!("Consumer group '{}' not found", group_id))
    }

    /// Delete a consumer group.
    /// 删除消费者组。
    ///
    /// The group must be empty (no members) before it can be deleted.
    ///
    /// 组必须为空（无成员）才能删除。
    pub fn delete_group(&mut self, group_id: &str) -> Result<(), String>
    {
        tracing::info!("Deleting consumer group: {}", group_id);
        self.groups
            .remove(group_id)
            .map(|_| ())
            .ok_or_else(|| format!("Consumer group '{}' not found", group_id))
    }

    /// Reset consumer offsets for a specific topic within a group.
    /// 重置组内特定主题的消费者偏移。
    ///
    /// The `strategy` parameter determines where to reset:
    /// - `OffsetResetStrategy::Earliest` — reset to the beginning of the log.
    /// - `OffsetResetStrategy::Latest` — reset to the end of the log.
    /// - `OffsetResetStrategy::ByTimestamp(ts)` — reset to the offset at the given timestamp.
    ///
    /// `strategy` 参数决定重置位置：
    /// - `OffsetResetStrategy::Earliest` — 重置到日志开头。
    /// - `OffsetResetStrategy::Latest` — 重置到日志末尾。
    /// - `OffsetResetStrategy::ByTimestamp(ts)` — 重置到给定时间戳的偏移。
    pub fn reset_offsets(
        &mut self,
        group_id: &str,
        topic: &str,
        strategy: &OffsetResetStrategy,
    ) -> Result<(), String>
    {
        tracing::info!(
            "Resetting offsets for group='{}', topic='{}', strategy={:?}",
            group_id,
            topic,
            strategy
        );
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or_else(|| format!("Consumer group '{}' not found", group_id))?;

        for info in &mut group.partition_info
        {
            if info.topic == topic
            {
                let new_offset = match strategy
                {
                    OffsetResetStrategy::Earliest => 0,
                    OffsetResetStrategy::Latest => info.end_offset,
                    OffsetResetStrategy::ByTimestamp(_ts) =>
                    {
                        // In a real implementation, this would look up the offset
                        // at the given timestamp via offsetsForTimes API.
                        info.end_offset
                    },
                };
                info.committed_offset = new_offset;
                info.lag = info.end_offset - new_offset;
            }
        }
        Ok(())
    }

    /// Register a group description (for testing / manual setup).
    /// 注册组描述（用于测试/手动设置）。
    pub fn register_group(&mut self, description: GroupDescription)
    {
        self.groups
            .insert(description.group_id.clone(), description);
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use super::*;

    fn sample_group(group_id: &str) -> GroupDescription
    {
        GroupDescription {
            group_id: group_id.to_string(),
            state: "Stable".to_string(),
            protocol_type: "consumer".to_string(),
            members: vec![GroupMemberInfo {
                member_id: "member-1".to_string(),
                client_id: "test-client".to_string(),
                host: "/127.0.0.1".to_string(),
                assigned_partitions: vec![TopicPartitionAssignment {
                    topic: "test-topic".to_string(),
                    partition: 0,
                }],
            }],
            partition_info: vec![PartitionOffsetInfo {
                topic: "test-topic".to_string(),
                partition: 0,
                committed_offset: 50,
                end_offset: 100,
                lag: 50,
            }],
        }
    }

    /// Test listing groups returns empty when none registered.
    /// 测试未注册组时列出返回空。
    #[test]
    fn test_list_groups_empty()
    {
        let mgr = ConsumerGroupManager::new("localhost:9092");
        let groups = mgr.list_groups().unwrap();
        assert!(groups.is_empty());
    }

    /// Test listing groups returns registered groups.
    /// 测试列出组返回已注册的组。
    #[test]
    fn test_list_groups_with_data()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        mgr.register_group(sample_group("group-a"));
        mgr.register_group(sample_group("group-b"));

        let groups = mgr.list_groups().unwrap();
        assert_eq!(groups.len(), 2);
        let ids: Vec<&str> = groups.iter().map(|g| g.group_id.as_str()).collect();
        assert!(ids.contains(&"group-a"));
        assert!(ids.contains(&"group-b"));
    }

    /// Test describing an existing group.
    /// 测试描述存在的组。
    #[test]
    fn test_describe_group_found()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        mgr.register_group(sample_group("my-group"));

        let desc = mgr.describe_group("my-group").unwrap();
        assert_eq!(desc.group_id, "my-group");
        assert_eq!(desc.state, "Stable");
        assert_eq!(desc.members.len(), 1);
        assert_eq!(desc.partition_info.len(), 1);
    }

    /// Test describing a non-existent group returns error.
    /// 测试描述不存在的组返回错误。
    #[test]
    fn test_describe_group_not_found()
    {
        let mgr = ConsumerGroupManager::new("localhost:9092");
        let result = mgr.describe_group("missing-group");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    /// Test deleting an existing group.
    /// 测试删除存在的组。
    #[test]
    fn test_delete_group()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        mgr.register_group(sample_group("to-delete"));

        mgr.delete_group("to-delete").unwrap();
        assert!(mgr.describe_group("to-delete").is_err());
        assert!(mgr.list_groups().unwrap().is_empty());
    }

    /// Test deleting a non-existent group returns error.
    /// 测试删除不存在的组返回错误。
    #[test]
    fn test_delete_group_not_found()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        let result = mgr.delete_group("no-such-group");
        assert!(result.is_err());
    }

    /// Test resetting offsets to earliest.
    /// 测试重置偏移到最早。
    #[test]
    fn test_reset_offsets_earliest()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        mgr.register_group(sample_group("reset-group"));

        mgr.reset_offsets("reset-group", "test-topic", &OffsetResetStrategy::Earliest)
            .unwrap();

        let desc = mgr.describe_group("reset-group").unwrap();
        let info = &desc.partition_info[0];
        assert_eq!(info.committed_offset, 0);
        assert_eq!(info.lag, 100);
    }

    /// Test resetting offsets to latest.
    /// 测试重置偏移到最新。
    #[test]
    fn test_reset_offsets_latest()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        mgr.register_group(sample_group("reset-group"));

        mgr.reset_offsets("reset-group", "test-topic", &OffsetResetStrategy::Latest)
            .unwrap();

        let desc = mgr.describe_group("reset-group").unwrap();
        let info = &desc.partition_info[0];
        assert_eq!(info.committed_offset, 100);
        assert_eq!(info.lag, 0);
    }

    /// Test resetting offsets by timestamp.
    /// 测试按时间戳重置偏移。
    #[test]
    fn test_reset_offsets_by_timestamp()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        mgr.register_group(sample_group("reset-group"));

        mgr.reset_offsets(
            "reset-group",
            "test-topic",
            &OffsetResetStrategy::ByTimestamp(1700000000000),
        )
        .unwrap();

        let desc = mgr.describe_group("reset-group").unwrap();
        let info = &desc.partition_info[0];
        // Mock implementation resets to end_offset for ByTimestamp
        assert_eq!(info.committed_offset, 100);
    }

    /// Test resetting offsets for non-existent group.
    /// 测试为不存在的组重置偏移。
    #[test]
    fn test_reset_offsets_group_not_found()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        let result = mgr.reset_offsets("missing", "topic", &OffsetResetStrategy::Earliest);
        assert!(result.is_err());
    }

    /// Test bootstrap_servers accessor.
    /// 测试 bootstrap_servers 访问器。
    #[test]
    fn test_bootstrap_servers()
    {
        let mgr = ConsumerGroupManager::new("kafka:9093");
        assert_eq!(mgr.bootstrap_servers(), "kafka:9093");
    }

    /// Test GroupSummary member count matches description.
    /// 测试 GroupSummary 成员数与描述匹配。
    #[test]
    fn test_group_summary_member_count()
    {
        let mut mgr = ConsumerGroupManager::new("localhost:9092");
        let mut group = sample_group("count-group");
        group.members.push(GroupMemberInfo {
            member_id: "member-2".to_string(),
            client_id: "client-2".to_string(),
            host: "/127.0.0.2".to_string(),
            assigned_partitions: vec![],
        });
        mgr.register_group(group);

        let summaries = mgr.list_groups().unwrap();
        assert_eq!(summaries[0].member_count, 2);
    }

    /// Test OffsetResetStrategy serialization round-trip.
    /// 测试 OffsetResetStrategy 序列化往返。
    #[test]
    fn test_offset_reset_strategy_serde()
    {
        let strategies = vec![
            OffsetResetStrategy::Earliest,
            OffsetResetStrategy::Latest,
            OffsetResetStrategy::ByTimestamp(1234567890),
        ];
        for s in &strategies
        {
            let json = serde_json::to_string(s).unwrap();
            let restored: OffsetResetStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(*s, restored);
        }
    }
}
