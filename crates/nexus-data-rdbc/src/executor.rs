//! Query execution
//! 查询执行
//!
//! # Overview / 概述
//!
//! Provides query execution for MyBatis-Plus style wrappers.
//! 提供 MyBatis-Plus 风格包装器的查询执行。
//!
//! SQL generation is delegated to the `sql_builder` module.
//! SQL 生成委托给 `sql_builder` 模块。

use crate::client::{DatabaseClient, QueryParam};
use crate::error::{Error, Result};
use crate::row::Row;
use crate::sql_builder;
use nexus_data_commons::{Page, PageRequest, QueryWrapper, UpdateWrapper};

/// Query executor — wraps a DatabaseClient for MyBatis-Plus style query execution.
/// 查询执行器 — 包装 DatabaseClient 用于 MyBatis-Plus 风格查询执行。
pub struct QueryExecutor<C: DatabaseClient> {
    client: C,
}

impl<C: DatabaseClient> QueryExecutor<C> {
    /// Create a new query executor
    /// 创建新的查询执行器
    pub fn new(client: C) -> Self {
        Self { client }
    }

    /// Get a reference to the underlying client
    /// 获取底层客户端的引用
    pub fn client(&self) -> &C {
        &self.client
    }

    // ── Select ────────────────────────────────────────────────────────

    /// Select a list of entities by wrapper
    /// 通过包装器选择实体列表
    pub async fn select_list<T: serde::de::DeserializeOwned>(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> Result<Vec<T>> {
        let (sql, params) = sql_builder::build_select_query(wrapper, table);
        let rows = self.client.fetch_all_params(&sql, &params).await?;
        self.map_rows(rows)
    }

    /// Select one entity by wrapper
    /// 通过包装器选择单个实体
    pub async fn select_one<T: serde::de::DeserializeOwned>(
        &self,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> Result<Option<T>> {
        let (sql, params) = sql_builder::build_select_query(wrapper, table);
        match self.client.fetch_one_params(&sql, &params).await? {
            Some(r) => Ok(Some(self.map_row(r)?)),
            None => Ok(None),
        }
    }

    /// Count entities by wrapper
    /// 通过包装器计数实体
    pub async fn count(&self, wrapper: &QueryWrapper, table: &str) -> Result<i64> {
        let (sql, params) = sql_builder::build_count_query(wrapper, table);
        let rows = self.client.fetch_all_params(&sql, &params).await?;
        let count = rows
            .first()
            .and_then(|r| r.get("cnt").and_then(|v| v.as_type::<i64>()))
            .ok_or_else(|| Error::RowMapping("count result missing 'cnt' column".into()))?;
        Ok(count)
    }

    /// Select with pagination
    /// 分页查询
    pub async fn select_page<T: serde::de::DeserializeOwned>(
        &self,
        page: &PageRequest,
        wrapper: &QueryWrapper,
        table: &str,
    ) -> Result<Page<T>> {
        let total = self.count(wrapper, table).await?;
        let (sql, params) = sql_builder::build_page_query(page, wrapper, table);
        let rows = self.client.fetch_all_params(&sql, &params).await?;
        let records = self.map_rows(rows)?;

        Ok(Page::new(records, page.page, page.size, total as u64))
    }

    /// Execute a raw select query
    /// 执行原始 SELECT 查询
    pub async fn select<T: serde::de::DeserializeOwned>(&self, sql: &str) -> Result<Vec<T>> {
        let rows = self.client.fetch_all(sql).await?;
        self.map_rows(rows)
    }

    // ── Insert ───────────────────────────────────────────────────────

    /// Insert an entity
    /// 插入实体
    pub async fn insert<T: serde::Serialize>(&self, entity: &T, table: &str) -> Result<u64> {
        let json =
            serde_json::to_value(entity).map_err(|e| Error::Deserialization(e.to_string()))?;
        let map = json
            .as_object()
            .ok_or_else(|| Error::Deserialization("entity must be an object".into()))?;

        let columns: Vec<&String> = map.keys().collect();
        let params: Vec<QueryParam> = map.values().cloned().map(QueryParam::from).collect();

        let placeholders: Vec<String> = (1..=params.len()).map(|i| format!("${}", i)).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", "),
            placeholders.join(", ")
        );

        self.client.execute_params(&sql, &params).await
    }

    // ── Update ───────────────────────────────────────────────────────

    /// Update by wrapper
    /// 通过包装器更新
    pub async fn update(&self, wrapper: &UpdateWrapper, table: &str) -> Result<u64> {
        let (sql, params) = sql_builder::build_update_query(wrapper, table);
        self.client.execute_params(&sql, &params).await
    }

    /// Execute a raw update/delete command
    /// 执行原始更新/删除命令
    pub async fn execute(&self, sql: &str) -> Result<u64> {
        self.client.execute_cmd(sql).await
    }

    // ── Delete ───────────────────────────────────────────────────────

    /// Delete by wrapper
    /// 通过包装器删除
    pub async fn delete(&self, wrapper: &QueryWrapper, table: &str) -> Result<u64> {
        let (sql, params) = sql_builder::build_delete_query(wrapper, table);
        self.client.execute_params(&sql, &params).await
    }

    // ── Row mapping helpers ──────────────────────────────────────────

    fn map_row<T: serde::de::DeserializeOwned>(&self, row: Row) -> Result<T> {
        row.deserialize()
    }

    fn map_rows<T: serde::de::DeserializeOwned>(&self, rows: Vec<Row>) -> Result<Vec<T>> {
        rows.into_iter().map(|r| self.map_row(r)).collect()
    }
}
