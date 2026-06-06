//! Transactional producer for Kafka.
//! Kafka的事务生产者。
//!
//! Supports exactly-once semantics (EOS) via the consume-transform-produce
//! pattern, allowing atomic writes across multiple topic-partitions.
//!
//! 通过 consume-transform-produce 模式支持精确一次语义（EOS），
//! 允许跨多个主题分区的原子写入。

use crate::{ProducerConfig, Record};

/// Offset and metadata for consume-transform-produce pattern.
/// consume-transform-produce 模式的偏移和元数据。
#[derive(Clone, Debug)]
pub struct TransactionOffset
{
    /// Topic name.
    /// 主题名称。
    pub topic: String,

    /// Partition number.
    /// 分区号。
    pub partition: i32,

    /// Offset to commit.
    /// 要提交的偏移。
    pub offset: i64,

    /// Optional metadata.
    /// 可选元数据。
    pub metadata: Option<String>,
}

impl TransactionOffset
{
    /// Create a new transaction offset.
    /// 创建新的事务偏移。
    pub fn new(topic: impl Into<String>, partition: i32, offset: i64) -> Self
    {
        Self {
            topic: topic.into(),
            partition,
            offset,
            metadata: None,
        }
    }

    /// Attach metadata to this offset entry.
    /// 为此偏移条目附加元数据。
    pub fn with_metadata(mut self, metadata: impl Into<String>) -> Self
    {
        self.metadata = Some(metadata.into());
        self
    }
}

/// State of a transaction.
/// 事务的状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionState
{
    /// No transaction in progress.
    /// 没有正在进行的事务。
    Idle,

    /// Transaction has been started.
    /// 事务已开始。
    Active,

    /// Transaction commit is in progress.
    /// 事务正在提交。
    Committing,

    /// Transaction abort is in progress.
    /// 事务正在中止。
    Aborting,
}

/// Transactional producer supporting exactly-once semantics.
/// 支持精确一次语义的事务生产者。
///
/// Wraps a regular `Producer` and adds transaction boundaries.
/// Messages sent within a transaction are only visible to consumers
/// after the transaction is committed.
///
/// 包装常规 `Producer` 并添加事务边界。
/// 在事务内发送的消息只有在事务提交后才对消费者可见。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// KafkaTransactionManager txManager = new KafkaTransactionManager(producerFactory);
/// txManager.beginTransaction();
/// kafkaTemplate.executeInTransaction(t -> {
///     t.send("topic", "key", "value");
///     t.sendOffsetsToTransaction(offsets, consumerGroupMetadata);
/// });
/// // Auto-commit on success, auto-abort on exception
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_kafka::{TransactionalProducer, ProducerConfig};
///
/// let producer = TransactionalProducer::new(
///     ProducerConfig::new(),
///     "my-txn-id",
/// );
///
/// producer.begin_transaction();
/// producer.send_in_transaction("topic", Some("key"), b"value");
/// producer.send_offsets_to_transaction(offsets, "my-group");
/// producer.commit_transaction();
/// ```
#[derive(Clone)]
pub struct TransactionalProducer
{
    /// Underlying producer configuration.
    /// 底层生产者配置。
    config: ProducerConfig,

    /// Transactional ID for exactly-once semantics.
    /// 用于精确一次语义的事务ID。
    transactional_id: String,

    /// Current transaction state.
    /// 当前事务状态。
    state: TransactionState,

    /// Messages buffered within the current transaction.
    /// 当前事务中缓冲的消息。
    pending_records: Vec<Record>,

    /// Offsets to be committed with the transaction (consume-transform-produce).
    /// 要随事务提交的偏移（consume-transform-produce）。
    pending_offsets: Vec<TransactionOffset>,

    /// Consumer group IDs associated with pending offsets.
    /// 与待提交偏移关联的消费者组ID。
    pending_group_ids: Vec<String>,
}

impl TransactionalProducer
{
    /// Create a new transactional producer.
    /// 创建新的事务生产者。
    ///
    /// The `transactional_id` must be unique across all producer instances
    /// to maintain exactly-once guarantees. Kafka uses this ID to fence
    /// off zombie instances from previous generations.
    ///
    /// `transactional_id` 必须在所有生产者实例中唯一，
    /// 以维护精确一次保证。Kafka 使用此 ID 隔离前代的僵尸实例。
    pub fn new(config: ProducerConfig, transactional_id: impl Into<String>) -> Self
    {
        Self {
            config,
            transactional_id: transactional_id.into(),
            state: TransactionState::Idle,
            pending_records: Vec::new(),
            pending_offsets: Vec::new(),
            pending_group_ids: Vec::new(),
        }
    }

