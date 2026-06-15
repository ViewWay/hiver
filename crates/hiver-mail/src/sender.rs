//! Mail sender trait and SMTP implementation.
//! 邮件发送器 trait 和 SMTP 实现。

use std::sync::Arc;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    transport::smtp::authentication::Credentials,
};
use tokio::sync::RwLock;

use crate::{
    config::{MailConfig, TlsMode},
    error::{MailError, MailResult},
    message::MailMessage,
};

/// Async mail sender trait — equivalent to Spring's `MailSender` / `JavaMailSender`.
/// 异步邮件发送器 trait — 等价于 Spring 的 `MailSender` / `JavaMailSender`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface JavaMailSender extends MailSender {
///     void send(MimeMessage mimeMessage) throws MailException;
///     MimeMessage createMimeMessage();
/// }
/// ```
#[async_trait::async_trait]
pub trait MailSender: Send + Sync
{
    /// Send a single email message.
    /// 发送单个邮件消息。
    async fn send(&self, message: MailMessage) -> MailResult<()>;

    /// Send multiple email messages.
    /// 发送多个邮件消息。
    async fn send_batch(&self, messages: Vec<MailMessage>) -> MailResult<Vec<MailResult<()>>>
    {
        let mut results = Vec::with_capacity(messages.len());
        for msg in messages
        {
            results.push(self.send(msg).await);
        }
        Ok(results)
    }
}

/// SMTP-based mail sender — equivalent to Spring's `JavaMailSenderImpl`.
/// 基于 SMTP 的邮件发送器 — 等价于 Spring 的 `JavaMailSenderImpl`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public JavaMailSender getJavaMailSender() {
///     JavaMailSenderImpl mailSender = new JavaMailSenderImpl();
///     mailSender.setHost("smtp.gmail.com");
///     mailSender.setPort(587);
///     mailSender.setUsername("user@gmail.com");
///     mailSender.setPassword("password");
///     return mailSender;
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run
/// use hiver_mail::{
///     config::MailConfig,
///     message::MailMessage,
///     sender::{MailSender, SmtpMailSender},
/// };
///
/// # async fn example() -> hiver_mail::error::MailResult<()> {
/// let config = MailConfig::builder()
///     .host("smtp.gmail.com")
///     .port(587)
///     .username("user@gmail.com")
///     .password("app-password")
///     .from("noreply@example.com")
///     .build();
///
/// let sender = SmtpMailSender::new(config)?;
///
/// let msg = MailMessage::builder()
///     .to("recipient@example.com")
///     .subject("Hello from Hiver")
///     .text("This is a test email.")
///     .build()?;
///
/// sender.send(msg).await?;
/// # Ok(())
/// # }
/// ```
pub struct SmtpMailSender
{
    config: MailConfig,
    transport: Arc<RwLock<AsyncSmtpTransport<Tokio1Executor>>>,
}

impl SmtpMailSender
{
    /// Create a new SMTP mail sender from configuration.
    /// 从配置创建新的 SMTP 邮件发送器。
    pub fn new(config: MailConfig) -> MailResult<Self>
    {
        config.validate()?;
        let transport = Self::build_transport(&config)?;
        Ok(Self {
            config,
            transport: Arc::new(RwLock::new(transport)),
        })
    }

    /// Get the configuration.
    /// 获取配置。
    pub fn config(&self) -> &MailConfig
    {
        &self.config
    }

    /// Test the SMTP connection.
    /// 测试 SMTP 连接。
    pub async fn test_connection(&self) -> MailResult<()>
    {
        let transport = self.transport.read().await;
        transport
            .test_connection()
            .await
            .map_err(|e| MailError::Transport(format!("connection test failed: {}", e)))?;
        Ok(())
    }

    fn build_transport(config: &MailConfig) -> MailResult<AsyncSmtpTransport<Tokio1Executor>>
    {
        let mut builder = match config.tls_mode
        {
            TlsMode::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                .port(config.port),
            TlsMode::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
                .map_err(|e| MailError::Transport(format!("TLS setup failed: {}", e)))?
                .port(config.port),
            TlsMode::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .map_err(|e| MailError::Transport(format!("TLS setup failed: {}", e)))?
                .port(config.port),
        };

        if let (Some(user), Some(pass)) = (&config.username, &config.password)
        {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }

        builder = builder.timeout(Some(std::time::Duration::from_secs(config.timeout_secs)));

        Ok(builder.build())
    }
}

