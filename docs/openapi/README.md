# OpenAPI Documentation / OpenAPI 文档

## Overview / 概述

`hiver-openapi` provides OpenAPI/Swagger documentation support for the Hiver Framework, equivalent to SpringDoc OpenAPI in Spring Boot.
`hiver-openapi` 为 Hiver 框架提供 OpenAPI/Swagger 文档支持，等价于 Spring Boot 中的 SpringDoc OpenAPI。

## Features / 功能

- **OpenAPI 3.0 Specification** - Full OpenAPI 3.0.3 support / 完整的 OpenAPI 3.0.3 支持
- **Swagger UI** - Interactive API documentation / 交互式 API 文档
- **Schema Generation** - Auto-generate schemas from structs / 从结构体自动生成模式
- **HTTP Framework Integration** - Easy integration with HTTP handlers / 与 HTTP 处理器轻松集成
- **Spring Boot Compatible** - Similar API to Spring annotations / 与 Spring 注解类似的 API

## Quick Start / 快速开始

### 1. Define Your API Schemas / 定义 API 模式

```rust
use hiver_openapi::ToSchema;
use serde::Serialize;

#[derive(Serialize, ToSchema)]
#[schema(description = "User entity")]
struct User {
    #[schema(description = "User ID", example = "1")]
    id: u64,

    #[schema(description = "User name", example = "John Doe")]
    name: String,

    #[schema(description = "User email", example = "user@example.com")]
    email: String,
}
```

### 2. Create OpenAPI Specification / 创建 OpenAPI 规范

```rust
use hiver_openapi::{OpenApi, OpenApiConfig, InfoConfig};

let openapi = OpenApi::new(OpenApiConfig {
    info: InfoConfig {
        title: "My API".to_string(),
        version: "1.0.0".to_string(),
        description: Some("My API description".to_string()),
        ..Default::default()
    },
    ..Default::default()
})
.add_schema("User", Schema::object());
```

### 3. Serve Swagger UI / 服务 Swagger UI

```rust
use hiver_openapi::{SwaggerUi, SwaggerConfig};

let swagger = SwaggerUi::with_config(
    openapi,
    SwaggerConfig::new()
        .path("/api-docs")
        .spec_path("/api-docs/openapi.json")
        .title("My API Documentation")
);

// Handle requests
let (body, status, headers) = swagger.handle("/api-docs");
```

## Spring Boot Comparison / Spring Boot 对比

| Hiver | Spring Boot | Description |
|-------|-------------|-------------|
| `#[derive(ToSchema)]` | `@Schema` | Define schema for struct / 为结构体定义模式 |
| `#[openapi(...)]` | `@OpenAPIDefinition` | Define OpenAPI spec / 定义 OpenAPI 规范 |
| `SwaggerUi` | `springdoc-openapi-ui` | Serve Swagger UI / 服务 Swagger UI |
| `OpenApiConfig` | `OpenAPIConfig` | OpenAPI configuration / OpenAPI 配置 |
| `InfoConfig` | `@Info` | API info / API 信息 |

## Examples / 示例

### Complete API Documentation Example / 完整 API 文档示例

```rust
use hiver_openapi::*;
use serde::{Deserialize, Serialize};

// Define request/response schemas
// 定义请求/响应模式
#[derive(Serialize, Deserialize, ToSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Serialize, ToSchema)]
struct UserResponse {
    id: u64,
    name: String,
    email: String,
}

// Create OpenAPI specification
// 创建 OpenAPI 规范
let openapi = OpenApiBuilder::new()
    .title("User Management API")
    .version("1.0.0")
    .description("API for managing users")
    .add_path(
        "/users",
        PathItem::new().post(
            Operation::new()
                .summary("Create user")
                .description("Create a new user")
                .add_response("201", Response::created("User created"))
                .add_response("400", Response::bad_request("Invalid input"))
                .request_body(RequestBody::new()
                    .description("User data")
                    .content("application/json", Schema::reference("CreateUserRequest"))
                )
        )
    )
    .add_path(
        "/users/{id}",
        PathItem::new().get(
            Operation::new()
                .summary("Get user")
                .description("Get user by ID")
                .add_parameter("id", Parameter::path("id")
                    .description("User ID")
                    .required(true)
                )
                .add_response("200", Response::ok("User found")
                    .json(Schema::reference("UserResponse"))
                )
                .add_response("404", Response::not_found("User not found"))
        )
    )
    .add_schema("UserResponse", Schema::object())
    .add_schema("CreateUserRequest", Schema::object())
    .build();

// Serve with Swagger UI
// 使用 Swagger UI 服务
let swagger = SwaggerUi::new(openapi);
```

### HTTP Framework Integration / HTTP 框架集成

```rust
use hiver_openapi::*;
use http::StatusCode;

// Create handler
// 创建处理器
let handler = OpenApiHandler::new(openapi);

// Handle requests
// 处理请求
let response = handler.handle("/swagger/openapi.json");
assert_eq!(response.status, StatusCode::OK);
assert!(response.body.contains("\"openapi\""));
```

