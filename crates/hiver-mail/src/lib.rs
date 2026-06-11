//! Hiver Mail — Spring Mail equivalent for the Hiver framework.
//! Hiver 邮件 — Hiver 框架的 Spring Mail 等价功能。
//!
//! Provides async email sending with SMTP, builder-pattern messages,
//! and an in-memory test sender. Equivalent to Spring Boot's `spring.mail.*`.
//!
//! 提供基于 SMTP 的异步邮件发送、构建器模式的消息、以及内存测试发送器。
//! 等价于 Spring Boot 的 `spring.mail.*`。
//!
//! # Spring Equivalent / Spring等价物
//!
//! | Hiver | Spring |
//! |-------|--------|
//! | `MailSender` | `JavaMailSender` |
//! | `SmtpMailSender` | `JavaMailSenderImpl` |
//! | `InMemoryMailSender` | *(no equivalent)* |
//! | `MailMessage` | `SimpleMailMessage` / `MimeMessage` |
//! | `MailConfig` | `spring.mail.*` properties |
//!
//! # Rust Advantage / Rust优势
//!
//! - Async-native SMTP (Spring uses synchronous `Transport.send()`)
//! - Built-in `InMemoryMailSender` for zero-cost testing
//! - Type-safe builder pattern (Spring uses mutable setters)
//!
//! # Quick Start / 快速开始
//!
//! ```rust,ignore
//! use hiver_mail::config::MailConfig;
//! use hiver_mail::sender::SmtpMailSender;
//! use hiver_mail::message::MailMessage;
//!
//! let config = MailConfig::builder()
//!     .host("smtp.gmail.com")
//!     .port(587)
//!     .username("user@gmail.com")
//!     .password("app-password")
//!     .from("noreply@example.com")
//!     .build();
//!
//! let sender = SmtpMailSender::new(config)?;
//!
//! let msg = MailMessage::builder()
//!     .to("recipient@example.com")
//!     .subject("Hello from Hiver")
//!     .html("<h1>Welcome</h1>")
//!     .build()?;
//!
//! sender.send(msg).await?;
//! ```

#![warn(missing_docs)]
#![allow(unreachable_pub)]

pub mod config;
pub mod error;
pub mod message;
pub mod sender;

pub use config::{MailConfig, MailConfigBuilder, TlsMode};
pub use error::{MailError, MailResult};
pub use message::{MailMessage, MailMessageBuilder};
pub use sender::{InMemoryMailSender, MailSender, SmtpMailSender};

/// Re-exports of commonly used types.
/// 常用类型的重新导出。
pub mod prelude
{
    pub use crate::{
        InMemoryMailSender, MailConfig, MailConfigBuilder, MailError, MailMessage,
        MailMessageBuilder, MailResult, MailSender, SmtpMailSender, TlsMode,
    };
}

/// Version of the mail module.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
