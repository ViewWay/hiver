//! Optimistic locking support for concurrent entity updates
//! 并发实体更新的乐观锁支持
//!
//! # Overview / 概述
//!
//! This module provides optimistic locking abstractions to prevent
//! lost updates when multiple transactions modify the same entity
//! concurrently. This mirrors Spring Data's `@Version` annotation
//! and optimistic locking mechanism.
//! 本模块提供乐观锁抽象，防止多个事务并发修改同一实体时
//! 发生更新丢失。这镜像了 Spring Data 的 `@Version` 注解
//! 和乐观锁机制。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data |
//! |-------|-------------|
//! | `Version` trait | `@Version` annotation |
//! | `OptimisticLockError` | `OptimisticLockingFailureException` |
//! | `Versioned<T>` | Version-managed entity |
//!
//! # How it works / 工作原理
//!
//! 1. Entity carries a version number (typically starting at 0). 实体携带一个版本号（通常从 0
//!    开始）。
//! 2. On read, the version is captured. 读取时捕获版本号。
//! 3. On update, the version is checked: if it has changed since read, the update is rejected with
//!    `OptimisticLockError`. 更新时检查版本号：如果自读取以来版本号已改变， 更新将被拒绝并返回
//!    `OptimisticLockError`。
//! 4. On successful update, the version is incremented. 更新成功后，版本号递增。

use std::fmt;

use crate::Error;

/// Error indicating an optimistic locking failure.
/// 表示乐观锁失败的错误。
///
/// Thrown when an entity's version has changed between read and write,
/// indicating that another transaction has modified the entity concurrently.
/// 当实体的版本在读取和写入之间发生变化时抛出，
/// 表明另一个事务已并发修改了该实体。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::optimistic_lock::OptimisticLockError;
///
/// let err = OptimisticLockError::new("User", "42", 1, 3);
/// assert_eq!(err.type_name(), "User");
/// assert_eq!(err.entity_id(), "42");
/// assert_eq!(err.expected_version(), 1);
/// assert_eq!(err.actual_version(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct OptimisticLockError
{
    /// Entity type name / 实体类型名
    type_name: String,
    /// Entity ID / 实体 ID
    entity_id: String,
    /// Expected version (the version when the entity was read) / 预期版本
    expected_version: i64,
    /// Actual version (the current version in the database) / 实际版本
    actual_version: i64,
}

impl OptimisticLockError
{
    /// Create a new optimistic lock error.
    /// 创建新的乐观锁错误。
    ///
    /// # Parameters / 参数
    ///
    /// - `type_name`: Entity type name / 实体类型名
    /// - `entity_id`: Entity identifier / 实体标识符
    /// - `expected_version`: Version expected by the caller / 调用者期望的版本
    /// - `actual_version`: Current version in the store / 存储中的当前版本
    pub fn new(
        type_name: impl Into<String>,
        entity_id: impl Into<String>,
        expected_version: i64,
        actual_version: i64,
    ) -> Self
    {
        Self {
            type_name: type_name.into(),
            entity_id: entity_id.into(),
            expected_version,
            actual_version,
        }
    }

    /// Get the entity type name.
    /// 获取实体类型名。
    pub fn type_name(&self) -> &str
    {
        &self.type_name
    }

    /// Get the entity ID.
    /// 获取实体 ID。
    pub fn entity_id(&self) -> &str
    {
        &self.entity_id
    }

    /// Get the expected version.
    /// 获取预期版本。
    pub fn expected_version(&self) -> i64
    {
        self.expected_version
    }

    /// Get the actual version.
    /// 获取实际版本。
    pub fn actual_version(&self) -> i64
    {
        self.actual_version
    }
}

impl fmt::Display for OptimisticLockError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "Optimistic lock failure for {}[{}]: expected version {}, but found {}",
            self.type_name, self.entity_id, self.expected_version, self.actual_version
        )
    }
}

impl std::error::Error for OptimisticLockError {}

impl From<OptimisticLockError> for Error
{
    fn from(err: OptimisticLockError) -> Self
    {
        Error::optimistic_locking_failure(err.type_name.clone(), err.entity_id.clone())
    }
}

