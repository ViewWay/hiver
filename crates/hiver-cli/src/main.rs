//! Hiver CLI - Project scaffolding tool.
//! Hiver CLI - 项目脚手架工具。
//!
//! Generate, build, and manage Hiver projects from the command line.
//! 从命令行生成、构建和管理 Hiver 项目。

mod cli;
mod commands;
mod templates;

use cli::Cli;

fn main()
{
    let cli = Cli::parse();
    if let Err(e) = cli.run()
    {
        eprintln!("{} {}", console::style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
