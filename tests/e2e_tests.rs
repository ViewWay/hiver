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
// ============================================================
// HTTP server end-to-end over a real socket.
// HTTP 服务端端到端(真实 socket)。
//
// Spawns the `http_server_example` binary (Hiver\'s own stack: custom runtime
// waker path → TCP accept → HTTP/1.1 parse → Router dispatch → handler →
// response encode) and asserts a real std::net client receives a correct HTTP
// response. This gates "can this framework actually serve a request?".
// 启动 `http_server_example` 二进制(Hiver 自有技术栈:自定义 runtime waker 路径
// → TCP accept → HTTP/1.1 解析 → Router 分发 → 处理程序 → 响应编码),并断言
// 真实 std::net 客户端收到正确的 HTTP 响应。把守"本框架能否真正服务请求?"。
// ============================================================

mod http_server_e2e
{
    use std::{
        io::{Read, Write},
        net::TcpStream,
        path::PathBuf,
        process::{Child, Command, Stdio},
        thread,
        time::Duration,
    };

    // Distinct port per example binary so the two e2e modules (this and
    // `annotated_app_e2e`) can run concurrently under `cargo test`'s default
    // multi-threaded runner without racing on the same listener.
    // 为每个示例二进制分配不同端口,使两个 e2e 模块(本模块与
    // `annotated_app_e2e`)可在 `cargo test` 默认多线程运行器下并发运行,
    // 而不在同一监听器上竞争。
    const PORT: u16 = 8091;

    /// Build the example binary on first use and return its path.
    /// 首次使用时构建示例二进制并返回其路径。
    fn example_bin() -> PathBuf
    {
        // The test harness runs with CARGO_MANIFEST_DIR = tests/. The binary
        // is built by `cargo test` into the workspace target dir.
        // 测试工具以 CARGO_MANIFEST_DIR = tests/ 运行。二进制由 `cargo test`
        // 构建到 workspace target 目录。
        let target = env!("CARGO_MANIFEST_DIR")
            .parse::<PathBuf>()
            .unwrap()
            .join("..")
            .join("target")
            .join("debug")
            .join("http_server_example");
        if !target.exists()
        {
            panic!(
                "http_server_example binary not found at {}.                  Build it first: \
                 cargo build -p hiver-examples --bin http_server_example",
                target.display()
            );
        }
        target
    }

    /// Spawn the example server and wait until it is accepting connections.
    /// 启动示例服务端并等待其接受连接。
    fn spawn_server() -> Child
    {
        let bin = example_bin();
        let addr = format!("127.0.0.1:{PORT}");
        let child = Command::new(&bin)
            // Pin this server to our dedicated port so it cannot collide with
            // `annotated_app_example` running in parallel.
            // 将本服务端固定到专用端口,使其不会与并发运行的
            // `annotated_app_example` 冲突。
            .env("HIVER_SERVER_PORT", PORT.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn {}: {e}", bin.display()));

        // Wait until the port is accepting (retry for up to ~5s). Treat
        // connection refusals and transient WouldBlock as "not ready yet".
        // 等待端口接受连接(最多重试约 5s)。将连接拒绝与瞬时 WouldBlock 视为
        // "尚未就绪"。
        for _ in 0..100
        {
            match TcpStream::connect_timeout(
                &addr.parse().unwrap(),
                Duration::from_millis(200),
            )
            {
                Ok(_) => return child,
                Err(_) => thread::sleep(Duration::from_millis(50)),
            }
        }
        panic!("server did not start listening on {addr}");
    }

    fn http_get(path: &str) -> String
    {
        // The server runs on a custom busy-poll runtime; connections may need a
        // few retries (connect can transiently return WouldBlock on macOS). Use
        // a bounded read (not read_to_end) so a misbehaving server can't hang us.
        // 服务端运行于自定义忙轮询 runtime;连接可能需要重试若干次
        // (在 macOS 上 connect 可能瞬时返回 WouldBlock)。使用有界读取
        // (非 read_to_end),使异常服务端无法挂起我们。
        let addr: std::net::SocketAddr = format!("127.0.0.1:{PORT}").parse().unwrap();
        let mut last = String::new();
        for _ in 0..40
        {
            let result = (|| -> std::io::Result<String> {
                let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(200))?;
                stream.set_read_timeout(Some(Duration::from_millis(800)))?;
                stream.set_write_timeout(Some(Duration::from_millis(800)))?;
                let req =
                    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
                stream.write_all(req.as_bytes())?;
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf)?;
                Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
            })();
            match result
            {
                Ok(resp) if resp.starts_with("HTTP") => return resp,
                Ok(other) =>
                {
                    last = other;
                    thread::sleep(Duration::from_millis(50));
                },
                Err(_) => thread::sleep(Duration::from_millis(50)),
            }
        }
        panic!("http_get({path}) failed after retries; last response: {last:?}");
    }

    #[test]
    fn server_serves_http_over_real_socket()
    {
        // One server, multiple requests — avoids two tests racing on port 8080.
        // 一个服务端,多个请求 —— 避免两个测试在 8080 端口上竞争。
        let mut child = spawn_server();

        // /hello -> 200 with the greeting body.
        // /hello -> 200 与问候正文。
        let resp = http_get("/hello");
        assert!(resp.starts_with("HTTP/1.1 200"), "hello status line wrong: {resp}");
        assert!(resp.contains("Hello from Hiver!"), "hello body missing: {resp}");

        // /echo -> 200.
        let resp = http_get("/echo");
        assert!(resp.starts_with("HTTP/1.1 200"), "echo status line wrong: {resp}");

        let _ = child.kill();
        let _ = child.wait();
    }
}

