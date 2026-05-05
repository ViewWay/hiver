//! AMQP message listener
//! AMQP消息监听器

use crate::{AmqpConnection, AmqpMessage};
use std::sync::Arc;
use tokio::sync::RwLock;

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
pub struct ListenerContainer {
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

impl ListenerContainer {
    /// Create new listener container
    /// 创建新的监听器容器
    pub fn new(connection: Arc<AmqpConnection>, queue: impl Into<String>) -> Self {
        Self {
            connection,
            queue: queue.into(),
            consumer_tag: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start consuming
    /// 开始消费
    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("Starting listener for queue: {}", self.queue);
        Ok(())
    }

    /// Stop consuming
    /// 停止消费
    pub async fn stop(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Stopping listener for queue: {}", self.queue);
        Ok(())
    }

    /// Check if running
    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get queue name
    /// 获取队列名称
    pub fn queue(&self) -> &str {
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
pub trait MessageHandler: Send + Sync {
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
    fn handle(&self, message: AmqpMessage) -> Result<(), String> {
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
pub struct Listener {
    /// Connection
    /// 连接
    connection: Arc<AmqpConnection>,

    /// Listener containers
    /// 监听器容器
    containers: Arc<RwLock<Vec<ListenerContainer>>>,
}

impl Listener {
    /// Create new listener
    /// 创建新的监听器
    pub fn new(connection: Arc<AmqpConnection>) -> Self {
        Self {
            connection,
            containers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add listener container
    /// 添加监听器容器
    pub async fn add_container(&self, container: ListenerContainer) {
        let mut containers = self.containers.write().await;
        containers.push(container);
    }

    /// Listen on a queue
    /// 监听队列
    pub async fn listen<H>(
        &self,
        queue: impl Into<String>,
        _handler: H,
    ) -> Result<(), String>
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
    pub async fn listen_fn<F>(
        &self,
        queue: impl Into<String>,
        f: F,
    ) -> Result<(), String>
    where
        F: Fn(AmqpMessage) -> Result<(), String> + Send + Sync + 'static,
    {
        let handler = FnHandler { handler: f };
        self.listen(queue, handler).await
    }

    /// Stop all listeners
    /// 停止所有监听器
    pub async fn stop_all(&self) -> Result<(), String> {
        let containers = self.containers.read().await;
        for container in containers.iter() {
            container.stop().await?;
        }
        Ok(())
    }

    /// Get listener count
    /// 获取监听器数量
    pub async fn listener_count(&self) -> usize {
        self.containers.read().await.len()
    }
}
