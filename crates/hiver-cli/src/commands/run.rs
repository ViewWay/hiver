//! `hiver run`, `build`, `check`, `test` command implementations.
//! `hiver run`、`build`、`check`、`test` 命令实现。

use std::process::Command;

use console::style;

use crate::cli::{BuildArgs, RunArgs, TestArgs};

/// Run `hiver run` (wraps cargo run).
/// 执行 `hiver run`（封装 cargo run）。
pub fn run(args: &RunArgs) -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = Command::new("cargo");
    cmd.arg("run");

    if args.release
    {
        cmd.arg("--release");
    }

    if !args.args.is_empty()
    {
        cmd.arg("--");
        cmd.args(&args.args);
    }

    println!("{} Running project / 运行项目", style(">").cyan());
    let status = cmd.status()?;
    if !status.success()
    {
        return Err("Build/run failed / 构建或运行失败".into());
    }
    Ok(())
}

/// Run `hiver build` (wraps cargo build).
/// 执行 `hiver build`（封装 cargo build）。
pub fn build(args: &BuildArgs) -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if args.release
    {
        cmd.arg("--release");
    }

    println!("{} Building project / 构建项目", style(">").cyan());
    let status = cmd.status()?;
    if !status.success()
    {
        return Err("Build failed / 构建失败".into());
    }
    Ok(())
}

/// Run `hiver check` (wraps cargo check).
/// 执行 `hiver check`（封装 cargo check）。
pub fn check() -> Result<(), Box<dyn std::error::Error>>
{
    println!("{} Checking project / 检查项目", style(">").cyan());
    let status = Command::new("cargo").args(["check"]).status()?;
    if !status.success()
    {
        return Err("Check failed / 检查失败".into());
    }
    Ok(())
}

/// Run `hiver test` (wraps cargo test).
/// 执行 `hiver test`（封装 cargo test）。
pub fn test(args: &TestArgs) -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = Command::new("cargo");
    cmd.arg("test");

    if let Some(filter) = &args.filter
    {
        cmd.arg(filter);
    }

    if args.nocapture
    {
        cmd.arg("--").arg("--nocapture");
    }

    println!("{} Running tests / 运行测试", style(">").cyan());
    let status = cmd.status()?;
    if !status.success()
    {
        return Err("Tests failed / 测试失败".into());
    }
    Ok(())
}
