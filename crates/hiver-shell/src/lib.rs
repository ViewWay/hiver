#![allow(clippy::indexing_slicing)]
//! hiver-shell — Spring Shell Equivalent: Interactive CLI REPL Framework
//! hiver-shell — 等价于Spring Shell：交互式CLI REPL框架
//!
//! # Overview / 概述
//!
//! `hiver-shell` provides an interactive CLI REPL (Read-Eval-Print Loop) framework,
//! equivalent to Spring Shell in the Spring ecosystem.
//!
//! `hiver-shell` 提供交互式CLI REPL（读取-求值-打印循环）框架，
//! `等价于Spring生态系统中的Spring` Shell。
//!
//! # Features / 功能
//!
//! - **REPL Loop** — Interactive command-line with history and tab completion **REPL循环** —
//!   带历史记录和Tab补全的交互式命令行
//! - **Command Registry** — Dynamic command registration with trait-based dispatch **命令注册表** —
//!   基于trait的动态命令注册
//! - **Built-in Commands** — help, clear, exit, stacktrace, script, echo, history **内置命令** —
//!   help、clear、exit、stacktrace、script、echo、history
//! - **Custom Prompts** — Colored, configurable prompts **自定义提示符** — 彩色、可配置的提示符
//! - **Input Validation** — Command input validation and sanitization **输入验证** —
//!   命令输入验证和清理
//! - **Multiple Output Formats** — Plain text, JSON, Table **多种输出格式** — 纯文本、JSON、表格
//! - **Proc Macros** — `#[shell_component]` and `#[shell_method]` for easy command definition
//!   **过程宏** — `#[shell_component]`和`#[shell_method]`用于轻松定义命令
//!
//! # Quick Start / 快速开始
//!
//! ```rust,no_run,ignore
//! use hiver_shell::{ShellBuilder, ShellConfig};
//! use hiver_shell::command::{Command, CommandMeta};
//! use hiver_shell::result::ShellResult;
//!
//! // Define a custom command / 定义自定义命令
//! struct GreetCommand;
//!
//! impl Command for GreetCommand {
//!     fn meta(&self) -> CommandMeta {
//!         CommandMeta::new("greet")
//!             .description("Greet someone / 问候某人")
//!     }
//!
//!     fn execute(&self, args: &[&str]) -> ShellResult<String> {
//!         let name = args.first().copied().unwrap_or("World");
//!         Ok(format!("Hello, {}!", name))
//!     }
//! }
//!
//! // Build and run the shell / 构建并运行shell
//! fn main() -> hiver_shell::result::ShellResult<()> {
//!     let mut shell = ShellBuilder::new()
//!         .app_name("my-app")
//!         .register(GreetCommand)
//!         .build();
//!     shell.run()
//! }
//! ```
//!
//! # Using Proc Macros / 使用过程宏
//!
//! ```rust,no_run,ignore
//! use hiver_shell_macros::shell_component;
//! use hiver_shell::command::{Command, CommandMeta};
//! use hiver_shell::result::ShellResult;
//!
//! #[shell_component]
//! struct MyCommands;
//!
//! impl MyCommands {
//!     #[shell_method("greet", "Greet someone")]
//!     fn greet(&self, args: &[&str]) -> ShellResult<String> {
//!         Ok(format!("Hello, {}!", args.first().unwrap_or(&"World")))
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;

pub mod builtin;
pub mod command;
pub mod completion;
pub mod prompt;
pub mod repl;
pub mod result;
pub mod validation;

// Re-exports for convenience / 便捷的重新导出
use std::sync::{Arc, Mutex};

pub use command::{Command, CommandBox, CommandMeta, CommandRegistry, ParameterMeta};
pub use prompt::{Banner, PromptColor, PromptStyle};
pub use repl::{Repl, ReplConfig};
pub use result::{
    JsonResult, OutputFormat, ResultHandler, ShellError, ShellOutput, ShellResult, TableResult,
    TextResult,
};
pub use validation::{InputValidator, ValidatedInput};

/// Shell configuration
/// Shell配置
///
/// Controls the behavior and appearance of the shell.
/// 控制shell的行为和外观。
#[derive(Debug)]
pub struct ShellConfig {
    /// Application name / 应用名称
    pub app_name: String,
    /// Prompt style / 提示符样式
    pub prompt: PromptStyle,
    /// Output format / 输出格式
    pub output_format: OutputFormat,
    /// History file path / 历史文件路径
    pub history_file: Option<String>,
    /// Whether to show the banner / 是否显示横幅
    pub show_banner: bool,
    /// Whether to register built-in commands / 是否注册内置命令
    pub register_builtins: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            app_name: "hiver".to_string(),
            prompt: PromptStyle::new(),
            output_format: OutputFormat::Plain,
            history_file: None,
            show_banner: true,
            register_builtins: true,
        }
    }
}

impl ShellConfig {
    /// Create a new config / 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// Set application name / 设置应用名称
    pub fn app_name(mut self, name: &str) -> Self {
        self.app_name = name.to_string();
        self
    }

    /// Set prompt style / 设置提示符样式
    pub fn prompt(mut self, prompt: PromptStyle) -> Self {
        self.prompt = prompt;
        self
    }

