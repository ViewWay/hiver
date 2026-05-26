//! Nexus Config - Configuration management module
//! Nexus配置 - 配置管理模块
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `@ConfigurationProperties` - `PropertiesConfig`
//! - `@Value` - Value extractor
//! - `Environment` - Environment
//! - `PropertySource` - `PropertySource`
//! - `@Profile` - Profile
//! - `ConfigFileApplicationListener` - `ConfigLoader`
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_config::{Config, PropertiesConfig};
//! use serde::Deserialize;
//!
//! #[derive(PropertiesConfig, Deserialize)]
//! #[prefix = "app.datasource"]
//! struct DataSourceConfig {
//!     url: String,
//!     username: String,
//!     password: String,
//! }
//!
//! #[derive(PropertiesConfig, Deserialize)]
//! #[prefix = "server"]
//! struct ServerConfig {
//!     port: u16,
//!     host: String,
//! }
//!
//! let config = Config::load().unwrap();
//! let server = config.get::<ServerConfig>().unwrap();
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

mod config;
mod encrypt;
mod environment;
mod error;
mod loader;
mod properties;
mod source;
mod value;

pub use config::{Config, ConfigBuilder, FileFormat, ReloadStrategy};
pub use encrypt::{ConfigEncryptor, EncryptError};
pub use environment::{ActiveProfiles, Environment, Profile};
pub use error::{ConfigError, ConfigResult};
pub use loader::{ConfigLoader, ConfigLoaderBuilder, Watcher};
pub use properties::{PropertiesConfig, PropertiesConfigRegistry};
pub use source::{PropertySource, PropertySourceBuilder, PropertySourceType};
pub use value::{Value, ValueExtractor};

/// Re-exports of commonly used types
/// 常用类型的重新导出
pub mod prelude {
    pub use super::{
        Config, ConfigBuilder, Environment, Profile, PropertiesConfig, PropertySource, Value,
        ValueExtractor,
    };
}

/// Version of the config module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
