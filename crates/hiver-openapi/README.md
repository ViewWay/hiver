# hiver-openapi

**OpenAPI/Swagger documentation support for the Hiver Framework.**

**Nexus框架的OpenAPI/Swagger文档支持。**

## Overview / 概述

`hiver-openapi` provides OpenAPI 3.0 specification generation with type-safe API documentation, similar to SpringDoc OpenAPI (Swagger 3).

`hiver-openapi` 提供OpenAPI 3.0规范生成，支持类型安全的API文档，类似于SpringDoc OpenAPI (Swagger 3)。

## Features / 功能

- **OpenAPI 3.0** - Full OpenAPI 3.0 specification support
- **Type-safe** - Compile-time validated schemas
- **Annotations** - Declarative API documentation
- **Swagger UI** - Built-in Swagger UI integration
- **Schema Generation** - Auto-generate schemas from types

- **OpenAPI 3.0** - 完整的OpenAPI 3.0规范支持
- **类型安全** - 编译时验证的模式
- **注解** - 声明式API文档
- **Swagger UI** - 内置Swagger UI集成
- **模式生成** - 从类型自动生成模式

## Equivalent to Spring Boot / 等价于 Spring Boot

| Spring Boot | Nexus |
|-------------|-------|
| `@OpenAPIDefinition` | `OpenApiConfig` |
| `@Operation` | `#[openapi]` |
| `@ApiResponse` | `response` attribute |
| `@Parameter` | `param` attribute |
| `@Schema` | `#[schema]` |
| SpringDoc | utoipa |

## Installation / 安装

```toml
[dependencies]
hiver-openapi = { version = "0.1", features = ["swagger"] }
serde = { version = "1.0", features = ["derive"] }
```

## Quick Start / 快速开始

### Basic Setup

```rust
use hiver_openapi::{OpenApi, OpenApiConfig};
use hiver_router::Router;
use serde::Serialize;

#[derive(Serialize)]
#[openapi]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[openapi(
    summary = "Get user by ID",
    description = "Returns a single user",
    tags = ["users"],
    responses(
        code = 200, description = "User found", response = User,
        code = 404, description = "User not found"
    )
)]
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(fetch_user(id).await)
}

#[tokio::main]
async fn main() {
    let config = OpenApiConfig::new()
        .title("My API")
        .version("1.0.0");

    let openapi = OpenApi::new(config);
    let spec = openapi.generate().await;
}
```

### Running Swagger UI

```rust
use hiver_openapi::openapi_routes;
use hiver_http::Server;

let app = Router::new()
    .merge(openapi_routes())
    .nest("/api", api_router());

Server::bind("127.0.0.1:8080")
    .run(app)
    .await;
```

Access Swagger UI at: `http://localhost:8080/swagger-ui`

## API Documentation / API 文档

### Core Types / 核心类型

| Type / 类型 | Description / 描述 |
|-------------|---------------------|
| `OpenApi` | OpenAPI specification |
| `OpenApiConfig` | Configuration |
| `Schema` | Schema definition |
| `Operation` | API operation |
| `Response` | API response |
| `Parameter` | Operation parameter |
| `RbacManager` | RBAC with caching |

### Derive Macros / 派生宏

| Macro / 宏 | Description / 描述 |
|-------------|---------------------|
| `#[openapi]` | Document API operation |
| `#[schema]` | Document schema type |
| `#[into_params]` | Document parameters |
| `#[to_response]` | Document response |

### Modules / 模块

| Module / 模块 | Description / 描述 |
|---------------|---------------------|
| `config` | Configuration types |
| `schema` | Schema definitions |
| `operation` | Operation definitions |
| `response` | Response definitions |
| `path` | Path definitions |
| `openapi` | OpenAPI builder |

## Configuration / 配置

### Basic Configuration

