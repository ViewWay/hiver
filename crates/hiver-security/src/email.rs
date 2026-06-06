//! Email service module.
//! 邮件服务模块。
//!
//! Provides configuration, message building, template rendering,
//! an `EmailSender` trait, and an async queue for outgoing emails.
//!
//! 提供配置、消息构建、模板渲染、`EmailSender` trait 和异步邮件发送队列。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_security::email::{EmailConfig, EmailMessage, EmailSender, SmtpEmailSender};
//!
//! let config = EmailConfig::new("smtp.example.com", 587, "user", "pass", "noreply@example.com");
//! let sender = SmtpEmailSender::new(config);
//!
//! let msg = EmailMessage::new()
//!     .to("alice@example.com")
//!     .subject("Hello")
//!     .body("Welcome!");
//! sender.send(msg).await?;
//! ```

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    sync::{Mutex, Notify},
};

/// Email error type.
/// 邮件错误类型。
#[derive(Error, Debug)]
pub enum EmailError
{
    /// Configuration error.
    /// 配置错误。
    #[error("Email configuration error: {0}")]
    ConfigError(String),

    /// SMTP connection or transmission error.
    /// SMTP 连接或传输错误。
    #[error("SMTP error: {0}")]
    SmtpError(String),

    /// Message construction error (missing fields, etc.).
    /// 消息构建错误（缺少字段等）。
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Template rendering error.
    /// 模板渲染错误。
    #[error("Template error: {0}")]
    TemplateError(String),

    /// Queue error.
    /// 队列错误。
    #[error("Queue error: {0}")]
    QueueError(String),
}

/// Email result type.
/// 邮件结果类型。
pub type EmailResult<T> = Result<T, EmailError>;

// ============================================================================
// EmailConfig / 邮件配置
// ============================================================================

/// Email service configuration.
/// 邮件服务配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig
{
    /// SMTP server hostname.
    /// SMTP 服务器主机名。
    pub smtp_host: String,

    /// SMTP server port.
    /// SMTP 服务器端口。
    pub smtp_port: u16,

    /// Authentication username.
    /// 认证用户名。
    pub username: String,

    /// Authentication password.
    /// 认证密码。
    pub password: String,

    /// Default "From" email address.
    /// 默认的发件人地址。
    pub from_address: String,

    /// Display name for the sender.
    /// 发件人显示名称。
    pub from_name: String,

    /// Whether to use TLS for the SMTP connection.
    /// 是否对 SMTP 连接使用 TLS。
    pub tls: bool,
}

impl EmailConfig
{
    /// Create a new email configuration.
    /// 创建新的邮件配置。
    pub fn new(
        smtp_host: impl Into<String>,
        smtp_port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
        from_address: impl Into<String>,
    ) -> Self
    {
        Self {
            smtp_host: smtp_host.into(),
            smtp_port,
            username: username.into(),
            password: password.into(),
            from_address: from_address.into(),
            from_name: String::new(),
            tls: true,
        }
    }

    /// Builder-style: set the display name.
    /// 构建器风格：设置显示名称。
    pub fn from_name(mut self, name: impl Into<String>) -> Self
    {
        self.from_name = name.into();
        self
    }

    /// Builder-style: enable or disable TLS.
    /// 构建器风格：启用或禁用 TLS。
    pub fn tls(mut self, tls: bool) -> Self
    {
        self.tls = tls;
        self
    }

    /// Validate required fields.
    /// 验证必填字段。
    pub fn validate(&self) -> EmailResult<()>
    {
        if self.smtp_host.is_empty()
        {
            return Err(EmailError::ConfigError("smtp_host is required".into()));
        }
        if self.from_address.is_empty() || !self.from_address.contains('@')
        {
            return Err(EmailError::ConfigError("from_address must be a valid email".into()));
        }
        Ok(())
    }
}

// ============================================================================
// EmailMessage & Attachment / 邮件消息与附件
// ============================================================================