    /// Get the transactional ID.
    /// 获取事务ID。
    pub fn transactional_id(&self) -> &str
    {
        &self.transactional_id
    }

    /// Get the current transaction state.
    /// 获取当前事务状态。
    pub fn state(&self) -> &TransactionState
    {
        &self.state
    }

    /// Get the underlying producer configuration.
    /// 获取底层生产者配置。
    pub fn config(&self) -> &ProducerConfig
    {
        &self.config
    }

    /// Begin a new transaction.
    /// 开始新事务。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if a transaction is already in progress.
    /// 如果事务正在进行中则返回错误。
    pub fn begin_transaction(&mut self) -> Result<(), String>
    {
        if self.state != TransactionState::Idle
        {
            return Err(format!(
                "Cannot begin transaction: current state is {:?}, expected Idle",
                self.state
            ));
        }
        tracing::info!("Beginning transaction: transactional_id={}", self.transactional_id);
        self.state = TransactionState::Active;
        self.pending_records.clear();
        self.pending_offsets.clear();
        self.pending_group_ids.clear();
        Ok(())
    }

    /// Commit the current transaction.
    /// 提交当前事务。
    ///
    /// All buffered messages and offsets are atomically committed.
    /// After this call, the producer returns to the `Idle` state
    /// and a new transaction can be started.
    ///
    /// 所有缓冲的消息和偏移将原子性地提交。
    /// 调用后，生产者返回 `Idle` 状态，可以开始新事务。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if no transaction is active.
    /// 如果没有活跃事务则返回错误。
    pub fn commit_transaction(&mut self) -> Result<(), String>
    {
        if self.state != TransactionState::Active
        {
            return Err(format!(
                "Cannot commit transaction: current state is {:?}, expected Active",
                self.state
            ));
        }
        self.state = TransactionState::Committing;

        let record_count = self.pending_records.len();
        let offset_count = self.pending_offsets.len();
        tracing::info!(
            "Committing transaction: transactional_id={}, records={}, offsets={}",
            self.transactional_id,
            record_count,
            offset_count
        );

        // Mock implementation: simulate flush of all pending records.
        // 模拟实现：模拟刷新所有待处理记录。
        for record in &self.pending_records
        {
            tracing::debug!(
                "Flushing record: topic={}, {} bytes",
                record.topic,
                record.payload.len()
            );
        }

        self.pending_records.clear();
        self.pending_offsets.clear();
        self.pending_group_ids.clear();
        self.state = TransactionState::Idle;
        Ok(())
    }

    /// Abort the current transaction.
    /// 中止当前事务。
    ///
    /// All buffered messages are discarded. After this call,
    /// the producer returns to the `Idle` state.
    ///
    /// 所有缓冲消息将被丢弃。调用后，生产者返回 `Idle` 状态。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if no transaction is active.
    /// 如果没有活跃事务则返回错误。
    pub fn abort_transaction(&mut self) -> Result<(), String>
    {
        if self.state != TransactionState::Active
        {
            return Err(format!(
                "Cannot abort transaction: current state is {:?}, expected Active",
                self.state
            ));
        }
        self.state = TransactionState::Aborting;

        let discarded = self.pending_records.len();
        tracing::warn!(
            "Aborting transaction: transactional_id={}, discarding {} records",
            self.transactional_id,
            discarded
        );

        self.pending_records.clear();
        self.pending_offsets.clear();
        self.pending_group_ids.clear();
        self.state = TransactionState::Idle;
        Ok(())
    }

    /// Send a message within the current transaction.
    /// 在当前事务内发送消息。
    ///
    /// The message is buffered and only delivered when the transaction
    /// is committed. If the transaction is aborted, the message is discarded.
    ///
    /// 消息被缓冲，仅在事务提交时传递。如果事务中止，消息将被丢弃。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if no transaction is active.
    /// 如果没有活跃事务则返回错误。
    pub fn send_in_transaction(
        &mut self,
        topic: &str,
        key: Option<&str>,
        value: &[u8],
    ) -> Result<(), String>
    {
        if self.state != TransactionState::Active
        {
            return Err(format!(
                "Cannot send in transaction: current state is {:?}, expected Active",
                self.state
            ));
        }
        let record = Record {
            topic: topic.to_string(),
            partition: None,
            key: key.map(|k| k.as_bytes().to_vec()),
            payload: value.to_vec(),
            headers: Vec::new(),
            timestamp: None,
        };
        tracing::debug!("Buffering transactional record: topic={}, {} bytes", topic, value.len());
        self.pending_records.push(record);
        Ok(())
    }