// ============================================================
// Annotated application end-to-end (full `#[hiver_main]` path).
// 注解驱动应用端到端(完整 `#[hiver_main]` 路径)。
//
// Spawns the `annotated_app_example` binary, which boots entirely via
// annotations: `#[hiver_main]` → runtime + auto-config + inventory route
// collection → Router → WebServer → bind + serve. Verifies that a route
// contributed by `#[get]` (not wired by hand) is actually discoverable and
// serves over a real socket.
// 启动 `annotated_app_example` 二进制,完全经注解启动:`#[hiver_main]` → runtime
// + 自动配置 + inventory 路由收集 → Router → WebServer → 绑定 + 服务。验证经
// `#[get]` 贡献(非手动接线)的路由确实可被发现,并经真实 socket 服务。
// ============================================================

mod annotated_app_e2e
{
    use std::{
        io::{Read, Write},
        net::TcpStream,
        path::PathBuf,
        process::{Child, Command, Stdio},
        thread,
        time::Duration,
    };

    // Distinct port so this module never contends with `http_server_e2e` when
    // both run concurrently. The `#[hiver_main]` runner maps `HIVER_SERVER_PORT`
    // → `server.port`, so the env var below overrides the bind address.
    // 独立端口,使本模块在与 `http_server_e2e` 并发运行时不与之争用。
    // `#[hiver_main]` 运行器将 `HIVER_SERVER_PORT` → `server.port`,故下面的
    // 环境变量会覆盖绑定地址。
    const PORT: u16 = 8092;

    /// Path to the annotated example binary (built by the test harness).
    /// 注解示例二进制路径(由测试工具构建)。
    fn example_bin() -> PathBuf
    {
        let target = env!("CARGO_MANIFEST_DIR")
            .parse::<PathBuf>()
            .unwrap()
            .join("..")
            .join("target")
            .join("debug")
            .join("annotated_app_example");
        if !target.exists()
        {
            panic!(
                "annotated_app_example binary not found at {}. Build it first: cargo build -p \
                 hiver-examples --bin annotated_app_example",
                target.display()
            );
        }
        target
    }

    /// Spawn the annotated app and wait until it accepts connections.
    /// 启动注解应用并等待其接受连接。
    fn spawn_app() -> Child
    {
        let bin = example_bin();
        let addr = format!("127.0.0.1:{PORT}");
        let child = Command::new(&bin)
            .env("HIVER_SERVER_PORT", PORT.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn {}: {e}", bin.display()));

        let addr: std::net::SocketAddr = addr.parse().unwrap();
        for _ in 0..100
        {
            match TcpStream::connect_timeout(&addr, Duration::from_millis(200))
            {
                Ok(_) => return child,
                Err(_) => thread::sleep(Duration::from_millis(50)),
            }
        }
        panic!("annotated app did not start listening on {addr}");
    }

    fn http_get(path: &str) -> String
    {
        let addr: std::net::SocketAddr = format!("127.0.0.1:{PORT}").parse().unwrap();
        let mut last = String::new();
        for _ in 0..40
        {
            let result = (|| -> std::io::Result<String> {
                let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(200))?;
                stream.set_read_timeout(Some(Duration::from_millis(800)))?;
                stream.set_write_timeout(Some(Duration::from_millis(800)))?;
                let req =
                    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
                stream.write_all(req.as_bytes())?;
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf)?;
                Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
            })();
            match result
            {
                Ok(resp) if resp.starts_with("HTTP") => return resp,
                Ok(other) =>
                {
                    last = other;
                    thread::sleep(Duration::from_millis(50));
                },
                Err(_) => thread::sleep(Duration::from_millis(50)),
            }
        }
        panic!("annotated http_get({path}) failed; last: {last:?}");
    }

    #[test]
    fn annotated_app_serves_inventory_collected_routes()
    {
        let mut child = spawn_app();

        // /hello is contributed by `#[get("/hello")]` and discovered via
        // inventory at startup (the `RouterAutoConfiguration` logs
        // "Registered N routes from inventory"). It is NOT wired by hand.
        // /hello 由 `#[get("/hello")]` 贡献,启动时经 inventory 发现
        // (`RouterAutoConfiguration` 打印 "Registered N routes from inventory")。
        // 它不是手动接线的。
        let resp = http_get("/hello");
        let _ = child.kill();
        let _ = child.wait();

        assert!(resp.starts_with("HTTP/1.1 200"), "hello status line wrong: {resp}");
        assert!(resp.contains("Hello from annotated Hiver!"), "hello body missing: {resp}");
    }
}
