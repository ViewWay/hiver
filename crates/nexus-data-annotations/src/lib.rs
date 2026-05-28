//! # Nexus Data Annotations
//!
//! Spring Data JPA style annotations for Nexus framework
//! Nexus 框架的 Spring Data JPA 风格注解
//!
//! ## Features / 功能
//!
//! - **`#[Entity]`** - Marks a struct as a JPA entity
//! - **`#[Table]`** - Specifies database table mapping
//! - **`#[Id]`** - Marks a field as primary key
//! - **`#[GeneratedValue]`** - Specifies ID generation strategy
//! - **`#[Column]`** - Specifies column mapping
//! - **`#[Query]`** - Custom SQL query for repository methods
//! - **`#[Insert]`**, **`#[Update]`**, **`#[Delete]`** - CRUD operation annotations
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_data_annotations::{Entity, Table, Id, GeneratedValue, Column};
//! use nexus_lombok::Data;
//! use serde::{Serialize, Deserialize};
//!
//! #[Entity]
//! #[Table(name = "users")]
//! #[Data]
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct User {
//!     #[Id]
//!     #[GeneratedValue(strategy = "AUTO")]
//!     #[Column(name = "id")]
//!     pub id: i64,
//!
//!     #[Column(name = "username", nullable = false)]
//!     pub username: String,
//!
//!     #[Column(name = "email")]
//!     pub email: String,
//!
//!     #[Column(name = "age")]
//!     pub age: i32,
//! }
//!
//! // Usage with MyBatis-Plus style repository / 与 MyBatis-Plus 风格的 repository 配合使用
//! #[nexus_mapper]
//! pub trait UserMapper: BaseMapper<User> {
//!     #[Query("SELECT * FROM users WHERE id = #{id}")]
//!     async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;
//!
//!     #[Query("SELECT * FROM users WHERE username LIKE #{username}%")]
//!     async fn find_by_username_starts_with(&self, username: &str) -> Result<Vec<User>, Error>;
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![allow(dead_code)] // Annotations/traits intended for external use via proc macros; 注解/Trait 供外部 proc macro 使用

use proc_macro::TokenStream;

// ========================================================================
// Internal Modules / 内部模块
// ========================================================================
// Note: These modules are private because proc-macro crates can only export
//       procedural macros, not regular modules or runtime types.
//       Runtime types should be in a separate library crate.
// 注意：这些模块是私有的，因为 proc-macro crate 只能导出过程宏，
//       不能导出常规模块或运行时类型。
//       运行时类型应该在单独的库 crate 中。

mod column;
mod entity;
mod id;
mod query;
mod relation;
mod repository;
mod transactional;
mod transactional_macro;
mod audit;
mod lifecycle;

// Pre-authorize macro module (conditionally compiled with security feature)
// Pre-authorize 宏模块（使用 security feature 条件编译）
#[cfg(feature = "security")]
mod pre_authorize_macro;

/// Marks a struct as a JPA entity
/// 将结构体标记为 JPA 实体
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::Entity;
///
/// #[Entity]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    entity::impl_entity(attr, item)
}

/// Specifies the database table for an entity
/// 指定实体的数据库表
///
/// # Attributes / 属性
///
/// - `name` - Table name (default: struct name in lowercase)
///   表名（默认：结构体名小写）
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::Table;
///
/// #[Entity]
/// #[Table(name = "users")]
/// pub struct User {
///     pub id: i64,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Table(attr: TokenStream, item: TokenStream) -> TokenStream {
    entity::impl_table(attr, item)
}

// ========================================================================
// Field Annotations / 字段注解
// ========================================================================

/// Marks a field as primary key
/// 将字段标记为主键
///
/// # Attributes / 属性
///
/// - `type` - Primary key type: "auto", "input", "`assign_id`"
///   主键类型："auto", "input", "`assign_id`"
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::{Id, GeneratedValue};
///
/// #[Entity]
/// pub struct User {
///     #[Id]
///     #[GeneratedValue(strategy = "AUTO")]
///     pub id: i64,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Id(attr: TokenStream, item: TokenStream) -> TokenStream {
    id::impl_id(attr, item)
}

