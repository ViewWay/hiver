//! Error types for nexus-ldap / nexus-ldap的错误类型
use thiserror::Error;

/// LDAP error types / LDAP错误类型
#[derive(Error, Debug)]
pub enum LdapError {
    /// Connection error / 连接错误
    #[error("LDAP connection error: {0}")]
    Connection(String),

    /// Authentication error / 认证错误
    #[error("LDAP authentication error: {0}")]
    Authentication(String),

    /// Operation error / 操作错误
    #[error("LDAP operation error: {0}")]
    Operation(String),

    /// Entry not found error / 条目未找到错误
    #[error("LDAP entry not found: {0}")]
    NotFound(String),

    /// Schema violation error / 模式违规错误
    #[error("LDAP schema violation: {0}")]
    SchemaViolation(String),
}

/// Result type for LDAP operations / LDAP操作的结果类型
pub type LdapResult<T> = Result<T, LdapError>;
