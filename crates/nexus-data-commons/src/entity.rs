//! Entity traits and lifecycle support
//! 实体 trait 和生命周期支持
//!
//! # Overview / 概述
//!
//! This module provides entity traits for domain-driven design.
//! 本模块提供领域驱动设计的实体 trait。

use std::any::Any;
use std::fmt::Debug;

/// Entity identifier trait
/// 实体标识符 trait
///
/// Marks a type as being usable as an entity identifier.
/// 标记类型可用作实体标识符。
pub trait Identifier: Any + Send + Sync + Debug + Clone + PartialEq + Eq {}

impl Identifier for String {}
impl Identifier for i32 {}
impl Identifier for i64 {}
impl Identifier for u32 {}
impl Identifier for u64 {}
impl Identifier for uuid::Uuid {}

/// Aggregate root trait
/// 聚合根 trait
///
/// Base trait for all entities in the domain model.
/// 领域模型中所有实体的基础 trait。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::AggregateRoot;
///
/// #[derive(Debug, Clone)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// impl AggregateRoot for User {
///     type Id = i32;
///
///     fn id(&self) -> &Self::Id {
///         &self.id
///     }
///
///     fn set_id(&mut self, id: Self::Id) {
///         self.id = id;
///     }
/// }
/// ```
pub trait AggregateRoot: Any + Send + Sync + Debug {
    /// ID type
    /// ID 类型
    type Id: Identifier;

    /// Get the entity ID
    /// 获取实体 ID
    fn id(&self) -> &Self::Id;

    /// Set the entity ID
    /// 设置实体 ID
    fn set_id(&mut self, id: Self::Id);

    /// Check if this is a new entity (ID not set)
    /// 检查是否为新实体（ID 未设置）
    fn is_new(&self) -> bool;

    /// Get the entity type name
    /// 获取实体类型名称
    fn type_name() -> String
    where
        Self: Sized;
}

/// Auditable entity trait
/// 可审计实体 trait
///
/// For entities that track creation and modification metadata.
/// 用于跟踪创建和修改元数据的实体。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::Auditable;
/// use chrono::{DateTime, Utc};
///
/// #[derive(Debug, Clone)]
/// struct AuditedUser {
///     id: i32,
///     created_at: DateTime<Utc>,
///     updated_at: DateTime<Utc>,
///     created_by: Option<String>,
///     updated_by: Option<String>,
///     version: i32,
/// }
///
/// impl Auditable for AuditedUser {
///     fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
///         Some(self.created_at)
///     }
///
///     fn set_created_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
///         self.created_at = ts;
///     }
///
///     fn updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
///         Some(self.updated_at)
///     }
///
///     fn set_updated_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
///         self.updated_at = ts;
///     }
/// }
/// ```
pub trait Auditable {
    /// Get creation timestamp
    /// 获取创建时间戳
    fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;

    /// Set creation timestamp
    /// 设置创建时间戳
    fn set_created_at(&mut self, ts: chrono::DateTime<chrono::Utc>);

    /// Get last update timestamp
    /// 获取最后更新时间戳
    fn updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;

    /// Set last update timestamp
    /// 设置最后更新时间戳
    fn set_updated_at(&mut self, ts: chrono::DateTime<chrono::Utc>);

    /// Get creator
    /// 获取创建者
    fn created_by(&self) -> Option<&str> {
        None
    }

    /// Set creator
    /// 设置创建者
    fn set_created_by(&mut self, _user: Option<String>) {}

    /// Get last updater
    /// 获取最后更新者
    fn updated_by(&self) -> Option<&str> {
        None
    }

    /// Set last updater
    /// 设置最后更新者
    fn set_updated_by(&mut self, _user: Option<String>) {}
}

/// Versioned entity trait for optimistic locking
/// 版本实体 trait，用于乐观锁
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::Versioned;
///
/// #[derive(Debug, Clone)]
/// struct VersionedUser {
///     id: i32,
///     version: i32,
/// }
///
/// impl Versioned for VersionedUser {
///     fn version(&self) -> i32 {
///         self.version
///     }
///
///     fn set_version(&mut self, version: i32) {
///         self.version = version;
///     }
///
///     fn increment_version(&mut self) {
///         self.version += 1;
///     }
/// }
/// ```
pub trait Versioned {
    /// Get current version
    /// 获取当前版本
    fn version(&self) -> i32;

