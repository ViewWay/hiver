//! Shell result types and output formatting
//! Shell结果类型和输出格式化
//!
//! # Equivalent to Spring Shell / 等价于 Spring Shell
//!
//! - `ResultHandler` — formats command output
//! - `TableResult` — tabular output like Spring Shell Table
//! - `JsonResult` — JSON formatted output

use std::fmt;

use colored::Colorize;
use serde::Serialize;

/// Shell result type alias
/// Shell结果类型别名
pub type ShellResult<T = ()> = Result<T, ShellError>;

/// Shell error types
/// Shell错误类型
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    /// Command not found / 命令未找到
    #[error("Command not found: {0} / 未找到命令: {0}")]
    CommandNotFound(String),

    /// Invalid arguments / 无效参数
    #[error("Invalid arguments: {0} / 无效参数: {0}")]
    InvalidArguments(String),

    /// IO error / IO错误
    #[error("IO error: {0} / IO错误: {0}")]
    Io(#[from] std::io::Error),

    /// Runtime error / 运行时错误
    #[error("Runtime error: {0} / 运行时错误: {0}")]
    Runtime(String),

    /// Script execution error / 脚本执行错误
    #[error("Script error: {0} / 脚本错误: {0}")]
    Script(String),

    /// Validation error / 验证错误
    #[error("Validation error: {0} / 验证错误: {0}")]
    Validation(String),

    /// Exit requested / 请求退出
    #[error("Exit requested / 请求退出")]
    ExitRequested,
}

/// Output format for command results
/// 命令结果的输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Plain text / 纯文本
    #[default]
    Plain,
    /// JSON format / JSON格式
    Json,
    /// Table format / 表格格式
    Table,
}

/// A command result that can be rendered in different formats
/// 可以以不同格式渲染的命令结果
pub trait ShellOutput: fmt::Display + fmt::Debug {
    /// Render as plain text / 渲染为纯文本
    fn render_plain(&self) -> String {
        format!("{self}")
    }

    /// Render as JSON / 渲染为JSON
    fn render_json(&self) -> String;

    /// Render as table / 渲染为表格
    fn render_table(&self) -> String {
        self.render_plain()
    }
}

/// Plain text result
/// 纯文本结果
#[derive(Debug, Clone)]
pub struct TextResult {
    /// The text content / 文本内容
    pub text: String,
}

impl TextResult {
    /// Create a new text result / 创建新的文本结果
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl fmt::Display for TextResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl ShellOutput for TextResult {
    fn render_json(&self) -> String {
        #[derive(Serialize)]
        struct TextOutput {
            result: String,
        }
        serde_json::to_string_pretty(&TextOutput {
            result: self.text.clone(),
        })
        .unwrap_or_else(|_| format!("{{\"result\": {:?}}}", self.text))
    }

    fn render_table(&self) -> String {
        self.text.clone()
    }
}

/// Table result for structured tabular data
/// 结构化表格数据的表格结果
///
/// # Equivalent to Spring Shell / 等价于 Spring Shell
/// Equivalent to `org.springframework.shell.table.Table`
#[derive(Debug, Clone)]
pub struct TableResult {
    /// Column headers / 列标题
    pub headers: Vec<String>,
    /// Row data / 行数据
    pub rows: Vec<Vec<String>>,
}

impl TableResult {
    /// Create a new table with headers / 创建带标题的新表格
    pub fn new(headers: Vec<&str>) -> Self {
        Self {
            headers: headers.iter().map(std::string::ToString::to_string).collect(),
            rows: Vec::new(),
        }
    }

    /// Add a row / 添加行
    pub fn row(mut self, values: Vec<&str>) -> Self {
        self.rows.push(values.iter().map(std::string::ToString::to_string).collect());
        self
    }

    /// Add a row from owned strings / 从拥有的字符串添加行
    pub fn row_owned(mut self, values: Vec<String>) -> Self {
        self.rows.push(values);
        self
    }
}

impl fmt::Display for TableResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_table())
    }
}

