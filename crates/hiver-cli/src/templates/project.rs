//! Project template generation.
//! 项目模板生成。

/// Generate Cargo.toml content.
/// 生成 Cargo.toml 内容。
pub fn generate_cargo_toml(name: &str, modules: &[String]) -> String
{
    let mut deps = vec![("hiver-runtime", r#""0.1""#)];

    for module in modules
    {
        match module.as_str()
        {
            "web" =>
            {
                deps.push(("hiver-http", r#""0.1""#));
                deps.push(("hiver-router", r#""0.1""#));
                deps.push(("hiver-middleware", r#""0.1""#));
                deps.push(("hiver-extractors", r#""0.1""#));
                deps.push(("hiver-response", r#""0.1""#));
                deps.push(("serde", r#"{ version = "1", features = ["derive"] }"#));
                deps.push(("serde_json", r#""0.1""#));
            },
            "security" => deps.push(("hiver-security", r#""0.1""#)),
            "data" =>
            {
                deps.push(("hiver-data-rdbc", r#""0.1""#));
                deps.push(("hiver-data-orm", r#""0.1""#));
                deps.push(("hiver-tx", r#""0.1""#));
            },
            "cache" => deps.push(("hiver-data-redis", r#""0.1""#)),
            "schedule" => deps.push(("hiver-schedule", r#""0.1""#)),
            "actuator" => deps.push(("hiver-actuator", r#""0.1""#)),
            "web3" => deps.push(("hiver-web3", r#""0.1""#)),
            "graphql" => deps.push(("hiver-graphql", r#""0.1""#)),
            "grpc" => deps.push(("hiver-grpc", r#""0.1""#)),
            "ai" => deps.push(("hiver-ai", r#""0.1""#)),
            _ =>
            {},
        }
    }

    if !modules.contains(&"web".to_string())
    {
        deps.push(("serde", r#"{ version = "1", features = ["derive"] }"#));
    }

    let deps_str = deps
        .iter()
        .map(|(name, version)| format!("{} = {}", name, version))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \
         \"1.85\"\n\n[dependencies]\n{}\n\ntracing = \"0.1\"\ntracing-subscriber = {{ version = \
         \"0.3\", features = [\"env-filter\"] }}\n",
        name, deps_str
    )
}

/// Generate main.rs content for web projects.
/// 为 Web 项目生成 main.rs 内容。
fn web_main_rs() -> String
{
    r##"//! Hiver application entry point.
//! Hiver 应用程序入口。

use hiver_http::{Body, Response, Server, StatusCode};
use hiver_router::Router;
use hiver_runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    runtime.block_on(async {
        let _server = Server::bind("127.0.0.1:8080")
            .run(app)
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
}

/// Hello handler / Hello 处理器
async fn hello(_req: hiver_http::Request) -> Result<Response, hiver_http::Error> {
    let body = r#"{"message":"Hello, Hiver!"}"#;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap())
}
"##
    .to_string()
}

/// Generate main.rs content for non-web projects.
/// 为非 Web 项目生成 main.rs 内容。
fn simple_main_rs() -> String
{
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
"#
    .to_string()
}

/// Generate main.rs content.
/// 生成 main.rs 内容。
pub fn generate_main_rs(modules: &[String]) -> String
{
    if modules.contains(&"web".to_string())
    {
        web_main_rs()
    }
    else
    {
        simple_main_rs()
    }
}

/// Generate application.toml content.
/// 生成 application.toml 内容。
pub fn generate_application_toml(modules: &[String]) -> String
{
    let mut config = String::from(
        r##"# Hiver Application Configuration
# Hiver 应用配置

[server]
host = "127.0.0.1"
port = 8080
workers = 4

[logging]
level = "info"
mode = "verbose"  # verbose for dev, simple for prod

"##,
    );

    if modules.contains(&"data".to_string())
    {
        config.push_str(
            r##"[data.source]
url = "postgresql://localhost:5432/mydb"
username = "user"
password = "pass"
max_connections = 20

"##,
        );
    }

    if modules.contains(&"cache".to_string())
    {
        config.push_str(
            r##"[cache]
type = "redis"
url = "redis://localhost:6379"

"##,
        );
    }

    if modules.contains(&"security".to_string())
    {
        config.push_str(
            r##"[security]
jwt_secret = "change-me-in-production"
jwt_expiration = 3600

"##,
        );
    }

    config
}