/// File attachment for an email.
/// 邮件的文件附件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment
{
    /// File name.
    /// 文件名。
    pub name: String,

    /// MIME content type (e.g., "application/pdf").
    /// MIME 内容类型（例如 "application/pdf"）。
    pub content_type: String,

    /// Raw attachment data.
    /// 原始附件数据。
    pub data: Vec<u8>,
}

impl Attachment
{
    /// Create a new attachment.
    /// 创建新的附件。
    pub fn new(name: impl Into<String>, content_type: impl Into<String>, data: Vec<u8>) -> Self
    {
        Self {
            name: name.into(),
            content_type: content_type.into(),
            data,
        }
    }
}

/// Email message builder.
/// 邮件消息构建器。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage
{
    /// Recipient addresses.
    /// 收件人地址。
    pub to: Vec<String>,

    /// Carbon-copy addresses.
    /// 抄送地址。
    pub cc: Vec<String>,

    /// Blind carbon-copy addresses.
    /// 密送地址。
    pub bcc: Vec<String>,

    /// Subject line.
    /// 主题行。
    pub subject: String,

    /// Plain-text body.
    /// 纯文本正文。
    pub body: String,

    /// HTML body (optional; overrides `body` in HTML-capable clients).
    /// HTML 正文（可选；在支持 HTML 的客户端中覆盖 `body`）。
    pub html_body: Option<String>,

    /// Attachments.
    /// 附件。
    pub attachments: Vec<Attachment>,
}

impl Default for EmailMessage
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl EmailMessage
{
    /// Create a new empty email message.
    /// 创建新的空邮件消息。
    pub fn new() -> Self
    {
        Self {
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            body: String::new(),
            html_body: None,
            attachments: Vec::new(),
        }
    }

    /// Builder-style: add a recipient.
    /// 构建器风格：添加收件人。
    pub fn to(mut self, addr: impl Into<String>) -> Self
    {
        self.to.push(addr.into());
        self
    }

    /// Builder-style: add a CC recipient.
    /// 构建器风格：添加抄送收件人。
    pub fn cc(mut self, addr: impl Into<String>) -> Self
    {
        self.cc.push(addr.into());
        self
    }

    /// Builder-style: add a BCC recipient.
    /// 构建器风格：添加密送收件人。
    pub fn bcc(mut self, addr: impl Into<String>) -> Self
    {
        self.bcc.push(addr.into());
        self
    }

    /// Builder-style: set the subject.
    /// 构建器风格：设置主题。
    pub fn subject(mut self, subject: impl Into<String>) -> Self
    {
        self.subject = subject.into();
        self
    }

    /// Builder-style: set the plain-text body.
    /// 构建器风格：设置纯文本正文。
    pub fn body(mut self, body: impl Into<String>) -> Self
    {
        self.body = body.into();
        self
    }

    /// Builder-style: set the HTML body.
    /// 构建器风格：设置 HTML 正文。
    pub fn html_body(mut self, html: impl Into<String>) -> Self
    {
        self.html_body = Some(html.into());
        self
    }

    /// Builder-style: add an attachment.
    /// 构建器风格：添加附件。
    pub fn attachment(mut self, attachment: Attachment) -> Self
    {
        self.attachments.push(attachment);
        self
    }

    /// Validate that the message has at least one recipient and a subject.
    /// 验证消息至少有一个收件人和主题。
    pub fn validate(&self) -> EmailResult<()>
    {
        if self.to.is_empty() && self.cc.is_empty() && self.bcc.is_empty()
        {
            return Err(EmailError::InvalidMessage(
                "Email must have at least one recipient".into(),
            ));
        }
        if self.subject.is_empty()
        {
            return Err(EmailError::InvalidMessage("Subject is required".into()));
        }
        Ok(())
    }

    /// Total number of recipients (to + cc + bcc).
    /// 收件人总数（to + cc + bcc）。
    pub fn recipient_count(&self) -> usize
    {
        self.to.len() + self.cc.len() + self.bcc.len()
    }
}

// ============================================================================
// EmailTemplate / 邮件模板
// ============================================================================

