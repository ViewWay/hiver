//! Project template generation.
//! 项目模板生成。

/// Generate Cargo.toml content.
/// 生成 Cargo.toml 内容。
pub fn generate_cargo_toml(name: &str, modules: &[String]) -> String {
    let mut deps = vec![("hiver-runtime", r#""0.1""#)];

    for module in modules {
        match module.as_str() {
            "web" => {
                deps.push(("hiver-http", r#""0.1""#));
                deps.push(("hiver-router", r#""0.1""#));
                deps.push(("hiver-middleware", r#""0.1""#));
                deps.push(("hiver-extractors", r#""0.1""#));
                deps.push(("hiver-response", r#""0.1""#));
                deps.push(("serde", r#"{ version = "1", features = ["derive"] }"#));
                deps.push(("serde_json", r#""0.1""#));
            },
            "security" => deps.push(("hiver-security", r#""0.1""#)),
            "data" => {
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
            _ => {},
        }
    }

    if !modules.contains(&"web".to_string()) {
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

/// Generate application.toml content.
/// 生成 application.toml 内容。
pub fn generate_application_toml(modules: &[String]) -> String {
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

    if modules.contains(&"data".to_string()) {
        config.push_str(
            r##"[data.source]
url = "postgresql://localhost:5432/mydb"
username = "user"
password = "pass"
max_connections = 20

"##,
        );
    }

    if modules.contains(&"cache".to_string()) {
        config.push_str(
            r##"[cache]
type = "redis"
url = "redis://localhost:6379"

"##,
        );
    }

    if modules.contains(&"security".to_string()) {
        config.push_str(
            r##"[security]
jwt_secret = "change-me-in-production"
jwt_expiration = 3600

"##,
        );
    }

    config
}
