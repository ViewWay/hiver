//! Project template generation.
//! 项目模板生成。

/// Generate Cargo.toml content.
/// 生成 Cargo.toml 内容。
pub fn generate_cargo_toml(name: &str, modules: &[String]) -> String {
    let mut deps = vec![
        ("hiver-runtime", "\"0.1\""),
    ];

    // Map modules to dependencies.
    // 将模块映射到依赖。
    for module in modules {
        match module.as_str() {
            "web" => {
                deps.push(("hiver-http", "\"0.1\""));
                deps.push(("hiver-router", "\"0.1\""));
                deps.push(("hiver-middleware", "\"0.1\""));
                deps.push(("hiver-extractors", "\"0.1\""));
                deps.push(("hiver-response", "\"0.1\""));
                deps.push(("serde", "{ version = \"1\", features = [\"derive\"] }"));
                deps.push(("serde_json", "\"1\""));
            }
            "security" => deps.push(("hiver-security", "\"0.1\"")),
            "data" => {
                deps.push(("hiver-data-rdbc", "\"0.1\""));
                deps.push(("hiver-data-orm", "\"0.1\""));
                deps.push(("hiver-tx", "\"0.1\""));
            }
            "cache" => deps.push(("hiver-data-redis", "\"0.1\"")),
            "schedule" => deps.push(("hiver-schedule", "\"0.1\"")),
            "actuator" => deps.push(("hiver-actuator", "\"0.1\"")),
            "web3" => deps.push(("hiver-web3", "\"0.1\"")),
            "graphql" => deps.push(("hiver-graphql", "\"0.1\"")),
            "grpc" => deps.push(("hiver-grpc", "\"0.1\"")),
            "ai" => deps.push(("hiver-ai", "\"0.1\"")),
            _ => {}
        }
    }

    // Always include tracing.
    // 始终包含 tracing。
    if !modules.contains(&"web".to_string()) {
        deps.push(("serde", "{ version = \"1\", features = [\"derive\"] }"));
    }

    let deps_str = deps.iter()
        .map(|(name, version)| format!("{} = {}", name, version))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
{deps}

tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
"#,
        name = name, deps = deps_str
    )
}

/// Generate main.rs content.
/// 生成 main.rs 内容。
pub fn generate_main_rs(modules: &[String]) -> String {
    let has_web = modules.contains(&"web".to_string());

    if has_web {
        let hello_body = r#"{"message":"Hello, Hiver!"}"#;
        format!(r#"//! Hiver application entry point.
//! Hiver 应用程序入口。

use hiver_http::{{Body, Response, Server, StatusCode}};
use hiver_router::Router;
use hiver_runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Initialize tracing / 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Build router / 构建路由
    let app = Router::new()
        .route("/", hiver_router::Method::GET, hello);

    // Create and run runtime / 创建并运行运行时
    let mut runtime = Runtime::new()?;

    tracing::info!("Server running on http://127.0.0.1:8080");

    runtime.block_on(async {{
        let _server = Server::bind("127.0.0.1:8080")
            .run(app)
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }})
}}

/// Hello handler / Hello 处理器
async fn hello(_req: hiver_http::Request) -> Result<Response, hiver_http::Error> {{
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from({hello_body}))
        .unwrap())
}}
"#, hello_body = hello_body)
    } else {
        r#"//! Hiver application entry point.
//! Hiver 应用程序入口。

use hiver_runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut runtime = Runtime::new()?;

    runtime.block_on(async {
        tracing::info!("Hiver application started");
        // TODO: Add your application logic here
        // TODO: 在这里添加你的应用逻辑
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}
"#.to_string()
    }
}

/// Generate application.toml content.
/// 生成 application.toml 内容。
pub fn generate_application_toml(modules: &[String]) -> String {
    let mut config = String::from(
r#"# Hiver Application Configuration
# Hiver 应用配置

[server]
host = "127.0.0.1"
port = 8080
workers = 4

[logging]
level = "info"
mode = "verbose"  # "verbose" (dev) or "simple" (prod)

"#,
    );

    if modules.contains(&"data".to_string()) {
        config.push_str(
r#"[data.source]
url = "postgresql://localhost:5432/mydb"
username = "user"
password = "pass"
max_connections = 20

"#,
        );
    }

    if modules.contains(&"cache".to_string()) {
        config.push_str(
r#"[cache]
type = "redis"
url = "redis://localhost:6379"

"#,
        );
    }

    if modules.contains(&"security".to_string()) {
        config.push_str(
r#"[security]
jwt_secret = "change-me-in-production"
jwt_expiration = 3600

"#,
        );
    }

    config
}
