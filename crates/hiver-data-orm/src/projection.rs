//! Projection support for ORM queries
//! ORM 查询的投影支持
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data |
//! |-------|-------------|
//! | `Projection` | Spring Data Projections |
//! | `InterfaceProjection` | Interface-based Projections |
//! | `DtoProjection<T>` | DTO Projections |
//!
//! Projections allow selecting a subset of columns from an entity instead of
//! fetching all fields. This improves performance by reducing data transfer.
//!
//! 投影允许从实体中选择部分列而不是获取所有字段。这通过减少数据传输来提高性能。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_data_orm::projection::{Projection, DtoProjection};
//!
//! // Select only name and email columns
//! let projected = User::query()
//!     .select(&["name", "email"])
//!     .project::<UserNameDto>(&client)
//!     .await?;
//! ```

use crate::Model;
use hiver_data_rdbc::DatabaseClient;
use std::marker::PhantomData;

// ──────────────────────────────────────────────────────────────────────────────
// Projection trait
// ──────────────────────────────────────────────────────────────────────────────

/// Projection trait — select a subset of columns instead of the full entity.
/// Projection trait — 选择部分列而非完整实体。
///
/// Equivalent to Spring Data Projections.
/// 等价于 Spring Data 投影。
///
/// A projection defines which columns to include in a query result and how
/// to map the result rows into a target type.
///
/// 投影定义查询结果中包含哪些列以及如何将结果行映射到目标类型。
pub trait Projection: Send + Sync {
    /// The source model type (the full entity)
    /// 源模型类型（完整实体）
    type Source: Model;

    /// The projected result type
    /// 投影结果类型
    type Target: Send + Sync;

    /// Return the column names to include in the SELECT clause.
    /// 返回 SELECT 子句中要包含的列名。
    fn columns() -> &'static [&'static str];

    /// Map a database row to the projected type.
    /// 将数据库行映射到投影类型。
    fn from_row(row: &hiver_data_rdbc::Row) -> crate::Result<Self::Target>;
}

// ──────────────────────────────────────────────────────────────────────────────
// Interface Projection
// ──────────────────────────────────────────────────────────────────────────────

/// Interface-based projection — project query results into a simpler struct.
/// 基于接口的投影 — 将查询结果投影到更简单的结构体。
///
/// Equivalent to Spring Data's interface-based projections where you define
/// a Java interface with getter methods for only the properties you need.
///
/// 等价于 Spring Data 基于接口的投影，其中定义仅包含所需属性的 getter 方法的 Java 接口。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_orm::projection::InterfaceProjection;
///
/// struct UserNameProjection;
///
/// impl InterfaceProjection for UserNameProjection {
///     type Source = User;
///     type Target = (String, String); // (name, email)
///
///     fn columns() -> &'static [&'static str] {
///         &["name", "email"]
///     }
///
///     fn from_row(row: &Row) -> Result<(String, String)> {
///         Ok((
///             row.get_as::<String>("name")?,
///             row.get_as::<String>("email")?,
///         ))
///     }
/// }
/// ```
pub trait InterfaceProjection: Send + Sync {
    /// The source model type
    /// 源模型类型
    type Source: Model;

    /// The projected result type
    /// 投影结果类型
    type Target: Send + Sync;

    /// Return the column names to include.
    /// 返回要包含的列名。
    fn columns() -> &'static [&'static str];

    /// Map a database row to the projected type.
    /// 将数据库行映射到投影类型。
    fn from_row(row: &hiver_data_rdbc::Row) -> crate::Result<Self::Target>;
}

// ──────────────────────────────────────────────────────────────────────────────
// DTO Projection
// ──────────────────────────────────────────────────────────────────────────────

/// DTO projection — map query results to a DTO (Data Transfer Object) type.
/// DTO 投影 — 将查询结果映射到 DTO（数据传输对象）类型。
///
/// Equivalent to Spring Data's class-based (DTO) projections.
/// 等价于 Spring Data 基于类的（DTO）投影。
///
/// The target type `T` must implement `serde::de::DeserializeOwned` so that
/// rows can be deserialized into it.
///
/// 目标类型 `T` 必须实现 `serde::de::DeserializeOwned`，以便行可以反序列化为它。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_orm::projection::DtoProjection;
///
/// #[derive(Deserialize)]
/// struct UserSummary {
///     name: String,
///     email: String,
/// }
///
/// let projection = DtoProjection::<User, UserSummary>::new(&["name", "email"]);
/// let results = projection.query(&client).await?;
/// ```
pub struct DtoProjection<M, T> {
    _phantom: PhantomData<(M, T)>,
    columns: Vec<String>,
}

