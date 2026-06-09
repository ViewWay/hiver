//! Command trait and registry
//! 命令trait和注册表
//!
//! # Equivalent to Spring Shell / 等价于 Spring Shell
//!
//! - `Command` trait — equivalent to `@ShellMethod` annotated methods
//! - `CommandRegistry` — equivalent to Spring Shell's command resolver
//! - Dynamic dispatch for extensibility

use std::{collections::HashMap, fmt};

use crate::result::ShellResult;

/// Metadata for a command parameter
/// 命令参数的元数据
#[derive(Debug, Clone)]
pub struct ParameterMeta
{
    /// Parameter name / 参数名
    pub name: String,
    /// Parameter description / 参数描述
    pub description: String,
    /// Whether the parameter is required / 参数是否必填
    pub required: bool,
    /// Default value (if any) / 默认值（如果有）
    pub default_value: Option<String>,
}

impl ParameterMeta
{
    /// Create a new required parameter / 创建新的必填参数
    pub fn required(name: &str, description: &str) -> Self
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required: true,
            default_value: None,
        }
    }

    /// Create a new optional parameter / 创建新的可选参数
    pub fn optional(name: &str, description: &str) -> Self
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            default_value: None,
        }
    }

    /// Create a parameter with a default value / 创建带默认值的参数
    pub fn with_default(name: &str, description: &str, default: &str) -> Self
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            default_value: Some(default.to_string()),
        }
    }
}

/// Command metadata
/// 命令元数据
#[derive(Debug, Clone)]
pub struct CommandMeta
{
    /// Primary command name / 主命令名
    pub name: String,
    /// Aliases for the command / 命令别名
    pub aliases: Vec<String>,
    /// Command description / 命令描述
    pub description: String,
    /// Command group / 命令分组
    pub group: String,
    /// Whether the command is hidden / 命令是否隐藏
    pub hidden: bool,
    /// Parameter metadata / 参数元数据
    pub parameters: Vec<ParameterMeta>,
}

impl CommandMeta
{
    /// Create new command metadata / 创建新的命令元数据
    pub fn new(name: &str) -> Self
    {
        Self {
            name: name.to_string(),
            aliases: Vec::new(),
            description: String::new(),
            group: "General".to_string(),
            hidden: false,
            parameters: Vec::new(),
        }
    }

    /// Set description / 设置描述
    pub fn description(mut self, desc: &str) -> Self
    {
        self.description = desc.to_string();
        self
    }

    /// Set group / 设置分组
    pub fn group(mut self, group: &str) -> Self
    {
        self.group = group.to_string();
        self
    }

    /// Set aliases / 设置别名
    pub fn aliases(mut self, aliases: &[&str]) -> Self
    {
        self.aliases = aliases.iter().map(ToString::to_string).collect();
        self
    }

    /// Set hidden / 设置隐藏
    pub fn hidden(mut self, hidden: bool) -> Self
    {
        self.hidden = hidden;
        self
    }

    /// Add a parameter / 添加参数
    pub fn parameter(mut self, param: ParameterMeta) -> Self
    {
        self.parameters.push(param);
        self
    }

    /// Check if a name matches this command (primary name or alias)
    /// 检查名称是否匹配此命令（主名或别名）
    pub fn matches(&self, name: &str) -> bool
    {
        self.name == name || self.aliases.iter().any(|a| a == name)
    }
}

impl fmt::Display for CommandMeta
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.name)?;
        if !self.aliases.is_empty()
        {
            write!(f, " ({})", self.aliases.join(", "))?;
        }
        if !self.description.is_empty()
        {
            write!(f, " - {}", self.description)?;
        }
        Ok(())
    }
}

/// Core command trait
/// 核心命令trait
///
/// # Equivalent to Spring Shell / 等价于 Spring Shell
/// Equivalent to `@ShellMethod` annotated methods in Spring Shell.
/// 等价于Spring Shell中用`@ShellMethod`注解的方法。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_shell::command::{Command, CommandMeta};
/// use hiver_shell::result::ShellResult;
///
/// struct GreetCommand;
///
/// impl Command for GreetCommand {
///     fn meta(&self) -> CommandMeta {
///         CommandMeta::new("greet")
///             .description("Greet someone / 问候某人")
///     }
///
///     fn execute(&self, args: &[&str]) -> ShellResult<String> {
///         let name = args.first().copied().unwrap_or("World");
///         Ok(format!("Hello, {}!", name))
///     }
/// }
/// ```
pub trait Command: Send + Sync
{
    /// Get command metadata / 获取命令元数据
    fn meta(&self) -> CommandMeta;

    /// Execute the command / 执行命令
    /// 执行命令
    ///
    /// # Arguments / 参数
    /// - `args` — command arguments (excluding command name)
    /// - `args` — 命令参数（不包含命令名）
    fn execute(&self, args: &[&str]) -> ShellResult<String>;

    /// Get command name (convenience method) / 获取命令名（便捷方法）
    fn name(&self) -> String
    {
        self.meta().name
    }

    /// Get command description / 获取命令描述
    fn description(&self) -> String
    {
        self.meta().description
    }
}

