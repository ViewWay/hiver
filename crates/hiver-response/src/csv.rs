//! CSV export module
//! CSV 导出模块
//!
//! # Overview / 概述
//!
//! This module provides CSV (Comma-Separated Values) export capabilities.
//! It generates RFC 4180 compliant CSV output with configurable delimiters,
//! quoting rules, and streaming support.
//!
//! 本模块提供 CSV（逗号分隔值）导出功能。
//! 它生成符合 RFC 4180 的 CSV 输出，支持可配置分隔符、引号规则和流式写入。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - OpenCSV / `CsvMapper`
//! - EasyExcel CSV export
//! - Spring Batch CSV `FlatFileItemWriter`
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_response::csv::{CsvExportConfig, CsvExporter};
//!
//! let config = CsvExportConfig::new()
//!     .header("Name")
//!     .header("Age")
//!     .header("Email");
//!
//! let mut exporter = CsvExporter::new(config);
//! exporter.add_row(vec!["Alice", "30", "alice@example.com"]);
//! exporter.add_row(vec!["Bob", "25", "bob@example.com"]);
//!
//! let csv_string = exporter.to_string().unwrap();
//! ```

use std::{
    io::{self, Write},
    path::Path,
};

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// CSV export error.
/// CSV 导出错误。
#[derive(Debug, thiserror::Error)]
pub enum CsvError
{
    /// I/O error during write.
    /// 写入过程中的 I/O 错误。
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Formatting error.
    /// 格式化错误。
    #[error("Format error: {0}")]
    Format(String),
}

/// Result type for CSV operations.
/// CSV 操作的结果类型。
pub type Result<T> = std::result::Result<T, CsvError>;

// ---------------------------------------------------------------------------
// Export configuration
// ---------------------------------------------------------------------------

/// CSV export configuration.
/// CSV 导出配置。
///
/// Equivalent to Spring's OpenCSV `CSVWriter` settings.
/// 等价于 Spring 的 OpenCSV `CSVWriter` 设置。
#[derive(Debug, Clone)]
pub struct CsvExportConfig
{
    /// Column headers / 列标题
    pub headers: Vec<String>,
    /// Field delimiter (default: comma) / 字段分隔符（默认：逗号）
    pub delimiter: char,
    /// Quote character (default: double-quote) / 引号字符（默认：双引号）
    pub quote_char: char,
    /// Line terminator / 行终止符
    pub line_terminator: String,
    /// Always quote fields / 总是引用字段
    pub always_quote: bool,
    /// Include BOM for Excel compatibility / 包含 BOM 以兼容 Excel
    pub include_bom: bool,
}

impl Default for CsvExportConfig
{
    fn default() -> Self
    {
        Self {
            headers: Vec::new(),
            delimiter: ',',
            quote_char: '"',
            line_terminator: "\r\n".to_string(),
            always_quote: false,
            include_bom: false,
        }
    }
}

impl CsvExportConfig
{
    /// Create a new default configuration.
    /// 创建新的默认配置。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add a column header.
    /// 添加列标题。
    pub fn header(mut self, name: impl Into<String>) -> Self
    {
        self.headers.push(name.into());
        self
    }

    /// Set multiple column headers.
    /// 设置多个列标题。
    pub fn headers(mut self, headers: Vec<String>) -> Self
    {
        self.headers = headers;
        self
    }

    /// Set the field delimiter character.
    /// 设置字段分隔符字符。
    ///
    /// Common values: `,` (default), `\t` (TSV), `;` (European CSV).
    /// 常用值：`,`（默认）、`\t`（TSV）、`;`（欧洲 CSV）。
    pub fn delimiter(mut self, delimiter: char) -> Self
    {
        self.delimiter = delimiter;
        self
    }

    /// Set the quote character.
    /// 设置引号字符。
    pub fn quote_char(mut self, ch: char) -> Self
    {
        self.quote_char = ch;
        self
    }

    /// Set the line terminator.
    /// 设置行终止符。
    pub fn line_terminator(mut self, term: impl Into<String>) -> Self
    {
        self.line_terminator = term.into();
        self
    }

    /// Always quote all fields, even if not required.
    /// 总是引用所有字段，即使不需要。
    pub fn always_quote(mut self, always: bool) -> Self
    {
        self.always_quote = always;
        self
    }

