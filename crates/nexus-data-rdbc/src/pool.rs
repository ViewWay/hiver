//! SQLx connection pool client
//! SQLx 连接池客户端
//!
//! # Overview / 概述
//!
//! Real DatabaseClient implementation using sqlx with Postgres/MySQL/SQLite support.
//! 使用 sqlx 的真实 DatabaseClient 实现，支持 Postgres/MySQL/SQLite。

use crate::client::DatabaseClient;
use crate::error::{Error, Result};
use crate::row::{ColumnValue, Row};
use sqlx::any::AnyRow;
use sqlx::{Any, Column as _, Row as _};

/// SQLx-based pool client using sqlx::Any for multi-database support.
/// 基于 SQLx 的连接池客户端，使用 sqlx::Any 支持多数据库。
pub struct SqlxPoolClient {
    pool: sqlx::AnyPool,
}

/// PostgreSQL specific pool client (alias)
pub type PgPoolClient = SqlxPoolClient;

/// MySQL specific pool client (alias)
#[cfg(feature = "mysql")]
pub type MySqlPoolClient = SqlxPoolClient;

/// SQLite specific pool client (alias)
#[cfg(feature = "sqlite")]
pub type SqlitePoolClient = SqlxPoolClient;

impl SqlxPoolClient {
    /// Create a new pool client from an existing AnyPool
    pub fn from_pool(pool: sqlx::AnyPool) -> Self {
        Self { pool }
    }

    /// Connect to a database (any supported dialect)
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = sqlx::any::AnyPoolOptions::new()
            .connect(database_url)
            .await
            .map_err(|e| Error::Connection(format!("connection failed: {}", e)))?;
        Ok(Self { pool })
    }

    /// Connect with custom pool options
    pub async fn connect_with_options(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| Error::Connection(format!("connection failed: {}", e)))?;
        Ok(Self { pool })
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &sqlx::AnyPool {
        &self.pool
    }
}

// DatabaseClient implementation

#[async_trait::async_trait]
impl DatabaseClient for SqlxPoolClient {
    async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>> {
        let db_rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(Error::from)?;

        db_rows
            .iter()
            .map(any_row_to_nexus_row)
            .collect()
    }

    async fn fetch_one(&self, sql: &str) -> Result<Option<Row>> {
        let db_row = sqlx::query(sql)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::from)?;

        match db_row {
            Some(row) => Ok(Some(any_row_to_nexus_row(&row)?)),
            None => Ok(None),
        }
    }

    async fn execute_cmd(&self, sql: &str) -> Result<u64> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(Error::from)?;

        Ok(result.rows_affected())
    }

    async fn begin_transaction(&self) -> Result<crate::Transaction> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Transaction(format!("begin transaction failed: {}", e)))?;

        Ok(crate::Transaction::new(std::sync::Arc::new(
            AnyTransactionInner {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx))),
            },
        )))
    }

    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Connection(format!("ping failed: {}", e)))?;
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

// Row conversion: sqlx AnyRow to Nexus Row

fn any_row_to_nexus_row(row: &AnyRow) -> Result<Row> {
    let columns = row.columns();
    let mut nexus_row = Row::new();

    for col in columns {
        let name = col.name().to_string();
        let type_name = format!("{}", col.type_info()).to_lowercase();

        let value = extract_any_column_value(row, col.ordinal(), &type_name)?;
        nexus_row = nexus_row.with_column(name, value);
    }

    Ok(nexus_row)
}

fn extract_any_column_value(row: &AnyRow, index: usize, type_name: &str) -> Result<ColumnValue> {
    match type_name {
        "boolean" | "bool" => {
            let val: bool = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::Bool(val))
        },
        "int2" | "smallint" | "tinyint" => {
            let val: i16 = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::I16(val))
        },
        "int4" | "integer" | "int" => {
            let val: i32 = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::I32(val))
        },
        "int8" | "bigint" => {
            let val: i64 = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::I64(val))
        },
        "float4" | "real" | "float" => {
            let val: f32 = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::F32(val))
        },
        "float8" | "double precision" | "double" => {
            let val: f64 = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::F64(val))
        },
        "varchar" | "text" | "char" | "bpchar" | "name" | "citext" | "string" => {
            let val: String = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::String(val))
        },
        "bytea" | "blob" | "binary" => {
            let val: Vec<u8> = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            Ok(ColumnValue::Bytes(val))
        },
        "uuid" => {
            let val: String = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            if let Ok(u) = uuid::Uuid::parse_str(&val) {
                Ok(ColumnValue::Uuid(u))
            } else {
                Ok(ColumnValue::String(val))
            }
        },
        "timestamp" | "timestamptz" | "datetime"
        | "timestamp without time zone" | "timestamp with time zone" => {
            let val: String = row
                .try_get(index)
                .map_err(|e| Error::RowMapping(format!("column {index}: {e}")))?;
            let formats = [
                "%Y-%m-%d %H:%M:%S",
                "%Y-%m-%d %H:%M:%S%.f",
                "%Y-%m-%dT%H:%M:%S",
                "%Y-%m-%dT%H:%M:%S%.f",
            ];
            let mut parsed = None;
            for fmt in &formats {
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&val, fmt) {
                    parsed = Some(dt);
                    break;
                }
            }
            if let Some(dt) = parsed {
                Ok(ColumnValue::NaiveDateTime(dt))
            } else {
                Ok(ColumnValue::String(val))
            }
        },
        _ => {
            let val: String = row.try_get(index).ok().unwrap_or_default();
            Ok(ColumnValue::String(val))
        },
    }
}

