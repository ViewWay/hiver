//! Email message types — equivalent to Spring's `SimpleMailMessage` / `MimeMessage`.
//! 邮件消息类型 — 等价于 Spring 的 `SimpleMailMessage` / `MimeMessage`。

use std::fmt;

use lettre::message::{MultiPart, SinglePart, header::ContentType};

use crate::error::{MailError, MailResult};

/// A simple email message.
/// 简单邮件消息。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// SimpleMailMessage msg = new SimpleMailMessage();
/// msg.setFrom("noreply@example.com");
/// msg.setTo("user@example.com");
/// msg.setSubject("Welcome");
/// msg.setText("Hello, World!");
/// mailSender.send(msg);
/// ```
///
/// # Example / 示例
///
/// ```rust,no_run
/// use hiver_mail::message::MailMessage;
///
/// let msg = MailMessage::builder()
///     .from("noreply@example.com")
///     .to("user@example.com")
///     .subject("Welcome")
///     .text("Hello, World!")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct MailMessage
{
    /// From address (overrides config default).
    /// 发件人地址（覆盖配置默认值）。
    pub from: Option<String>,

    /// From display name.
    /// 发件人显示名。
    pub from_name: Option<String>,

    /// Reply-To address.
    /// 回复地址。
    pub reply_to: Option<String>,

    /// To recipients.
    /// 收件人。
    pub to: Vec<String>,

    /// CC recipients.
    /// 抄送。
    pub cc: Vec<String>,

    /// BCC recipients.
    /// 密送。
    pub bcc: Vec<String>,

    /// Email subject.
    /// 邮件主题。
    pub subject: String,

    /// Plain text body.
    /// 纯文本正文。
    pub text: Option<String>,

    /// HTML body.
    /// HTML 正文。
    pub html: Option<String>,

    /// Custom headers.
    /// 自定义头部。
    pub headers: Vec<(String, String)>,
}

impl MailMessage
{
    /// Create a message builder.
    /// 创建消息构建器。
    pub fn builder() -> MailMessageBuilder
    {
        MailMessageBuilder::default()
    }

    /// Convert to a lettre `Message`.
    /// 转换为 lettre `Message`。
    pub(crate) fn to_lettre(&self, default_from: Option<&str>) -> MailResult<lettre::Message>
    {
        let from_addr = self.from.as_deref().or(default_from).ok_or_else(|| {
            MailError::BuildError("no From address (set in message or config)".to_string())
        })?;

        let from = if let Some(ref name) = self.from_name
        {
            format!("{} <{}>", name, from_addr)
                .parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?
        }
        else
        {
            from_addr
                .parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?
        };

        let to_str = self.to.first().ok_or_else(|| {
            MailError::BuildError("at least one To recipient is required".to_string())
        })?;

        let to: lettre::message::Mailbox = to_str
            .parse()
            .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;

        let builder = lettre::Message::builder()
            .from(from)
            .to(to)
            .subject(&self.subject);

        // Add CC
        let mut builder = builder;
        for cc in &self.cc
        {
            let addr: lettre::message::Mailbox = cc
                .parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;
            builder = builder.cc(addr);
        }

        // Add BCC
        for bcc in &self.bcc
        {
            let addr: lettre::message::Mailbox = bcc
                .parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;
            builder = builder.bcc(addr);
        }

        // Add Reply-To
        if let Some(ref reply_to) = self.reply_to
        {
            let addr: lettre::message::Mailbox = reply_to
                .parse()
                .map_err(|e: lettre::address::AddressError| MailError::InvalidAddress(e.to_string()))?;
            builder = builder.reply_to(addr);
        }

        // Build body
        let message = match (&self.text, &self.html)
        {
            (Some(text), Some(html)) =>
            {
                // Multipart: text + HTML
                builder.multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text.clone()),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html.clone()),
                        ),
                )?
            }
            (Some(text), None) =>
            {
                // Plain text only
                builder.singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text.clone()),
                )?
            }
            (None, Some(html)) =>
            {
                // HTML only
                builder.singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html.clone()),
                )?
            }
            (None, None) =>
            {
                return Err(MailError::BuildError(
                    "email body is required (text or html)".to_string(),
                ));
            }
        };

        Ok(message)
    }
}

/// Builder for `MailMessage`.
/// `MailMessage` 的构建器。
#[derive(Default)]
pub struct MailMessageBuilder
{
    from: Option<String>,
    from_name: Option<String>,
    reply_to: Option<String>,
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: Option<String>,
    text: Option<String>,
    html: Option<String>,
    headers: Vec<(String, String)>,
}