    /// Include UTF-8 BOM for Excel compatibility.
    /// 包含 UTF-8 BOM 以兼容 Excel。
    pub fn include_bom(mut self, include: bool) -> Self
    {
        self.include_bom = include;
        self
    }
}

// ---------------------------------------------------------------------------
// CSV exporter
// ---------------------------------------------------------------------------

/// CSV exporter for generating CSV output.
/// 用于生成 CSV 输出的导出器。
///
/// Equivalent to Spring's OpenCSV `CSVWriter` or EasyExcel CSV export.
/// 等价于 Spring 的 OpenCSV `CSVWriter` 或 EasyExcel CSV 导出。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_response::csv::{CsvExportConfig, CsvExporter};
///
/// let config = CsvExportConfig::new()
///     .header("ID")
///     .header("Name");
///
/// let mut exporter = CsvExporter::new(config);
/// exporter.add_row(vec!["1", "Alice"]);
/// exporter.add_row(vec!["2", "Bob"]);
///
/// let output = exporter.to_string().unwrap();
/// assert!(output.contains("ID,Name"));
/// assert!(output.contains("1,Alice"));
/// ```
pub struct CsvExporter
{
    config: CsvExportConfig,
    rows: Vec<Vec<String>>,
}

impl CsvExporter
{
    /// Create a new CSV exporter with the given configuration.
    /// 使用给定配置创建新的 CSV 导出器。
    pub fn new(config: CsvExportConfig) -> Self
    {
        Self {
            config,
            rows: Vec::new(),
        }
    }

    /// Add a data row.
    /// 添加数据行。
    pub fn add_row(&mut self, fields: Vec<impl Into<String>>)
    {
        self.rows.push(fields.into_iter().map(Into::into).collect());
    }

    /// Add a pre-formatted row (already `String`).
    /// 添加预格式化行（已是 `String`）。
    pub fn add_row_str(&mut self, fields: Vec<String>)
    {
        self.rows.push(fields);
    }

    /// Returns the number of data rows (excluding header).
    /// 返回数据行数（不包括标题）。
    pub fn row_count(&self) -> usize
    {
        self.rows.len()
    }

    /// Generate the CSV output as a string.
    /// 将 CSV 输出生成为字符串。
    pub fn to_string(&self) -> Result<String>
    {
        let mut buf = String::new();

        // UTF-8 BOM for Excel compatibility
        // UTF-8 BOM 用于 Excel 兼容性
        if self.config.include_bom
        {
            buf.push('\u{FEFF}');
        }

        // Header row
        // 标题行
        if !self.config.headers.is_empty()
        {
            let line = format_csv_line(&self.config.headers, &self.config);
            buf.push_str(&line);
            buf.push_str(&self.config.line_terminator);
        }

        // Data rows
        // 数据行
        for row in &self.rows
        {
            let line = format_csv_line(row, &self.config);
            buf.push_str(&line);
            buf.push_str(&self.config.line_terminator);
        }

        Ok(buf)
    }

    /// Generate the CSV output as bytes.
    /// 将 CSV 输出生成为字节。
    pub fn to_bytes(&self) -> Result<Vec<u8>>
    {
        Ok(self.to_string()?.into_bytes())
    }

    /// Write the CSV output to a writer.
    /// 将 CSV 输出写入 writer。
    pub fn write_to_writer(&self, mut writer: impl Write) -> Result<()>
    {
        let data = self.to_bytes()?;
        writer.write_all(&data)?;
        Ok(())
    }