/// Simple email template engine with `{{variable}}` substitution.
/// 简单的邮件模板引擎，支持 `{{variable}}` 替换。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate
{
    /// Template string containing `{{key}}` placeholders.
    /// 包含 `{{key}}` 占位符的模板字符串。
    pub template: String,

    /// Variable map for substitution.
    /// 用于替换的变量映射。
    pub variables: HashMap<String, String>,
}

impl EmailTemplate
{
    /// Create a new template.
    /// 创建新的模板。
    pub fn new(template: impl Into<String>) -> Self
    {
        Self {
            template: template.into(),
            variables: HashMap::new(),
        }
    }

    /// Set a template variable.
    /// 设置模板变量。
    pub fn variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Set multiple variables at once.
    /// 一次设置多个变量。
    pub fn variables(mut self, vars: HashMap<String, String>) -> Self
    {
        self.variables = vars;
        self
    }

    /// Render the template by replacing `{{key}}` placeholders with values.
    /// 通过将 `{{key}}` 占位符替换为值来渲染模板。
    ///
    /// Unknown variables are left as-is.
    /// 未知的变量保持原样。
    pub fn render(&self) -> EmailResult<String>
    {
        let mut result = self.template.clone();
        for (key, value) in &self.variables
        {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        Ok(result)
    }

    /// Render the template and produce an `EmailMessage`.
    /// 渲染模板并生成 `EmailMessage`。
    pub fn render_message(
        &self,
        to: impl Into<String>,
        subject: impl Into<String>,
    ) -> EmailResult<EmailMessage>
    {
        let body = self.render()?;
        Ok(EmailMessage::new().to(to).subject(subject).body(body))
    }
}

// ============================================================================
// EmailSender trait / 邮件发送器 trait
// ============================================================================

/// Trait for sending email messages.
/// 发送邮件消息的 trait。
///
/// Implementations may use SMTP, an external API (SendGrid, SES, etc.),
/// or a test double.
/// 实现可以使用 SMTP、外部 API（SendGrid、SES 等）或测试替身。
#[async_trait::async_trait]
pub trait EmailSender: Send + Sync
{
    /// Send an email message.
    /// 发送邮件消息。
    async fn send(&self, message: EmailMessage) -> EmailResult<()>;
}

/// SMTP-based email sender.
/// 基于 SMTP 的邮件发送器。
///
/// Opens a plain TCP connection to the SMTP server and performs the basic
/// SMTP protocol handshake (EHLO, MAIL FROM, RCPT TO, DATA, QUIT).
/// If the connection or any SMTP command fails, returns an error.
///
/// 打开到 SMTP 服务器的普通 TCP 连接并执行基本 SMTP 协议握手
/// （EHLO, MAIL FROM, RCPT TO, DATA, QUIT）。
/// 如果连接或任何 SMTP 命令失败，则返回错误。
#[derive(Debug, Clone)]
pub struct SmtpEmailSender
{
    config: EmailConfig,
}

impl SmtpEmailSender
{
    /// Create a new SMTP sender.
    /// 创建新的 SMTP 发送器。
    pub fn new(config: EmailConfig) -> Self
    {
        Self { config }
    }

    /// Get a reference to the configuration.
    /// 获取配置的引用。
    pub fn config(&self) -> &EmailConfig
    {
        &self.config
    }

    /// Read a complete SMTP response (multi-line `250-...` / final `250 ...`).
    /// 读取完整的 SMTP 响应（多行 `250-...` / 最终 `250 ...`）。
    async fn read_response<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) -> EmailResult<u16>
    {
        let mut line = String::new();
        let mut code: u16;
        loop
        {
            line.clear();
            let n = reader
                .read_line(&mut line)
                .await
                .map_err(|e| EmailError::SmtpError(format!("failed to read SMTP response: {e}")))?;
            if n == 0
            {
                return Err(EmailError::SmtpError("SMTP connection closed unexpectedly".into()));
            }
            if line.len() >= 4
            {
                code = line[..3]
                    .parse::<u16>()
                    .map_err(|_| EmailError::SmtpError(format!("invalid SMTP response: {line}")))?;
                // A space after the code means this is the final line of the response.
                if line.as_bytes()[3] == b' '
                {
                    break;
                }
            }
        }
        Ok(code)
    }

