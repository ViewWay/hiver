# Testing / 测试

> **Status**: Phase 3+ Available ✅  
> **状态**: 第3阶段+可用 ✅

Nexus provides comprehensive testing support for your applications.

Nexus 为您的应用程序提供全面的测试支持。

---

## Overview / 概述

Testing strategies:

测试策略：

- **Unit Tests** / **单元测试** - Test individual components
- **Integration Tests** / **集成测试** - Test component interactions
- **E2E Tests** / **端到端测试** - Test full application flow

---

## Unit Testing / 单元测试

### Testing Handlers / 测试处理器

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use nexus_test::TestClient;

    #[test]
    fn test_handler() -> std::io::Result<()> {
        let runtime = nexus_runtime::Runtime::new()?;
        runtime.block_on(async {
            let response = handler(Request::default()).await;
            assert_eq!(response.status(), StatusCode::OK);
            Ok(())
        })
    }
}
```

### Testing Extractors / 测试提取器

```rust
#[test]
fn test_path_extractor() -> std::io::Result<()> {
    let runtime = nexus_runtime::Runtime::new()?;
    runtime.block_on(async {
        let req = Request::builder()
            .uri("/users/123")
            .build();

        let id: Path<u64> = Path::from_request(&req).await.unwrap();
        assert_eq!(id.0, 123);
        Ok(())
    })
}
```

---

## Integration Testing / 集成测试

### Test Client / 测试客户端

```rust
use nexus_test::TestClient;

#[test]
fn test_api() -> std::io::Result<()> {
    let runtime = nexus_runtime::Runtime::new()?;
    runtime.block_on(async {
        let app = create_app();
        let client = TestClient::new(app);

        // Test GET / 测试 GET
        let response = client.get("/api/users").send().await;
        assert_eq!(response.status(), 200);

        // Test POST / 测试 POST
        let response = client.post("/api/users")
            .json(&user_data)
            .send()
            .await;
        assert_eq!(response.status(), 201);

        Ok(())
    })
}
```

---

## E2E Testing / 端到端测试

```rust
#[test]
fn test_full_flow() -> std::io::Result<()> {
    let runtime = nexus_runtime::Runtime::new()?;
    runtime.block_on(async {
        // Start test server / 启动测试服务器
        let server = start_test_server().await;

        // Test complete flow / 测试完整流程
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:8080/api/users")
            .json(&create_user_request)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 201);

        Ok(())
    })
}
```

---

## Best Practices / 最佳实践

1. **Test in isolation** / **隔离测试**
2. **Use test fixtures** / **使用测试夹具**
3. **Mock external dependencies** / **模拟外部依赖**
4. **Test error cases** / **测试错误情况**

---

*← [Previous / 上一页](./web3.md) | [Next / 下一页](../reference/api.md) →*
