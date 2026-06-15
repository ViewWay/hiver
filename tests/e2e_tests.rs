//! End-to-end integration tests for critical Hiver flows.
//! 关键 Hiver 流程的端到端集成测试。

// ============================================================
// 1. MCP Client <-> Server roundtrip
// ============================================================

mod mcp_e2e
{
    use std::collections::HashMap;

    use hiver_mcp::{
        client::McpClient, server::McpServer, transport::MemoryTransport, types::CallToolResult,
    };

    async fn setup() -> (McpClient, tokio::task::JoinHandle<()>)
    {
        let server = McpServer::builder("e2e-server", "1.0.0")
            .instructions("E2E test server")
            .build();

        server
            .tools()
            .register(hiver_mcp::registry::FunctionTool::new(
                "add",
                "Add two numbers",
                |args| async move {
                    let a = args["a"].as_i64().unwrap_or(0);
                    let b = args["b"].as_i64().unwrap_or(0);
                    Ok(CallToolResult::text(format!("{}", a + b)))
                },
            ))
            .await;

        server
            .resources()
            .register(hiver_mcp::registry::StaticTextResource::new(
                "test:///info",
                "Info",
                "Hiver E2E Test",
            ))
            .await;

        server
            .prompts()
            .register(
                hiver_mcp::registry::StaticPrompt::new("greet", "Hello, {{name}}!")
                    .argument("name", "Name", true),
            )
            .await;

        let (client_transport, server_transport) = MemoryTransport::pair();
        let handle = tokio::spawn(async move {
            server.run(server_transport).await.unwrap();
        });

        let client = McpClient::new();
        client.connect(client_transport).await.unwrap();
        (client, handle)
    }