/// Specifies the strategy for generating ID values
/// 指定 ID 值的生成策略
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::{GeneratedValue, Id};
///
/// #[Entity]
/// pub struct User {
///     #[Id]
///     #[GeneratedValue(strategy = "AUTO")]
///     pub id: i64,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn GeneratedValue(attr: TokenStream, item: TokenStream) -> TokenStream {
    id::impl_generated_value(attr, item)
}

/// Specifies database column mapping
/// 指定数据库列映射
///
/// # Attributes / 属性
///
/// - `name` - Column name (default: field name)
///   列名（默认：字段名）
/// - `nullable` - Whether column can be null (default: true)
///   列是否可为 null（默认：true）
/// - `unique` - Whether column has unique constraint (default: false)
///   列是否有唯一约束（默认：false）
/// - `length` - Column length for string types
///   字符串类型的列长度
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::Column;
///
/// #[Entity]
/// pub struct User {
///     #[Column(name = "username", nullable = false, unique = true)]
///     pub username: String,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Column(attr: TokenStream, item: TokenStream) -> TokenStream {
    column::impl_column(attr, item)
}

// ========================================================================
// Relation Annotations / 关系注解
// ========================================================================

/// Marks a one-to-many relationship
/// 标记一对多关系
///
/// # Attributes / 属性
///
/// - `target_entity` - Target entity type name / 目标实体类型名
/// - `mapped_by` - Field name in target entity / 目标实体中的字段名
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::{Entity, OneToMany};
///
/// #[Entity]
/// pub struct User {
///     pub id: i64,
///     #[OneToMany(target_entity = "Order", mapped_by = "user_id")]
///     pub orders: Vec<Order>,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn OneToMany(attr: TokenStream, item: TokenStream) -> TokenStream {
    relation::impl_one_to_many(attr, item)
}

/// Marks a many-to-one relationship
/// 标记多对一关系
///
/// # Attributes / 属性
///
/// - `target_entity` - Target entity type name / 目标实体类型名
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::{Entity, ManyToOne};
///
/// #[Entity]
/// pub struct Order {
///     pub id: i64,
///     #[ManyToOne(target_entity = "User")]
///     pub user: User,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn ManyToOne(attr: TokenStream, item: TokenStream) -> TokenStream {
    relation::impl_many_to_one(attr, item)
}

/// Marks a many-to-many relationship
/// 标记多对多关系
///
/// # Attributes / 属性
///
/// - `target_entity` - Target entity type name / 目标实体类型名
/// - `mapped_by` - Field name in target entity / 目标实体中的字段名
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::{Entity, ManyToMany};
///
/// #[Entity]
/// pub struct User {
///     pub id: i64,
///     #[ManyToMany(target_entity = "Role", mapped_by = "users")]
///     pub roles: Vec<Role>,
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn ManyToMany(attr: TokenStream, item: TokenStream) -> TokenStream {
    relation::impl_many_to_many(attr, item)
}

// ========================================================================
// Query Annotations / 查询注解
// ========================================================================

/// Specifies a custom SQL query for a repository method
/// 为 repository 方法指定自定义 SQL 查询
///
/// # Parameters / 参数
///
/// Supports parameter binding with:
/// 支持以下参数绑定：
///
/// - `:param` - Named parameter (recommended)
///   命名参数（推荐）
/// - `#{param}` - MyBatis-Plus style
///   MyBatis-Plus 风格
/// - `$1, $2` - Positional parameter (PostgreSQL style)
///   位置参数（PostgreSQL 风格）
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::Query;
/// use nexus_data::Repository;
///
/// trait UserRepository: Repository<User, i64> {
///     #[Query("SELECT * FROM users WHERE id = :id")]
///     async fn find_by_id(&self, id: i64) -> Result<Option<User>, Error>;
///
///     #[Query("SELECT * FROM users WHERE username LIKE :pattern%")]
///     async fn find_by_username_starts_with(&self, pattern: &str) -> Result<Vec<User>, Error>;
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Query(attr: TokenStream, item: TokenStream) -> TokenStream {
    query::impl_query(attr, item)
}

/// Marks a method as an insert operation
/// 将方法标记为插入操作
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::Insert;
///
/// trait UserRepository {
///     #[Insert("INSERT INTO users (username, email) VALUES (:username, :email)")]
///     async fn insert_user(&self, username: &str, email: &str) -> Result<u64, Error>;
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Insert(attr: TokenStream, item: TokenStream) -> TokenStream {
    query::impl_insert(attr, item)
}