/// Type-erased command box
/// 类型擦除的命令盒子
pub type CommandBox = Box<dyn Command>;

/// Command registry — manages all registered commands
/// 命令注册表 — 管理所有已注册的命令
///
/// # Equivalent to Spring Shell / 等价于 Spring Shell
/// Equivalent to Spring Shell's command resolver and registry.
/// 等价于Spring Shell的命令解析器和注册表。
#[derive(Default)]
pub struct CommandRegistry
{
    /// Commands indexed by primary name / 以主名索引的命令
    commands: HashMap<String, CommandBox>,
    /// Alias to primary name mapping / 别名到主名的映射
    aliases: HashMap<String, String>,
}

impl CommandRegistry
{
    /// Create a new empty registry / 创建新的空注册表
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Register a command / 注册命令
    pub fn register<C: Command + 'static>(&mut self, command: C)
    {
        let meta = command.meta();
        let primary = meta.name.clone();

        // Register aliases / 注册别名
        for alias in &meta.aliases
        {
            self.aliases.insert(alias.clone(), primary.clone());
        }

        self.commands.insert(primary, Box::new(command));
    }

    /// Register a boxed command / 注册已装箱的命令
    pub fn register_boxed(&mut self, command: CommandBox)
    {
        let meta = command.meta();
        let primary = meta.name.clone();

        for alias in &meta.aliases
        {
            self.aliases.insert(alias.clone(), primary.clone());
        }

        self.commands.insert(primary, command);
    }

    /// Look up a command by name or alias / 通过名称或别名查找命令
    pub fn get(&self, name: &str) -> Option<&dyn Command>
    {
        // Try primary name first, then aliases
        // 先尝试主名，然后尝试别名
        if let Some(cmd) = self.commands.get(name)
        {
            return Some(cmd.as_ref());
        }
        if let Some(primary) = self.aliases.get(name)
            && let Some(cmd) = self.commands.get(primary)
        {
            return Some(cmd.as_ref());
        }
        None
    }

    /// Get all command metadata / 获取所有命令元数据
    pub fn all_commands(&self) -> Vec<CommandMeta>
    {
        let mut metas: Vec<CommandMeta> = self
            .commands
            .values()
            .map(|c| c.meta())
            .filter(|m| !m.hidden)
            .collect();
        metas.sort_by(|a, b| a.name.cmp(&b.name));
        metas
    }

    /// Get all command metadata including hidden / 获取所有命令元数据（含隐藏）
    pub fn all_commands_including_hidden(&self) -> Vec<CommandMeta>
    {
        let mut metas: Vec<CommandMeta> = self.commands.values().map(|c| c.meta()).collect();
        metas.sort_by(|a, b| a.name.cmp(&b.name));
        metas
    }

    /// List all command names / 列出所有命令名
    pub fn command_names(&self) -> Vec<&str>
    {
        let mut names: Vec<&str> = self.commands.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Check if a command exists / 检查命令是否存在
    pub fn contains(&self, name: &str) -> bool
    {
        self.commands.contains_key(name) || self.aliases.contains_key(name)
    }

    /// Get the number of registered commands / 获取已注册命令的数量
    pub fn len(&self) -> usize
    {
        self.commands.len()
    }

    /// Check if the registry is empty / 检查注册表是否为空
    pub fn is_empty(&self) -> bool
    {
        self.commands.is_empty()
    }

    /// Execute a command line string / 执行命令行字符串
    ///
    /// Parses the input, looks up the command, and executes it.
    /// 解析输入，查找命令并执行。
    pub fn execute_line(&self, line: &str) -> ShellResult<String>
    {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty()
        {
            return Ok(String::new());
        }

        #[allow(clippy::indexing_slicing)]
        let cmd_name = parts[0];
        #[allow(clippy::indexing_slicing)]
        let args = &parts[1..];

        match self.get(cmd_name)
        {
            Some(cmd) => cmd.execute(args),
            None => Err(crate::result::ShellError::CommandNotFound(cmd_name.to_string())),
        }
    }
}

impl fmt::Debug for CommandRegistry
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("CommandRegistry")
            .field("command_count", &self.commands.len())
            .field("alias_count", &self.aliases.len())
            .finish()
    }
}

/// Macro-based command registration helper
/// 基于宏的命令注册辅助
///
/// Use the `#[shell_method]` attribute macro from `hiver-shell-macros`
/// for automatic command registration.
/// 使用`hiver-shell-macros`中的`#[shell_method]`属性宏进行自动命令注册。
#[macro_export]
macro_rules! shell_command {
    ($name:expr, $desc:expr, $handler:expr) => {{
        struct DynCommand;
        impl $crate::command::Command for DynCommand
        {
            fn meta(&self) -> $crate::command::CommandMeta
            {
                $crate::command::CommandMeta::new($name).description($desc)
            }

            fn execute(&self, args: &[&str]) -> $crate::result::ShellResult<String>
            {
                $handler(args)
            }
        }
        Box::new(DynCommand) as $crate::command::CommandBox
    }};
}
