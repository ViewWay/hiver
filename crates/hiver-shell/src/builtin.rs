//! Built-in shell commands
//! 内置shell命令
//!
//! # Equivalent to Spring Shell / 等价于 Spring Shell
//!
//! - `help` — list available commands (equivalent to `help`)
//! - `clear` — clear the screen (equivalent to `clear`)
//! - `exit` / `quit` — exit the shell (equivalent to `exit`)
//! - `stacktrace` — show last error stacktrace
//! - `script` — execute commands from a file
//! - `echo` — echo text
//! - `history` — show command history

use std::backtrace::Backtrace;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use colored::Colorize;

use crate::command::{Command, CommandMeta};
use crate::result::{ShellError, ShellResult};

/// Maximum history entries to keep
/// 保留的最大历史记录数
const MAX_HISTORY: usize = 100;

/// Shared state for builtins
/// 内置命令的共享状态
#[derive(Debug)]
#[derive(Default)]
pub struct BuiltinState {
    /// Command history / 命令历史
    pub history: VecDeque<String>,
    /// Last error stacktrace / 最后一个错误的堆栈跟踪
    pub last_error: Option<String>,
}


impl BuiltinState {
    /// Create new state / 创建新状态
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a history entry / 添加历史记录条目
    pub fn add_history(&mut self, line: &str) {
        if !line.trim().is_empty() {
            self.history.push_back(line.to_string());
            while self.history.len() > MAX_HISTORY {
                self.history.pop_front();
            }
        }
    }

    /// Record an error / 记录错误
    pub fn record_error(&mut self, error: &ShellError) {
        let backtrace = Backtrace::capture();
        self.last_error = Some(format!("Error: {error}\n{backtrace}"));
    }
}

// ============================================================================
// Help Command / 帮助命令
// ============================================================================

/// Help command — displays available commands
/// 帮助命令 — 显示可用命令
///
/// # Equivalent to Spring Shell / 等价于 Spring Shell
/// Equivalent to `help` command in Spring Shell.
pub struct HelpCommand {
    /// Shared state / 共享状态
    state: Arc<Mutex<BuiltinState>>,
}

impl HelpCommand {
    /// Create a new help command / 创建新的帮助命令
    pub fn new(state: Arc<Mutex<BuiltinState>>) -> Self {
        Self { state }
    }
}

impl Command for HelpCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new("help")
            .description("Show available commands / 显示可用命令")
            .aliases(&["h", "?"])
            .group("Built-in")
            .parameter(crate::command::ParameterMeta::optional(
                "command",
                "Show help for a specific command / 显示特定命令的帮助",
            ))
    }

    fn execute(&self, args: &[&str]) -> ShellResult<String> {
        // Help without arguments — show general help
        if args.is_empty() {
            let output = format!(
                "{}\n\n{}\n{}\n{}\n{}\n",
                "Available Commands / 可用命令:".green().bold(),
                "  help          Show available commands / 显示可用命令",
                "  help <cmd>    Show help for a command / 显示命令帮助",
                "  exit          Exit the shell / 退出shell",
                "Type a command and press Enter to execute."
            );
            return Ok(output);
        }

        // Help for a specific command
        #[allow(clippy::indexing_slicing)]
        let cmd_name = args[0];
        let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let msg = if state.history.is_empty() {
            format!("No additional help available for: {cmd_name}")
        } else {
            format!("Help for '{cmd_name}': Use '{cmd_name}' with appropriate arguments.")
        };
        Ok(msg)
    }
}

// ============================================================================
// Clear Command / 清屏命令
// ============================================================================

/// Clear command — clears the terminal screen
/// 清屏命令 — 清除终端屏幕
pub struct ClearCommand;

impl Command for ClearCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new("clear")
            .description("Clear the terminal screen / 清除终端屏幕")
            .aliases(&["cls"])
            .group("Built-in")
    }

    fn execute(&self, _args: &[&str]) -> ShellResult<String> {
        // Return ANSI escape sequence to clear screen
        // 返回ANSI转义序列以清屏
        Ok("\x1b[2J\x1b[H".to_string())
    }
}

// ============================================================================
// Exit Command / 退出命令
// ============================================================================

/// Exit command — exits the shell
/// 退出命令 — 退出shell
pub struct ExitCommand;

impl Command for ExitCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new("exit")
            .description("Exit the shell / 退出shell")
            .aliases(&["quit", "q"])
            .group("Built-in")
    }

    fn execute(&self, _args: &[&str]) -> ShellResult<String> {
        Err(ShellError::ExitRequested)
    }
}

// ============================================================================
// Stacktrace Command / 堆栈跟踪命令
// ============================================================================

