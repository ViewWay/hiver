//! Auditing support for entity lifecycle tracking
//! 实体生命周期跟踪的审计支持
//!
//! # Overview / 概述
//!
//! This module provides auditing annotations, traits, and a handler
//! for automatically tracking who created or last modified an entity
//! and when. This mirrors Spring Data's `@CreatedDate`, `@LastModifiedDate`,
//! `@CreatedBy`, `@LastModifiedBy`, and `AuditorAware<T>` abstractions.
//! 本模块提供审计注解、trait 和处理器，用于自动跟踪实体的创建者、
//! 最后修改者以及时间。这镜像了 Spring Data 的
//! `@CreatedDate`、`@LastModifiedDate`、`@CreatedBy`、`@LastModifiedBy`
//! 和 `AuditorAware<T>` 抽象。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data |
//! |-------|-------------|
//! | `CreatedDate` | `@CreatedDate` |
//! | `LastModifiedDate` | `@LastModifiedDate` |
//! | `CreatedBy` | `@CreatedBy` |
//! | `LastModifiedBy` | `@LastModifiedBy` |
//! | `AuditorAware<T>` | `AuditorAware<T>` |
//! | `AuditingHandler` | `AuditingHandler` / `IsNewAwareAuditingHandler` |

use chrono::{DateTime, Utc};

/// Marker trait for entities with a created-date field.
/// 带有创建日期字段的实体的标记 trait。
///
/// Annotate entity fields that should be automatically populated
/// with the creation timestamp.
/// 标注应自动填充创建时间戳的实体字段。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::auditing::CreatedDate;
///
/// struct User {
///     created_at: DateTime<Utc>,  // impl CreatedDate
/// }
/// ```
pub trait CreatedDate {}

/// Marker trait for entities with a last-modified-date field.
/// 带有最后修改日期字段的实体的标记 trait。
///
/// Annotate entity fields that should be automatically updated
/// with the modification timestamp.
/// 标注应自动更新修改时间戳的实体字段。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::auditing::LastModifiedDate;
///
/// struct User {
///     updated_at: DateTime<Utc>,  // impl LastModifiedDate
/// }
/// ```
pub trait LastModifiedDate {}

/// Marker trait for entities with a created-by field.
/// 带有创建者字段的实体的标记 trait。
///
/// Annotate entity fields that should be automatically populated
/// with the current auditor (e.g., user ID or username).
/// 标注应自动填充当前审计者（如用户 ID 或用户名）的实体字段。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::auditing::CreatedBy;
///
/// struct User {
///     created_by: String,  // impl CreatedBy
/// }
/// ```
pub trait CreatedBy {}

/// Marker trait for entities with a last-modified-by field.
/// 带有最后修改者字段的实体的标记 trait。
///
/// Annotate entity fields that should be automatically updated
/// with the current auditor on modification.
/// 标注在修改时应自动更新当前审计者的实体字段。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::auditing::LastModifiedBy;
///
/// struct User {
///     updated_by: String,  // impl LastModifiedBy
/// }
/// ```
pub trait LastModifiedBy {}

/// Trait for providing the current auditor.
/// 提供当前审计者的 trait。
///
/// Implementations resolve the current user/principal from the
/// security context or other authentication mechanism.
/// 实现从安全上下文或其他身份验证机制解析当前用户/主体。
///
/// This mirrors Spring Data's `AuditorAware<T>` interface.
/// 这镜像了 Spring Data 的 `AuditorAware<T>` 接口。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::auditing::AuditorAware;
///
/// struct SecurityAuditor;
///
/// impl AuditorAware<String> for SecurityAuditor
/// {
///     fn current_auditor(&self) -> Option<String>
///     {
///         // In a real app, read from security context
///         // 在实际应用中，从安全上下文读取
///         Some("admin".to_string())
///     }
/// }
/// ```
pub trait AuditorAware<T>: Send + Sync
{
    /// Returns the current auditor.
    /// 返回当前审计者。
    ///
    /// Returns `None` if no auditor is available (e.g., anonymous access).
    /// 如果没有可用的审计者（如匿名访问），返回 `None`。
    fn current_auditor(&self) -> Option<T>;
}

