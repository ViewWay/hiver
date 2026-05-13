//! Nexus Transaction - Transaction management module
//! Nexus事务 - 事务管理模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@Transactional` - Transactional
//! - `TransactionTemplate` - `TransactionTemplate`
//! - `TransactionManager` - `TransactionManager`
//! - `PlatformTransactionManager` - `PlatformTransactionManager`
//! - `@EnableTransactionManagement` - `EnableTransactionManagement`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_tx::{Transactional, TransactionTemplate};
//!
//! struct UserService {
//!     // ... fields
//! }
//!
//! impl UserService {
//!     // Equivalent to @Transactional
//!     #[transactional]
//!     async fn create_user(&self, user: User) -> Result<User, Error> {
//!         // Database operations
//!         Ok(user)
//!     }
//!
//!     // With specific isolation level
//!     #[transactional(isolation = "SERIALIZABLE")]
//!     async fn transfer_money(&self, from: u64, to: u64, amount: f64) -> Result<(), Error> {
//!         // Transfer logic
//!         Ok(())
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow dead_code: This is a framework library with many public APIs that are
// provided for users but not used internally. This is expected and intentional.
// 允许 dead_code：这是一个框架库，包含许多公共 API 供用户使用但内部未使用。
// 这是预期且有意的设计。
#![allow(dead_code)]

#[cfg(test)]
mod tests;

mod error;
mod isolation;
mod manager;
pub mod synchronization;
#[cfg(feature = "sqlx")]
mod sqlx_manager;
mod propagation;
mod request_ext;
mod status;
mod template;
mod transaction;
mod transactional;

pub use error::{TransactionError, TransactionResult};
pub use isolation::IsolationLevel;
pub use manager::{NoopTransactionManager, TransactionDefinition, TransactionManager, TransactionManagerBuilder, global_tx_manager, set_global_tx_manager};
pub use propagation::Propagation;
pub use request_ext::{
    TransactionContextExt, get_transaction_from_request, has_active_transaction_in_request,
};
pub use status::TransactionStatus;
pub use template::TransactionTemplate;
pub use transaction::Transaction;
pub use transactional::{Transactional, TransactionalOptions};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        IsolationLevel, Propagation, Transaction, TransactionError, TransactionManager,
        TransactionResult, TransactionStatus, TransactionTemplate, Transactional,
    };
}

/// Version of the transaction module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default transaction timeout in seconds
/// 默认事务超时时间（秒）
pub const DEFAULT_TX_TIMEOUT_SECS: u64 = 30;

/// Default transaction name
/// 默认事务名称
pub const DEFAULT_TX_NAME: &str = "default";

#[cfg(feature = "sqlx")]
pub use sqlx_manager::SqlxTransactionManager;
