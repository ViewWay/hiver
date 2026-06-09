//! Migration information and history
//! 迁移信息和历史

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{Checksum, Description, MigratedVersion, MigrationType, Version};

/// Migration entry in history
/// 历史中的迁移条目
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEntry
{
    /// Installed rank
    /// 安装排名
    pub installed_rank: i32,

    /// Version
    /// 版本
    pub version: Version,

    /// Description
    /// 描述
    pub description: Description,

    /// Migration type
    /// 迁移类型
    pub migration_type: MigrationType,

    /// Checksum
    /// 校验和
    pub checksum: Option<Checksum>,

    /// Installed by
    /// 安装者
    pub installed_by: Option<String>,

    /// Installation time
    /// 安装时间
    pub installed_on: Option<DateTime<Utc>>,

    /// Execution time in milliseconds
    /// 执行时间（毫秒）
    pub execution_time: i64,

    /// Success flag
    /// 是否成功
    pub success: bool,
}

impl MigrationEntry
{
    /// Create a new migration entry
    /// 创建新的迁移条目
    pub fn new(
        version: Version,
        description: Description,
        migration_type: MigrationType,
        checksum: Option<Checksum>,
    ) -> Self
    {
        Self {
            installed_rank: 0,
            version,
            description,
            migration_type,
            checksum,
            installed_by: None,
            installed_on: None,
            execution_time: 0,
            success: true,
        }
    }
}

/// Migration information
/// 迁移信息
#[derive(Debug, Clone)]
pub struct Info
{
    /// Schema history table exists
    /// 历史表是否存在
    pub schema_exists: bool,

    /// Current schema version
    /// 当前架构版本
    pub current_version: Option<Version>,

    /// All migrations
    /// 所有迁移
    pub all: Vec<MigrationEntry>,

    /// Pending migrations
    /// 待执行迁移
    pub pending: Vec<MigrationEntry>,

    /// Applied migrations
    /// 已应用迁移
    pub applied: HashMap<Version, MigrationEntry>,
}

impl Info
{
    /// Create new info
    /// 创建新信息
    pub fn new() -> Self
    {
        Self {
            schema_exists: false,
            current_version: None,
            all: Vec::new(),
            pending: Vec::new(),
            applied: HashMap::new(),
        }
    }

    /// Get all migrations
    /// 获取所有迁移
    pub fn all(&self) -> &[MigrationEntry]
    {
        &self.all
    }

    /// Get pending migrations
    /// 获取待执行迁移
    pub fn pending(&self) -> &[MigrationEntry]
    {
        &self.pending
    }

    /// Get applied migrations
    /// 获取已应用迁移
    pub fn applied(&self) -> Vec<&MigrationEntry>
    {
        self.applied.values().collect()
    }

    /// Check if migration is applied
    /// 检查迁移是否已应用
    pub fn is_applied(&self, version: &Version) -> bool
    {
        self.applied.contains_key(version)
    }

    /// Get current version
    /// 获取当前版本
    pub fn current_version(&self) -> Option<&Version>
    {
        self.current_version.as_ref()
    }

    /// Get number of pending migrations
    /// 获取待执行迁移数量
    pub fn pending_count(&self) -> usize
    {
        self.pending.len()
    }

    /// Get number of applied migrations
    /// 获取已应用迁移数量
    pub fn applied_count(&self) -> usize
    {
        self.applied.len()
    }
}

impl Default for Info
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Migration result after execution
/// 迁移执行结果
#[derive(Debug, Clone)]
pub struct MigrationResult
{
    /// Migrations executed
    /// 执行的迁移
    pub migrations_executed: Vec<MigratedVersion>,

    /// Target version reached
    /// 目标版本
    pub target_version: Option<Version>,

    /// Total execution time
    /// 总执行时间
    pub total_execution_time_ms: i64,

    /// Success flag
    /// 是否成功
    pub success: bool,

    /// Warning messages
    /// 警告信息
    pub warnings: Vec<String>,
}

impl MigrationResult
{
    /// Create a new successful result
    /// 创建新的成功结果
    pub fn success(migrations: Vec<MigratedVersion>, target: Option<Version>, time_ms: i64)
    -> Self
    {
        Self {
            migrations_executed: migrations,
            target_version: target,
            total_execution_time_ms: time_ms,
            success: true,
            warnings: Vec::new(),
        }
    }

    /// Create a new failed result
    /// 创建新的失败结果
    pub fn failed(error: String) -> Self
    {
        Self {
            migrations_executed: Vec::new(),
            target_version: None,
            total_execution_time_ms: 0,
            success: false,
            warnings: vec![error],
        }
    }

    /// Add a warning
    /// 添加警告
    pub fn add_warning(&mut self, warning: String)
    {
        self.warnings.push(warning);
    }

    /// Get number of executed migrations
    /// 获取执行的迁移数量
    pub fn executed_count(&self) -> usize
    {
        self.migrations_executed.len()
    }

    /// Check if warnings exist
    /// 检查是否有警告
    pub fn has_warnings(&self) -> bool
    {
        !self.warnings.is_empty()
    }
}

/// Baseline information
/// 基线信息
#[derive(Debug, Clone)]
pub struct BaselineInfo
{
    /// Baseline version
    /// 基线版本
    pub version: Version,

    /// Baseline description
    /// 基线描述
    pub description: Option<String>,

    /// Baseline applied at
    /// 基线应用时间
    pub applied_at: Option<DateTime<Utc>>,
}

impl BaselineInfo
{
    /// Create new baseline info
    /// 创建新的基线信息
    pub fn new(version: Version) -> Self
    {
        Self {
            version,
            description: None,
            applied_at: None,
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    #[test]
    fn test_migration_entry_creation()
    {
        let entry = MigrationEntry::new(
            "V1".to_string(),
            "Create users table".to_string(),
            MigrationType::SQL,
            Some(12345),
        );

        assert_eq!(entry.version, "V1");
        assert_eq!(entry.description, "Create users table");
        assert_eq!(entry.migration_type, MigrationType::SQL);
        assert!(entry.success);
    }

    #[test]
    fn test_info_pending_count()
    {
        let mut info = Info::new();
        info.pending.push(MigrationEntry::new(
            "V2".to_string(),
            "Add email column".to_string(),
            MigrationType::SQL,
            None,
        ));

        assert_eq!(info.pending_count(), 1);
        assert_eq!(info.applied_count(), 0);
    }

    #[test]
    fn test_migration_result_success()
    {
        let migrations = vec![MigratedVersion {
            version: "V1".to_string(),
            description: "Initial schema".to_string(),
            execution_time_ms: 100,
            success: true,
        }];

        let result = MigrationResult::success(migrations, Some("V1".to_string()), 100);

        assert!(result.success);
        assert_eq!(result.executed_count(), 1);
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_migration_result_failed()
    {
        let result = MigrationResult::failed("Connection lost".to_string());

        assert!(!result.success);
        assert!(result.has_warnings());
        assert_eq!(result.executed_count(), 0);
    }

    #[test]
    fn test_baseline_info()
    {
        let baseline = BaselineInfo::new("V0".to_string());
        assert_eq!(baseline.version, "V0");
        assert!(baseline.description.is_none());
        assert!(baseline.applied_at.is_none());
    }
}