impl ShellOutput for TableResult {
    fn render_plain(&self) -> String {
        self.render_table()
    }

    fn render_json(&self) -> String {
        use serde_json::Value;
        let mut data = Vec::new();
        for row in &self.rows {
            let mut map = serde_json::Map::new();
            for (i, header) in self.headers.iter().enumerate() {
                let val = row.get(i).cloned().unwrap_or_default();
                map.insert(header.clone(), Value::String(val));
            }
            data.push(Value::Object(map));
        }
        serde_json::to_string_pretty(&data).unwrap_or_else(|_| "[]".to_string())
    }

    fn render_table(&self) -> String {
        if self.headers.is_empty() {
            return String::new();
        }

        // Calculate column widths / 计算列宽
        let mut widths: Vec<usize> = self.headers.iter().map(std::string::String::len).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        let mut output = String::new();

        // Header line / 标题行
        let header_line: String = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let padded = format!("{:<width$}", h, width = widths[i]);
                padded.bold().to_string()
            })
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&header_line);
        output.push('\n');

        // Separator line / 分隔线
        let separator: String = widths
            .iter()
            .map(|&w| "-".repeat(w))
            .collect::<Vec<_>>()
            .join("-+-");
        output.push_str(&separator);
        output.push('\n');

        // Data rows / 数据行
        for row in &self.rows {
            let line: String = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let w = widths.get(i).copied().unwrap_or(0);
                    format!("{:<width$}", cell, width = w)
                })
                .collect::<Vec<_>>()
                .join(" | ");
            output.push_str(&line);
            output.push('\n');
        }

        output
    }
}

/// JSON result wrapper
/// JSON结果包装器
#[derive(Debug, Clone)]
pub struct JsonResult {
    /// The JSON value / JSON值
    pub value: serde_json::Value,
}

impl JsonResult {
    /// Create from any serializable value / 从任何可序列化值创建
    pub fn new(value: &impl Serialize) -> Self {
        Self {
            value: serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Create from raw JSON value / 从原始JSON值创建
    pub fn from_value(value: serde_json::Value) -> Self {
        Self { value }
    }

    /// Create from a JSON string / 从JSON字符串创建
    pub fn from_str(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            value: serde_json::from_str(json)?,
        })
    }
}

impl fmt::Display for JsonResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_json())
    }
}

impl ShellOutput for JsonResult {
    fn render_plain(&self) -> String {
        self.render_json()
    }

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.value).unwrap_or_else(|_| "null".to_string())
    }

    fn render_table(&self) -> String {
        self.render_json()
    }
}

/// Result handler for formatting command output
/// 命令输出格式化的结果处理器
///
/// # Equivalent to Spring Shell / 等价于 Spring Shell
/// Equivalent to `org.springframework.shell.result.ResultHandler`
#[derive(Debug, Default)]
pub struct ResultHandler {
    /// Current output format / 当前输出格式
    pub format: OutputFormat,
}

impl ResultHandler {
    /// Create a new result handler / 创建新的结果处理器
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a specific format / 使用指定格式创建
    pub fn with_format(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Handle a shell output / 处理shell输出
    pub fn handle(&self, output: &dyn ShellOutput) -> String {
        match self.format {
            OutputFormat::Plain => output.render_plain(),
            OutputFormat::Json => output.render_json(),
            OutputFormat::Table => output.render_table(),
        }
    }

    /// Handle a simple string result / 处理简单字符串结果
    pub fn handle_str(&self, text: &str) -> String {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::json!({ "result": text });
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| text.to_string())
            }
            _ => text.to_string(),
        }
    }

    /// Handle an error / 处理错误
    pub fn handle_error(&self, error: &ShellError) -> String {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "error": true,
                    "message": format!("{error}")
                });
                serde_json::to_string_pretty(&json)
                    .unwrap_or_else(|_| format!("{{\"error\": true, \"message\": \"{error}\"}}"))
            }
            _ => format!("{} {}", "ERROR:".red().bold(), error),
        }
    }
}
