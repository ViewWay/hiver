//! Partition handling for parallel step execution.
//! 分区处理，用于并行步骤执行。
//!
//! Equivalent to Spring Batch's `PartitionHandler` and `Partitioner`.
//! 等价于 Spring Batch 的 PartitionHandler 和 Partitioner。

use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    error::{BatchError, BatchResult},
    execution::{BatchStatus, ExitStatus, StepExecution},
};

/// Metadata for a single partition.
/// 单个分区的元数据。
#[derive(Debug, Clone)]
pub struct PartitionInfo
{
    /// Partition name (e.g. "partition-0").
    pub name: String,
    /// Grid size hint.
    pub grid_size: usize,
    /// Extra context data for the partition worker.
    pub context: HashMap<String, String>,
}

impl PartitionInfo
{
    /// Create a new partition info.
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            grid_size: 1,
            context: HashMap::new(),
        }
    }

    /// Set grid size.
    pub fn with_grid_size(mut self, size: usize) -> Self
    {
        self.grid_size = size;
        self
    }

    /// Add context data.
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Partitioner creates partition metadata for a step.
/// 分区器为步骤创建分区元数据。
#[async_trait]
pub trait Partitioner: Send + Sync
{
    /// Create partitions with the given grid size.
    async fn partition(&self, grid_size: usize) -> BatchResult<Vec<PartitionInfo>>;
}

/// Range-based partitioner — splits `[0..total)` into `grid_size` chunks.
/// 基于范围的分区器 —— 将范围拆分为多个块。
pub struct RangePartitioner
{
    /// Total number of items.
    pub total: usize,
}

impl RangePartitioner
{
    /// Create a new range partitioner.
    pub fn new(total: usize) -> Self
    {
        Self { total }
    }
}

#[async_trait]
impl Partitioner for RangePartitioner
{
    async fn partition(&self, grid_size: usize) -> BatchResult<Vec<PartitionInfo>>
    {
        if grid_size == 0
        {
            return Err(BatchError::Other("grid_size must be > 0".into()));
        }
        let chunk = self.total.div_ceil(grid_size);
        let mut partitions = Vec::with_capacity(grid_size);
        let mut start = 0usize;
        for i in 0..grid_size
        {
            let end = (start + chunk).min(self.total);
            if start >= self.total
            {
                break;
            }
            partitions.push(
                PartitionInfo::new(format!("partition-{i}"))
                    .with_context("start", start.to_string())
                    .with_context("end", end.to_string()),
            );
            start = end;
        }
        Ok(partitions)
    }
}

/// Result of a single partition execution.
/// 单个分区执行的结果。
#[derive(Debug, Clone)]
pub struct PartitionResult
{
    /// Partition name.
    pub name: String,
    /// Execution status.
    pub status: BatchStatus,
    /// Exit status.
    pub exit_status: ExitStatus,
    /// Step execution details.
    pub step_execution: StepExecution,
}

/// Handler that executes a single partitioned step.
/// 执行单个分区步骤的处理器。
#[async_trait]
pub trait PartitionHandler: Send + Sync
{
    /// Execute a single partition and return its result.
    async fn execute_partition(&self, partition: &PartitionInfo) -> BatchResult<PartitionResult>;
}

/// Collects results from all partitions.
/// 收集所有分区的结果。
#[derive(Debug, Default)]
pub struct PartitionCollector
{
    results: Vec<PartitionResult>,
}

impl PartitionCollector
{
    /// Create a new collector.
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add a partition result.
    pub fn add(&mut self, result: PartitionResult)
    {
        self.results.push(result);
    }

    /// Number of completed partitions.
    pub fn count(&self) -> usize
    {
        self.results.len()
    }

    /// Check if all partitions completed successfully.
    pub fn all_completed(&self) -> bool
    {
        self.results
            .iter()
            .all(|r| r.status == BatchStatus::Completed)
    }

    /// Aggregate exit status — COMPLETED if all succeeded, FAILED if any failed.
    pub fn aggregate_status(&self) -> ExitStatus
    {
        if self.all_completed()
        {
            ExitStatus::completed()
        }
        else
        {
            ExitStatus::failed()
        }
    }

    /// Total read count across all partitions.
    pub fn total_read_count(&self) -> usize
    {
        self.results
            .iter()
            .map(|r| r.step_execution.read_count())
            .sum()
    }

    /// Total write count across all partitions.
    pub fn total_write_count(&self) -> usize
    {
        self.results
            .iter()
            .map(|r| r.step_execution.write_count())
            .sum()
    }
}

#[cfg(test)]
mod tests
{
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn test_range_partitioner()
    {
        let partitioner = RangePartitioner::new(100);
        let partitions = partitioner.partition(4).await.unwrap();
        assert_eq!(partitions.len(), 4);
        assert_eq!(partitions[0].context.get("start").unwrap(), "0");
        assert_eq!(partitions[0].context.get("end").unwrap(), "25");
    }

    #[tokio::test]
    async fn test_range_partitioner_uneven()
    {
        let partitioner = RangePartitioner::new(10);
        let partitions = partitioner.partition(3).await.unwrap();
        assert_eq!(partitions.len(), 3);
    }

    #[tokio::test]
    async fn test_range_partitioner_zero_grid()
    {
        let partitioner = RangePartitioner::new(10);
        assert!(partitioner.partition(0).await.is_err());
    }

    #[tokio::test]
    async fn test_range_partitioner_more_grids_than_items()
    {
        let partitioner = RangePartitioner::new(2);
        let partitions = partitioner.partition(5).await.unwrap();
        assert_eq!(partitions.len(), 2);
    }

    #[test]
    fn test_partition_collector_mixed()
    {
        let mut collector = PartitionCollector::new();
        let se1 = StepExecution::new("p0", Uuid::new_v4());
        let mut se2 = StepExecution::new("p1", Uuid::new_v4());
        se2.set_status(BatchStatus::Failed);

        collector.add(PartitionResult {
            name: "p0".into(),
            status: BatchStatus::Completed,
            exit_status: ExitStatus::completed(),
            step_execution: se1,
        });
        collector.add(PartitionResult {
            name: "p1".into(),
            status: BatchStatus::Failed,
            exit_status: ExitStatus::failed(),
            step_execution: se2,
        });

        assert_eq!(collector.count(), 2);
        assert!(!collector.all_completed());
        assert_eq!(collector.aggregate_status().code, "FAILED");
    }

    #[test]
    fn test_partition_collector_all_success()
    {
        let mut collector = PartitionCollector::new();
        collector.add(PartitionResult {
            name: "p0".into(),
            status: BatchStatus::Completed,
            exit_status: ExitStatus::completed(),
            step_execution: StepExecution::new("p0", Uuid::new_v4()),
        });
        assert!(collector.all_completed());
        assert_eq!(collector.aggregate_status().code, "COMPLETED");
    }
}