impl MailMessageBuilder
{
    /// Set From address.
    /// 设置发件人地址。
    pub fn from(mut self, addr: impl Into<String>) -> Self
    {
        self.from = Some(addr.into());
        self
    }

    /// Set From display name.
    /// 设置发件人显示名。
    pub fn from_name(mut self, name: impl Into<String>) -> Self
    {
        self.from_name = Some(name.into());
        self
    }

    /// Set Reply-To address.
    /// 设置回复地址。
    pub fn reply_to(mut self, addr: impl Into<String>) -> Self
    {
        self.reply_to = Some(addr.into());
        self
    }

    /// Add a To recipient.
    /// 添加收件人。
    pub fn to(mut self, addr: impl Into<String>) -> Self
    {
        self.to.push(addr.into());
        self
    }

    /// Add a CC recipient.
    /// 添加抄送。
    pub fn cc(mut self, addr: impl Into<String>) -> Self
    {
        self.cc.push(addr.into());
        self
    }

    /// Add a BCC recipient.
    /// 添加密送。
    pub fn bcc(mut self, addr: impl Into<String>) -> Self
    {
        self.bcc.push(addr.into());
        self
    }

    /// Set subject.
    /// 设置主题。
    pub fn subject(mut self, subject: impl Into<String>) -> Self
    {
        self.subject = Some(subject.into());
        self
    }

    /// Set plain text body.
    /// 设置纯文本正文。
    pub fn text(mut self, text: impl Into<String>) -> Self
    {
        self.text = Some(text.into());
        self
    }

    /// Set HTML body.
    /// 设置 HTML 正文。
    pub fn html(mut self, html: impl Into<String>) -> Self
    {
        self.html = Some(html.into());
        self
    }

    /// Add a custom header.
    /// 添加自定义头部。
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Build the message.
    /// 构建消息。
    pub fn build(self) -> MailResult<MailMessage>
    {
        let subject = self.subject.ok_or_else(|| {
            MailError::BuildError("subject is required".to_string())
        })?;

        Ok(MailMessage {
            from: self.from,
            from_name: self.from_name,
            reply_to: self.reply_to,
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            subject,
            text: self.text,
            html: self.html,
            headers: self.headers,
        })
    }
}

impl fmt::Display for MailMessage
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "MailMessage {{ from: {:?}, to: {:?}, subject: {:?} }}",
            self.from, self.to, self.subject
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_simple_message()
    {
        let msg = MailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .subject("Test")
            .text("Hello, World!")
            .build()
            .unwrap();

        assert_eq!(msg.from.as_deref(), Some("noreply@example.com"));
        assert_eq!(msg.to, vec!["user@example.com"]);
        assert_eq!(msg.subject, "Test");
        assert_eq!(msg.text.as_deref(), Some("Hello, World!"));
    }

    #[test]
    fn test_html_message()
    {
        let msg = MailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .subject("HTML Test")
            .html("<h1>Hello</h1>")
            .build()
            .unwrap();

        assert!(msg.html.is_some());
        assert!(msg.text.is_none());
    }

    #[test]
    fn test_multipart_message()
    {
        let msg = MailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .cc("cc@example.com")
            .bcc("bcc@example.com")
            .reply_to("reply@example.com")
            .subject("Multipart")
            .text("Plain text")
            .html("<p>HTML</p>")
            .build()
            .unwrap();

        assert_eq!(msg.cc.len(), 1);
        assert_eq!(msg.bcc.len(), 1);
        assert!(msg.text.is_some());
        assert!(msg.html.is_some());
    }

    #[test]
    fn test_missing_subject()
    {
        let result = MailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .text("body")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_to_lettre_message()
    {
        let msg = MailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .subject("Test")
            .text("Hello")
            .build()
            .unwrap();

        // Verify lettre message can be built successfully
        let lettre_msg = msg.to_lettre(None).unwrap();
        let bytes = lettre_msg.formatted();
        assert!(std::str::from_utf8(&bytes).unwrap().contains("Subject: Test"));
    }

    #[test]
    fn test_to_lettre_uses_default_from()
    {
        let msg = MailMessage::builder()
            .to("user@example.com")
            .subject("Test")
            .text("Hello")
            .build()
            .unwrap();

        // Verify it uses default from
        let lettre_msg = msg.to_lettre(Some("default@example.com")).unwrap();
        let bytes = lettre_msg.formatted();
        assert!(std::str::from_utf8(&bytes).unwrap().contains("From:"));
    }

    #[test]
    fn test_to_lettre_no_from()
    {
        let msg = MailMessage::builder()
            .to("user@example.com")
            .subject("Test")
            .text("Hello")
            .build()
            .unwrap();

        assert!(msg.to_lettre(None).is_err());
    }
}