// Transaction inner for sqlx Any

use std::sync::Arc;
use tokio::sync::Mutex;

struct AnyTransactionInner {
    inner: Arc<Mutex<Option<sqlx::Transaction<'static, Any>>>>,
}

async fn take_tx(
    inner: &Arc<Mutex<Option<sqlx::Transaction<'static, Any>>>>,
) -> std::result::Result<sqlx::Transaction<'static, Any>, Box<dyn std::error::Error + Send + Sync>> {
    inner
        .lock()
        .await
        .take()
        .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> { "transaction already consumed".into() })
}

async fn put_tx(
    inner: &Arc<Mutex<Option<sqlx::Transaction<'static, Any>>>>,
    tx: sqlx::Transaction<'static, Any>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut guard = inner.lock().await;
    *guard = Some(tx);
    Ok(())
}

impl crate::transaction::TransactionInner for AnyTransactionInner {
    fn execute(
        &self,
        sql: &str,
    ) -> futures_util::future::BoxFuture<'_, std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>>> {
        let inner = self.inner.clone();
        let sql = sql.to_string();
        Box::pin(async move {
            let mut tx = take_tx(&inner).await?;
            let result = sqlx::query(&sql)
                .execute(&mut *tx)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
            put_tx(&inner, tx).await?;
            Ok(result.rows_affected())
        })
    }
    fn fetch_all(
        &self,
        sql: &str,
    ) -> futures_util::future::BoxFuture<'_, std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>>> {
        let inner = self.inner.clone();
        let sql = sql.to_string();
        Box::pin(async move {
            let mut tx = take_tx(&inner).await?;
            let db_rows = sqlx::query(&sql)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
            let rows: std::result::Result<Vec<Row>, crate::error::Error> = db_rows
                .iter()
                .map(|r| any_row_to_nexus_row(r))
                .collect();
            let rows = rows.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
            put_tx(&inner, tx).await?;
            Ok(rows)
        })
    }
    fn fetch_one(
        &self,
        sql: &str,
    ) -> futures_util::future::BoxFuture<'_, std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>>> {
        let inner = self.inner.clone();
        let sql = sql.to_string();
        Box::pin(async move {
            let mut tx = take_tx(&inner).await?;
            let db_row = sqlx::query(&sql)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
            let result = match db_row {
                Some(row) => Ok(Some(any_row_to_nexus_row(&row)?)),
                None => Ok(None),
            };
            put_tx(&inner, tx).await?;
            result
        })
    }
    fn commit(
        &self,
    ) -> futures_util::future::BoxFuture<'_, std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let tx = take_tx(&inner).await?;
            tx.commit()
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })
        })
    }
    fn rollback(
        &self,
    ) -> futures_util::future::BoxFuture<'_, std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let tx = take_tx(&inner).await?;
            tx.rollback()
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })
        })
    }
    fn isolation_level(&self) -> crate::transaction::IsolationLevel {
        crate::transaction::IsolationLevel::ReadCommitted
    }
    fn is_committed(&self) -> bool {
        let guard = self.inner.try_lock();
        guard.map_or(false, |tx| tx.is_none())
    }
    fn is_rolled_back(&self) -> bool {
        let guard = self.inner.try_lock();
        guard.map_or(false, |tx| tx.is_none())
    }
    fn clone_box(&self) -> Box<dyn crate::transaction::TransactionInner> {
        Box::new(AnyTransactionInner {
            inner: self.inner.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_types_compile() {
        let _client: Option<super::SqlxPoolClient> = None;
    }
}