/// Stacktrace command — shows the last error stacktrace
/// 堆栈跟踪命令 — 显示最后一个错误的堆栈跟踪
pub struct StacktraceCommand {
    /// Shared state / 共享状态
    state: Arc<Mutex<BuiltinState>>,
}

impl StacktraceCommand {
    /// Create a new stacktrace command / 创建新的堆栈跟踪命令
    pub fn new(state: Arc<Mutex<BuiltinState>>) -> Self {
        Self { state }
    }
}

impl Command for StacktraceCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new("stacktrace")
            .description("Show last error stacktrace / 显示最后一次错误的堆栈跟踪")
            .aliases(&["st"])
            .group("Built-in")
    }

    fn execute(&self, _args: &[&str]) -> ShellResult<String> {
        let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        match &state.last_error {
            Some(trace) => Ok(trace.clone()),
            None => Ok("No error recorded / 没有记录的错误".to_string()),
        }
    }
}

// ============================================================================
// Script Command / 脚本命令
// ============================================================================

/// Script command — executes commands from a file
/// 脚本命令 — 从文件执行命令
pub struct ScriptCommand {
    /// Shared state / 共享状态
    state: Arc<Mutex<BuiltinState>>,
}

impl ScriptCommand {
    /// Create a new script command / 创建新的脚本命令
    pub fn new(state: Arc<Mutex<BuiltinState>>) -> Self {
        Self { state }
    }
}

impl Command for ScriptCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new("script")
            .description("Execute commands from a file / 从文件执行命令")
            .group("Built-in")
            .parameter(crate::command::ParameterMeta::required(
                "file",
                "Path to the script file / 脚本文件路径",
            ))
    }

    fn execute(&self, args: &[&str]) -> ShellResult<String> {
        let path = args.first().ok_or_else(|| {
            ShellError::InvalidArguments(
                "Usage: script <file> / 用法: script <文件>".to_string(),
            )
        })?;

        let canonical = std::path::Path::new(path).canonicalize().map_err(|e| {
            ShellError::Script(format!(
                "Invalid script path '{}': {e} / 无法解析脚本路径 '{}': {e}",
                path, path
            ))
        })?;

        let content = std::fs::read_to_string(&canonical).map_err(|e| {
            ShellError::Script(format!(
                "Failed to read script file '{}': {e} / \
                 无法读取脚本文件 '{}': {e}",
                path, path
            ))
        })?;

        let mut results = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Record in history / 记录到历史
            {
                let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                state.add_history(trimmed);
            }

            results.push(format!("Line {}: {}", i + 1, trimmed));
        }

        if results.is_empty() {
            Ok("Script file is empty / 脚本文件为空".to_string())
        } else {
            Ok(format!(
                "Script loaded {} commands from '{}'\n{}",
                results.len(),
                path,
                results.join("\n")
            ))
        }
    }
}

// ============================================================================
// Echo Command / 回显命令
// ============================================================================

/// Echo command — prints text
/// 回显命令 — 打印文本
pub struct EchoCommand;

impl Command for EchoCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new("echo")
            .description("Echo text / 回显文本")
            .group("Built-in")
    }

    fn execute(&self, args: &[&str]) -> ShellResult<String> {
        Ok(args.join(" "))
    }
}

// ============================================================================
// History Command / 历史命令
// ============================================================================

/// History command — shows command history
/// 历史命令 — 显示命令历史
pub struct HistoryCommand {
    /// Shared state / 共享状态
    state: Arc<Mutex<BuiltinState>>,
}

impl HistoryCommand {
    /// Create a new history command / 创建新的历史命令
    pub fn new(state: Arc<Mutex<BuiltinState>>) -> Self {
        Self { state }
    }
}

impl Command for HistoryCommand {
    fn meta(&self) -> CommandMeta {
        CommandMeta::new("history")
            .description("Show command history / 显示命令历史")
            .aliases(&["hist"])
            .group("Built-in")
    }

    fn execute(&self, _args: &[&str]) -> ShellResult<String> {
        let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.history.is_empty() {
            return Ok("No history / 没有历史记录".to_string());
        }

        let output: String = state
            .history
            .iter()
            .enumerate()
            .map(|(i, line)| format!("  {:4}  {}", i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(output)
    }
}

/// Register all built-in commands into the registry
/// 将所有内置命令注册到注册表中
pub fn register_builtins(
    registry: &mut crate::command::CommandRegistry,
    state: &Arc<Mutex<BuiltinState>>,
) {
    registry.register(ClearCommand);
    registry.register(ExitCommand);
    registry.register(EchoCommand);
    registry.register(HelpCommand::new(state.clone()));
    registry.register(StacktraceCommand::new(state.clone()));
    registry.register(ScriptCommand::new(state.clone()));
    registry.register(HistoryCommand::new(state.clone()));
}
