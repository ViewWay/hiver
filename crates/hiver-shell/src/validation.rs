//! Input validation
//! 输入验证
//!
//! # Equivalent to Spring Shell / 等价于 Spring Shell
//! Input validation and sanitization for shell commands.

use crate::result::{ShellError, ShellResult};

/// Input validator for shell commands
/// Shell命令的输入验证器
#[derive(Debug, Default)]
pub struct InputValidator {
    /// Maximum input length / 最大输入长度
    pub max_length: usize,
    /// Whether to allow empty input / 是否允许空输入
    pub allow_empty: bool,
}

impl InputValidator {
    /// Create a new validator / 创建新的验证器
    pub fn new() -> Self {
        Self {
            max_length: 4096,
            allow_empty: true,
        }
    }

    /// Set maximum length / 设置最大长度
    pub fn max_length(mut self, len: usize) -> Self {
        self.max_length = len;
        self
    }

    /// Set whether empty input is allowed / 设置是否允许空输入
    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// Validate input / 验证输入
    pub fn validate(&self, input: &str) -> ShellResult<ValidatedInput> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            if self.allow_empty {
                return Ok(ValidatedInput {
                    raw: input.to_string(),
                    command: String::new(),
                    args: Vec::new(),
                });
            }
            return Err(ShellError::Validation(
                "Input cannot be empty / 输入不能为空".to_string(),
            ));
        }

        if trimmed.len() > self.max_length {
            return Err(ShellError::Validation(format!(
                "Input exceeds maximum length of {} / 输入超过最大长度 {}",
                self.max_length, self.max_length
            )));
        }

        // Check for null bytes / 检查空字节
        if trimmed.contains('\0') {
            return Err(ShellError::Validation(
                "Input contains null bytes / 输入包含空字节".to_string(),
            ));
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        #[allow(clippy::indexing_slicing)]
        let command = parts[0].to_string();
        #[allow(clippy::indexing_slicing)]
        let args: Vec<String> = parts[1..].iter().map(|s| (*s).to_string()).collect();

        // Validate command name / 验证命令名
        validate_command_name(&command)?;

        Ok(ValidatedInput {
            raw: input.to_string(),
            command,
            args,
        })
    }
}

/// Validate a command name / 验证命令名
pub fn validate_command_name(name: &str) -> ShellResult<()> {
    if name.is_empty() {
        return Err(ShellError::Validation(
            "Command name cannot be empty / 命令名不能为空".to_string(),
        ));
    }

    // Command names should start with a letter or underscore
    // 命令名应该以字母或下划线开头
    if !name
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_')
    {
        return Err(ShellError::Validation(format!(
            "Command name must start with a letter or underscore: {name} / \
             命令名必须以字母或下划线开头: {name}"
        )));
    }

    // Command names should only contain alphanumeric, hyphens, underscores, colons
    // 命令名只应包含字母数字、连字符、下划线、冒号
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
    {
        return Err(ShellError::Validation(format!(
            "Command name contains invalid characters: {name} / \
             命令名包含无效字符: {name}"
        )));
    }

    Ok(())
}

/// Validated and parsed input
/// 已验证和解析的输入
#[derive(Debug, Clone)]
pub struct ValidatedInput {
    /// Raw input string / 原始输入字符串
    pub raw: String,
    /// Parsed command name / 解析的命令名
    pub command: String,
    /// Parsed arguments / 解析的参数
    pub args: Vec<String>,
}

impl ValidatedInput {
    /// Get args as string slices / 获取参数作为字符串切片
    pub fn args_slices(&self) -> Vec<&str> {
        self.args.iter().map(String::as_str).collect()
    }

    /// Check if this is empty input / 检查是否为空输入
    pub fn is_empty(&self) -> bool {
        self.command.is_empty()
    }
}