/// Convenience trait for entities that support full auditing.
/// 支持完整审计的实体的便捷 trait。
///
/// Combines created-date, last-modified-date, created-by,
/// and last-modified-by into a single trait with sensible defaults.
/// 将创建日期、最后修改日期、创建者和最后修改者组合为一个带有合理默认值的 trait。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_data_commons::auditing::AuditingEntity;
///
/// struct User {
///     created_at: DateTime<Utc>,
///     updated_at: DateTime<Utc>,
///     created_by: Option<String>,
///     updated_by: Option<String>,
/// }
///
/// impl AuditingEntity for User {
///     fn created_at(&self) -> Option<DateTime<Utc>> { Some(self.created_at) }
///     fn set_created_at(&mut self, ts: DateTime<Utc>) { self.created_at = ts; }
///     fn updated_at(&self) -> Option<DateTime<Utc>> { Some(self.updated_at) }
///     fn set_updated_at(&mut self, ts: DateTime<Utc>) { self.updated_at = ts; }
///     fn created_by(&self) -> Option<&str> { self.created_by.as_deref() }
///     fn set_created_by(&mut self, user: Option<String>) { self.created_by = user; }
///     fn updated_by(&self) -> Option<&str> { self.updated_by.as_deref() }
///     fn set_updated_by(&mut self, user: Option<String>) { self.updated_by = user; }
/// }
/// ```
pub trait AuditingEntity
{
    /// Get the creation timestamp.
    /// 获取创建时间戳。
    fn created_at(&self) -> Option<DateTime<Utc>>;

    /// Set the creation timestamp.
    /// 设置创建时间戳。
    fn set_created_at(&mut self, ts: DateTime<Utc>);

    /// Get the last modification timestamp.
    /// 获取最后修改时间戳。
    fn updated_at(&self) -> Option<DateTime<Utc>>;

    /// Set the last modification timestamp.
    /// 设置最后修改时间戳。
    fn set_updated_at(&mut self, ts: DateTime<Utc>);

    /// Get the creator.
    /// 获取创建者。
    fn created_by(&self) -> Option<&str>;

    /// Set the creator.
    /// 设置创建者。
    fn set_created_by(&mut self, user: Option<String>);

    /// Get the last modifier.
    /// 获取最后修改者。
    fn updated_by(&self) -> Option<&str>;

    /// Set the last modifier.
    /// 设置最后修改者。
    fn set_updated_by(&mut self, user: Option<String>);

    /// Mark this entity as new (set timestamps and auditor for creation).
    /// 将此实体标记为新实体（设置创建的时间戳和审计者）。
    fn mark_new(&mut self, auditor: Option<String>)
    {
        let now = Utc::now();
        self.set_created_at(now);
        self.set_updated_at(now);
        self.set_created_by(auditor.clone());
        self.set_updated_by(auditor);
    }

    /// Mark this entity as modified (update modification timestamp and auditor).
    /// 将此实体标记为已修改（更新修改时间戳和审计者）。
    fn mark_modified(&mut self, auditor: Option<String>)
    {
        self.set_updated_at(Utc::now());
        self.set_updated_by(auditor);
    }
}

/// Handler that sets auditing fields on entities.
/// 设置实体审计字段的处理器。
///
/// This mirrors Spring Data's `AuditingHandler` which automatically
/// populates auditing fields before saving entities.
/// 这镜像了 Spring Data 的 `AuditingHandler`，它在保存实体之前
/// 自动填充审计字段。
///
/// # Example / 示例
///
/// ```rust
/// use chrono::{DateTime, Utc};
/// use hiver_data_commons::auditing::{AuditingEntity, AuditingHandler, AuditorAware};
///
/// struct TestAuditor;
/// impl AuditorAware<String> for TestAuditor
/// {
///     fn current_auditor(&self) -> Option<String>
///     {
///         Some("system".to_string())
///     }
/// }
///
/// #[derive(Debug, Clone)]
/// struct AuditedUser
/// {
///     created_at: Option<DateTime<Utc>>,
///     updated_at: Option<DateTime<Utc>>,
///     created_by: Option<String>,
///     updated_by: Option<String>,
/// }
///
/// impl AuditingEntity for AuditedUser
/// {
///     fn created_at(&self) -> Option<DateTime<Utc>>
///     {
///         self.created_at
///     }
///
///     fn set_created_at(&mut self, ts: DateTime<Utc>)
///     {
///         self.created_at = Some(ts);
///     }
///
///     fn updated_at(&self) -> Option<DateTime<Utc>>
///     {
///         self.updated_at
///     }
///
///     fn set_updated_at(&mut self, ts: DateTime<Utc>)
///     {
///         self.updated_at = Some(ts);
///     }
///
///     fn created_by(&self) -> Option<&str>
///     {
///         self.created_by.as_deref()
///     }
///
///     fn set_created_by(&mut self, user: Option<String>)
///     {
///         self.created_by = user;
///     }
///
///     fn updated_by(&self) -> Option<&str>
///     {
///         self.updated_by.as_deref()
///     }
///
///     fn set_updated_by(&mut self, user: Option<String>)
///     {
///         self.updated_by = user;
///     }
/// }
///
/// let handler = AuditingHandler::new(TestAuditor);
/// let mut user = AuditedUser {
///     created_at: None,
///     updated_at: None,
///     created_by: None,
///     updated_by: None,
/// };
///
/// handler.mark_created(&mut user);
/// assert!(user.created_at().is_some());
/// assert_eq!(user.created_by(), Some("system"));
/// ```
pub struct AuditingHandler<A>
{
    /// The auditor-aware provider.
    /// 审计者感知提供者。
    auditor_aware: A,
}