### Custom Swagger UI Configuration / 自定义 Swagger UI 配置

```rust
use hiver_openapi::{SwaggerConfig, ModelRendering, SyntaxHighlightTheme};

let config = SwaggerConfig::new()
    .path("/docs")
    .spec_path("/docs/spec.json")
    .title("API Documentation")
    .logo_url("https://example.com/logo.png")
    .display_request_duration(true)
    .default_models_expand_depth(2)
    .default_model_rendering(ModelRendering::Model)
    .try_it_out_enabled(true)
    .persist_authorization(true)
    .syntax_highlight_theme(SyntaxHighlightTheme::Monokai);

let swagger = SwaggerUi::with_config(openapi, config);
```

## API Reference / API 参考

### Main Types / 主要类型

| Type | Description |
|------|-------------|
| `OpenApi` | Root OpenAPI specification / 根 OpenAPI 规范 |
| `OpenApiBuilder` | Builder for creating OpenApi specs / 用于创建 OpenApi 规范的构建器 |
| `OpenApiConfig` | Configuration for OpenApi spec / OpenApi 规范的配置 |
| `InfoConfig` | API information / API 信息 |
| `SwaggerUi` | Swagger UI handler / Swagger UI 处理器 |
| `SwaggerConfig` | Configuration for Swagger UI / Swagger UI 的配置 |
| `OpenApiHandler` | HTTP handler for OpenAPI / OpenAPI 的 HTTP 处理器 |
| `OpenApiRouter` | Router integration helper / 路由器集成助手 |

### Schema Types / 模式类型

| Type | Description |
|------|-------------|
| `Schema` | OpenAPI schema definition / OpenAPI 模式定义 |
| `SchemaType` | Schema type (string, number, etc.) / 模式类型 |
| `SchemaFormat` | Schema format (int32, float, etc.) / 模式格式 |
| `SchemaProperty` | Schema property wrapper / 模式属性包装器 |

### Operation Types / 操作类型

| Type | Description |
|------|-------------|
| `Operation` | API operation definition / API 操作定义 |
| `Parameter` | Operation parameter / 操作参数 |
| `ParameterLocation` | Parameter location (path, query, etc.) / 参数位置 |
| `RequestBody` | Request body definition / 请求体定义 |
| `Response` | Response definition / 响应定义 |
| `SecurityScheme` | Security scheme definition / 安全方案定义 |

### Path Types / 路径类型

| Type | Description |
|------|-------------|
| `PathItem` | Path item with operations / 包含操作的路径项 |
| `PathMethod` | HTTP method (GET, POST, etc.) / HTTP 方法 |
| `Components` | Reusable components / 可重用组件 |

## Macros / 宏

### `#[derive(ToSchema)]`

Define OpenAPI schema for a struct.
为结构体定义 OpenAPI 模式。

```rust
#[derive(ToSchema)]
struct User {
    id: u64,
    name: String,
}
```

### `#[openapi(...)]`

Define OpenAPI specification at module or struct level.
在模块或结构体级别定义 OpenAPI 规范。

```rust
#[openapi(
    paths(
        get_user = (
            path = "/users/{id}",
            method = get,
            operation_id = "getUser",
            responses = (
                (status = 200, description = "Success", body = User)
            )
        )
    )
)]
struct ApiDoc;
```

## Configuration Options / 配置选项

### SwaggerConfig / Swagger UI 配置

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | `String` | `"/swagger"` | Swagger UI path / Swagger UI 路径 |
| `spec_path` | `String` | `"/swagger/openapi.json"` | OpenAPI spec path / OpenAPI 规范路径 |
| `title` | `Option<String>` | `None` | Page title / 页面标题 |
| `logo_url` | `Option<String>` | `None` | Logo URL / logo URL |
| `display_request_duration` | `bool` | `false` | Show request duration / 显示请求持续时间 |
| `default_models_expand_depth` | `Option<usize>` | `None` | Default models expand depth / 默认模型展开深度 |
| `default_model_rendering` | `ModelRendering` | `Example` | Default model rendering / 默认模型渲染 |
| `display_operation_id` | `bool` | `false` | Display operation ID / 显示操作 ID |
| `try_it_out_enabled` | `bool` | `true` | Enable "Try it out" / 启用"尝试"功能 |
| `persist_authorization` | `bool` | `false` | Persist authorization / 持久化授权 |
| `syntax_highlight_theme` | `SyntaxHighlightTheme` | `Monokai` | Syntax highlight theme / 语法高亮主题 |

### SyntaxHighlightTheme Options / 语法高亮主题选项

- `Agate`, `Artsy`, `AtomOneDark`, `AtomOneLight`
- `GithubDark`, `GithubLight`
- `Monokai`, `Nord`, `Obsidian`
- `TomorrowNight`, `VsCodeDark`, `VsCodeLight`

## See Also / 另请参阅

- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger UI Documentation](https://swagger.io/tools/swagger-ui/)
- [utoipa Documentation](https://docs.rs/utoipa/)
