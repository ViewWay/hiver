//! AMQP configuration
//! AMQP配置

use serde::{Deserialize, Serialize};

/// AMQP configuration
/// AMQP配置
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Configuration
/// public class RabbitConfig {
///     @Bean
///     public ConnectionFactory connectionFactory() {
///         return new CachingConnectionFactory("localhost");
///     }
///
///     @Bean
///     public RabbitTemplate rabbitTemplate() {
///         return new RabbitTemplate(connectionFactory());
///     }
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmqpConfig
{
    /// Connection URL
    /// 连接URL
    pub url: String,

    /// Host
    /// 主机
    pub host: String,

    /// Port
    /// 端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// Username
    /// 用户名
    pub username: String,

    /// Password
    /// 密码
    pub password: String,

    /// Virtual host
    /// 虚拟主机
    #[serde(default = "default_vhost")]
    pub vhost: String,

    /// Use SSL/TLS
    /// 使用SSL/TLS
    #[serde(default)]
    pub ssl: bool,

    /// Connection timeout in seconds
    /// 连接超时时间（秒）
    #[serde(default = "default_timeout")]
    pub connection_timeout_secs: u64,

    /// Heartbeat timeout in seconds
    /// 心跳超时时间（秒）
    #[serde(default = "default_heartbeat")]
    pub heartbeat_secs: u16,

    /// Channel max
    /// 通道最大数量
    #[serde(default = "default_channel_max")]
    pub channel_max: u16,

    /// Frame max
    /// 帧最大大小
    #[serde(default = "default_frame_max")]
    pub frame_max: u32,

    /// Automatic recovery
    /// 自动恢复
    #[serde(default = "default_auto_recovery")]
    pub automatic_recovery: bool,

    /// Network recovery interval in milliseconds
    /// 网络恢复间隔（毫秒）
    #[serde(default = "default_recovery_interval")]
    pub network_recovery_interval_ms: u64,
}

impl Default for AmqpConfig
{
    fn default() -> Self
    {
        Self {
            url: "amqp://localhost:5672".to_string(),
            host: "localhost".to_string(),
            port: default_port(),
            username: "guest".to_string(),
            password: "guest".to_string(),
            vhost: default_vhost(),
            ssl: false,
            connection_timeout_secs: default_timeout(),
            heartbeat_secs: default_heartbeat(),
            channel_max: default_channel_max(),
            frame_max: default_frame_max(),
            automatic_recovery: default_auto_recovery(),
            network_recovery_interval_ms: default_recovery_interval(),
        }
    }
}

impl AmqpConfig
{
    /// Create new AMQP configuration
    /// 创建新的AMQP配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set connection URL
    /// 设置连接URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self
    {
        self.url = url.into();
        self
    }

    /// Set host and port
    /// 设置主机和端口
    pub fn with_host(mut self, host: impl Into<String>, port: u16) -> Self
    {
        self.host = host.into();
        self.port = port;
        self.url.clear();
        self
    }

    /// Set credentials
    /// 设置凭据
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self
    {
        self.username = username.into();
        self.password = password.into();
        self.url.clear();
        self
    }

    /// Set virtual host
    /// 设置虚拟主机
    pub fn with_vhost(mut self, vhost: impl Into<String>) -> Self
    {
        self.vhost = vhost.into();
        self
    }

    /// Enable SSL
    /// 启用SSL
    pub fn with_ssl(mut self, ssl: bool) -> Self
    {
        self.ssl = ssl;
        if ssl && self.port == 5672
        {
            self.port = 5671;
        }
        self.url.clear();
        self
    }

    /// Build connection URL
    /// 构建连接URL
    pub fn build_url(&self) -> String
    {
        if !self.url.is_empty() && self.url.starts_with("amqp")
        {
            return self.url.clone();
        }

        let protocol = if self.ssl { "amqps" } else { "amqp" };
        let username = urlencoding::encode(&self.username);
        let password = urlencoding::encode(&self.password);
        format!(
            "{}://{}:{}@{}:{}{}",
            protocol, username, password, self.host, self.port, self.vhost
        )
    }
}

/// Connection configuration
/// 连接配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionConfig
{
    /// Prefetch count (`QoS`)
    /// 预取数量（QoS）
    #[serde(default = "default_prefetch")]
    pub prefetch_count: u16,

    /// Confirm select (publisher confirms)
    /// 确认选择（发布者确认）
    #[serde(default)]
    pub publisher_confirms: bool,

    /// Confirm timeout in seconds
    /// 确认超时时间（秒）
    #[serde(default = "default_confirm_timeout")]
    pub confirm_timeout_secs: u64,
}

impl Default for ConnectionConfig
{
    fn default() -> Self
    {
        Self {
            prefetch_count: default_prefetch(),
            publisher_confirms: false,
            confirm_timeout_secs: default_confirm_timeout(),
        }
    }
}

impl ConnectionConfig
{
    /// Create new connection configuration
    /// 创建新的连接配置
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Set prefetch count
    /// 设置预取数量
    pub fn with_prefetch(mut self, count: u16) -> Self
    {
        self.prefetch_count = count;
        self
    }