    /// Write the CSV output to a file.
    /// 将 CSV 输出写入文件。
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()>
    {
        let data = self.to_bytes()?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CsvTable trait
// ---------------------------------------------------------------------------

/// Trait for types that can be auto-exported to CSV.
/// 可自动导出到 CSV 的类型的 trait。
///
/// Equivalent to Spring Batch's `FieldSetMapper` or OpenCSV's bean mapping.
/// 等价于 Spring Batch 的 `FieldSetMapper` 或 OpenCSV 的 Bean 映射。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_response::csv::{CsvTable, export_to_csv};
///
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// impl CsvTable for User {
///     fn csv_headers() -> Vec<String> {
///         vec!["Name".into(), "Email".into()]
///     }
///
///     fn csv_row(&self) -> Vec<String> {
///         vec![self.name.clone(), self.email.clone()]
///     }
/// }
/// ```
pub trait CsvTable
{
    /// Return the column headers for this type.
    /// 返回此类型的列标题。
    fn csv_headers() -> Vec<String>;

    /// Convert this instance into a row of string fields.
    /// 将此实例转换为一行字符串字段。
    fn csv_row(&self) -> Vec<String>;
}

/// Export a slice of `CsvTable` implementors to CSV string.
/// 将 `CsvTable` 实现者切片导出为 CSV 字符串。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_response::csv::{CsvExportConfig, CsvTable, export_to_csv};
///
/// let data = vec![User { name: "Alice".into(), email: "a@b.com".into() }];
/// let csv = export_to_csv(&data, CsvExportConfig::new()).unwrap();
/// ```
pub fn export_to_csv<T: CsvTable>(data: &[T], config: CsvExportConfig) -> Result<String>
{
    let mut new_config = config;
    if new_config.headers.is_empty() && !data.is_empty()
    {
        new_config.headers = T::csv_headers();
    }

    let mut exporter = CsvExporter::new(new_config);
    for item in data
    {
        exporter.add_row_str(item.csv_row());
    }
    exporter.to_string()
}

// ---------------------------------------------------------------------------
// CSV line formatting
// ---------------------------------------------------------------------------

/// Format a single CSV line from field values.
/// 将字段值格式化为单行 CSV。
fn format_csv_line(fields: &[String], config: &CsvExportConfig) -> String
{
    let mut line = String::new();
    for (i, field) in fields.iter().enumerate()
    {
        if i > 0
        {
            line.push(config.delimiter);
        }
        if config.always_quote || needs_quoting(field, config)
        {
            let quoted = quote_field(field, config);
            line.push_str(&quoted);
        }
        else
        {
            line.push_str(field);
        }
    }
    line
}

/// Check if a field value requires quoting per RFC 4180.
/// 检查字段值是否需要按 RFC 4180 加引号。
fn needs_quoting(field: &str, config: &CsvExportConfig) -> bool
{
    field.contains(config.delimiter)
        || field.contains(config.quote_char)
        || field.contains('\n')
        || field.contains('\r')
}

/// Quote a field value, escaping internal quote characters.
/// 引用字段值，转义内部引号字符。
fn quote_field(field: &str, config: &CsvExportConfig) -> String
{
    let escaped =
        field.replace(config.quote_char, &format!("{}{}", config.quote_char, config.quote_char));
    format!("{}{}{}", config.quote_char, escaped, config.quote_char)
}

// ---------------------------------------------------------------------------
// IntoResponse integration
// ---------------------------------------------------------------------------

/// Wrapper type that enables CSV output to be used as an HTTP response.
/// 包装类型，使 CSV 输出可以用作 HTTP 响应。
///
/// Equivalent to Spring's `ResponseEntity` with `Content-Type: text/csv`.
/// 等价于 Spring 的 `ResponseEntity` 配合 `Content-Type: text/csv`。
pub struct Csv(pub CsvExporter);

impl crate::IntoResponse for Csv
{
    fn into_response(self) -> crate::Response
    {
        match self.0.to_bytes()
        {
            Ok(bytes) => crate::Response::builder()
                .header("content-type", "text/csv; charset=utf-8")
                .header("content-disposition", "attachment; filename=\"export.csv\"")
                .body(bytes)
                .unwrap_or_else(|_| crate::Response::new()),
            Err(_) => crate::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to generate CSV")
                .unwrap_or_else(|_| crate::Response::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_csv_config_builder()
    {
        let config = CsvExportConfig::new()
            .header("Name")
            .header("Age")
            .delimiter(';')
            .always_quote(true)
            .include_bom(true);

        assert_eq!(config.headers, vec!["Name", "Age"]);
        assert_eq!(config.delimiter, ';');
        assert!(config.always_quote);
        assert!(config.include_bom);
    }

    #[test]
    fn test_simple_export()
    {
        let config = CsvExportConfig::new().header("ID").header("Name");

        let mut exporter = CsvExporter::new(config);
        exporter.add_row(vec!["1", "Alice"]);
        exporter.add_row(vec!["2", "Bob"]);

        let output = exporter.to_string().unwrap();
        assert!(output.contains("ID,Name\r\n"));
        assert!(output.contains("1,Alice\r\n"));
        assert!(output.contains("2,Bob\r\n"));
    }

    #[test]
    fn test_field_quoting()
    {
        let config = CsvExportConfig::new().header("Name");

        let mut exporter = CsvExporter::new(config);
        exporter.add_row(vec!["Alice, Jr."]);
        exporter.add_row(vec!["Bob \"The Builder\""]);
        exporter.add_row(vec!["Charlie\nSmith"]);

        let output = exporter.to_string().unwrap();
        assert!(output.contains("\"Alice, Jr.\""));
        assert!(output.contains("\"Bob \"\"The Builder\"\"\""));
        assert!(output.contains("\"Charlie\nSmith\""));
    }

    #[test]
    fn test_always_quote()
    {
        let config = CsvExportConfig::new().header("A").always_quote(true);

        let mut exporter = CsvExporter::new(config);
        exporter.add_row(vec!["hello"]);

        let output = exporter.to_string().unwrap();
        assert!(output.contains("\"A\""));
        assert!(output.contains("\"hello\""));
    }

    #[test]
    fn test_tsv_export()
    {
        let config = CsvExportConfig::new()
            .header("Name")
            .header("Value")
            .delimiter('\t');

        let mut exporter = CsvExporter::new(config);
        exporter.add_row(vec!["Alice", "100"]);

        let output = exporter.to_string().unwrap();
        assert!(output.contains("Name\tValue\r\n"));
        assert!(output.contains("Alice\t100\r\n"));
    }

    #[test]
    fn test_bom_output()
    {
        let config = CsvExportConfig::new().header("Col").include_bom(true);

        let mut exporter = CsvExporter::new(config);
        exporter.add_row(vec!["val"]);

        let output = exporter.to_string().unwrap();
        assert!(output.starts_with('\u{FEFF}'));
    }

    #[test]
    fn test_no_bom_by_default()
    {
        let config = CsvExportConfig::new().header("Col");
        let exporter = CsvExporter::new(config);
        let output = exporter.to_string().unwrap();
        assert!(!output.starts_with('\u{FEFF}'));
    }

    #[test]
    fn test_row_count()
    {
        let config = CsvExportConfig::new();
        let mut exporter = CsvExporter::new(config);
        assert_eq!(exporter.row_count(), 0);
        exporter.add_row(vec!["a"]);
        exporter.add_row(vec!["b"]);
        assert_eq!(exporter.row_count(), 2);
    }

    #[test]
    fn test_empty_export()
    {
        let config = CsvExportConfig::new().header("Col1");
        let exporter = CsvExporter::new(config);
        let output = exporter.to_string().unwrap();
        assert_eq!(output, "Col1\r\n");
    }

    #[test]
    fn test_export_to_bytes()
    {
        let config = CsvExportConfig::new().header("A");
        let mut exporter = CsvExporter::new(config);
        exporter.add_row(vec!["1"]);
        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_write_to_file()
    {
        let config = CsvExportConfig::new().header("Name");
        let mut exporter = CsvExporter::new(config);
        exporter.add_row(vec!["Alice"]);

        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join("hiver_csv_test.csv");
        exporter.write_to(&path).unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Name"));
        assert!(content.contains("Alice"));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_csv_table_trait()
    {
        struct Item
        {
            name: String,
            qty: u32,
        }

        impl CsvTable for Item
        {
            fn csv_headers() -> Vec<String>
            {
                vec!["Name".into(), "Quantity".into()]
            }

            fn csv_row(&self) -> Vec<String>
            {
                vec![self.name.clone(), self.qty.to_string()]
            }
        }

        let data = vec![
            Item {
                name: "Apple".into(),
                qty: 10,
            },
            Item {
                name: "Banana".into(),
                qty: 20,
            },
        ];

        let csv = export_to_csv(&data, CsvExportConfig::new()).unwrap();
        assert!(csv.contains("Name,Quantity\r\n"));
        assert!(csv.contains("Apple,10\r\n"));
        assert!(csv.contains("Banana,20\r\n"));
    }

    #[test]
    fn test_needs_quoting()
    {
        let config = CsvExportConfig::new();
        assert!(needs_quoting("a,b", &config));
        assert!(needs_quoting("a\"b", &config));
        assert!(needs_quoting("a\nb", &config));
        assert!(needs_quoting("a\rb", &config));
        assert!(!needs_quoting("hello", &config));
    }

    #[test]
    fn test_quote_field()
    {
        let config = CsvExportConfig::new();
        assert_eq!(quote_field("hello", &config), "\"hello\"");
        assert_eq!(quote_field("a\"b", &config), "\"a\"\"b\"");
    }
}
