# Testing / 测试

> **Status**: Phase 3+ Available ✅
> **状态**: 第3阶段+可用 ✅

Hiver provides comprehensive testing support including unit tests, integration tests, and data layer testing.
Hiver 提供全面的测试支持，包括单元测试、集成测试和数据层测试。

---

## Overview / 概述

Testing strategies:
测试策略：

- **Unit Tests** / **单元测试** — Test individual handlers and extractors
- **Integration Tests** / **集成测试** — Test with `TestClient`
- **E2E Tests** / **端到端测试** — Test full application flow
- **Data Layer Tests** / **数据层测试** — Test repositories and ORM models

---

## Unit Testing / 单元测试

### Testing Handlers / 测试处理器

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler() -> std::io::Result<()> {
        let rt = hiver_runtime::Runtime::new()?;
        rt.block_on(async {
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
    let rt = hiver_runtime::Runtime::new()?;
    rt.block_on(async {
        let req = Request::builder()
            .uri("/users/123")
            .build();

        let id: Path<u64> = Path::from_request(&req).await.unwrap();
        assert_eq!(id.0, 123);
        Ok(())
    })
}
```

### Testing JSON Serialization / 测试 JSON 序列化

```rust
#[test]
fn test_json_response() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let json = serde_json::to_string(&user).unwrap();
    assert!(json.contains("Alice"));

    let deserialized: User = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, 1);
}
```

---

## Integration Testing / 集成测试

### TestClient / 测试客户端

```rust
use hiver_test::TestClient;

#[test]
fn test_api() -> std::io::Result<()> {
    let rt = hiver_runtime::Runtime::new()?;
    rt.block_on(async {
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

        // Test with headers / 测试带请求头
        let response = client.get("/api/profile")
            .header("Authorization", "Bearer token123")
            .send()
            .await;
        assert_eq!(response.status(), 200);

        Ok(())
    })
}
```

### Testing Middleware / 测试中间件

```rust
#[test]
fn test_cors_middleware() -> std::io::Result<()> {
    let rt = hiver_runtime::Runtime::new()?;
    rt.block_on(async {
        let cors = Cors::new()
            .allow_origin("*")
            .allow_methods(["GET", "POST"]);

        let app = Router::new()
            .middleware(Arc::new(cors))
            .get("/", || async { "ok" });

        let client = TestClient::new(app);
        let response = client.get("/").send().await;

        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
        Ok(())
    })
}
```

---

## Data Layer Testing / 数据层测试

### Testing Repository / 测试仓库

```rust
use hiver_data_rdbc::DatabaseClient;
use hiver_data_orm::prelude::*;

#[test]
fn test_repository_crud() -> Result<()> {
    let rt = hiver_runtime::Runtime::new()?;
    rt.block_on(async {
        // Use test database / 使用测试数据库
        let client = DatabaseClient::connect("sqlite::memory:").await?;
        let repo = UserRepository::new(client);

        // Create / 创建
        let user = repo.save(User {
            name: "Alice".to_string(),
            email: "alice@test.com".to_string(),
            ..Default::default()
        }).await?;
        assert!(user.id > 0);

        // Read / 读取
        let found = repo.find_by_id(user.id).await?;
        assert_eq!(found.unwrap().name, "Alice");

        // Update / 更新
        let updated = repo.save(User {
            id: user.id,
            name: "Alice Updated".to_string(),
            email: "alice@test.com".to_string(),
        }).await?;
        assert_eq!(updated.name, "Alice Updated");

        // Delete / 删除
        repo.delete_by_id(user.id).await?;
        assert!(repo.find_by_id(user.id).await?.is_none());

        Ok(())
    })
}
```

### Testing Query Methods / 测试查询方法

```rust
#[test]
fn test_derived_queries() -> Result<()> {
    let rt = hiver_runtime::Runtime::new()?;
    rt.block_on(async {
        let repo = setup_test_repo().await?;

        // Test findBy... / 测试 findBy...
        let users = repo.find_by_email("alice@test.com").await?;
        assert_eq!(users.len(), 1);

        // Test findBy...And... / 测试 findBy...And...
        let users = repo.find_by_name_and_email("Alice", "alice@test.com").await?;
        assert_eq!(users.len(), 1);

        // Test pagination / 测试分页
        let page = repo.find_all(PageRequest::of(0, 10)).await?;
        assert!(page.total_elements() > 0);

        Ok(())
    })
}
```

### Testing Migrations / 测试迁移

```rust
use hiver_flyway::{Flyway, MigrationConfig};

#[test]
fn test_migrations() -> Result<()> {
    let rt = hiver_runtime::Runtime::new()?;
    rt.block_on(async {
        let mut flyway = Flyway::new(MigrationConfig {
            locations: vec!["migrations".to_string()],
            ..Default::default()
        });

        let result = flyway.migrate(&mut conn).await?;
        assert_eq!(result.successful_migrations, 3);
        Ok(())
    })
}
```

---

## E2E Testing / 端到端测试

```rust
#[test]
fn test_full_flow() -> std::io::Result<()> {
    let rt = hiver_runtime::Runtime::new()?;
    rt.block_on(async {
        let server = start_test_server().await;
        let client = reqwest::Client::new();

        // Create user / 创建用户
        let response = client
            .post("http://localhost:8080/api/users")
            .json(&json!({"name": "Alice", "email": "alice@test.com"}))
            .send().await.unwrap();
        assert_eq!(response.status(), 201);

        // Get user / 获取用户
        let response = client
            .get("http://localhost:8080/api/users/1")
            .send().await.unwrap();
        assert_eq!(response.status(), 200);

        Ok(())
    })
}
```

---

## Test Organization / 测试组织

```
my-app/
├── src/
│   ├── lib.rs
│   └── handler.rs        # #[cfg(test)] mod tests { ... }
├── tests/
│   ├── api_test.rs        # Integration tests
│   └── data_test.rs       # Data layer tests
└── migrations/
    ├── V1__create_users.sql
    └── V2__add_posts.sql
```

---

## Best Practices / 最佳实践

1. **Test in isolation** — mock external dependencies / 隔离测试，模拟外部依赖
2. **Use `sqlite::memory:`** for data tests — fast, no setup / 数据测试用内存 SQLite
3. **Test error cases** — verify error responses / 测试错误情况
4. **Use `TestClient`** for handler tests — no real HTTP needed / 用 TestClient 测试处理器
5. **Run migrations** before data tests — ensure schema is current / 数据测试前先跑迁移

---

*← [Previous / 上一页](./web3.md) | [Next / 下一页](../reference/api.md) →*