    /// Enable publisher confirms
    /// 启用发布者确认
    pub fn with_publisher_confirms(mut self, enabled: bool) -> Self
    {
        self.publisher_confirms = enabled;
        self
    }
}

fn default_port() -> u16
{
    5672
}

fn default_vhost() -> String
{
    "/".to_string()
}

fn default_timeout() -> u64
{
    10
}

fn default_heartbeat() -> u16
{
    60
}

fn default_channel_max() -> u16
{
    2047
}

fn default_frame_max() -> u32
{
    131_072
}

fn default_auto_recovery() -> bool
{
    true
}

fn default_recovery_interval() -> u64
{
    5000
}

fn default_prefetch() -> u16
{
    1
}

fn default_confirm_timeout() -> u64
{
    30
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    /// Test default AmqpConfig values / 测试默认 AmqpConfig 值
    #[test]
    fn test_amqp_config_defaults()
    {
        let config = AmqpConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5672);
        assert_eq!(config.username, "guest");
        assert_eq!(config.password, "guest");
        assert_eq!(config.vhost, "/");
        assert!(!config.ssl);
        assert_eq!(config.connection_timeout_secs, 10);
        assert_eq!(config.heartbeat_secs, 60);
        assert_eq!(config.channel_max, 2047);
        assert_eq!(config.frame_max, 131_072);
        assert!(config.automatic_recovery);
        assert_eq!(config.network_recovery_interval_ms, 5000);
    }

    /// Test AmqpConfig builder pattern / 测试 AmqpConfig 构建器模式
    #[test]
    fn test_amqp_config_builder()
    {
        let config = AmqpConfig::new()
            .with_host("rabbit.example.com", 5673)
            .with_credentials("admin", "secret")
            .with_vhost("/production");

        assert_eq!(config.host, "rabbit.example.com");
        assert_eq!(config.port, 5673);
        assert_eq!(config.username, "admin");
        assert_eq!(config.password, "secret");
        assert_eq!(config.vhost, "/production");
    }

    /// Test build_url generates correct AMQP URL / 测试 build_url 生成正确的 AMQP URL
    #[test]
    fn test_amqp_config_build_url()
    {
        let config = AmqpConfig::new()
            .with_host("rabbit.example.com", 5672)
            .with_credentials("admin", "secret");

        let url = config.build_url();
        assert!(url.starts_with("amqp://"));
        assert!(url.contains("admin:secret@"));
        assert!(url.contains("rabbit.example.com:5672"));
    }

    /// Test SSL changes protocol to amqps and port to 5671 / 测试 SSL 将协议改为 amqps 并将端口改为
    /// 5671
    #[test]
    fn test_amqp_config_ssl()
    {
        let config = AmqpConfig::default().with_ssl(true);
        assert!(config.ssl);
        assert_eq!(config.port, 5671);

        let url = config.build_url();
        assert!(url.starts_with("amqps://"));
    }

    /// Test build_url returns raw URL when already amqp:// / 测试已有 amqp:// URL 时直接返回
    #[test]
    fn test_amqp_config_build_url_passthrough()
    {
        let config = AmqpConfig::default().with_url("amqp://custom:9999/vhost");
        assert_eq!(config.build_url(), "amqp://custom:9999/vhost");
    }

    /// Test build_url URL-encodes special characters in credentials / 测试 build_url
    /// 对凭据中的特殊字符进行 URL 编码
    #[test]
    fn test_amqp_config_build_url_encodes_credentials()
    {
        let config = AmqpConfig::new().with_credentials("user@domain", "p@ss:w0rd");
        let url = config.build_url();
        // '@' and ':' should be percent-encoded in the user/password portion
        assert!(url.contains("user%40domain"));
        assert!(url.contains("p%40ss%3Aw0rd"));
    }

    /// Test ConnectionConfig defaults / 测试 ConnectionConfig 默认值
    #[test]
    fn test_connection_config_defaults()
    {
        let config = ConnectionConfig::default();
        assert_eq!(config.prefetch_count, 1);
        assert!(!config.publisher_confirms);
        assert_eq!(config.confirm_timeout_secs, 30);
    }

    /// Test ConnectionConfig builder / 测试 ConnectionConfig 构建器
    #[test]
    fn test_connection_config_builder()
    {
        let config = ConnectionConfig::new()
            .with_prefetch(10)
            .with_publisher_confirms(true);
        assert_eq!(config.prefetch_count, 10);
        assert!(config.publisher_confirms);
    }

    /// Test AmqpConfig serialization round-trip / 测试 AmqpConfig 序列化往返
    #[test]
    fn test_amqp_config_serde_roundtrip()
    {
        let config = AmqpConfig::new()
            .with_host("broker.local", 5672)
            .with_credentials("user", "pass")
            .with_vhost("/test");
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AmqpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.host, deserialized.host);
        assert_eq!(config.port, deserialized.port);
        assert_eq!(config.username, deserialized.username);
        assert_eq!(config.vhost, deserialized.vhost);
    }
}
