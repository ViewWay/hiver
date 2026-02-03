//! AMQP connection management
//! AMQP连接管理

use crate::{AmqpConfig, ConnectionConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

/// AMQP connection
/// AMQP连接
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Autowired
/// private ConnectionFactory connectionFactory;
///
/// Connection connection = connectionFactory.createConnection();
/// ```
#[derive(Clone)]
pub struct AmqpConnection {
    /// Connection configuration
    /// 连接配置
    config: AmqpConfig,

    /// Connection state
    /// 连接状态
    state: Arc<RwLock<ConnectionState>>,
}

/// Connection state
/// 连接状态
#[derive(Clone, Debug)]
pub enum ConnectionState {
    /// Not connected
    /// 未连接
    Disconnected,

    /// Connecting
    /// 连接中
    Connecting,

    /// Connected
    /// 已连接
    Connected {
        /// Connection ID
        /// 连接ID
        id: String,

        /// Connected at timestamp
        /// 连接时间戳
        connected_at: u64,
    },

    /// Connection failed
    /// 连接失败
    Failed {
        /// Error message
        /// 错误消息
        error: String,
    },
}

impl AmqpConnection {
    /// Create new AMQP connection
    /// 创建新的AMQP连接
    pub fn new(config: AmqpConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
        }
    }

    /// Get connection state
    /// 获取连接状态
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Check if connected
    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        matches!(*self.state.read().await, ConnectionState::Connected { .. })
    }

    /// Get connection URL
    /// 获取连接URL
    pub fn url(&self) -> String {
        self.config.build_url()
    }
}

/// Connection manager
/// 连接管理器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public CachingConnectionFactory connectionFactory() {
///     CachingConnectionFactory factory = new CachingConnectionFactory();
///     factory.setHost("localhost");
///     factory.setPort(5672);
///     return factory;
/// }
/// ```
#[derive(Clone)]
pub struct ConnectionManager {
    /// Connection configuration
    /// 连接配置
    config: AmqpConfig,

    /// Connection configuration
    /// 连接配置
    conn_config: ConnectionConfig,

    /// Active connections
    /// 活动连接
    connections: Arc<RwLock<Vec<AmqpConnection>>>,
}

impl ConnectionManager {
    /// Create new connection manager
    /// 创建新的连接管理器
    pub fn new(config: AmqpConfig) -> Self {
        Self {
            config,
            conn_config: ConnectionConfig::default(),
            connections: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with connection configuration
    /// 使用连接配置创建
    pub fn with_connection_config(mut self, conn_config: ConnectionConfig) -> Self {
        self.conn_config = conn_config;
        self
    }

    /// Get configuration
    /// 获取配置
    pub fn config(&self) -> &AmqpConfig {
        &self.config
    }

    /// Get connection configuration
    /// 获取连接配置
    pub fn connection_config(&self) -> &ConnectionConfig {
        &self.conn_config
    }

    /// Create a new connection
    /// 创建新连接
    pub async fn create_connection(&self) -> Result<AmqpConnection, String> {
        let conn = AmqpConnection::new(self.config.clone());
        {
            let mut connections = self.connections.write().await;
            connections.push(conn.clone());
        }
        Ok(conn)
    }

    /// Close all connections
    /// 关闭所有连接
    pub async fn close_all(&self) {
        let mut connections = self.connections.write().await;
        connections.clear();
    }

    /// Get active connection count
    /// 获取活动连接数
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

impl From<AmqpConfig> for ConnectionManager {
    fn from(config: AmqpConfig) -> Self {
        Self::new(config)
    }
}