    /// Set output format / 设置输出格式
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Set history file / 设置历史文件
    pub fn history_file(mut self, path: impl Into<String>) -> Self {
        self.history_file = Some(path.into());
        self
    }

    /// Set show banner / 设置显示横幅
    pub fn show_banner(mut self, show: bool) -> Self {
        self.show_banner = show;
        self
    }

    /// Set register builtins / 设置注册内置命令
    pub fn register_builtins(mut self, register: bool) -> Self {
        self.register_builtins = register;
        self
    }
}

/// Builder for creating a Shell instance
/// 创建Shell实例的构建器
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_shell::ShellBuilder;
///
/// let shell = ShellBuilder::new()
///     .app_name("my-app")
///     .history_file("~/.my-app-history")
///     .show_banner(true)
///     .build();
/// ```
pub struct ShellBuilder {
    /// Shell configuration / Shell配置
    config: ShellConfig,
    /// Command registry / 命令注册表
    registry: CommandRegistry,
}

impl ShellBuilder {
    /// Create a new `ShellBuilder` with default configuration
    /// `创建带默认配置的新ShellBuilder`
    pub fn new() -> Self {
        Self {
            config: ShellConfig::default(),
            registry: CommandRegistry::new(),
        }
    }

    /// Set the application name / 设置应用名称
    pub fn app_name(mut self, name: &str) -> Self {
        self.config.app_name = name.to_string();
        self.config.prompt = PromptStyle::new().app_name(name);
        self
    }

    /// Set the prompt style / 设置提示符样式
    pub fn prompt(mut self, prompt: PromptStyle) -> Self {
        self.config.prompt = prompt;
        self
    }

    /// Set the output format / 设置输出格式
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.config.output_format = format;
        self
    }

    /// Set the history file path / 设置历史文件路径
    pub fn history_file(mut self, path: impl Into<String>) -> Self {
        self.config.history_file = Some(path.into());
        self
    }

    /// Set whether to show the banner / 设置是否显示横幅
    pub fn show_banner(mut self, show: bool) -> Self {
        self.config.show_banner = show;
        self
    }

    /// Set whether to register built-in commands / 设置是否注册内置命令
    pub fn register_builtins(mut self, register: bool) -> Self {
        self.config.register_builtins = register;
        self
    }

    /// Register a command / 注册命令
    pub fn register<C: Command + 'static>(mut self, command: C) -> Self {
        self.registry.register(command);
        self
    }

    /// Register a boxed command / 注册已装箱的命令
    pub fn register_boxed(mut self, command: CommandBox) -> Self {
        self.registry.register_boxed(command);
        self
    }

    /// Build the Shell instance / 构建Shell实例
    pub fn build(self) -> Shell {
        let mut registry = self.registry;
        let config = self.config;
        let state = Arc::new(Mutex::new(builtin::BuiltinState::new()));

        if config.register_builtins {
            builtin::register_builtins(&mut registry, &state);
        }

        let repl_config = ReplConfig::new()
            .prompt(config.prompt.clone())
            .output_format(config.output_format)
            .show_banner(config.show_banner);

        let repl_config = if let Some(ref history_file) = config.history_file {
            repl_config.history_file(history_file.as_str())
        } else {
            repl_config
        };

        Shell {
            repl: Repl::with_config(registry, repl_config),
            config,
            state,
        }
    }
}

impl Default for ShellBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ShellBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellBuilder")
            .field("config", &self.config)
            .field("command_count", &self.registry.len())
            .finish()
    }
}

/// The main Shell instance
/// 主Shell实例
///
/// # Equivalent to Spring Shell / 等价于 Spring Shell
/// Equivalent to Spring Shell's `Shell` which manages the lifecycle and
/// execution of shell commands.
/// 等价于Spring Shell管理shell命令生命周期和执行的`Shell`。
pub struct Shell {
    /// The REPL engine / REPL引擎
    repl: Repl,
    /// Shell configuration / Shell配置
    config: ShellConfig,
    /// Shared builtin state / 共享的内置命令状态
    state: Arc<Mutex<builtin::BuiltinState>>,
}

impl Shell {
    /// Run the interactive REPL / 运行交互式REPL
    pub fn run(&mut self) -> ShellResult<()> {
        self.repl.run()
    }

    /// Execute a single command line / 执行单行命令
    pub fn execute(&self, line: &str) -> ShellResult<String> {
        // Record in history / 记录到历史
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.add_history(line);
        }
        let result = self.repl.execute_line(line);
        if let Err(ref e) = result {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.record_error(e);
        }
        result
    }

    /// Execute a script (multiple lines) / 执行脚本（多行）
    pub fn execute_script(&self, script: &str) -> Vec<ShellResult<String>> {
        self.repl.execute_script(script)
    }

    /// Get the shell configuration / 获取Shell配置
    pub fn config(&self) -> &ShellConfig {
        &self.config
    }

    /// Get command registry reference / 获取命令注册表引用
    pub fn registry(&self) -> &CommandRegistry {
        self.repl.registry()
    }

    /// Create a builder for this shell / 为此shell创建构建器
    pub fn builder() -> ShellBuilder {
        ShellBuilder::new()
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shell")
            .field("config", &self.config)
            .field("command_count", &self.repl.registry().len())
            .finish()
    }
}
