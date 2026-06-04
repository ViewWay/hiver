//! AMQP message listener
//! AMQP消息监听器

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{AmqpConnection, AmqpMessage};

/// Message listener container
/// 消息监听器容器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @RabbitListener(queues = "my_queue")
/// public void handleMessage(Message message) {
///     // Process message
/// }
/// ```
pub struct ListenerContainer
{
    /// Connection
    /// 连接
    connection: Arc<AmqpConnection>,

    /// Queue name
    /// 队列名称
    queue: String,

    /// Consumer tag
    /// 消费者标签
    consumer_tag: Option<String>,

    /// Running state
    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl ListenerContainer
{
    /// Create new listener container
    /// 创建新的监听器容器
    pub fn new(connection: Arc<AmqpConnection>, queue: impl Into<String>) -> Self
    {
        Self {
            connection,
            queue: queue.into(),
            consumer_tag: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start consuming
    /// 开始消费
    pub async fn start(&self) -> Result<(), String>
    {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("Starting listener for queue: {}", self.queue);
        Ok(())
    }

    /// Stop consuming
    /// 停止消费
    pub async fn stop(&self) -> Result<(), String>
    {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Stopping listener for queue: {}", self.queue);
        Ok(())
    }

    /// Check if running
    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool
    {
        *self.running.read().await
    }

    /// Get queue name
    /// 获取队列名称
    pub fn queue(&self) -> &str
    {
        &self.queue
    }
}

/// Message handler trait
/// 消息处理器trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @RabbitListener(queues = "my_queue")
/// @RabbitHandler
/// public void handleMessage(String message) {
///     // Process message
/// }
/// ```
pub trait MessageHandler: Send + Sync
{
    /// Handle message
    /// 处理消息
    fn handle(&self, message: AmqpMessage) -> Result<(), String>;
}

/// Function-based message handler
/// 基于函数的消息处理器
pub(crate) struct FnHandler<F>
where
    F: Fn(AmqpMessage) -> Result<(), String> + Send + Sync,
{
    handler: F,
}

impl<F> MessageHandler for FnHandler<F>
where
    F: Fn(AmqpMessage) -> Result<(), String> + Send + Sync,
{
    fn handle(&self, message: AmqpMessage) -> Result<(), String>
    {
        (self.handler)(message)
    }
}

/// Message listener
/// 消息监听器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Component
/// public class MyListener {
///
///     @RabbitListener(queues = "my_queue")
///     public void handleMessage(Message message) {
///         // Process message
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Listener
{
    /// Connection
    /// 连接
    connection: Arc<AmqpConnection>,

    /// Listener containers
    /// 监听器容器
    containers: Arc<RwLock<Vec<ListenerContainer>>>,
}

impl Listener
{
    /// Create new listener
    /// 创建新的监听器
    pub fn new(connection: Arc<AmqpConnection>) -> Self
    {
        Self {
            connection,
            containers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add listener container
    /// 添加监听器容器
    pub async fn add_container(&self, container: ListenerContainer)
    {
        let mut containers = self.containers.write().await;
        containers.push(container);
    }

    /// Listen on a queue
    /// 监听队列
    pub async fn listen<H>(&self, queue: impl Into<String>, _handler: H) -> Result<(), String>
    where
        H: MessageHandler + 'static,
    {
        let queue = queue.into();
        let container = ListenerContainer::new(self.connection.clone(), queue);
        container.start().await?;

        // In a real implementation, this would start a task that consumes messages
        // and calls the handler
        // 在实际实现中，这将启动一个任务来消费消息并调用处理器
        self.add_container(container).await;
        Ok(())
    }

    /// Listen with a function
    /// 使用函数监听
    pub async fn listen_fn<F>(&self, queue: impl Into<String>, f: F) -> Result<(), String>
    where
        F: Fn(AmqpMessage) -> Result<(), String> + Send + Sync + 'static,
    {
        let handler = FnHandler { handler: f };
        self.listen(queue, handler).await
    }

    /// Stop all listeners
    /// 停止所有监听器
    pub async fn stop_all(&self) -> Result<(), String>
    {
        let containers = self.containers.read().await;
        for container in containers.iter()
        {
            container.stop().await?;
        }
        Ok(())
    }

    /// Get listener count
    /// 获取监听器数量
    pub async fn listener_count(&self) -> usize
    {
        self.containers.read().await.len()
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use super::*;
    use crate::{AmqpConfig, AmqpConnection, AmqpMessage, Message};

    /// Helper to create a Listener for testing / 创建测试用 Listener 的辅助函数
    fn create_listener() -> Listener
    {
        let config = AmqpConfig::default();
        let conn = AmqpConnection::new(config);
        Listener::new(Arc::new(conn))
    }

    /// Helper to create a ListenerContainer for testing / 创建测试用 ListenerContainer 的辅助函数
    fn create_container(queue: &str) -> ListenerContainer
    {
        let config = AmqpConfig::default();
        let conn = AmqpConnection::new(config);
        ListenerContainer::new(Arc::new(conn), queue)
    }

    /// Test ListenerContainer starts and stops / 测试 ListenerContainer 启动和停止
    #[tokio::test]
    async fn test_container_start_stop()
    {
        let container = create_container("test_queue");
        assert!(!container.is_running().await);
        assert_eq!(container.queue(), "test_queue");

        container.start().await.unwrap();
        assert!(container.is_running().await);

        container.stop().await.unwrap();
        assert!(!container.is_running().await);
    }

    /// Test ListenerContainer queue accessor / 测试 ListenerContainer queue 访问器
    #[tokio::test]
    async fn test_container_queue_accessor()
    {
        let container = create_container("orders_queue");
        assert_eq!(container.queue(), "orders_queue");
    }

    /// Test Listener::listen_fn registers a container / 测试 Listener::listen_fn 注册监听器
    #[tokio::test]
    async fn test_listener_listen_fn()
    {
        let listener = create_listener();
        assert_eq!(listener.listener_count().await, 0);

        listener.listen_fn("my_queue", |_msg| Ok(())).await.unwrap();
        assert_eq!(listener.listener_count().await, 1);
    }

    /// Test Listener::stop_all stops all containers / 测试 Listener::stop_all 停止所有容器
    #[tokio::test]
    async fn test_listener_stop_all()
    {
        let listener = create_listener();
        listener.listen_fn("queue_a", |_msg| Ok(())).await.unwrap();
        listener.listen_fn("queue_b", |_msg| Ok(())).await.unwrap();
        assert_eq!(listener.listener_count().await, 2);

        listener.stop_all().await.unwrap();
    }

    /// Test FnHandler delegates to closure / 测试 FnHandler 委托给闭包
    #[test]
    fn test_fn_handler_delegates()
    {
        let handler = FnHandler {
            handler: |msg: AmqpMessage| {
                let body = msg.payload_as_string();
                if body.contains("error")
                {
                    Err("simulated error".to_string())
                }
                else
                {
                    Ok(())
                }
            },
        };

        let ok_msg = AmqpMessage::new(Message::from_string("good message"));
        assert!(handler.handle(ok_msg).is_ok());

        let err_msg = AmqpMessage::new(Message::from_string("has error"));
        assert!(handler.handle(err_msg).is_err());
    }
}
