//! REPL (Read-Eval-Print Loop) implementation
//! REPL（读取-求值-打印循环）实现
//!
//! # Equivalent to Spring Shell / 等价于 Spring Shell
//! Core REPL loop using rustyline, similar to Spring Shell's `ShellRunner`.


use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

use crate::command::CommandRegistry;
use crate::completion::{CompletionProvider, ShellCompleter};
use crate::prompt::PromptStyle;
use crate::result::{OutputFormat, ResultHandler, ShellError, ShellResult};
use crate::validation::InputValidator;

/// REPL configuration
/// REPL配置
#[derive(Debug)]
pub struct ReplConfig {
    /// Prompt style / 提示符样式
    pub prompt: PromptStyle,
    /// Output format / 输出格式
    pub output_format: OutputFormat,
    /// History file path / 历史文件路径
    pub history_file: Option<String>,
    /// Maximum history entries / 最大历史记录条目
    pub max_history: usize,
    /// Welcome banner / 欢迎横幅
    pub show_banner: bool,
    /// Input validator / 输入验证器
    pub validator: InputValidator,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: PromptStyle::new(),
            output_format: OutputFormat::Plain,
            history_file: None,
            max_history: 100,
            show_banner: true,
            validator: InputValidator::new(),
        }
    }
}

impl ReplConfig {
    /// Create a new REPL config / 创建新的REPL配置
    pub fn new() -> Self {
        Self::default()
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

    /// Set max history / 设置最大历史记录
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Set show banner / 设置显示横幅
    pub fn show_banner(mut self, show: bool) -> Self {
        self.show_banner = show;
        self
    }
}

/// The REPL engine
/// REPL引擎
///
/// # Equivalent to Spring Shell / 等价于 Spring Shell
/// Equivalent to Spring Shell's `ShellRunner` which manages the read-eval-print loop.
/// 等价于Spring Shell管理读取-求值-打印循环的`ShellRunner`。
pub struct Repl {
    /// Command registry / 命令注册表
    registry: CommandRegistry,
    /// REPL configuration / REPL配置
    config: ReplConfig,
    /// Result handler / 结果处理器
    result_handler: ResultHandler,
}

impl Repl {
    /// Create a new REPL / 创建新的REPL
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            config: ReplConfig::new(),
            result_handler: ResultHandler::new(),
        }
    }

    /// Create with configuration / 使用配置创建
    pub fn with_config(registry: CommandRegistry, config: ReplConfig) -> Self {
        let result_handler = ResultHandler::with_format(config.output_format);
        Self {
            registry,
            config,
            result_handler,
        }
    }

    /// Get a reference to the command registry / 获取命令注册表的引用
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Get a mutable reference to the command registry / 获取命令注册表的可变引用
    pub fn registry_mut(&mut self) -> &mut CommandRegistry {
        &mut self.registry
    }

    /// Run the REPL loop / 运行REPL循环
    ///
    /// This is the main entry point for the interactive shell.
    /// 这是交互式shell的主要入口点。
    pub fn run(&mut self) -> ShellResult<()> {
        let completion_provider = CompletionProvider::new(&self.registry);
        let completer = ShellCompleter::new(completion_provider);

        let config = rustyline::Config::builder()
            .max_history_size(self.config.max_history)
            .unwrap_or_default()
            .auto_add_history(true)
            .build();

        let mut editor: Editor<ShellCompleter, DefaultHistory> =
            Editor::with_config(config).map_err(|e| ShellError::Runtime(e.to_string()))?;
        editor.set_helper(Some(completer));

        // Load history / 加载历史
        if let Some(ref history_file) = self.config.history_file {
            let _ = editor.load_history(history_file);
        }

        // Show banner / 显示横幅
        if self.config.show_banner {
            let banner = crate::prompt::Banner::new();
            print!("{}", banner.render());
        }

        let prompt_str = self.config.prompt.render();

        // Main REPL loop / 主REPL循环
        loop {
            let readline = editor.readline(&prompt_str);

            match readline {
                Ok(line) => {
                    let line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }

                    // Validate input / 验证输入
                    let validated = match self.config.validator.validate(&line) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("{}", self.result_handler.handle_error(&e));
                            continue;
                        }
                    };

                    if validated.is_empty() {
                        continue;
                    }

                    // Execute / 执行
                    match self.registry.execute_line(&line) {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(ShellError::ExitRequested) => {
                            println!("{}", "Goodbye! / 再见!".dimmed());
                            // Save history / 保存历史
                            if let Some(ref history_file) = self.config.history_file {
                                let _ = editor.save_history(history_file);
                            }
                            return Ok(());
                        }
                        Err(e) => {
                            eprintln!("{}", self.result_handler.handle_error(&e));
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C — just continue
                    println!("{}", "^C".dimmed());
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D — exit
                    println!("{}", "Goodbye! / 再见!".dimmed());
                    if let Some(ref history_file) = self.config.history_file {
                        let _ = editor.save_history(history_file);
                    }
                    return Ok(());
                }
                Err(e) => {
                    return Err(ShellError::Runtime(format!(
                        "Readline error: {e} / 读取错误: {e}"
                    )));
                }
            }
        }
    }

    /// Execute a single line (non-interactive mode)
    /// 执行单行（非交互模式）
    pub fn execute_line(&self, line: &str) -> ShellResult<String> {
        self.registry.execute_line(line)
    }

    /// Execute a script (multiple lines)
    /// 执行脚本（多行）
    pub fn execute_script(&self, script: &str) -> Vec<ShellResult<String>> {
        script
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#')
            })
            .map(|line| self.registry.execute_line(line))
            .collect()
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Repl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repl")
            .field("command_count", &self.registry.len())
            .field("config", &self.config)
            .finish()
    }
}