impl<A> AuditingHandler<A>
where
    A: AuditorAware<String>,
{
    /// Create a new auditing handler with the given auditor provider.
    /// 使用给定的审计者提供者创建新的审计处理器。
    pub fn new(auditor_aware: A) -> Self
    {
        Self { auditor_aware }
    }

    /// Get the current auditor from the configured provider.
    /// 从配置的提供者获取当前审计者。
    pub fn current_auditor(&self) -> Option<String>
    {
        self.auditor_aware.current_auditor()
    }

    /// Mark an entity as newly created, setting all audit fields.
    /// 将实体标记为新创建，设置所有审计字段。
    pub fn mark_created(&self, entity: &mut impl AuditingEntity)
    {
        let auditor = self.current_auditor();
        entity.mark_new(auditor);
    }

    /// Mark an entity as modified, updating modification audit fields.
    /// 将实体标记为已修改，更新修改审计字段。
    pub fn mark_modified(&self, entity: &mut impl AuditingEntity)
    {
        let auditor = self.current_auditor();
        entity.mark_modified(auditor);
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    struct TestAuditor
    {
        current: Option<String>,
    }

    impl TestAuditor
    {
        fn new(current: Option<String>) -> Self
        {
            Self { current }
        }
    }

    impl AuditorAware<String> for TestAuditor
    {
        fn current_auditor(&self) -> Option<String>
        {
            self.current.clone()
        }
    }

    #[derive(Debug, Clone)]
    struct TestAuditedEntity
    {
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        created_by: Option<String>,
        updated_by: Option<String>,
    }

    impl AuditingEntity for TestAuditedEntity
    {
        fn created_at(&self) -> Option<DateTime<Utc>>
        {
            self.created_at
        }

        fn set_created_at(&mut self, ts: DateTime<Utc>)
        {
            self.created_at = Some(ts);
        }

        fn updated_at(&self) -> Option<DateTime<Utc>>
        {
            self.updated_at
        }

        fn set_updated_at(&mut self, ts: DateTime<Utc>)
        {
            self.updated_at = Some(ts);
        }

        fn created_by(&self) -> Option<&str>
        {
            self.created_by.as_deref()
        }

        fn set_created_by(&mut self, user: Option<String>)
        {
            self.created_by = user;
        }

        fn updated_by(&self) -> Option<&str>
        {
            self.updated_by.as_deref()
        }

        fn set_updated_by(&mut self, user: Option<String>)
        {
            self.updated_by = user;
        }
    }

    fn new_entity() -> TestAuditedEntity
    {
        TestAuditedEntity {
            created_at: None,
            updated_at: None,
            created_by: None,
            updated_by: None,
        }
    }

    #[test]
    fn test_auditing_entity_mark_new()
    {
        let mut entity = new_entity();
        entity.mark_new(Some("admin".to_string()));

        assert!(entity.created_at().is_some());
        assert!(entity.updated_at().is_some());
        assert_eq!(entity.created_by(), Some("admin"));
        assert_eq!(entity.updated_by(), Some("admin"));
    }

    #[test]
    fn test_auditing_entity_mark_modified()
    {
        let mut entity = new_entity();
        entity.mark_new(Some("admin".to_string()));

        entity.mark_modified(Some("editor".to_string()));

        assert_eq!(entity.created_by(), Some("admin"));
        assert_eq!(entity.updated_by(), Some("editor"));
    }

    #[test]
    fn test_auditing_handler_mark_created()
    {
        let handler = AuditingHandler::new(TestAuditor::new(Some("system".to_string())));
        let mut entity = new_entity();

        handler.mark_created(&mut entity);

        assert!(entity.created_at().is_some());
        assert!(entity.updated_at().is_some());
        assert_eq!(entity.created_by(), Some("system"));
        assert_eq!(entity.updated_by(), Some("system"));
    }

    #[test]
    fn test_auditing_handler_mark_modified()
    {
        let handler = AuditingHandler::new(TestAuditor::new(Some("system".to_string())));
        let mut entity = new_entity();

        handler.mark_created(&mut entity);
        handler.mark_modified(&mut entity);

        assert_eq!(entity.updated_by(), Some("system"));
    }

    #[test]
    fn test_auditing_handler_no_auditor()
    {
        let handler = AuditingHandler::new(TestAuditor::new(None));
        let mut entity = new_entity();

        handler.mark_created(&mut entity);

        assert!(entity.created_at().is_some());
        assert!(entity.updated_at().is_some());
        assert!(entity.created_by().is_none());
        assert!(entity.updated_by().is_none());
    }

    #[test]
    fn test_auditor_aware()
    {
        let auditor = TestAuditor::new(Some("alice".to_string()));
        assert_eq!(auditor.current_auditor(), Some("alice".to_string()));

        let no_auditor = TestAuditor::new(None);
        assert!(no_auditor.current_auditor().is_none());
    }
}