    /// Send consumer offsets to the current transaction (consume-transform-produce pattern).
    /// 将消费者偏移发送到当前事务（consume-transform-produce 模式）。
    ///
    /// This allows atomically committing both produced messages and consumed
    /// offsets in a single transaction, enabling exactly-once processing.
    ///
    /// 这允许在单个事务中原子性地提交产生的消息和消费的偏移，
    /// 实现精确一次处理。
    ///
    /// # Arguments / 参数
    ///
    /// - `offsets` — List of `(topic, partition, offset)` entries to commit.
    /// - `group_id` — Consumer group ID these offsets belong to.
    ///
    /// - `offsets` — 要提交的 `(topic, partition, offset)` 条目列表。
    /// - `group_id` — 这些偏移所属的消费者组ID。
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if no transaction is active.
    /// 如果没有活跃事务则返回错误。
    pub fn send_offsets_to_transaction(
        &mut self,
        offsets: Vec<TransactionOffset>,
        group_id: &str,
    ) -> Result<(), String>
    {
        if self.state != TransactionState::Active
        {
            return Err(format!(
                "Cannot send offsets to transaction: current state is {:?}, expected Active",
                self.state
            ));
        }
        tracing::debug!("Adding {} offsets for group '{}' to transaction", offsets.len(), group_id);
        self.pending_offsets.extend(offsets);
        if !self.pending_group_ids.contains(&group_id.to_string())
        {
            self.pending_group_ids.push(group_id.to_string());
        }
        Ok(())
    }

    /// Get the number of pending records in the current transaction.
    /// 获取当前事务中待处理记录的数量。
    pub fn pending_count(&self) -> usize
    {
        self.pending_records.len()
    }

    /// Get the number of pending offsets in the current transaction.
    /// 获取当前事务中待处理偏移的数量。
    pub fn pending_offset_count(&self) -> usize
    {
        self.pending_offsets.len()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    /// Test creating a transactional producer.
    /// 测试创建事务生产者。
    #[test]
    fn test_new_transactional_producer()
    {
        let producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        assert_eq!(producer.transactional_id(), "txn-1");
        assert_eq!(producer.state(), &TransactionState::Idle);
        assert_eq!(producer.pending_count(), 0);
    }

    /// Test begin_transaction transitions state to Active.
    /// 测试 begin_transaction 将状态转为 Active。
    #[test]
    fn test_begin_transaction()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        producer.begin_transaction().unwrap();
        assert_eq!(producer.state(), &TransactionState::Active);
    }

