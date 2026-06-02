//! `hiver generate` command implementation.
//! `hiver generate` 命令实现。

use std::fs;
use std::path::Path;

use console::style;

use crate::cli::GenerateArgs;

/// Run the `hiver generate` command.
/// 执行 `hiver generate` 命令。
pub fn run(args: &GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let gen_type = args.gen_type.to_lowercase();
    let name = &args.name;

    // Convert name to snake_case for file names and Rust identifiers.
    // 将名称转换为 snake_case 用于文件名和 Rust 标识符。
    let snake_name = to_snake_case(name);
    let pascal_name = to_pascal_case(name);

    let (code, subdir) = match gen_type.as_str() {
        "controller" | "c" => (generate_controller(&pascal_name, &snake_name), "controller"),
        "service" | "s"    => (generate_service(&pascal_name, &snake_name), "service"),
        "repository" | "r" => (generate_repository(&pascal_name, &snake_name), "repository"),
        "entity" | "e"     => (generate_entity(&pascal_name, &snake_name), "entity"),
        "middleware" | "m"  => (generate_middleware(&pascal_name, &snake_name), "middleware"),
        "config" | "cfg"   => (generate_config(&pascal_name, &snake_name), "config"),
        _ => return Err(format!(
            "Unknown type '{}'. Available: controller, service, repository, entity, middleware, config\n\
             未知类型 '{}'。可用：controller, service, repository, entity, middleware, config",
            gen_type, gen_type
        ).into()),
    };

    // Write file to src/<subdir>/<name>.rs.
    // 写入文件到 src/<subdir>/<name>.rs。
    let dir = Path::new("src").join(subdir);
    fs::create_dir_all(&dir)?;

    let file_path = dir.join(format!("{}.rs", snake_name));
    if file_path.exists() {
        return Err(format!(
            "File '{}' already exists / 文件 '{}' 已存在",
            file_path.display(),
            file_path.display()
        )
        .into());
    }

    fs::write(&file_path, code)?;

    println!(
        "{} Generated {} '{}' -> {}",
        style("✓").green().bold(),
        style(&gen_type).cyan(),
        style(name).green(),
        style(file_path.display()).dim(),
    );

    Ok(())
}

/// Generate a controller template.
/// 生成控制器模板。
fn generate_controller(pascal: &str, snake: &str) -> String {
    format!(
        r#"//! {pascal} controller.
//! {pascal} 控制器。

/// {pascal} controller / {pascal} 控制器
pub struct {pascal}Controller;

impl {pascal}Controller {{
    /// GET /{snake}
    /// List all {snake} / 列出所有 {snake}
    pub async fn list() -> impl IntoResponse {{
        // TODO: Implement list / TODO: 实现 list
        Json(vec::<String>::new())
    }}

    /// GET /{snake}/:id
    /// Get {snake} by ID / 按 ID 获取 {snake}
    pub async fn get(Path(id): Path<u64>) -> impl IntoResponse {{
        // TODO: Implement get / TODO: 实现 get
        Json(serde_json::json!({{"id": id}}))
    }}

    /// POST /{snake}
    /// Create {snake} / 创建 {snake}
    pub async fn create() -> impl IntoResponse {{
        // TODO: Implement create / TODO: 实现 create
        StatusCode::CREATED
    }}
}}
"#,
        pascal = pascal,
        snake = snake
    )
}

/// Generate a service template.
/// 生成服务模板。
fn generate_service(pascal: &str, _snake: &str) -> String {
    format!(
        r#"//! {pascal} service.
//! {pascal} 服务。

/// {pascal} service trait / {pascal} 服务 trait
pub trait {pascal}Service {{
    /// Find by ID / 按 ID 查找
    fn find_by_id(&self, id: u64) -> Option<String>;

    /// List all / 列出所有
    fn list_all(&self) -> Vec<String>;
}}

/// Default {pascal} service implementation.
/// 默认 {pascal} 服务实现。
pub struct Default{pascal}Service;

impl {pascal}Service for Default{pascal}Service {{
    fn find_by_id(&self, _id: u64) -> Option<String> {{
        // TODO: Implement / TODO: 实现
        None
    }}

    fn list_all(&self) -> Vec<String> {{
        // TODO: Implement / TODO: 实现
        vec![]
    }}
}}
"#,
        pascal = pascal
    )
}

/// Generate a repository template.
/// 生成仓储模板。
fn generate_repository(pascal: &str, _snake: &str) -> String {
    format!(
        r#"//! {pascal} repository.
//! {pascal} 仓储。

/// {pascal} repository trait / {pascal} 仓储 trait
pub trait {pascal}Repository {{
    /// Find by ID / 按 ID 查找
    async fn find_by_id(&self, id: i64) -> Result<Option<String>, Box<dyn std::error::Error>>;

    /// Find all / 查找全部
    async fn find_all(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;

    /// Save / 保存
    async fn save(&self, entity: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Delete by ID / 按 ID 删除
    async fn delete_by_id(&self, id: i64) -> Result<bool, Box<dyn std::error::Error>>;
}}
"#,
        pascal = pascal
    )
}

/// Generate an entity template.
/// 生成实体模板。
fn generate_entity(pascal: &str, _snake: &str) -> String {
    format!(
        r#"//! {pascal} entity.
//! {pascal} 实体。

use serde::{{Deserialize, Serialize}};

/// {pascal} entity / {pascal} 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal} {{
    /// ID / 标识
    pub id: Option<i64>,
    /// Name / 名称
    pub name: String,
    /// Created at / 创建时间
    pub created_at: Option<String>,
    /// Updated at / 更新时间
    pub updated_at: Option<String>,
}}
"#,
        pascal = pascal
    )
}

/// Generate a middleware template.
/// 生成中间件模板。
fn generate_middleware(pascal: &str, _snake: &str) -> String {
    format!(
        r#"//! {pascal} middleware.
//! {pascal} 中间件。

/// {pascal} middleware / {pascal} 中间件
pub struct {pascal}Middleware;

impl {pascal}Middleware {{
    /// Create new instance / 创建新实例
    pub fn new() -> Self {{
        Self
    }}
}}

impl Default for {pascal}Middleware {{
    fn default() -> Self {{
        Self::new()
    }}
}}
"#,
        pascal = pascal
    )
}

/// Generate a config template.
/// 生成配置模板。
fn generate_config(pascal: &str, _snake: &str) -> String {
    format!(
        r#"//! {pascal} configuration.
//! {pascal} 配置。

use serde::{{Deserialize, Serialize}};

/// {pascal} config / {pascal} 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal}Config {{
    /// Enabled / 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}}

fn default_enabled() -> bool {{
    true
}}

impl Default for {pascal}Config {{
    fn default() -> Self {{
        Self {{
            enabled: default_enabled(),
        }}
    }}
}}
"#,
        pascal = pascal
    )
}

/// Convert to snake_case.
/// 转换为 snake_case。
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    // Handle kebab-case input.
    result.replace('-', "_")
}

/// Convert to PascalCase.
/// 转换为 PascalCase。
fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