```rust
use hiver_openapi::OpenApiConfig;

let config = OpenApiConfig::new()
    .title("My API")
    .version("1.0.0")
    .description("API description")
    .contact(ContactConfig::new()
        .name("API Team")
        .email("api@example.com"))
    .license(LicenseConfig::new("MIT"))
    .add_server(ServerConfig::new("http://localhost:8080")
        .description("Local server"))
    .add_tag(TagConfig::new("users")
        .description("User operations"));
```

### Security Configuration

```rust
let config = OpenApiConfig::new()
    .add_security_scheme(
        "bearerAuth".to_string(),
        SecuritySchemeConfig::Http {
            scheme: "bearer".to_string(),
            bearer_format: Some("JWT".to_string()),
        }
    );
```

## Schema Documentation / 模式文档

### Documenting Types

```rust
use hiver_openapi::ToSchema;
use serde::Serialize;

#[derive(Serialize, ToSchema)]
#[schema(description = "User object")]
struct User {
    /// User ID
    #[schema(example = 1)]
    id: u64,

    /// Username
    #[schema(example = "alice", min_length = 3, max_length = 20)]
    username: String,

    /// Email address
    #[schema(example = "alice@example.com", format = "email")]
    email: String,
}
```

### Nested Schemas

```rust
#[derive(Serialize, ToSchema)]
struct Address {
    street: String,
    city: String,
    country: String,
}

#[derive(Serialize, ToSchema)]
struct User {
    id: u64,
    name: String,
    #[schema(description = "User address")]
    address: Address,
}
```

## Operation Documentation / 操作文档

### Documenting Endpoints

```rust
#[openapi(
    summary = "Create user",
    description = "Creates a new user account",
    operation_id = "createUser",
    tags = ["users"],
    deprecated = false,
    responses(
        code = 201, description = "User created", response = User,
        code = 400, description = "Invalid input",
        code = 409, description = "User already exists"
    )
)]
async fn create_user(Json(user): Json<CreateUser>) -> Response {
    // ...
}
```

### Documenting Parameters

```rust
#[openapi(
    summary = "Search users",
    params(
        name = "q",
        description = "Search query",
        required = false,
        example = "john"
    )
)]
async fn search_users(Query(q): Query<Option<String>>) -> Json<Vec<User>> {
    // ...
}
```

## Response Documentation / 响应文档

### Success Response

```rust
use hiver_openapi::ApiResponse;

#[openapi(
    summary = "Get user",
    responses(
        code = 200,
        description = "User found",
        response = User,
        headers(
            name = "X-Rate-Limit",
            description = "Rate limit remaining",
            example = "100"
        )
    )
)]
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    // ...
}
```

### Error Response

```rust
#[openapi(
    responses(
        code = 404,
        description = "User not found",
        response = ErrorResponse
    )
)]
async fn get_user(Path(id): Path<u64>) -> Result<Json<User>> {
    // ...
}
```

## Swagger UI Integration / Swagger UI集成

### Enable Swagger UI

```toml
[dependencies]
hiver-openapi = { version = "0.1", features = ["swagger"] }
```

### Access Documentation

```bash
# OpenAPI spec JSON
curl http://localhost:8080/api-docs/openapi.json

# Swagger UI
open http://localhost:8080/swagger-ui/
```

### Custom Styling

```rust
use hiver_openapi::SwaggerUi;

let swagger_ui = SwaggerUi::new("/api-docs/openapi.json")
    .title("My API Docs")
    .theme("dark")
    .persist_authorization("cookie");
```

## Examples / 示例

- `basic_api.rs` - Basic API documentation
- `schemas.rs` - Schema documentation
- `security.rs` - Security documentation

## License / 许可证

MIT License. See [LICENSE](https://github.com/hiver-rs/nexus/blob/main/LICENSE) for details.

---

**Spring Boot Equivalence**: SpringDoc OpenAPI, Swagger 3

**Spring Boot 等价物**: SpringDoc OpenAPI, Swagger 3