/// Trait for versioned entities supporting optimistic locking.
/// 支持乐观锁的版本化实体 trait。
///
/// Entities implementing this trait carry a monotonically increasing
/// version number that is checked on update to prevent lost updates.
/// 实现此 trait 的实体携带一个单调递增的版本号，
/// 在更新时进行检查以防止更新丢失。
///
/// This is the Rust equivalent of Spring Data's `@Version` annotation.
/// 这是 Spring Data `@Version` 注解的 Rust 等价物。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::optimistic_lock::Version;
///
/// #[derive(Debug, Clone)]
/// struct VersionedUser
/// {
///     id: i32,
///     name: String,
///     version: i64,
/// }
///
/// impl Version for VersionedUser
/// {
///     fn version(&self) -> i64
///     {
///         self.version
///     }
///
///     fn set_version(&mut self, version: i64)
///     {
///         self.version = version;
///     }
/// }
/// ```
pub trait Version
{
    /// Get the current version number.
    /// 获取当前版本号。
    fn version(&self) -> i64;

    /// Set the version number.
    /// 设置版本号。
    fn set_version(&mut self, version: i64);

    /// Increment the version number by one.
    /// 将版本号递增一。
    fn increment_version(&mut self)
    {
        let current = self.version();
        self.set_version(current + 1);
    }
}

/// A wrapper that pairs an entity with its version for safe updates.
/// 将实体与其版本配对的安全更新包装器。
///
/// This type is used when reading an entity for later update. The version
/// is captured at read time and can be verified at write time.
/// 此类型用于读取实体以供后续更新。版本在读取时捕获，
/// 并可在写入时进行验证。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::optimistic_lock::{OptimisticLockError, Version, Versioned};
///
/// #[derive(Debug, Clone)]
/// struct User
/// {
///     id: i32,
///     name: String,
///     ver: i64,
/// }
///
/// impl Version for User
/// {
///     fn version(&self) -> i64
///     {
///         self.ver
///     }
///
///     fn set_version(&mut self, v: i64)
///     {
///         self.ver = v;
///     }
/// }
///
/// let user = User {
///     id: 1,
///     name: "Alice".into(),
///     ver: 3,
/// };
/// let versioned = Versioned::new(user);
///
/// assert_eq!(versioned.version(), 3);
/// assert_eq!(versioned.entity().name, "Alice");
///
/// // Verify version before update
/// let current_db_version: i64 = 3;
/// versioned
///     .verify_version("User", "1", current_db_version)
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Versioned<T: Version>
{
    /// The versioned entity.
    /// 版本化实体。
    entity: T,
}

impl<T: Version> Versioned<T>
{
    /// Create a new versioned wrapper around an entity.
    /// 创建围绕实体的新版本化包装器。
    pub fn new(entity: T) -> Self
    {
        Self { entity }
    }

    /// Get a reference to the inner entity.
    /// 获取内部实体的引用。
    pub fn entity(&self) -> &T
    {
        &self.entity
    }

    /// Get a mutable reference to the inner entity.
    /// 获取内部实体的可变引用。
    pub fn entity_mut(&mut self) -> &mut T
    {
        &mut self.entity
    }

    /// Get the captured version.
    /// 获取捕获的版本。
    pub fn version(&self) -> i64
    {
        self.entity.version()
    }

    /// Verify that the captured version matches the current database version.
    /// 验证捕获的版本是否匹配当前数据库版本。
    ///
    /// Returns `Ok(())` if versions match, or `Err(OptimisticLockError)` if they differ.
    /// 如果版本匹配返回 `Ok(())`，如果不同返回 `Err(OptimisticLockError)`。
    pub fn verify_version(
        &self,
        type_name: &str,
        entity_id: &str,
        current_db_version: i64,
    ) -> Result<(), OptimisticLockError>
    {
        let expected = self.version();
        if expected == current_db_version
        {
            Ok(())
        }
        else
        {
            Err(OptimisticLockError::new(type_name, entity_id, expected, current_db_version))
        }
    }

    /// Consume the wrapper, increment the version, and return the entity.
    /// 消费包装器，递增版本，并返回实体。
    ///
    /// Call this after a successful update to prepare for future updates.
    /// 在成功更新后调用此方法，为未来的更新做准备。
    pub fn into_updated(mut self) -> T
    {
        self.entity.increment_version();
        self.entity
    }

    /// Consume the wrapper and return the inner entity without modification.
    /// 消费包装器并返回内部实体，不做修改。
    pub fn into_inner(self) -> T
    {
        self.entity
    }
}

