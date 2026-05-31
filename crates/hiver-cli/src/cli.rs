//! CLI argument definitions.
//! CLI 参数定义。

use clap::{Parser, Subcommand};

use crate::commands;

/// Hiver CLI - Project scaffolding tool.
/// Hiver CLI - 项目脚手架工具。
#[derive(Parser)]
#[command(name = "hiver")]
#[command(version, about = "Hiver project scaffolding tool / Hiver 项目脚手架工具")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Parse arguments and execute the command.
    /// 解析参数并执行命令。
    pub fn parse() -> Self {
        Parser::parse()
    }

    /// Run the selected command.
    /// 执行选中的命令。
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Commands::New(args) => commands::new::run(&args),
            Commands::Run(args) => commands::run::run(&args),
            Commands::Build(args) => commands::run::build(&args),
            Commands::Add(args) => commands::add::run(&args),
            Commands::Generate(args) => commands::generate::run(&args),
            Commands::Check => commands::run::check(),
            Commands::Test(args) => commands::run::test(&args),
        }
    }
}

/// Available commands.
/// 可用命令。
#[derive(Subcommand)]
enum Commands {
    /// Create a new Hiver project / 创建新的 Hiver 项目
    #[command(alias = "n")]
    New(NewArgs),

    /// Run the project (cargo run) / 运行项目
    #[command(alias = "r")]
    Run(RunArgs),

    /// Build the project (cargo build) / 构建项目
    #[command(alias = "b")]
    Build(BuildArgs),

    /// Add a module dependency / 添加模块依赖
    #[command(alias = "a")]
    Add(AddArgs),

    /// Generate code from templates / 从模板生成代码
    #[command(alias = "g")]
    Generate(GenerateArgs),

    /// Check compilation (cargo check) / 检查编译
    #[command(alias = "c")]
    Check,

    /// Run tests (cargo test) / 运行测试
    #[command(alias = "t")]
    Test(TestArgs),
}

/// Arguments for `hiver new`.
/// `hiver new` 的参数。
#[derive(clap::Args)]
pub struct NewArgs {
    /// Project name / 项目名称
    pub name: String,

    /// Project path (defaults to current directory) / 项目路径
    #[arg(short, long)]
    pub path: Option<String>,

    /// Web module / Web 模块
    #[arg(long)]
    pub web: bool,

    /// Security module / 安全模块
    #[arg(long)]
    pub security: bool,

    /// Data module (RDBC + ORM) / 数据模块
    #[arg(long)]
    pub data: bool,

    /// Cache module (Redis) / 缓存模块
    #[arg(long)]
    pub cache: bool,

    /// Schedule module / 调度模块
    #[arg(long)]
    pub schedule: bool,

    /// Actuator module / 运维模块
    #[arg(long)]
    pub actuator: bool,

    /// Web3 module / Web3 模块
    #[arg(long)]
    pub web3: bool,

    /// GraphQL module / GraphQL 模块
    #[arg(long)]
    pub graphql: bool,

    /// gRPC module / gRPC 模块
    #[arg(long)]
    pub grpc: bool,

    /// AI module / AI 模块
    #[arg(long)]
    pub ai: bool,

    /// All modules / 全部模块
    #[arg(long)]
    pub all: bool,

    /// Skip interactive prompts / 跳过交互式提示
    #[arg(long)]
    pub no_interactive: bool,
}

/// Arguments for `hiver run`.
/// `hiver run` 的参数。
#[derive(clap::Args)]
pub struct RunArgs {
    /// Run in release mode / 以 release 模式运行
    #[arg(long, short)]
    pub release: bool,

    /// Arguments to pass to the binary / 传递给二进制的参数
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

/// Arguments for `hiver build`.
/// `hiver build` 的参数。
#[derive(clap::Args)]
pub struct BuildArgs {
    /// Build in release mode / 以 release 模式构建
    #[arg(long, short)]
    pub release: bool,
}

/// Arguments for `hiver add`.
/// `hiver add` 的参数。
#[derive(clap::Args)]
pub struct AddArgs {
    /// Module name (web, security, data, cache, schedule, actuator, web3, graphql, grpc, ai)
    /// 模块名称
    pub module: String,
}

/// Arguments for `hiver generate`.
/// `hiver generate` 的参数。
#[derive(clap::Args)]
pub struct GenerateArgs {
    /// Type of code to generate (controller, service, repository, entity, middleware, config)
    /// 要生成的代码类型
    pub gen_type: String,

    /// Name for the generated code / 生成代码的名称
    pub name: String,
}

/// Arguments for `hiver test`.
/// `hiver test` 的参数。
#[derive(clap::Args)]
pub struct TestArgs {
    /// Test name filter / 测试名称过滤
    #[arg(long, short)]
    pub filter: Option<String>,

    /// Show output / 显示输出
    #[arg(long)]
    pub nocapture: bool,
}
