//! Integration tests for hiver-amqp
//! 测试模块

#[cfg(test)]
mod tests
{
    use crate::*;

    // ── Constants tests / 常量测试 ───────────────────────────────

    /// Test module-level constants / 测试模块级常量
    #[test]
    fn test_module_constants()
    {
        assert_eq!(DEFAULT_AMQP_PORT, 5672);
        assert_eq!(DEFAULT_AMQP_SSL_PORT, 5671);
        assert_eq!(DEFAULT_VHOST, "/");
        assert_eq!(DEFAULT_EXCHANGE_TYPE, ExchangeType::Direct);
        assert!(DEFAULT_QUEUE_DURABLE);
        assert_eq!(DEFAULT_DELIVERY_MODE, DeliveryMode::Persistent);
    }

    /// Test VERSION is set / 测试 VERSION 已设置
    #[test]
    fn test_version_is_set()
    {
        assert!(!VERSION.is_empty());
    }

    // ── Full publish-consume flow (mock) / 完整发布-消费流程（模拟） ──

    /// Test full message lifecycle: config -> connection -> publisher -> publish
    /// 测试完整消息生命周期：配置 -> 连接 -> 发布者 -> 发布
    #[tokio::test]
    async fn test_full_publish_flow()
    {
        let config = AmqpConfig::new()
            .with_host("localhost", 5672)
            .with_credentials("guest", "guest")
            .with_vhost("/");

        let conn = AmqpConnection::new(config);
        let publisher = Publisher::new(std::sync::Arc::new(conn));

        let result = publisher.publish("orders", "order.created", b"{\"id\":1}");
        assert!(result.is_ok());
    }

    /// Test full listen flow: config -> connection -> listener -> listen_fn
    /// 测试完整监听流程：配置 -> 连接 -> 监听器 -> listen_fn
    #[tokio::test]
    async fn test_full_listen_flow()
    {
        let config = AmqpConfig::default();
        let conn = AmqpConnection::new(config);
        let listener = Listener::new(std::sync::Arc::new(conn));

        listener
            .listen_fn("events", |msg| {
                let _body = msg.payload_as_string();
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(listener.listener_count().await, 1);

        listener.stop_all().await.unwrap();
    }

    // ── ConnectionManager integration / ConnectionManager 集成测试 ──

    /// Test ConnectionManager lifecycle with multiple connections
    /// 测试 ConnectionManager 多连接生命周期
    #[tokio::test]
    async fn test_connection_manager_lifecycle()
    {
        let config = AmqpConfig::new().with_host("broker.test", 5672);
        let manager = ConnectionManager::new(config);

        let c1 = manager.create_connection().await.unwrap();
        let c2 = manager.create_connection().await.unwrap();
        assert_eq!(manager.connection_count().await, 2);

        // Connections start disconnected / 连接初始状态为未连接
        assert!(!c1.is_connected().await);
        assert!(!c2.is_connected().await);

        manager.close_all().await;
        assert_eq!(manager.connection_count().await, 0);
    }

    // ── Exchange + Queue + Binding integration / Exchange + Queue + Binding 集成 ──

    /// Test declaring exchange, queue, and binding them together
    /// 测试声明交换机、队列并绑定
    #[test]
    fn test_exchange_queue_binding_integration()
    {
        let exchange = Exchange::topic("events")
            .with_durable(true)
            .with_alternate_exchange("events.dlq");

        let queue = Queue::durable("event_processor")
            .with_message_ttl(60000)
            .with_dead_letter_exchange("events.dlq");

        let binding = BindingBuilder::bind_queue(queue.clone())
            .to(exchange.clone())
            .with("order.#")
            .build();

        assert_eq!(binding.source_name(), "events");
        assert_eq!(binding.destination_name(), "event_processor");
        assert_eq!(binding.routing_key, "order.#");
        assert_eq!(exchange.exchange_type, ExchangeType::Topic);
        assert!(queue.arguments.contains_key("x-message-ttl"));
    }

    // ── Message converter integration / 消息转换器集成 ──

    /// Test JsonMessageConverter with Message and AmqpMessage
    /// 测试 JsonMessageConverter 与 Message 和 AmqpMessage 集成
    #[test]
    fn test_json_converter_integration()
    {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct OrderEvent
        {
            order_id: u64,
            status: String,
        }

        let converter = JsonMessageConverter::new();
        let event = OrderEvent {
            order_id: 1001,
            status: "created".to_string(),
        };

        let msg = converter.to_message(&event).unwrap();
        assert_eq!(msg.properties.content_type.as_deref(), Some("application/json"));

        // Wrap in AmqpMessage / 包装为 AmqpMessage
        let amqp_msg = AmqpMessage {
            message: msg,
            exchange: "orders".to_string(),
            routing_key: "order.created".to_string(),
            delivery_tag: 1,
            redelivered: false,
        };

        assert_eq!(amqp_msg.exchange, "orders");
        assert!(amqp_msg.ack().is_ok());

        // Deserialize back / 反序列化回去
        let back: OrderEvent = converter.convert_from_message(&amqp_msg.message).unwrap();
        assert_eq!(back, event);
    }

    // ── Publisher with JSON round-trip / Publisher JSON 往返 ──

    /// Test publisher converts and sends JSON payload
    /// 测试发布者转换并发送 JSON 负载
    #[test]
    fn test_publisher_json_roundtrip()
    {
        let config = AmqpConfig::default();
        let conn = AmqpConnection::new(config);
        let publisher = Publisher::new(std::sync::Arc::new(conn));

        #[derive(serde::Serialize)]
        struct Notification
        {
            user_id: u64,
            message: String,
        }

        let notif = Notification {
            user_id: 42,
            message: "Welcome!".to_string(),
        };

        let result = publisher.convert_and_send("notifications", "notify.user", &notif);
        assert!(result.is_ok());
    }

    // ── Smoke tests (original, kept) / 冒烟测试（保留原有） ──────

    #[test]
    fn smoke_test()
    {
        assert!(true, "hiver-amqp test infrastructure is working");
    }

    #[test]
    fn test_basic_math()
    {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_vec_operations()
    {
        let v: Vec<i32> = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
        assert_eq!(v.iter().sum::<i32>(), 6);
    }
}
