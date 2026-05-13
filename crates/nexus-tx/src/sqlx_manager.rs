//! SQLx-backed transaction manager.
//! 基于 SQLx 的事务管理器。

#[cfg(feature = "sqlx")]
mod imp {
    use std::sync::Arc;

    use async_trait::async_trait;
    use sqlx::{Pool, Postgres};

    use crate::manager::{TransactionDefinition, TransactionManager};
    use crate::status::TransactionStatus;
    use crate::synchronization::{self, LiveTransaction};
    use crate::{Propagation, TransactionError, TransactionResult};

    struct SqlxLiveTx {
        conn: sqlx::pool::PoolConnection<Postgres>,
    }

    impl LiveTransaction for SqlxLiveTx {
        fn commit_boxed(
            mut self: Box<Self>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TransactionResult<()>> + Send>> {
            Box::pin(async move {
                sqlx::query("COMMIT")
                    .execute(&mut *self.conn)
                    .await
                    .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
                Ok(())
            })
        }

        fn rollback_boxed(
            mut self: Box<Self>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TransactionResult<()>> + Send>> {
            Box::pin(async move {
                sqlx::query("ROLLBACK")
                    .execute(&mut *self.conn)
                    .await
                    .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
                Ok(())
            })
        }
    }

    /// SQLx `Postgres` transaction manager.
    /// SQLx `Postgres` 事务管理器。
    #[derive(Clone)]
    pub struct SqlxTransactionManager {
        pool: Arc<Pool<Postgres>>,
    }

    impl SqlxTransactionManager {
        /// Create from an existing pool.
        /// 从已有连接池创建。
        pub fn new(pool: Pool<Postgres>) -> Self {
            Self { pool: Arc::new(pool) }
        }

        /// Connect and create a manager.
        /// 连接并创建管理器。
        pub async fn connect(url: &str) -> TransactionResult<Self> {
            let pool = Pool::<Postgres>::connect(url)
                .await
                .map_err(|e| TransactionError::CreationFailed(e.to_string()))?;
            Ok(Self::new(pool))
        }
    }

    #[async_trait]
    impl TransactionManager for SqlxTransactionManager {
        async fn begin(&self, definition: &TransactionDefinition) -> TransactionResult<TransactionStatus> {
            if matches!(definition.propagation, Propagation::Required | Propagation::Supports) {
                if let Some(existing) = synchronization::current_status().await {
                    return Ok(existing);
                }
            }

            let mut conn = self
                .pool
                .acquire()
                .await
                .map_err(|e| TransactionError::CreationFailed(e.to_string()))?;

            sqlx::query("BEGIN")
                .execute(&mut *conn)
                .await
                .map_err(|e| TransactionError::CreationFailed(e.to_string()))?;

            let status = TransactionStatus::new(&definition.name);
            synchronization::bind_transaction(status.clone(), Box::new(SqlxLiveTx { conn })).await;
            Ok(status)
        }

        async fn commit(&self, status: TransactionStatus) -> TransactionResult<()> {
            if let Some(tx) = synchronization::take_transaction(&status).await {
                tx.commit_boxed().await?;
            }
            status.mark_completed();
            Ok(())
        }

        async fn rollback(&self, status: TransactionStatus) -> TransactionResult<()> {
            if let Some(tx) = synchronization::take_transaction(&status).await {
                tx.rollback_boxed().await?;
            }
            status.mark_completed();
            Ok(())
        }

        fn name(&self) -> &str {
            "sqlx-postgres"
        }
    }
}

#[cfg(feature = "sqlx")]
pub use imp::SqlxTransactionManager;