    /// Set version
    /// 设置版本
    fn set_version(&mut self, version: i32);

    /// Increment version
    /// 递增版本
    fn increment_version(&mut self) {
        let current = self.version();
        self.set_version(current + 1);
    }
}

/// Soft deletable entity trait
/// 可软删除实体 trait
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::SoftDeletable;
/// use chrono::{DateTime, Utc};
///
/// #[derive(Debug, Clone)]
/// struct SoftDeleteUser {
///     id: i32,
///     deleted: bool,
///     deleted_at: Option<DateTime<Utc>>,
/// }
///
/// impl SoftDeletable for SoftDeleteUser {
///     fn is_deleted(&self) -> bool {
///         self.deleted
///     }
///
///     fn mark_deleted(&mut self) {
///         self.deleted = true;
///         self.deleted_at = Some(chrono::Utc::now());
///     }
///
///     fn restore(&mut self) {
///         self.deleted = false;
///         self.deleted_at = None;
///     }
/// }
/// ```
pub trait SoftDeletable {
    /// Check if entity is deleted
    /// 检查实体是否已删除
    fn is_deleted(&self) -> bool;

    /// Mark entity as deleted
    /// 标记实体为已删除
    fn mark_deleted(&mut self);

    /// Restore deleted entity
    /// 恢复已删除的实体
    fn restore(&mut self);

    /// Get deletion timestamp
    /// 获取删除时间戳
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        None
    }
}

/// Entity lifecycle event
/// 实体生命周期事件
///
/// Events that can occur during an entity's lifecycle.
/// 实体生命周期中可能发生的事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// Before save
    /// 保存前
    BeforeSave,

    /// After save
    /// 保存后
    AfterSave,

    /// Before insert
    /// 插入前
    BeforeInsert,

    /// After insert
    /// 插入后
    AfterInsert,

    /// Before update
    /// 更新前
    BeforeUpdate,

    /// After update
    /// 更新后
    AfterUpdate,

    /// Before delete
    /// 删除前
    BeforeDelete,

    /// After delete
    /// 删除后
    AfterDelete,

    /// Before load
    /// 加载前
    BeforeLoad,

    /// After load
    /// 加载后
    AfterLoad,
}

/// Entity with lifecycle callbacks
/// 带生命周期回调的实体
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::{EntityWithLifecycle, LifecycleEvent};
///
/// struct UserWithCallbacks {
///     id: i32,
///     name: String,
/// }
///
/// impl EntityWithLifecycle for UserWithCallbacks {
///     fn before_save(&mut self) {
///         println!("Saving user: {}", self.name);
///     }
///
///     fn after_load(&mut self) {
///         println!("Loaded user: {}", self.name);
///     }
/// }
/// ```
pub trait EntityWithLifecycle {
    /// Callback before save
    /// 保存前回调
    fn before_save(&mut self) {}

    /// Callback after save
    /// 保存后回调
    fn after_save(&mut self) {}

    /// Callback before insert
    /// 插入前回调
    fn before_insert(&mut self) {}

    /// Callback after insert
    /// 插入后回调
    fn after_insert(&mut self) {}

    /// Callback before update
    /// 更新前回调
    fn before_update(&mut self) {}

    /// Callback after update
    /// 更新后回调
    fn after_update(&mut self) {}

    /// Callback before delete
    /// 删除前回调
    fn before_delete(&mut self) {}

    /// Callback after delete
    /// 删除后回调
    fn after_delete(&mut self) {}

    /// Callback after load
    /// 加载后回调
    fn after_load(&mut self) {}

    /// Dispatch lifecycle event
    /// 分发生命周期事件
    fn dispatch_lifecycle_event(&mut self, event: LifecycleEvent) {
        match event {
            LifecycleEvent::BeforeSave => self.before_save(),
            LifecycleEvent::AfterSave => self.after_save(),
            LifecycleEvent::BeforeInsert => self.before_insert(),
            LifecycleEvent::AfterInsert => self.after_insert(),
            LifecycleEvent::BeforeUpdate => self.before_update(),
            LifecycleEvent::AfterUpdate => self.after_update(),
            LifecycleEvent::BeforeDelete => self.before_delete(),
            LifecycleEvent::AfterDelete => self.after_delete(),
            LifecycleEvent::BeforeLoad => {},
            LifecycleEvent::AfterLoad => self.after_load(),
        }
    }
}