/// Trait for performing a version-checked update.
/// 执行版本检查更新的 trait。
///
/// Implementors provide a way to atomically compare-and-set
/// the entity version in the data store.
/// 实现者提供在数据存储中原子化比较并设置实体版本的方法。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::optimistic_lock::VersionCheckedUpdate;
///
/// async fn save_versioned(&self, versioned: Versioned<User>) -> Result<User, Error> {
///     let entity = versioned.entity();
///     let expected = versioned.version();
///     // UPDATE users SET ... WHERE id = ? AND version = ?
///     let rows = db.execute("UPDATE users SET name = ?, version = version + 1 WHERE id = ? AND version = ?",
///         &[&entity.name, &entity.id, &expected]).await?;
///     if rows == 0 {
///         return Err(OptimisticLockError::new("User", &entity.id.to_string(), expected, ...).into());
///     }
///     Ok(versioned.into_updated())
/// }
/// ```
pub trait VersionCheckedUpdate<T: Version>
{
    /// Error type for the update operation.
    /// 更新操作的错误类型。
    type Error: std::fmt::Debug + Send + Sync;

    /// Attempt to update the entity only if the version matches.
    /// 仅当版本匹配时尝试更新实体。
    ///
    /// Returns `Ok(updated_entity)` on success, or `Err` if the version
    /// has changed (optimistic lock failure).
    /// 成功返回 `Ok(updated_entity)`，如果版本已改变返回 `Err`（乐观锁失败）。
    fn update_if_version_matches(&self, versioned: Versioned<T>) -> Result<T, Self::Error>;
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use super::*;

    #[derive(Debug, Clone)]
    struct TestUser
    {
        id: i32,
        name: String,
        ver: i64,
    }

    impl Version for TestUser
    {
        fn version(&self) -> i64
        {
            self.ver
        }

        fn set_version(&mut self, v: i64)
        {
            self.ver = v;
        }
    }

    fn make_user(id: i32, name: &str, ver: i64) -> TestUser
    {
        TestUser {
            id,
            name: name.to_string(),
            ver,
        }
    }

    #[test]
    fn test_version_trait()
    {
        let mut user = make_user(1, "Alice", 0);
        assert_eq!(user.version(), 0);
        user.increment_version();
        assert_eq!(user.version(), 1);
        user.set_version(5);
        assert_eq!(user.version(), 5);
    }

    #[test]
    fn test_versioned_new()
    {
        let user = make_user(1, "Alice", 3);
        let versioned = Versioned::new(user);
        assert_eq!(versioned.version(), 3);
        assert_eq!(versioned.entity().name, "Alice");
    }

    #[test]
    fn test_versioned_verify_success()
    {
        let user = make_user(1, "Alice", 3);
        let versioned = Versioned::new(user);
        let result = versioned.verify_version("User", "1", 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_versioned_verify_failure()
    {
        let user = make_user(1, "Alice", 3);
        let versioned = Versioned::new(user);
        let result = versioned.verify_version("User", "1", 5);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.type_name(), "User");
        assert_eq!(err.entity_id(), "1");
        assert_eq!(err.expected_version(), 3);
        assert_eq!(err.actual_version(), 5);
    }

    #[test]
    fn test_versioned_into_updated()
    {
        let user = make_user(1, "Alice", 3);
        let versioned = Versioned::new(user);
        let updated = versioned.into_updated();
        assert_eq!(updated.version(), 4);
    }

    #[test]
    fn test_versioned_into_inner()
    {
        let user = make_user(1, "Alice", 3);
        let versioned = Versioned::new(user);
        let inner = versioned.into_inner();
        assert_eq!(inner.version(), 3);
    }

    #[test]
    fn test_versioned_entity_mut()
    {
        let user = make_user(1, "Alice", 3);
        let mut versioned = Versioned::new(user);
        versioned.entity_mut().name = "Bob".to_string();
        assert_eq!(versioned.entity().name, "Bob");
    }

    #[test]
    fn test_optimistic_lock_error_display()
    {
        let err = OptimisticLockError::new("User", "42", 1, 3);
        let msg = err.to_string();
        assert!(msg.contains("User"));
        assert!(msg.contains("42"));
        assert!(msg.contains("1"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn test_optimistic_lock_error_into_data_error()
    {
        let err = OptimisticLockError::new("User", "42", 1, 3);
        let data_err: Error = err.into();
        assert!(matches!(data_err, Error::OptimisticLockingFailure { .. }));
    }
}
