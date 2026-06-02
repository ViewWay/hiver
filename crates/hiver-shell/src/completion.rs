//! Tab completion support
//! Tab补全支持
//!
//! # Equivalent to Spring Shell / 等价于 Spring Shell
//! Provides command name completion, similar to Spring Shell's Completer.

use std::collections::HashSet;

use crate::command::CommandRegistry;

/// Completion provider for the shell
/// Shell的补全提供者
pub struct CompletionProvider {
    /// Reference to command names for completion / 用于补全的命令名引用
    command_names: Vec<String>,
}

impl CompletionProvider {
    /// Create a new completion provider from the registry
    /// 从注册表创建新的补全提供者
    pub fn new(registry: &CommandRegistry) -> Self {
        let names = registry.all_commands();
        let mut command_names: Vec<String> = names.iter().map(|m| m.name.clone()).collect();
        // Also include aliases / 也包括别名
        for meta in &names {
            command_names.extend(meta.aliases.iter().cloned());
        }
        // Deduplicate / 去重
        let set: HashSet<String> = command_names.into_iter().collect();
        let mut unique: Vec<String> = set.into_iter().collect();
        unique.sort();
        Self {
            command_names: unique,
        }
    }

    /// Get completions for a partial input
    /// 获取部分输入的补全
    pub fn complete(&self, input: &str) -> Vec<String> {
        if input.is_empty() {
            return self.command_names.clone();
        }

        let lower = input.to_lowercase();
        self.command_names
            .iter()
            .filter(|name| name.to_lowercase().starts_with(&lower))
            .cloned()
            .collect()
    }

    /// Complete the first word of a line
    /// 补全行的第一个单词
    pub fn complete_line(&self, line: &str) -> Vec<String> {
        let trimmed = line.trim_start();
        // Only complete the first word (command name)
        // 只补全第一个单词（命令名）
        if (trimmed.contains(' ') || trimmed.is_empty() && !line.contains(' '))
            && let Some(_space_pos) = trimmed.find(' ')
        {
            // Already typed a command and space — no subcommand completion
            return Vec::new();
        }

        let prefix = trimmed.split_whitespace().next().unwrap_or("");
        self.complete(prefix)
    }

    /// Get all command names / 获取所有命令名
    pub fn command_names(&self) -> &[String] {
        &self.command_names
    }

    /// Refresh from registry / 从注册表刷新
    pub fn refresh(&mut self, registry: &CommandRegistry) {
        *self = Self::new(registry);
    }
}

/// A rustyline-compatible completer
/// 兼容rustyline的补全器
pub struct ShellCompleter {
    /// The completion provider / 补全提供者
    provider: std::sync::Mutex<CompletionProvider>,
}

impl ShellCompleter {
    /// Create a new shell completer / 创建新的shell补全器
    pub fn new(provider: CompletionProvider) -> Self {
        Self {
            provider: std::sync::Mutex::new(provider),
        }
    }

    /// Get completions for rustyline / 为rustyline获取补全
    pub fn get_completions(&self, input: &str) -> rustyline::Result<Vec<String>> {
        let provider = self
            .provider
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(provider.complete_line(input))
    }
}

impl rustyline::completion::Completer for ShellCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<String>)> {
        let input = &line[..pos];
        let provider = self
            .provider
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let completions = provider.complete_line(input);
        let word_start = input.rfind(' ').map_or(0, |p| p + 1);
        Ok((word_start, completions))
    }
}

// No-op implementations required by rustyline::Helper
impl rustyline::hint::Hinter for ShellCompleter {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl rustyline::highlight::Highlighter for ShellCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Borrowed(line)
    }
    #[allow(single_use_lifetimes)]
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        std::borrow::Cow::Borrowed(prompt)
    }
}

impl rustyline::validate::Validator for ShellCompleter {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl rustyline::Helper for ShellCompleter {}