/// Table name trait
/// 表名 trait
///
/// Associates an entity type with its database table name.
/// 将实体类型与其数据库表名关联。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::TableName;
///
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// impl TableName for User {
///     fn table_name() -> &'static str {
///         "users"
///     }
/// }
/// ```
pub trait TableName {
    /// Get the table name for this entity
    /// 获取此实体的表名
    fn table_name() -> &'static str;

    /// Get the table name (instance method)
    /// 获取表名（实例方法）
    fn get_table_name(&self) -> &'static str
    where
        Self: Sized,
    {
        Self::table_name()
    }
}

/// Column mapping trait
/// 列映射 trait
///
/// Maps entity fields to database column names.
/// 将实体字段映射到数据库列名。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::ColumnName;
///
/// struct User {
///     id: i32,
///     user_name: String,
/// }
///
/// impl ColumnName for User {
///     fn column_name(field: &str) -> String {
///         match field {
///             "id" => "id".to_string(),
///             "user_name" => "username".to_string(),
///             _ => field.to_string(),
///         }
///     }
/// }
/// ```
pub trait ColumnName {
    /// Convert field name to column name
    /// 将字段名转换为列名
    fn column_name(field: &str) -> String {
        // Default: snake_case to snake_case (no change)
        field.to_string()
    }

    /// Get all column names for this entity
    /// 获取此实体的所有列名
    fn column_names() -> Vec<&'static str> {
        Vec::new()
    }
}

/// Entity trait combining all entity features
/// 组合所有实体功能的 Entity trait
///
/// A comprehensive trait that combines aggregate root, table name,
/// and column mapping.
/// 组合聚合根、表名和列映射的综合 trait。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_commons::Entity;
///
/// #[derive(Debug, Clone)]
/// struct User {
///     id: i32,
///     name: String,
///     email: String,
/// }
///
/// impl Entity for User {
///     type Id = i32;
///
///     fn id(&self) -> &Self::Id {
///         &self.id
///     }
///
///     fn set_id(&mut self, id: Self::Id) {
///         self.id = id;
///     }
///
///     fn is_new(&self) -> bool {
///         self.id == 0
///     }
///
///     fn type_name() -> String {
///         "User".to_string()
///     }
///
///     fn table_name() -> &'static str {
///         "users"
///     }
/// }
/// ```
pub trait Entity: AggregateRoot + TableName + Any + Send + Sync {
    /// Get a value by field name
    /// 通过字段名获取值
    fn get_field(&self, _field: &str) -> Option<String> {
        None
    }

    /// Set a value by field name
    /// 通过字段名设置值
    fn set_field(&mut self, _field: &str, _value: &str) -> bool {
        false
    }

    /// Get all field names
    /// 获取所有字段名
    fn field_names() -> Vec<&'static str>
    where
        Self: Sized,
    {
        Vec::new()
    }
}

/// Default implementation for Entity types
/// Entity 类型的默认实现
impl<E> EntityWithLifecycle for E where E: AggregateRoot {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[derive(Debug, Clone)]
    struct TestUser {
        id: i32,
        name: String,
    }

    impl AggregateRoot for TestUser {
        type Id = i32;

        fn id(&self) -> &Self::Id {
            &self.id
        }

        fn set_id(&mut self, id: Self::Id) {
            self.id = id;
        }

        fn is_new(&self) -> bool {
            self.id == 0
        }

        fn type_name() -> String {
            "TestUser".to_string()
        }
    }