    /// Send an SMTP command and read the response code.
    /// 发送 SMTP 命令并读取响应码。
    async fn send_command<W: tokio::io::AsyncWrite + Unpin, R: tokio::io::AsyncBufRead + Unpin>(
        writer: &mut W,
        reader: &mut R,
        command: &str,
    ) -> EmailResult<u16>
    {
        writer
            .write_all(command.as_bytes())
            .await
            .map_err(|e| EmailError::SmtpError(format!("failed to write SMTP command: {e}")))?;
        writer
            .flush()
            .await
            .map_err(|e| EmailError::SmtpError(format!("failed to flush SMTP command: {e}")))?;
        Self::read_response(reader).await
    }
}

#[async_trait::async_trait]
impl EmailSender for SmtpEmailSender
{
    async fn send(&self, message: EmailMessage) -> EmailResult<()>
    {
        self.config.validate()?;
        message.validate()?;

        let addr = format!("{}:{}", self.config.smtp_host, self.config.smtp_port);
        let stream = tokio::net::TcpStream::connect(&addr).await.map_err(|e| {
            EmailError::SmtpError(format!("failed to connect to SMTP server {addr}: {e}"))
        })?;

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = tokio::io::BufReader::new(reader);

        // Read server greeting.
        let greeting_code = Self::read_response(&mut reader).await?;
        if !(greeting_code >= 200 && greeting_code < 300)
        {
            return Err(EmailError::SmtpError(format!(
                "SMTP server greeting failed with code {greeting_code}"
            )));
        }

        // EHLO.
        let hostname = "hiver.local";
        let ehlo_code =
            Self::send_command(&mut writer, &mut reader, &format!("EHLO {hostname}\r\n")).await?;
        if ehlo_code != 250
        {
            return Err(EmailError::SmtpError(format!("SMTP EHLO rejected with code {ehlo_code}")));
        }

        // MAIL FROM.
        let mail_from_cmd = format!("MAIL FROM:<{}>\r\n", self.config.from_address);
        let mail_code = Self::send_command(&mut writer, &mut reader, &mail_from_cmd).await?;
        if mail_code != 250
        {
            return Err(EmailError::SmtpError(format!(
                "SMTP MAIL FROM rejected with code {mail_code}"
            )));
        }

        // RCPT TO (for each recipient).
        for recipient in message
            .to
            .iter()
            .chain(message.cc.iter())
            .chain(message.bcc.iter())
        {
            let rcpt_cmd = format!("RCPT TO:<{recipient}>\r\n");
            let rcpt_code = Self::send_command(&mut writer, &mut reader, &rcpt_cmd).await?;
            if rcpt_code != 250
            {
                return Err(EmailError::SmtpError(format!(
                    "SMTP RCPT TO <{recipient}> rejected with code {rcpt_code}"
                )));
            }
        }

        // DATA.
        let data_code = Self::send_command(&mut writer, &mut reader, "DATA\r\n").await?;
        if data_code != 354
        {
            return Err(EmailError::SmtpError(format!("SMTP DATA rejected with code {data_code}")));
        }

        // Build a minimal RFC 5322 message.
        let mut data_payload = String::new();
        data_payload.push_str(&format!("From: {}\r\n", self.config.from_address));
        for to_addr in &message.to
        {
            data_payload.push_str(&format!("To: {to_addr}\r\n"));
        }
        for cc_addr in &message.cc
        {
            data_payload.push_str(&format!("Cc: {cc_addr}\r\n"));
        }
        data_payload.push_str(&format!("Subject: {}\r\n", message.subject));
        data_payload.push_str("Content-Type: text/plain; charset=utf-8\r\n");
        data_payload.push_str("\r\n");
        // Dot-stuffing: lines starting with "." get an extra "." prepended.
        for line in message.body.lines()
        {
            if line.starts_with('.')
            {
                data_payload.push('.');
            }
            data_payload.push_str(line);
            data_payload.push_str("\r\n");
        }
        if !message.body.ends_with('\n')
        {
            data_payload.push_str("\r\n");
        }
        data_payload.push_str(".\r\n");

        writer
            .write_all(data_payload.as_bytes())
            .await
            .map_err(|e| {
                EmailError::SmtpError(format!("failed to write SMTP DATA payload: {e}"))
            })?;
        writer.flush().await.map_err(|e| {
            EmailError::SmtpError(format!("failed to flush SMTP DATA payload: {e}"))
        })?;

        let data_end_code = Self::read_response(&mut reader).await?;
        if data_end_code != 250
        {
            return Err(EmailError::SmtpError(format!(
                "SMTP DATA end rejected with code {data_end_code}"
            )));
        }

        // QUIT.
        let _ = Self::send_command(&mut writer, &mut reader, "QUIT\r\n").await;

        tracing::info!(
            "[SMTP] Sent email via {}:{} | From: {} | To: {:?} | Subject: {}",
            self.config.smtp_host,
            self.config.smtp_port,
            self.config.from_address,
            message.to,
            message.subject,
        );

        Ok(())
    }
}

