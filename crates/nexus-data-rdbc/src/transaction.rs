//! Transaction support
//! 事务支持
//!
//! # Overview / 概述
//!
//! Transaction management for database operations.
//! 数据库操作的事务管理。

#![allow(dead_code)]

use crate::client::DatabaseClient;
use crate::error::{Error, Result};
use crate::row::Row;
use futures_util::future::BoxFuture;
use std::sync::Arc;

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    pub fn as_sql(&self) -> &str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// Internal trait for transaction operations
pub(crate) trait TransactionInner: Send + Sync {
    fn execute(
        &self,
        sql: &str,
    ) -> std::result::Result<u64, Box<dyn std::error::Error + Send + Sync>>;
    fn fetch_all(
        &self,
        sql: &str,
    ) -> std::result::Result<Vec<Row>, Box<dyn std::error::Error + Send + Sync>>;
    fn fetch_one(
        &self,
        sql: &str,
    ) -> std::result::Result<Option<Row>, Box<dyn std::error::Error + Send + Sync>>;
    fn commit(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn rollback(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn isolation_level(&self) -> IsolationLevel;
    fn is_committed(&self) -> bool;
    fn is_rolled_back(&self) -> bool;
    fn clone_box(&self) -> Box<dyn TransactionInner>;
}

/// Transaction
pub struct Transaction {
    inner: Arc<dyn TransactionInner>,
    committed: bool,
    rolled_back: bool,
}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            committed: self.committed,
            rolled_back: self.rolled_back,
        }
    }
}

impl Transaction {
    pub fn new(inner: Arc<dyn TransactionInner>) -> Self {
        Self {
            inner,
            committed: false,
            rolled_back: false,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.committed
            && !self.rolled_back
            && !self.inner.is_committed()
            && !self.inner.is_rolled_back()
    }

    pub fn is_committed(&self) -> bool {
        self.committed || self.inner.is_committed()
    }

    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back || self.inner.is_rolled_back()
    }

    pub fn isolation_level(&self) -> IsolationLevel {
        self.inner.isolation_level()
    }

    pub async fn execute(&self, sql: &str) -> Result<u64> {
        if !self.is_active() {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .execute(sql)
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    pub async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>> {
        if !self.is_active() {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .fetch_all(sql)
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    pub async fn fetch_one(&self, sql: &str) -> Result<Option<Row>> {
        if !self.is_active() {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .fetch_one(sql)
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    pub async fn commit(&self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .commit()
            .map_err(|e| Error::Transaction(e.to_string()))
    }

    pub async fn rollback(&self) -> Result<()> {
        if !self.is_active() {
            return Err(Error::Transaction("Transaction is not active".into()));
        }
        self.inner
            .rollback()
            .map_err(|e| Error::Transaction(e.to_string()))
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if self.is_active() {
            let _ = self.inner.rollback();
        }
    }
}

/// Transaction manager
pub struct TransactionManager;

impl TransactionManager {
    pub fn new() -> Self {
        Self
    }

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
        match result {
            Ok(val) => {
                tx.commit().await?;
                Ok(val)
            },
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            },
        }
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level_sql() {
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
    }
}