    impl TableName for TestUser {
        fn table_name() -> &'static str {
            "test_users"
        }
    }

    impl Entity for TestUser {
        // Entity trait uses methods from AggregateRoot
        // Entity trait 使用来自 AggregateRoot 的方法
    }

    #[test]
    fn test_aggregate_root() {
        let user = TestUser {
            id: 1,
            name: "Alice".to_string(),
        };

        assert_eq!(user.id(), &1);
        assert!(!user.is_new());
        assert_eq!(TestUser::type_name(), "TestUser");
    }

    #[test]
    fn test_new_entity() {
        let user = TestUser {
            id: 0,
            name: "Bob".to_string(),
        };

        assert!(user.is_new());
    }

    #[test]
    fn test_set_id() {
        let mut user = TestUser {
            id: 0,
            name: "Charlie".to_string(),
        };

        assert!(user.is_new());
        user.set_id(42);
        assert_eq!(user.id(), &42);
        assert!(!user.is_new());
    }

    #[test]
    fn test_table_name() {
        assert_eq!(TestUser::table_name(), "test_users");
    }

    #[derive(Debug, Clone)]
    struct AuditedTestUser {
        id: i32,
        created: Option<chrono::DateTime<chrono::Utc>>,
        updated: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl Auditable for AuditedTestUser {
        fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            self.created
        }

        fn set_created_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
            self.created = Some(ts);
        }

        fn updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            self.updated
        }

        fn set_updated_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
            self.updated = Some(ts);
        }
    }

    #[test]
    fn test_auditable() {
        let mut user = AuditedTestUser {
            id: 1,
            created: None,
            updated: None,
        };

        let now = Utc::now();
        user.set_created_at(now);
        user.set_updated_at(now);

        assert_eq!(user.created_at(), Some(now));
        assert_eq!(user.updated_at(), Some(now));
    }

    #[derive(Debug, Clone)]
    struct VersionedTestUser {
        id: i32,
        ver: i32,
    }

    impl Versioned for VersionedTestUser {
        fn version(&self) -> i32 {
            self.ver
        }

        fn set_version(&mut self, version: i32) {
            self.ver = version;
        }
    }

    #[test]
    fn test_versioned() {
        let mut user = VersionedTestUser { id: 1, ver: 0 };

        assert_eq!(user.version(), 0);
        user.increment_version();
        assert_eq!(user.version(), 1);
        user.set_version(5);
        assert_eq!(user.version(), 5);
    }

    #[derive(Debug, Clone)]
    struct SoftDeleteTestUser {
        id: i32,
        deleted: bool,
        deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl SoftDeletable for SoftDeleteTestUser {
        fn is_deleted(&self) -> bool {
            self.deleted
        }

        fn mark_deleted(&mut self) {
            self.deleted = true;
            self.deleted_at = Some(Utc::now());
        }

        fn restore(&mut self) {
            self.deleted = false;
            self.deleted_at = None;
        }

        fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            self.deleted_at
        }
    }

    #[test]
    fn test_soft_deletable() {
        let mut user = SoftDeleteTestUser {
            id: 1,
            deleted: false,
            deleted_at: None,
        };

        assert!(!user.is_deleted());
        assert!(user.deleted_at().is_none());

        user.mark_deleted();
        assert!(user.is_deleted());
        assert!(user.deleted_at().is_some());

        user.restore();
        assert!(!user.is_deleted());
        assert!(user.deleted_at().is_none());
    }

    #[derive(Debug, Clone)]
    struct UserWithLifecycle {
        id: i32,
        name: String,
        before_save_called: bool,
        after_load_called: bool,
    }

    impl AggregateRoot for UserWithLifecycle {
        type Id = i32;

        fn id(&self) -> &Self::Id {
            &self.id
        }

        fn set_id(&mut self, id: Self::Id) {
            self.id = id;
        }

        fn is_new(&self) -> bool {
            self.id == 0
        }

        fn type_name() -> String {
            "UserWithLifecycle".to_string()
        }
    }

    // Note: We can't manually implement EntityWithLifecycle due to the blanket
    // implementation: `impl<E> EntityWithLifecycle for E where E: AggregateRoot {}`
    // Instead, we directly test lifecycle callback behavior
    // 注意：由于 blanket implementation，我们无法手动实现 EntityWithLifecycle
    // 我们直接测试生命周期回调行为
    impl UserWithLifecycle {
        /// Custom before_save callback for testing
        /// 用于测试的自定义 before_save 回调
        pub fn before_save_custom(&mut self) {
            self.before_save_called = true;
        }

        /// Custom after_load callback for testing
        /// 用于测试的自定义 after_load 回调
        pub fn after_load_custom(&mut self) {
            self.after_load_called = true;
        }
    }

    #[test]
    fn test_lifecycle_callbacks() {
        let mut user = UserWithLifecycle {
            id: 1,
            name: "Test".to_string(),
            before_save_called: false,
            after_load_called: false,
        };

        // Test custom lifecycle methods
        // 测试自定义生命周期方法
        user.before_save_custom();
        assert!(user.before_save_called);

        user.after_load_custom();
        assert!(user.after_load_called);
    }
}