    /// Test begin_transaction fails when already active.
    /// 测试在 Active 状态下 begin_transaction 失败。
    #[test]
    fn test_begin_transaction_already_active()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        producer.begin_transaction().unwrap();
        let result = producer.begin_transaction();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected Idle"));
    }

    /// Test commit_transaction transitions state back to Idle.
    /// 测试 commit_transaction 将状态转回 Idle。
    #[test]
    fn test_commit_transaction()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        producer.begin_transaction().unwrap();
        producer.commit_transaction().unwrap();
        assert_eq!(producer.state(), &TransactionState::Idle);
    }

    /// Test commit_transaction fails when idle.
    /// 测试在 Idle 状态下 commit_transaction 失败。
    #[test]
    fn test_commit_transaction_when_idle()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        let result = producer.commit_transaction();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected Active"));
    }

    /// Test abort_transaction discards records and returns to Idle.
    /// 测试 abort_transaction 丢弃记录并返回 Idle。
    #[test]
    fn test_abort_transaction()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        producer.begin_transaction().unwrap();
        producer
            .send_in_transaction("topic", Some("key"), b"value")
            .unwrap();
        assert_eq!(producer.pending_count(), 1);

        producer.abort_transaction().unwrap();
        assert_eq!(producer.state(), &TransactionState::Idle);
        assert_eq!(producer.pending_count(), 0);
    }

    /// Test abort_transaction fails when idle.
    /// 测试在 Idle 状态下 abort_transaction 失败。
    #[test]
    fn test_abort_transaction_when_idle()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        let result = producer.abort_transaction();
        assert!(result.is_err());
    }

    /// Test send_in_transaction buffers records.
    /// 测试 send_in_transaction 缓冲记录。
    #[test]
    fn test_send_in_transaction()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        producer.begin_transaction().unwrap();

        producer
            .send_in_transaction("topic-a", Some("k1"), b"v1")
            .unwrap();
        producer
            .send_in_transaction("topic-b", None, b"v2")
            .unwrap();

        assert_eq!(producer.pending_count(), 2);
    }

    /// Test send_in_transaction fails when idle.
    /// 测试在 Idle 状态下 send_in_transaction 失败。
    #[test]
    fn test_send_in_transaction_when_idle()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        let result = producer.send_in_transaction("topic", Some("key"), b"value");
        assert!(result.is_err());
    }

    /// Test send_offsets_to_transaction buffers offsets.
    /// 测试 send_offsets_to_transaction 缓冲偏移。
    #[test]
    fn test_send_offsets_to_transaction()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        producer.begin_transaction().unwrap();

        let offsets = vec![
            TransactionOffset::new("source-topic", 0, 100),
            TransactionOffset::new("source-topic", 1, 200),
        ];
        producer
            .send_offsets_to_transaction(offsets, "my-group")
            .unwrap();

        assert_eq!(producer.pending_offset_count(), 2);
    }

    /// Test send_offsets_to_transaction fails when idle.
    /// 测试在 Idle 状态下 send_offsets_to_transaction 失败。
    #[test]
    fn test_send_offsets_to_transaction_when_idle()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-1");
        let result = producer.send_offsets_to_transaction(vec![], "group");
        assert!(result.is_err());
    }

    /// Test full transaction lifecycle: begin -> send -> commit.
    /// 测试完整事务生命周期：开始 -> 发送 -> 提交。
    #[test]
    fn test_full_lifecycle_commit()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-lifecycle");

        producer.begin_transaction().unwrap();
        producer
            .send_in_transaction("topic", Some("key"), b"value")
            .unwrap();
        producer
            .send_offsets_to_transaction(vec![TransactionOffset::new("src", 0, 50)], "group-1")
            .unwrap();
        producer.commit_transaction().unwrap();

        assert_eq!(producer.state(), &TransactionState::Idle);
        assert_eq!(producer.pending_count(), 0);
        assert_eq!(producer.pending_offset_count(), 0);
    }

    /// Test full transaction lifecycle: begin -> send -> abort.
    /// 测试完整事务生命周期：开始 -> 发送 -> 中止。
    #[test]
    fn test_full_lifecycle_abort()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-lifecycle");

        producer.begin_transaction().unwrap();
        producer
            .send_in_transaction("topic", Some("key"), b"value")
            .unwrap();
        producer.abort_transaction().unwrap();

        assert_eq!(producer.state(), &TransactionState::Idle);
        assert_eq!(producer.pending_count(), 0);
    }

    /// Test multiple sequential transactions.
    /// 测试多个连续事务。
    #[test]
    fn test_multiple_transactions()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-multi");

        // Transaction 1
        producer.begin_transaction().unwrap();
        producer.send_in_transaction("t1", None, b"v1").unwrap();
        producer.commit_transaction().unwrap();

        // Transaction 2
        producer.begin_transaction().unwrap();
        producer.send_in_transaction("t2", None, b"v2").unwrap();
        producer.commit_transaction().unwrap();

        assert_eq!(producer.state(), &TransactionState::Idle);
    }

    /// Test TransactionOffset builder.
    /// 测试 TransactionOffset 构建器。
    #[test]
    fn test_transaction_offset_builder()
    {
        let offset = TransactionOffset::new("topic", 3, 999).with_metadata("my-meta");
        assert_eq!(offset.topic, "topic");
        assert_eq!(offset.partition, 3);
        assert_eq!(offset.offset, 999);
        assert_eq!(offset.metadata, Some("my-meta".to_string()));
    }

    /// Test TransactionOffset without metadata.
    /// 测试不带元数据的 TransactionOffset。
    #[test]
    fn test_transaction_offset_no_metadata()
    {
        let offset = TransactionOffset::new("topic", 0, 0);
        assert!(offset.metadata.is_none());
    }

    /// Test producer clone preserves state.
    /// 测试生产者克隆保留状态。
    #[test]
    fn test_producer_clone()
    {
        let mut producer = TransactionalProducer::new(ProducerConfig::new(), "txn-clone");
        producer.begin_transaction().unwrap();
        producer
            .send_in_transaction("topic", None, b"data")
            .unwrap();

        let cloned = producer.clone();
        assert_eq!(cloned.transactional_id(), "txn-clone");
        assert_eq!(cloned.state(), &TransactionState::Active);
        assert_eq!(cloned.pending_count(), 1);
    }

    /// Test config accessor.
    /// 测试配置访问器。
    #[test]
    fn test_config_accessor()
    {
        let config = ProducerConfig::new();
        let producer = TransactionalProducer::new(config.clone(), "txn-cfg");
        assert_eq!(producer.config().bootstrap_servers, config.bootstrap_servers);
    }
}