// ============================================================================
// EmailQueue / 邮件队列
// ============================================================================

/// Async email queue for deferred sending.
/// 用于延迟发送的异步邮件队列。
///
/// Messages are enqueued and processed in batches.
/// 消息入队后批量处理。
#[derive(Debug)]
pub struct EmailQueue
{
    sender: Arc<Mutex<Vec<EmailMessage>>>,
    notify: Arc<Notify>,
}

impl Default for EmailQueue
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl EmailQueue
{
    /// Create a new empty email queue.
    /// 创建新的空邮件队列。
    pub fn new() -> Self
    {
        Self {
            sender: Arc::new(Mutex::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Enqueue a message for later sending.
    /// 将消息入队以供后续发送。
    pub async fn enqueue(&self, message: EmailMessage)
    {
        let mut queue = self.sender.lock().await;
        queue.push(message);
        self.notify.notify_one();
    }

    /// Drain the queue and attempt to send all pending messages via the provided sender.
    /// 排空队列并通过提供的发送器尝试发送所有待处理消息。
    ///
    /// Returns the number of messages successfully sent.
    /// 返回成功发送的消息数。
    pub async fn process_queue(&self, sender: &dyn EmailSender) -> EmailResult<usize>
    {
        let mut queue = self.sender.lock().await;
        let batch = std::mem::take(&mut *queue);
        drop(queue);

        let mut sent = 0usize;
        for message in &batch
        {
            if sender.send(message.clone()).await.is_ok()
            {
                sent += 1;
            }
        }

        Ok(sent)
    }

    /// Number of messages currently in the queue.
    /// 当前队列中的消息数。
    pub async fn len(&self) -> usize
    {
        self.sender.lock().await.len()
    }

    /// Check if the queue is empty.
    /// 检查队列是否为空。
    pub async fn is_empty(&self) -> bool
    {
        self.sender.lock().await.is_empty()
    }

    /// Clear all pending messages.
    /// 清除所有待处理消息。
    pub async fn clear(&self)
    {
        self.sender.lock().await.clear();
    }
}

// ============================================================================
// Tests / 测试
// ============================================================================

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    // ── EmailConfig / 邮件配置 ──

    #[test]
    fn test_config_new()
    {
        let cfg = EmailConfig::new("smtp.host", 587, "user", "pass", "from@host");
        assert_eq!(cfg.smtp_host, "smtp.host");
        assert_eq!(cfg.smtp_port, 587);
        assert!(cfg.tls);
    }

    #[test]
    fn test_config_builder()
    {
        let cfg = EmailConfig::new("smtp.host", 25, "u", "p", "f@h")
            .from_name("Hiver")
            .tls(false);
        assert_eq!(cfg.from_name, "Hiver");
        assert!(!cfg.tls);
    }

    #[test]
    fn test_config_validate_ok()
    {
        let cfg = EmailConfig::new("smtp.host", 587, "u", "p", "from@host");
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_config_validate_missing_host()
    {
        let cfg = EmailConfig::new("", 587, "u", "p", "from@host");
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_from()
    {
        let cfg = EmailConfig::new("smtp.host", 587, "u", "p", "no-at-sign");
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_validate_empty_from()
    {
        let cfg = EmailConfig::new("smtp.host", 587, "u", "p", "");
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_config_serialization()
    {
        let cfg = EmailConfig::new("smtp.host", 587, "u", "p", "from@host");
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: EmailConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.smtp_host, cfg.smtp_host);
    }

    // ── EmailMessage / 邮件消息 ──

    #[test]
    fn test_message_builder()
    {
        let msg = EmailMessage::new()
            .to("alice@ex.com")
            .to("bob@ex.com")
            .cc("cc@ex.com")
            .bcc("bcc@ex.com")
            .subject("Test")
            .body("Hello")
            .html_body("<p>Hello</p>");

        assert_eq!(msg.to.len(), 2);
        assert_eq!(msg.cc.len(), 1);
        assert_eq!(msg.bcc.len(), 1);
        assert_eq!(msg.subject, "Test");
        assert!(msg.html_body.is_some());
        assert_eq!(msg.recipient_count(), 4);
    }

    #[test]
    fn test_message_validate_ok()
    {
        let msg = EmailMessage::new().to("a@b.com").subject("Hi").body("Body");
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_message_validate_no_recipient()
    {
        let msg = EmailMessage::new().subject("Hi");
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_validate_no_subject()
    {
        let msg = EmailMessage::new().to("a@b.com");
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_serialization()
    {
        let msg = EmailMessage::new().to("a@b.com").subject("S").body("B");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: EmailMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.subject, "S");
    }

    // ── Attachment / 附件 ──

    #[test]
    fn test_attachment_new()
    {
        let att = Attachment::new("file.pdf", "application/pdf", vec![0, 1, 2]);
        assert_eq!(att.name, "file.pdf");
        assert_eq!(att.data.len(), 3);
    }

    // ── EmailTemplate / 邮件模板 ──

    #[test]
    fn test_template_render()
    {
        let tmpl = EmailTemplate::new("Hello {{name}}, welcome to {{app}}!")
            .variable("name", "Alice")
            .variable("app", "Hiver");
        let rendered = tmpl.render().unwrap();
        assert_eq!(rendered, "Hello Alice, welcome to Hiver!");
    }

    #[test]
    fn test_template_unknown_variable_unchanged()
    {
        let tmpl = EmailTemplate::new("Hello {{name}}! {{unknown}}").variable("name", "Bob");
        let rendered = tmpl.render().unwrap();
        assert_eq!(rendered, "Hello Bob! {{unknown}}");
    }

    #[test]
    fn test_template_no_variables()
    {
        let tmpl = EmailTemplate::new("Static content");
        assert_eq!(tmpl.render().unwrap(), "Static content");
    }

    #[test]
    fn test_template_render_message()
    {
        let tmpl = EmailTemplate::new("Hi {{user}}, code: {{code}}")
            .variable("user", "Eve")
            .variable("code", "12345");
        let msg = tmpl.render_message("eve@ex.com", "Your code").unwrap();
        assert_eq!(msg.to[0], "eve@ex.com");
        assert_eq!(msg.subject, "Your code");
        assert!(msg.body.contains("Eve"));
        assert!(msg.body.contains("12345"));
    }

    #[test]
    fn test_template_multiple_same_variable()
    {
        let tmpl = EmailTemplate::new("{{x}} and {{x}}").variable("x", "val");
        let rendered = tmpl.render().unwrap();
        assert_eq!(rendered, "val and val");
    }

    #[test]
    fn test_template_set_variables_batch()
    {
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), "1".to_string());
        vars.insert("b".to_string(), "2".to_string());
        let tmpl = EmailTemplate::new("{{a}}-{{b}}").variables(vars);
        assert_eq!(tmpl.render().unwrap(), "1-2");
    }

    // ── SmtpEmailSender / SMTP 邮件发送器 ──

    #[tokio::test]
    async fn test_smtp_sender_connection_failure()
    {
        // Non-existent host will fail at TCP connect.
        let cfg = EmailConfig::new("smtp.nonexistent.invalid", 587, "u", "p", "from@host");
        let sender = SmtpEmailSender::new(cfg);
        let msg = EmailMessage::new().to("a@b.com").subject("S").body("B");
        let err = sender.send(msg).await.unwrap_err();
        assert!(err.to_string().contains("SMTP"));
    }

    #[tokio::test]
    async fn test_smtp_sender_invalid_config()
    {
        let cfg = EmailConfig::new("", 587, "u", "p", "bad");
        let sender = SmtpEmailSender::new(cfg);
        let msg = EmailMessage::new().to("a@b.com").subject("S").body("B");
        assert!(sender.send(msg).await.is_err());
    }

    #[tokio::test]
    async fn test_smtp_sender_invalid_message()
    {
        let cfg = EmailConfig::new("smtp.host", 587, "u", "p", "from@host");
        let sender = SmtpEmailSender::new(cfg);
        let msg = EmailMessage::new(); // no recipients, no subject
        assert!(sender.send(msg).await.is_err());
    }

    #[test]
    fn test_smtp_sender_config_access()
    {
        let cfg = EmailConfig::new("h", 25, "u", "p", "f@h");
        let sender = SmtpEmailSender::new(cfg);
        assert_eq!(sender.config().smtp_host, "h");
    }

    // ── EmailQueue / 邮件队列 ──

    #[tokio::test]
    async fn test_queue_enqueue_and_len()
    {
        let queue = EmailQueue::new();
        assert!(queue.is_empty().await);

        let msg = EmailMessage::new().to("a@b.com").subject("S").body("B");
        queue.enqueue(msg).await;
        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_queue_process_all_fail()
    {
        // All sends fail because the host is unreachable; queue is still drained.
        let queue = EmailQueue::new();
        let cfg = EmailConfig::new("smtp.nonexistent.invalid", 587, "u", "p", "from@host");
        let sender = SmtpEmailSender::new(cfg);

        for i in 0..3
        {
            let msg = EmailMessage::new()
                .to(format!("a{}@b.com", i))
                .subject(format!("S{}", i))
                .body("B");
            queue.enqueue(msg).await;
        }

        let sent = queue.process_queue(&sender).await.unwrap();
        assert_eq!(sent, 0);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_queue_clear()
    {
        let queue = EmailQueue::new();
        let msg = EmailMessage::new().to("a@b.com").subject("S").body("B");
        queue.enqueue(msg).await;
        assert_eq!(queue.len().await, 1);
        queue.clear().await;
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_queue_process_with_failures()
    {
        // Use a sender with invalid config so all sends fail
        let queue = EmailQueue::new();
        let bad_cfg = EmailConfig::new("", 587, "u", "p", "bad");
        let sender = SmtpEmailSender::new(bad_cfg);

        let msg = EmailMessage::new().to("a@b.com").subject("S").body("B");
        queue.enqueue(msg).await;

        let sent = queue.process_queue(&sender).await.unwrap();
        assert_eq!(sent, 0);
        assert!(queue.is_empty().await); // messages were drained even on failure
    }

    #[tokio::test]
    async fn test_queue_process_empty()
    {
        let queue = EmailQueue::new();
        let cfg = EmailConfig::new("smtp.host", 587, "u", "p", "from@host");
        let sender = SmtpEmailSender::new(cfg);
        let sent = queue.process_queue(&sender).await.unwrap();
        assert_eq!(sent, 0);
    }
}