    #[tokio::test]
    async fn test_mcp_initialize()
    {
        let (client, handle) = setup().await;
        let info = client.server_info().await.unwrap();
        assert_eq!(info.server_info.name, "e2e-server");
        assert_eq!(info.protocol_version, "2025-03-26");
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_mcp_tool_call_roundtrip()
    {
        let (client, handle) = setup().await;
        let result = client
            .call_tool("add", serde_json::json!({"a": 3, "b": 7}))
            .await
            .unwrap();
        assert!(!result.is_error);
        match &result.content[0]
        {
            hiver_mcp::types::Content::Text { text } => assert_eq!(text, "10"),
            _ => panic!("Expected text"),
        }
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_mcp_resource_read()
    {
        let (client, handle) = setup().await;
        let result = client.read_resource("test:///info").await.unwrap();
        assert_eq!(result.contents[0].text.as_deref(), Some("Hiver E2E Test"));
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_mcp_prompt_get()
    {
        let (client, handle) = setup().await;
        let result = client
            .get_prompt("greet", HashMap::from([("name".into(), "World".into())]))
            .await
            .unwrap();
        match &result.messages[0].content
        {
            hiver_mcp::types::Content::Text { text } => assert_eq!(text, "Hello, World!"),
            _ => panic!("Expected text"),
        }
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_mcp_ping()
    {
        let (client, handle) = setup().await;
        client.ping().await.unwrap();
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_mcp_unknown_tool_error()
    {
        let (client, handle) = setup().await;
        let result = client.call_tool("nope", serde_json::json!({})).await;
        assert!(result.is_err());
        client.disconnect().await.unwrap();
        let _ = handle.await;
    }
}

// ============================================================
// 2. Cloud Bus publish/subscribe (LocalBus)
// ============================================================

mod cloud_bus_e2e
{
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use hiver_cloud_bus::{BusEvent, BusEventType, CloudBus, LocalBus};

    #[tokio::test]
    async fn test_local_bus_pub_sub()
    {
        let bus = LocalBus::new();
        let count = Arc::new(AtomicUsize::new(0));
        let c1 = count.clone();

        bus.subscribe(Box::new(move |evt| {
            assert!(matches!(evt.event_type, BusEventType::ConfigRefresh));
            c1.fetch_add(1, Ordering::Relaxed);
        }))
        .await
        .unwrap();

        let event = BusEvent::config_refresh("service-a");
        bus.publish(event).await.unwrap();

        assert_eq!(count.load(Ordering::Relaxed), 1);
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_local_bus_multiple_subscribers()
    {
        let bus = LocalBus::new();
        let count = Arc::new(AtomicUsize::new(0));

        for _ in 0..3
        {
            let c = count.clone();
            bus.subscribe(Box::new(move |_| {
                c.fetch_add(1, Ordering::Relaxed);
            }))
            .await
            .unwrap();
        }

        bus.publish(BusEvent::new(BusEventType::Custom("deploy".to_string()), "svc"))
            .await
            .unwrap();

        assert_eq!(count.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_bus_event_serialization_roundtrip()
    {
        let event = BusEvent::config_refresh("svc")
            .with_header("region", "us-east-1")
            .with_payload(serde_json::json!({"key": "val"}));

        let bytes = event.to_bytes().unwrap();
        let restored = BusEvent::from_bytes(&bytes).unwrap();

        assert_eq!(restored.source, "svc");
        assert!(restored.headers.contains_key("region"));
        assert!(!restored.payload.is_null());
    }

    #[test]
    fn test_local_bus_name()
    {
        let bus = LocalBus::new();
        assert_eq!(bus.name(), "local");
    }
}

// ============================================================
// 3. Cloud Stream InMemoryBinder produce/consume
// ============================================================

mod cloud_stream_e2e
{
    use hiver_cloud_stream::{InMemoryBinder, StreamBinder, StreamMessage};

    #[tokio::test]
    async fn test_produce_consume_roundtrip()
    {
        let binder = InMemoryBinder::new();
        let producer = binder.create_producer("orders").await.unwrap();
        let consumer = binder
            .create_consumer("orders", "order-service")
            .await
            .unwrap();

        let msg = StreamMessage::new(b"order-123".to_vec())
            .with_key("order-123")
            .with_header("content-type", "application/json");

        producer.send(msg).await.unwrap();

        let received = consumer.receive().await.unwrap().unwrap();
        assert_eq!(received.as_str(), Some("order-123"));
        assert_eq!(received.key.as_deref(), Some("order-123"));
    }

    #[tokio::test]
    async fn test_fifo_ordering()
    {
        let binder = InMemoryBinder::new();
        let producer = binder.create_producer("fifo-test").await.unwrap();
        let consumer = binder.create_consumer("fifo-test", "g1").await.unwrap();

        for i in 0..10
        {
            producer
                .send(StreamMessage::new(format!("msg-{i}").into_bytes()))
                .await
                .unwrap();
        }

        for i in 0..10
        {
            let msg = consumer.receive().await.unwrap().unwrap();
            assert_eq!(msg.as_str(), Some(&*format!("msg-{i}")));
        }

        assert!(consumer.receive().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_json_message()
    {
        let binder = InMemoryBinder::new();
        let producer = binder.create_producer("json-topic").await.unwrap();
        let consumer = binder.create_consumer("json-topic", "g").await.unwrap();

        let data = serde_json::json!({"user": "alice", "score": 42});
        let msg = StreamMessage::from_json(&data).unwrap();
        producer.send(msg).await.unwrap();

        let received = consumer.receive().await.unwrap().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&received.payload).unwrap();
        assert_eq!(parsed["user"], "alice");
        assert_eq!(parsed["score"], 42);
    }

    #[test]
    fn test_binder_name()
    {
        let binder = InMemoryBinder::new();
        assert_eq!(binder.name(), "in-memory");
    }
}

// ============================================================
// 4. Modulith dependency graph
// ============================================================

mod modulith_e2e
{
    use hiver_modulith::{DependencyGraph, ModuleDependency};

    #[test]
    fn test_diamond_dependency_no_cycle()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("A", "B"));
        graph.add_dependency(ModuleDependency::new("A", "C"));
        graph.add_dependency(ModuleDependency::new("B", "D"));
        graph.add_dependency(ModuleDependency::new("C", "D"));

        assert!(graph.is_acyclic());
        let order = graph.topological_sort().unwrap();
        assert_eq!(order.len(), 4);
        let pos = |name: &str| order.iter().position(|m| m == name).unwrap();
        assert!(pos("D") < pos("B"));
        assert!(pos("D") < pos("C"));
        assert!(pos("B") < pos("A"));
        assert!(pos("C") < pos("A"));
    }

    #[test]
    fn test_cyclic_dependency_detected()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("A", "B"));
        graph.add_dependency(ModuleDependency::new("B", "C"));
        graph.add_dependency(ModuleDependency::new("C", "A"));

        assert!(!graph.is_acyclic());
        assert!(graph.topological_sort().is_none());
        assert!(!graph.detect_cycles().is_empty());
    }

    #[test]
    fn test_linear_chain()
    {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(ModuleDependency::new("A", "B"));
        graph.add_dependency(ModuleDependency::new("B", "C"));
        graph.add_dependency(ModuleDependency::new("C", "D"));

        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec!["D", "C", "B", "A"]);
    }

    #[test]
    fn test_empty_graph()
    {
        let graph = DependencyGraph::new();
        assert!(graph.is_acyclic());
        assert!(graph.topological_sort().unwrap().is_empty());
    }
}

// ============================================================
// 5. SpEL parser depth limit (P0 fix verification)
// ============================================================

mod spel_e2e
{
    use hiver_spel::{SpelContext, SpelEvaluator};

    #[test]
    fn test_spel_has_role()
    {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        let eval = SpelEvaluator::new("hasRole('ADMIN')");
        assert!(eval.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_spel_and_or()
    {
        let mut ctx = SpelContext::new();
        ctx.add_role("ADMIN");
        ctx.add_role("USER");

        let eval = SpelEvaluator::new("hasRole('ADMIN') and hasRole('USER')");
        assert!(eval.evaluate(&ctx).unwrap());

        let eval2 = SpelEvaluator::new("hasRole('ADMIN') or hasRole('GUEST')");
        assert!(eval2.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_spel_not()
    {
        let ctx = SpelContext::new();
        let eval = SpelEvaluator::new("not hasRole('ADMIN')");
        assert!(eval.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_spel_depth_limit_no_overflow()
    {
        // Build deeply nested expression — should not stack overflow.
        let depth = 70;
        let expr = (0..depth).map(|_| "not").collect::<Vec<_>>().join(" ") + " hasRole('X')";
        let _ = SpelEvaluator::new(&expr);
    }

    #[test]
    fn test_spel_permit_all_deny_all()
    {
        let ctx = SpelContext::new();
        assert!(SpelEvaluator::new("permitAll").evaluate(&ctx).unwrap());
        assert!(!SpelEvaluator::new("denyAll").evaluate(&ctx).unwrap());
    }
}

// ============================================================
// 6. DevTools — Profile, BuildInfo
// ============================================================

mod devtools_e2e
{
    use hiver_devtools::{BuildInfo, Profile};

    #[test]
    fn test_profile_detection()
    {
        let profile = Profile::current();
        assert!(matches!(profile, Profile::Dev | Profile::Test));
    }

    #[test]
    fn test_build_info()
    {
        let info = BuildInfo::new();
        assert!(!info.version.is_empty());
        assert!(!info.target.is_empty());
    }
}

// ============================================================
// 7. ResponseEntity (Spring-style response)
// ============================================================

mod response_e2e
{
    use hiver_response::response_entity::ResponseEntity;

    #[test]
    fn test_response_entity_ok()
    {
        let entity: ResponseEntity<String> = ResponseEntity::ok_with("hello".to_string());
        assert_eq!(entity.status_code().as_u16(), 200);
    }

    #[test]
    fn test_response_entity_not_found()
    {
        let entity: ResponseEntity<()> = ResponseEntity::not_found();
        assert_eq!(entity.status_code().as_u16(), 404);
    }

    #[test]
    fn test_response_entity_with_headers()
    {
        let entity: ResponseEntity<&str> = ResponseEntity::ok()
            .header("content-type", "application/json")
            .header("x-request-id", "abc123")
            .body("data");
        assert_eq!(entity.headers().len(), 2);
    }
}
