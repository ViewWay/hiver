//! Prompt styling and customization
//! 提示符样式和自定义
//!
//! # Equivalent to Spring Shell / 等价于 Spring Shell
//! Customizable prompt with colors, similar to Spring Shell's `PromptProvider`.

use colored::Colorize;

/// Prompt style configuration
/// 提示符样式配置
#[derive(Debug, Clone)]
pub struct PromptStyle {
    /// Prompt prefix / 提示符前缀
    pub prefix: String,
    /// Prompt suffix / 提示符后缀
    pub suffix: String,
    /// Color for the prompt / 提示符颜色
    pub color: PromptColor,
    /// Whether to show a timestamp / 是否显示时间戳
    pub show_timestamp: bool,
    /// Application name / 应用名称
    pub app_name: String,
}

impl Default for PromptStyle {
    fn default() -> Self {
        Self {
            prefix: "nexus".to_string(),
            suffix: "> ".to_string(),
            color: PromptColor::Cyan,
            show_timestamp: false,
            app_name: "nexus".to_string(),
        }
    }
}

impl PromptStyle {
    /// Create a new prompt style / 创建新的提示符样式
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the prefix / 设置前缀
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// Set the suffix / 设置后缀
    pub fn suffix(mut self, suffix: &str) -> Self {
        self.suffix = suffix.to_string();
        self
    }

    /// Set the color / 设置颜色
    pub fn color(mut self, color: PromptColor) -> Self {
        self.color = color;
        self
    }

    /// Set the application name / 设置应用名称
    pub fn app_name(mut self, name: &str) -> Self {
        self.app_name = name.to_string();
        self.prefix = name.to_string();
        self
    }

    /// Show timestamp / 显示时间戳
    pub fn show_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    /// Render the prompt string / 渲染提示符字符串
    pub fn render(&self) -> String {
        let base = if self.show_timestamp {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{} [{:.0}]", self.prefix, ts.as_secs())
        } else {
            self.prefix.clone()
        };

        let colored_base = match self.color {
            PromptColor::Cyan => base.cyan().bold().to_string(),
            PromptColor::Green => base.green().bold().to_string(),
            PromptColor::Yellow => base.yellow().bold().to_string(),
            PromptColor::Blue => base.blue().bold().to_string(),
            PromptColor::Magenta => base.magenta().bold().to_string(),
            PromptColor::Red => base.red().bold().to_string(),
            PromptColor::White => base.white().bold().to_string(),
            PromptColor::Default => base.bold().to_string(),
        };

        format!("{}{}", colored_base, self.suffix)
    }
}

/// Prompt color options
/// 提示符颜色选项
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromptColor {
    /// Cyan (default) / 青色（默认）
    #[default]
    Cyan,
    /// Green / 绿色
    Green,
    /// Yellow / 黄色
    Yellow,
    /// Blue / 蓝色
    Blue,
    /// Magenta / 品红
    Magenta,
    /// Red / 红色
    Red,
    /// White / 白色
    White,
    /// Default terminal color / 默认终端颜色
    Default,
}

/// Banner displayed at shell startup
/// Shell启动时显示的横幅
#[derive(Debug, Clone)]
pub struct Banner {
    /// Banner lines / 横幅行
    pub lines: Vec<String>,
    /// Whether to show the banner / 是否显示横幅
    pub enabled: bool,
}

impl Default for Banner {
    fn default() -> Self {
        Self {
            lines: vec![
                format!(
                    "{}",
                    "  _   _                     ".bright_black()
                ),
                format!(
                    "{}",
                    " | \\ | |_   _ _ __   ___ _ __ ".bright_black()
                ),
                format!(
                    "{}",
                    " |  \\| | | | | '_ \\ / _ \\ '__|".bright_black()
                ),
                format!(
                    "{}",
                    " | |\\  | |_| | | | |  __/ |   ".bright_black()
                ),
                format!(
                    "{}",
                    " |_| \\_|\\__, |_| |_|\\___|_|   ".bright_black()
                ),
                format!(
                    "{}",
                    "       |___/                  ".bright_black()
                ),
            ],
            enabled: true,
        }
    }
}

impl Banner {
    /// Create a new banner / 创建新横幅
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a custom banner / 创建自定义横幅
    pub fn custom(lines: Vec<&str>) -> Self {
        Self {
            lines: lines.iter().map(std::string::ToString::to_string).collect(),
            enabled: true,
        }
    }

    /// Disable the banner / 禁用横幅
    pub fn disabled() -> Self {
        Self {
            lines: Vec::new(),
            enabled: false,
        }
    }

    /// Set enabled / 设置启用
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Render the banner / 渲染横幅
    pub fn render(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let mut output = self.lines.join("\n");
        output.push('\n');
        output.push_str(&format!(
            "  {} {}\n",
            "Nexus Shell".cyan().bold(),
            "v0.1.0-alpha".dimmed()
        ));
        output.push_str(&format!(
            "  {} {}\n",
            "Type".dimmed(),
            "'help' for available commands.".dimmed()
        ));
        output
    }
}