#[async_trait::async_trait]
impl MailSender for SmtpMailSender
{
    async fn send(&self, message: MailMessage) -> MailResult<()>
    {
        let lettre_msg = message.to_lettre(self.config.from.as_deref())?;

        let transport = self.transport.read().await;
        transport
            .send(lettre_msg)
            .await
            .map_err(|e| MailError::Transport(format!("send failed: {}", e)))?;

        Ok(())
    }

    async fn send_batch(&self, messages: Vec<MailMessage>) -> MailResult<Vec<MailResult<()>>>
    {
        let mut results = Vec::with_capacity(messages.len());
        for msg in messages
        {
            results.push(self.send(msg).await);
        }
        Ok(results)
    }
}

impl Clone for SmtpMailSender
{
    fn clone(&self) -> Self
    {
        let transport = match Self::build_transport(&self.config)
        {
            Ok(t) => t,
            Err(_) => self.transport.blocking_read().clone(),
        };
        Self {
            config: self.config.clone(),
            transport: Arc::new(RwLock::new(transport)),
        }
    }
}

/// A no-op mail sender for testing — logs messages instead of sending.
/// 用于测试的空邮件发送器 — 记录消息而非发送。
///
/// # Rust Advantage / Rust优势
///
/// Spring does not have a built-in test mail sender. Hiver provides one
/// for zero-cost testing without an SMTP server.
///
/// Spring 没有内置的测试邮件发送器。Hiver 提供了一个无需 SMTP 服务器的零成本测试方案。
pub struct InMemoryMailSender
{
    sent: Arc<RwLock<Vec<MailMessage>>>,
}

impl InMemoryMailSender
{
    /// Create a new in-memory mail sender.
    /// 创建新的内存邮件发送器。
    pub fn new() -> Self
    {
        Self {
            sent: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get all sent messages.
    /// 获取所有已发送的消息。
    pub async fn sent_messages(&self) -> Vec<MailMessage>
    {
        self.sent.read().await.clone()
    }

    /// Get the count of sent messages.
    /// 获取已发送消息的数量。
    pub async fn sent_count(&self) -> usize
    {
        self.sent.read().await.len()
    }

    /// Clear all sent messages.
    /// 清除所有已发送的消息。
    pub async fn clear(&self)
    {
        self.sent.write().await.clear();
    }
}

impl Default for InMemoryMailSender
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MailSender for InMemoryMailSender
{
    async fn send(&self, message: MailMessage) -> MailResult<()>
    {
        tracing::info!(
            "InMemoryMailSender: recording message to {:?}, subject: {:?}",
            message.to,
            message.subject
        );
        self.sent.write().await.push(message);
        Ok(())
    }
}

impl Clone for InMemoryMailSender
{
    fn clone(&self) -> Self
    {
        Self {
            sent: self.sent.clone(),
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_in_memory_sender()
    {
        let sender = InMemoryMailSender::new();

        let msg = MailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .subject("Test")
            .text("Hello")
            .build()
            .unwrap();

        sender.send(msg).await.unwrap();
        assert_eq!(sender.sent_count().await, 1);

        let messages = sender.sent_messages().await;
        assert_eq!(messages[0].subject, "Test");
    }

    #[tokio::test]
    async fn test_in_memory_batch()
    {
        let sender = InMemoryMailSender::new();

        let msgs: Vec<MailMessage> = (0..3)
            .map(|i| {
                MailMessage::builder()
                    .from("noreply@example.com")
                    .to("user@example.com")
                    .subject(format!("Test {}", i))
                    .text("body")
                    .build()
                    .unwrap()
            })
            .collect();

        sender.send_batch(msgs).await.unwrap();
        assert_eq!(sender.sent_count().await, 3);
    }

    #[tokio::test]
    async fn test_in_memory_clear()
    {
        let sender = InMemoryMailSender::new();

        let msg = MailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .subject("Test")
            .text("Hello")
            .build()
            .unwrap();

        sender.send(msg).await.unwrap();
        assert_eq!(sender.sent_count().await, 1);

        sender.clear().await;
        assert_eq!(sender.sent_count().await, 0);
    }

    #[test]
    fn test_smtp_sender_creation()
    {
        let config = MailConfig::builder()
            .host("smtp.example.com")
            .port(587)
            .username("user")
            .password("pass")
            .build();

        let sender = SmtpMailSender::new(config);
        assert!(sender.is_ok());
    }

    #[test]
    fn test_smtp_sender_invalid_config()
    {
        let config = MailConfig::default();
        assert!(SmtpMailSender::new(config).is_err());
    }
}