/// Marks a method as an update operation
/// 将方法标记为更新操作
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::Update;
///
/// trait UserRepository {
///     #[Update("UPDATE users SET email = :email WHERE id = :id")]
///     async fn update_email(&self, id: i64, email: &str) -> Result<u64, Error>;
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Update(attr: TokenStream, item: TokenStream) -> TokenStream {
    query::impl_update(attr, item)
}

/// Marks a method as a delete operation
/// 将方法标记为删除操作
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_annotations::Delete;
///
/// trait UserRepository {
///     #[Delete("DELETE FROM users WHERE id = :id")]
///     async fn delete_by_id(&self, id: i64) -> Result<u64, Error>;
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    query::impl_delete(attr, item)
}

// ============================================================================
// Transactional Annotation / 事务注解
// ============================================================================

/// Marks a method or function to be executed within a transaction
/// 将方法或函数标记为在事务中执行
///
/// # Attributes / 属性
///
/// - `isolation` - Transaction isolation level (Default, `ReadUncommitted`, `ReadCommitted`, `RepeatableRead`, Serializable)
///   事务隔离级别
/// - `timeout` - Transaction timeout in seconds
///   事务超时时间（秒）
/// - `propagation` - Transaction propagation behavior (Required, Supports, Mandatory, `RequiresNew`, `NotSupported`, Never, Nested)
///   事务传播行为
/// - `read_only` - Whether transaction is read-only
///   事务是否只读
/// - `max_retries` - Max retry attempts for serialization failures
///   序列化失败的最大重试次数
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_data_annotations::Transactional;
///
/// #[Transactional]
/// async fn create_user(&self, user: User) -> Result<(), Error> {
///     // Automatically executed in a transaction
///     Ok(())
/// }
///
/// #[Transactional(isolation = ReadCommitted, timeout = 60)]
/// async fn transfer_funds(&self, from: i64, to: i64, amount: i64) -> Result<(), Error> {
///     // Executed with READ COMMITTED isolation and 60s timeout
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Transactional(attr: TokenStream, item: TokenStream) -> TokenStream {
    transactional_macro::impl_transactional(attr, item)
}

// ========================================================================
// Re-exports / 重新导出
// ========================================================================
// Note: In a proc-macro crate, only macros can be exported via
// #[proc_macro_attribute] and #[proc_macro_derive] functions.
// Runtime types cannot be exported from proc-macro crates.
// 注意：在 proc-macro crate 中，只有宏可以通过 #[proc_macro_attribute]
// 和 #[proc_macro_derive] 函数导出。运行时类型无法从 proc-macro crate 导出。
//
// For runtime types, users should depend on a separate library crate.
// 对于运行时类型，用户应该依赖单独的库 crate。

// ============================================================================
// Security Annotations / 安全注解
// ============================================================================
// Security annotations are only available when the "security" feature is enabled.
// 安全注解仅在启用 "security" feature 时可用。
#[cfg(feature = "security")]
/// Method-level security annotation
/// 方法级安全注解
///
/// # Attributes / 属性
///
/// - `expression` - Security expression to evaluate
///   要评估的安全表达式
///
/// # Supported Expressions / 支持的表达式
///
/// - `has_role('ROLE_NAME')` - Check if user has role
///   检查用户是否拥有角色
/// - `has_permission('PERMISSION_NAME')` - Check if user has permission
///   检查用户是否拥有权限
/// - `is_admin()` - Check if user is admin
///   检查用户是否为管理员
/// - `#param == value` - Check parameter values
///   检查参数值
/// - `expr1 and expr2` - Logical AND
///   逻辑与
/// - `expr1 or expr2` - Logical OR
///   逻辑或
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_data_annotations::PreAuthorize;
///
/// impl UserService {
///     #[PreAuthorize("has_role('ADMIN')")]
///     async fn delete_user(&self, id: i64) -> Result<(), Error> {
///         // Only ADMIN role can execute
///         // 只有 ADMIN 角色可以执行
///         self.repository.delete(id).await
///     }
///
///     #[PreAuthorize("has_role('ADMIN') or #id == auth.user_id()")]
///     async fn update_profile(&self, auth: &AuthContext, id: i64, data: UpdateData) -> Result<(), Error> {
///         // Admin or owner can modify
///         // 管理员或本人可以修改
///         self.repository.update(id, data).await
///     }
/// }
/// ```
#[cfg(feature = "security")]
#[proc_macro_attribute]
pub fn PreAuthorize(attr: TokenStream, item: TokenStream) -> TokenStream {
    pre_authorize_macro::pre_authorize(attr, item)
}