impl<M, T> DtoProjection<M, T>
where
    M: Model,
    T: serde::de::DeserializeOwned + Send + Sync,
{
    /// Create a new DTO projection with the specified columns.
    /// 使用指定列创建新的 DTO 投影。
    pub fn new(columns: &[&str]) -> Self {
        Self {
            _phantom: PhantomData,
            columns: columns.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Execute the projected query and return all results as DTOs.
    /// 执行投影查询并以 DTO 形式返回所有结果。
    pub async fn query<C: DatabaseClient>(&self, client: &C) -> crate::Result<Vec<T>> {
        let col_list = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };
        let sql = format!("SELECT {} FROM {}", col_list, M::table_name());
        let rows = client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Projection query failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            results.push(
                row.deserialize()
                    .map_err(|e| crate::Error::validation(format!("DTO deserialize: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Execute the projected query with a WHERE condition.
    /// 使用 WHERE 条件执行投影查询。
    pub async fn query_where<C: DatabaseClient>(
        &self,
        client: &C,
        condition: &str,
        params: &[hiver_data_rdbc::QueryParam],
    ) -> crate::Result<Vec<T>> {
        let col_list = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut cond = condition.to_string();
        for (param_idx, _) in (1u32..).zip(params.iter()) {
            cond = cond.replacen('?', &format!("${param_idx}"), 1);
        }

        let sql = format!("SELECT {} FROM {} WHERE {}", col_list, M::table_name(), cond);
        let rows = client
            .fetch_all_params(&sql, params)
            .await
            .map_err(|e| crate::Error::query_build(format!("Projection query failed: {e}")))?;
        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            results.push(
                row.deserialize()
                    .map_err(|e| crate::Error::validation(format!("DTO deserialize: {e}")))?,
            );
        }
        Ok(results)
    }

    /// Get the columns included in this projection.
    /// 获取此投影中包含的列。
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Closed Projection (subset of columns mapped to a flat struct)
// ──────────────────────────────────────────────────────────────────────────────

/// Closed projection — a projection that exposes only a fixed set of
/// properties from the aggregate root.
///
/// 闭投影 — 仅暴露聚合根中固定属性集的投影。
///
/// Equivalent to Spring Data's "closed" interface-based projection.
/// 等价于 Spring Data 的"闭"接口投影。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_orm::projection::ClosedProjection;
///
/// struct UserEmailOnly;
///
/// impl ClosedProjection for UserEmailOnly {
///     type Source = User;
///     type Target = String;
///
///     fn columns() -> &'static [&'static str] {
///         &["email"]
///     }
///
///     fn from_row(row: &Row) -> Result<String> {
///         row.get_as::<String>("email").map_err(|e| Error::unknown(e.to_string()))
///     }
/// }
/// ```
pub trait ClosedProjection: Send + Sync {
    /// The source model type
    /// 源模型类型
    type Source: Model;

    /// The projected result type
    /// 投影结果类型
    type Target: Send + Sync;

    /// Return the column names to include.
    /// 返回要包含的列名。
    fn columns() -> &'static [&'static str];

    /// Map a database row to the projected type.
    /// 将数据库行映射到投影类型。
    fn from_row(row: &hiver_data_rdbc::Row) -> crate::Result<Self::Target>;
}

// ──────────────────────────────────────────────────────────────────────────────
// Dynamic Projection
// ──────────────────────────────────────────────────────────────────────────────

/// Dynamic projection — runtime-selectable projection type.
/// 动态投影 — 运行时可选的投影类型。
///
/// Allows choosing the projection type at runtime rather than compile time,
/// similar to Spring Data's `crystal()`.
///
/// 允许在运行时而非编译时选择投影类型，类似于 Spring Data 的 `crystal()`。
pub struct DynamicProjection<M: Model> {
    _phantom: PhantomData<M>,
    columns: Vec<String>,
}

impl<M: Model> DynamicProjection<M> {
    /// Create a new dynamic projection.
    /// 创建新的动态投影。
    pub fn new(columns: &[&str]) -> Self {
        Self {
            _phantom: PhantomData,
            columns: columns.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Execute and return raw rows for flexible mapping.
    /// 执行并返回原始行以进行灵活映射。
    pub async fn execute<C: DatabaseClient>(
        &self,
        client: &C,
    ) -> crate::Result<Vec<hiver_data_rdbc::Row>> {
        let col_list = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };
        let sql = format!("SELECT {} FROM {}", col_list, M::table_name());
        client
            .fetch_all(&sql)
            .await
            .map_err(|e| crate::Error::query_build(format!("Dynamic projection failed: {e}")))
    }

    /// Execute with a WHERE condition.
    /// 使用 WHERE 条件执行。
    pub async fn execute_where<C: DatabaseClient>(
        &self,
        client: &C,
        condition: &str,
        params: &[hiver_data_rdbc::QueryParam],
    ) -> crate::Result<Vec<hiver_data_rdbc::Row>> {
        let col_list = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut cond = condition.to_string();
        for (param_idx, _) in (1u32..).zip(params.iter()) {
            cond = cond.replacen('?', &format!("${param_idx}"), 1);
        }

        let sql = format!("SELECT {} FROM {} WHERE {}", col_list, M::table_name(), cond);
        client
            .fetch_all_params(&sql, params)
            .await
            .map_err(|e| crate::Error::query_build(format!("Dynamic projection failed: {e}")))
    }

    /// Get the columns included in this projection.
    /// 获取此投影中包含的列。
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Column, ColumnType, Model, ModelMeta};

    #[derive(Debug, Clone)]
    struct TestUser;

    impl Model for TestUser {
        fn meta() -> ModelMeta {
            let mut meta = ModelMeta::new("users");
            meta.columns
                .push(Column::new("id", ColumnType::I64).primary_key());
            meta.columns.push(Column::new("name", ColumnType::String));
            meta.columns.push(Column::new("email", ColumnType::String));
            meta
        }
    }

    #[test]
    fn test_dto_projection_creation() {
        let projection = DtoProjection::<TestUser, serde_json::Value>::new(&["name", "email"]);
        assert_eq!(projection.columns().len(), 2);
        assert_eq!(projection.columns()[0], "name");
        assert_eq!(projection.columns()[1], "email");
    }

    #[test]
    fn test_dto_projection_all_columns() {
        let projection = DtoProjection::<TestUser, serde_json::Value>::new(&[]);
        assert!(projection.columns().is_empty());
    }

    #[test]
    fn test_dynamic_projection_creation() {
        let projection = DynamicProjection::<TestUser>::new(&["id", "name"]);
        assert_eq!(projection.columns().len(), 2);
    }
}
