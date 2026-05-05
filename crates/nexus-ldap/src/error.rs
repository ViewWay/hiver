//! Error types for nexus-ldap / nexus-ldap的错误类型
use thiserror::Error;

/// LDAP error types / LDAP错误类型
#[derive(Error, Debug)]
pub enum LdapError {
    #[error("LDAP connection error: {0}")]
    Connection(String),

    #[error("LDAP authentication error: {0}")]
    Authentication(String),

    #[error("LDAP operation error: {0}")]
    Operation(String),

    #[error("LDAP entry not found: {0}")]
    NotFound(String),

    #[error("LDAP schema violation: {0}")]
    SchemaViolation(String),
}

/// Result type for LDAP operations / LDAP操作的结果类型
pub type LdapResult<T> = Result<T, LdapError>;