/// Alias for @PreAuthorize annotation
/// @PreAuthorize 注解的别名
#[cfg(feature = "security")]
#[proc_macro_attribute]
pub fn PreAuthorizeMacroFn(attr: TokenStream, item: TokenStream) -> TokenStream {
    pre_authorize_macro::pre_authorize(attr, item)
}

// ============================================================================
// Audit Annotations / 审计注解
// ============================================================================

/// Marks a field as auto-populated with creation timestamp
/// 标记字段自动填充创建时间戳
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @CreatedDate
/// private Instant createdAt;
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn CreatedDate(attr: TokenStream, item: TokenStream) -> TokenStream {
    audit::impl_created_date(attr, item)
}

/// Marks a field as auto-updated with modification timestamp
/// 标记字段自动更新修改时间戳
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @LastModifiedDate
/// private Instant updatedAt;
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn LastModifiedDate(attr: TokenStream, item: TokenStream) -> TokenStream {
    audit::impl_last_modified_date(attr, item)
}

/// Marks a field as auto-populated with creator identity
/// 标记字段自动填充创建者标识
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @CreatedBy
/// private String createdBy;
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn CreatedBy(attr: TokenStream, item: TokenStream) -> TokenStream {
    audit::impl_created_by(attr, item)
}

/// Marks a field as auto-updated with modifier identity
/// 标记字段自动更新修改者标识
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @LastModifiedBy
/// private String updatedBy;
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn LastModifiedBy(attr: TokenStream, item: TokenStream) -> TokenStream {
    audit::impl_last_modified_by(attr, item)
}

// ============================================================================
// Lifecycle Callback Annotations / 生命周期回调注解
// ============================================================================

/// Marks a method to be called before entity persist (INSERT).
/// 标记方法在实体持久化（INSERT）之前调用。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @PrePersist
/// protected void onCreate() { ... }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn PrePersist(attr: TokenStream, item: TokenStream) -> TokenStream {
    lifecycle::impl_pre_persist(attr, item)
}

/// Marks a method to be called after entity persist.
/// 标记方法在实体持久化之后调用。
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn PostPersist(attr: TokenStream, item: TokenStream) -> TokenStream {
    lifecycle::impl_post_persist(attr, item)
}

/// Marks a method to be called before entity update.
/// 标记方法在实体更新之前调用。
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn PreUpdate(attr: TokenStream, item: TokenStream) -> TokenStream {
    lifecycle::impl_pre_update(attr, item)
}

/// Marks a method to be called after entity update.
/// 标记方法在实体更新之后调用。
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn PostUpdate(attr: TokenStream, item: TokenStream) -> TokenStream {
    lifecycle::impl_post_update(attr, item)
}

/// Marks a method to be called before entity remove (DELETE).
/// 标记方法在实体删除之前调用。
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn PreRemove(attr: TokenStream, item: TokenStream) -> TokenStream {
    lifecycle::impl_pre_remove(attr, item)
}

/// Marks a method to be called after entity is loaded from database.
/// 标记方法在实体从数据库加载之后调用。
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn PostLoad(attr: TokenStream, item: TokenStream) -> TokenStream {
    lifecycle::impl_post_load(attr, item)
}

// ============================================================================
// Additional Field Annotations / 附加字段注解
// ============================================================================

/// Marks a field as non-persistent (not mapped to a database column).
/// 标记字段为非持久化（不映射到数据库列）。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Transient
/// private String computedField;
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Transient(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Specifies the foreign key column for a relationship.
/// 指定关系的外键列。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @JoinColumn(name = "user_id")
/// private User user;
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn JoinColumn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Specifies the join table for a many-to-many relationship.
/// 指定多对多关系的连接表。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @JoinTable(name = "user_roles",
///     joinColumns = @JoinColumn(name = "user_id"),
///     inverseJoinColumns = @JoinColumn(name = "role_id"))
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn JoinTable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
