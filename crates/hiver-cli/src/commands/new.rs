//! `hiver new` command implementation.
//! `hiver new` 命令实现。

use std::{fs, path::Path};

use console::style;
use dialoguer::MultiSelect;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{cli::NewArgs, templates::{arch::Architecture, project}};

/// Module definitions with display names and feature mappings.
/// 模块定义，包含显示名称和 feature 映射。
const MODULES: &[(&str, &str, &str)] = &[
    ("web", "Web (HTTP + Router + Middleware)", "Web 模块 (HTTP + 路由 + 中间件)"),
    ("security", "Security (JWT + OAuth2)", "安全模块 (JWT + OAuth2)"),
    ("data", "Data (RDBC + ORM + Flyway)", "数据模块 (RDBC + ORM + Flyway)"),
    ("cache", "Cache (Redis)", "缓存模块 (Redis)"),
    ("schedule", "Scheduling (Cron)", "调度模块 (Cron)"),
    ("actuator", "Actuator (Health + Metrics)", "运维模块 (健康检查 + 指标)"),
    ("web3", "Web3 (Blockchain)", "Web3 模块 (区块链)"),
    ("graphql", "GraphQL", "GraphQL 模块"),
    ("grpc", "gRPC", "gRPC 模块"),
    ("ai", "AI (Chat + Embedding + Agent)", "AI 模块 (聊天 + 向量 + Agent)"),
];

/// Run the `hiver new` command.
/// 执行 `hiver new` 命令。
pub fn run(args: &NewArgs) -> Result<(), Box<dyn std::error::Error>>
{
    // Validate architecture.
    // 验证架构。
    let arch = Architecture::from_str_opt(&args.arch).ok_or_else(|| {
        format!(
            "Unknown architecture '{}'. Valid: {}",
            args.arch,
            Architecture::valid_names()
        )
    })?;

    let project_dir = args.path.as_deref().unwrap_or(&args.name);
    let project_path = Path::new(project_dir);

    if project_path.exists()
    {
        return Err(format!(
            "Directory '{}' already exists / 目录 '{}' 已存在",
            project_dir, project_dir
        )
        .into());
    }

    // Determine which modules to include.
    // 确定要包含的模块。
    let modules = if args.all
    {
        MODULES
            .iter()
            .map(|(k, _, _)| k.to_string())
            .collect::<Vec<_>>()
    }
    else if args.web
        || args.security
        || args.data
        || args.cache
        || args.schedule
        || args.actuator
        || args.web3
        || args.graphql
        || args.grpc
        || args.ai
    {
        let mut selected = Vec::new();
        if args.web { selected.push("web"); }
        if args.security { selected.push("security"); }
        if args.data { selected.push("data"); }
        if args.cache { selected.push("cache"); }
        if args.schedule { selected.push("schedule"); }
        if args.actuator { selected.push("actuator"); }
        if args.web3 { selected.push("web3"); }
        if args.graphql { selected.push("graphql"); }
        if args.grpc { selected.push("grpc"); }
        if args.ai { selected.push("ai"); }
        selected.iter().map(|s| s.to_string()).collect()
    }
    else if !args.no_interactive
    {
        select_modules_interactive()?
    }
    else
    {
        vec!["web".to_string()]
    };

    println!(
        "{} Creating Hiver project '{}' [{}] / 创建 Hiver 项目 '{}' [{}]",
        style(">").cyan(),
        style(&args.name).green().bold(),
        style(&args.arch).yellow(),
        style(&args.name).green().bold(),
        style(&args.arch).yellow(),
    );

    let pb = ProgressBar::new(5);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {bar:40.cyan/blue} {pos}/{len}")?
            .progress_chars("##-"),
    );

    // Step 1: Create architecture-specific directory structure.
    pb.set_message("Creating directories / 创建目录");
    crate::templates::arch::create_arch_dirs(project_path, arch, &modules)?;
    pb.inc(1);

    // Step 2: Generate Cargo.toml.
    pb.set_message("Generating Cargo.toml / 生成 Cargo.toml");
    let cargo_toml = project::generate_cargo_toml(&args.name, &modules);
    fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    pb.inc(1);

    // Step 3: Generate architecture-specific main.rs.
    pb.set_message("Generating source files / 生成源代码");
    let main_rs = crate::templates::arch::generate_arch_main_rs(arch, &modules);
    fs::write(project_path.join("src").join("main.rs"), main_rs)?;
    pb.inc(1);

    // Step 4: Generate config.
    pb.set_message("Generating config / 生成配置");
    let app_toml = project::generate_application_toml(&modules);
    fs::write(project_path.join("resources").join("application.toml"), app_toml)?;
    pb.inc(1);

    // Step 5: Initialize git.
    pb.set_message("Initializing git / 初始化 git");
    let _ = std::process::Command::new("git")
        .args(["init", project_dir])
        .output();
    pb.inc(1);

    pb.finish_with_message("Done / 完成");

    println!();
    println!(
        "{} Project '{}' [{}] created successfully! / 项目 '{}' [{}] 创建成功！",
        style("✓").green().bold(),
        style(&args.name).green(),
        style(&args.arch).yellow(),
        style(&args.name).green(),
        style(&args.arch).yellow(),
    );
    println!();
    println!("  cd {}", args.name);
    println!("  hiver run");
    println!();

    Ok(())
}

/// Interactive module selection.
/// 交互式模块选择。
fn select_modules_interactive() -> Result<Vec<String>, Box<dyn std::error::Error>>
{
    println!(
        "{} Select modules to include (use space to toggle, enter to confirm)",
        style(">").cyan()
    );
    println!("{} 选择要包含的模块（空格切换，回车确认）", style(">").cyan());
    println!();

    let items: Vec<String> = MODULES
        .iter()
        .map(|(_, en, zh)| format!("{} / {}", en, zh))
        .collect();

    let defaults: Vec<bool> = vec![
        true, false, false, false, false, false, false, false, false, false,
    ];

    let selections = MultiSelect::new()
        .items(&items)
        .defaults(&defaults)
        .interact()?;

    Ok(selections
        .iter()
        .map(|&i| MODULES[i].0.to_string())
        .collect())
}
