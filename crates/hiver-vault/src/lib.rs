//! hiver-vault — Spring Vault equivalent for `HashiCorp` Vault secrets management
//! hiver-vault — `HashiCorp` Vault 密钥管理（Spring Vault 等价实现）
//!
//! # Overview / 概述
//!
//! `hiver-vault` provides a complete Rust client for `HashiCorp` Vault,
//! equivalent to Spring Vault in the Java/Spring ecosystem.
//!
//! `hiver-vault` 提供完整的 `HashiCorp` Vault Rust 客户端，
//! 等价于 Java/Spring 生态中的 Spring Vault。
//!
//! # Features / 功能
//!
//! - **`VaultClient`** — HTTP client with TLS and token auth / 支持 TLS 和 Token 认证的 HTTP 客户端
//! - **Auth** — Token & `AppRole` authentication / Token 和 `AppRole` 认证
//! - **Secret** — CRUD operations on secrets / 密钥的增删改查操作
//! - **Lease** — Lease renewal and revocation / 租约续订和撤销
//! - **KV** — KV v1 and v2 secret backends / KV v1 和 v2 密钥后端
//! - **Transit** — Encrypt/decrypt as a service / 加密/解密服务
//! - **PKI** — Certificate management / 证书管理
//! - **Health** — Health check and seal status / 健康检查和封印状态
//!
//! # Quick Start / 快速开始
//!
//! ```rust,no_run,ignore
//! use hiver_vault::{VaultClient, VaultConfig, auth::TokenAuth};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = VaultConfig::builder()
//!         .address("https://127.0.0.1:8200")
//!         .token("my-root-token")
//!         .build()?;
//!
//!     let client = VaultClient::connect(config)?;
//!
//!     // Read a secret / 读取密钥
//!     let secret = client.kv_v2("secret")
//!         .read("myapp/config")
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

pub mod auth;
pub mod auth_jwt;
pub mod client;
pub mod error;
pub mod health;
pub mod kv;
pub mod kv_v2;
pub mod lease;
pub mod pki;
pub mod secret;
pub mod transit;

// Re-exports / 重导出
pub use auth_jwt::{JwtAuth, JwtAuthManager, JwtRoleConfig};
pub use client::{VaultClient, VaultConfig};
pub use error::VaultError;

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;
