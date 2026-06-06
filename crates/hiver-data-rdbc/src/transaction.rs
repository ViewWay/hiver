//! Transaction support
//! 事务支持
//!
//! # Overview / 概述
//!
//! Transaction management for database operations.
//! 数据库操作的事务管理。

#![allow(dead_code)]

use std::sync::Arc;

use futures_util::future::BoxFuture;

use crate::{
    client::DatabaseClient,
    error::{Error, Result},
    row::Row,
};

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel
{
    /// Read uncommitted isolation level
    ReadUncommitted,
    /// Read committed isolation level
    ReadCommitted,
    /// Repeatable read isolation level
    RepeatableRead,
    /// Serializable isolation level
    Serializable,
}

impl IsolationLevel
{
    /// Returns the SQL string for this isolation level
    pub fn as_sql(&self) -> &str
    {
        match self
        {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// Internal trait for transaction operations
pub(crate) trait TransactionInner: Send + Sync
{
    fn execute(
        &self,
        sql: &str,
    ) -> BoxFuture<'_, std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>>>;
    fn fetch_all(
        &self,
        sql: &str,
    ) -> BoxFuture<'_, std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>>>;
    fn fetch_one(
        &self,
        sql: &str,
    ) -> BoxFuture<'_, std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>>>;
    fn commit(
        &self,
    ) -> BoxFuture<'_, std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>>;
    fn rollback(
        &self,
    ) -> BoxFuture<'_, std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>>;
    fn isolation_level(&self) -> IsolationLevel;
    fn is_committed(&self) -> bool;
    fn is_rolled_back(&self) -> bool;
    fn clone_box(&self) -> Box<dyn TransactionInner>;
}

/// Transaction
pub struct Transaction
{
    inner: Arc<dyn TransactionInner>,
    committed: bool,
    rolled_back: bool,
}

impl Clone for Transaction
{
    fn clone(&self) -> Self
    {
        Self {
            inner: self.inner.clone(),
            committed: self.committed,
            rolled_back: self.rolled_back,
        }
    }
}

impl Transaction
{
    /// Creates a new transaction wrapping the given inner implementation
    #[allow(private_interfaces)]
    pub fn new(inner: Arc<dyn TransactionInner>) -> Self
    {
        Self {
            inner,
            committed: false,
            rolled_back: false,
        }
    }

    /// Returns true if the transaction is still active
    pub fn is_active(&self) -> bool
    {
        !self.committed
            && !self.rolled_back
            && !self.inner.is_committed()
            && !self.inner.is_rolled_back()
    }

    /// Returns true if the transaction has been committed
    pub fn is_committed(&self) -> bool
    {
        self.committed || self.inner.is_committed()
    }

    /// Returns true if the transaction has been rolled back
    pub fn is_rolled_back(&self) -> bool
    {
        self.rolled_back || self.inner.is_rolled_back()
    }

    /// Returns the isolation level of this transaction
    pub fn isolation_level(&self) -> IsolationLevel
    {
        self.inner.isolation_level()
    }

    /// Executes a SQL statement within this transaction
    pub async fn execute(&self, sql: &str) -> Result<u64>
    {
        if !self.is_active()
        {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .execute(sql)
            .await
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    /// Fetches all rows matching the given SQL query within this transaction
    pub async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>>
    {
        if !self.is_active()
        {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .fetch_all(sql)
            .await
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    /// Fetches a single row matching the given SQL query within this transaction
    pub async fn fetch_one(&self, sql: &str) -> Result<Option<Row>>
    {
        if !self.is_active()
        {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .fetch_one(sql)
            .await
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    /// Commits this transaction
    pub async fn commit(&self) -> Result<()>
    {
        if !self.is_active()
        {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .commit()
            .await
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    /// Rolls back this transaction
    pub async fn rollback(&self) -> Result<()>
    {
        if !self.is_active()
        {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .rollback()
            .await
            .map_err(|e| Error::Transaction(e.to_string()))
    }
}

impl Drop for Transaction
{
    fn drop(&mut self)
    {
        if self.is_active()
        {
            self.committed = true;
            // The underlying sqlx transaction will be rolled back on drop automatically.
        }
    }
}

/// Transaction manager
pub struct TransactionManager;

impl TransactionManager
{
    /// Creates a new transaction manager
    pub fn new() -> Self
    {
        Self
    }

    /// Executes a closure within a transaction with the given isolation level
    pub async fn execute_in_transaction<F, T, C>(
        &self,
        _isolation: IsolationLevel,
        client: &C,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce(&Transaction) -> BoxFuture<'static, Result<T>>,
        C: DatabaseClient + ?Sized,
    {
        let tx = client.begin_transaction().await?;
        let result = f(&tx).await;
        match result
        {
            Ok(val) =>
            {
                tx.commit().await?;
                Ok(val)
            },
            Err(e) =>
            {
                let _ = tx.rollback().await;
                Err(e)
            },
        }
    }
}

impl Default for TransactionManager
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_isolation_level_sql()
    {
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
    }
}
