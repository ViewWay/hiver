//! `hiver add` command implementation.
//! `hiver add` 命令实现。

use std::fs;
use std::path::Path;

use console::style;

use crate::cli::AddArgs;

/// Module to Cargo dependency mapping.
/// 模块到 Cargo 依赖的映射。
const MODULE_DEPS: &[(&str, &[&str])] = &[
    ("web",        &["hiver-http", "hiver-router", "hiver-middleware"]),
    ("security",   &["hiver-security"]),
    ("data",       &["hiver-data-rdbc", "hiver-data-orm", "hiver-tx", "hiver-flyway"]),
    ("cache",      &["hiver-data-redis"]),
    ("schedule",   &["hiver-schedule"]),
    ("actuator",   &["hiver-actuator"]),
    ("web3",       &["hiver-web3"]),
    ("graphql",    &["hiver-graphql"]),
    ("grpc",       &["hiver-grpc"]),
    ("ai",         &["hiver-ai"]),
];

/// Run the `hiver add` command.
/// 执行 `hiver add` 命令。
pub fn run(args: &AddArgs) -> Result<(), Box<dyn std::error::Error>> {
    let module = args.module.to_lowercase();

    let deps = MODULE_DEPS.iter()
        .find(|(name, _)| *name == module)
        .ok_or_else(|| format!(
            "Unknown module '{}'. Available: web, security, data, cache, schedule, actuator, web3, graphql, grpc, ai\n\
             未知模块 '{}'。可用：web, security, data, cache, schedule, actuator, web3, graphql, grpc, ai",
            module, module
        ))?
        .1;

    // Find Cargo.toml in current directory.
    // 在当前目录查找 Cargo.toml。
    let cargo_path = Path::new("Cargo.toml");
    if !cargo_path.exists() {
        return Err("No Cargo.toml found in current directory / 当前目录未找到 Cargo.toml".into());
    }

    let content = fs::read_to_string(cargo_path)?;
    let mut doc = content.parse::<toml::Value>()?;

    // Ensure [dependencies] section exists.
    // 确保 [dependencies] 段存在。
    if doc.get("dependencies").is_none() {
        doc.as_table_mut().unwrap().insert("dependencies".to_string(), toml::Value::Table(toml::map::Map::new()));
    }

    let deps_table = doc["dependencies"].as_table_mut().unwrap();
    let version = "\"0.1\"";

    for dep in deps {
        if !deps_table.contains_key(*dep) {
            deps_table.insert(dep.to_string(), toml::Value::String(version.to_string()));
            println!(
                "  {} Added dependency: {}",
                style("+").green(),
                style(dep).green().bold()
            );
        } else {
            println!(
                "  {} Already exists: {}",
                style("→").yellow(),
                style(dep).yellow()
            );
        }
    }

    let output = toml::to_string_pretty(&doc)?;
    fs::write(cargo_path, output)?;

    println!(
        "\n{} Module '{}' added successfully / 模块 '{}' 添加成功",
        style("✓").green().bold(),
        style(&module).green(),
        style(&module).green(),
    );

    Ok(())
}
