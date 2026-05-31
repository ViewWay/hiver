//! Excel export module
//! Excel 导出模块
//!
//! # Overview / 概述
//!
//! This module provides Excel (.xlsx) export capabilities using the OOXML specification.
//! It generates xlsx files as ZIP archives containing the required XML parts,
//! without requiring a full spreadsheet library.
//!
//! 本模块提供基于 OOXML 规范的 Excel (.xlsx) 导出功能。
//! 它将 xlsx 文件生成为包含所需 XML 部分的 ZIP 归档，
//! 无需完整的电子表格库。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_response::excel::{ExcelExportConfig, ExcelExporter, ExcelCell};
//!
//! let config = ExcelExportConfig::new("Users")
//!     .header("Name")
//!     .header("Age")
//!     .header("Email");
//!
//! let mut exporter = ExcelExporter::new(config);
//! exporter.add_row(vec![
//!     ExcelCell::Text("Alice".into()),
//!     ExcelCell::Number(30.0),
//!     ExcelCell::Text("alice@example.com".into()),
//! ]);
//!
//! let bytes = exporter.to_bytes().unwrap();
//! ```

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::{Cursor, Write};
use std::path::Path;
use std::time::SystemTime;

use zip::write::SimpleFileOptions;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Excel export error.
/// Excel 导出错误。
#[derive(Debug, thiserror::Error)]
pub enum ExcelError {
    /// I/O error during ZIP creation.
    /// ZIP 创建过程中的 I/O 错误。
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// ZIP error during xlsx generation.
    /// xlsx 生成过程中的 ZIP 错误。
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Invalid configuration.
    /// 无效的配置。
    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Result type for Excel operations.
/// Excel 操作的结果类型。
pub type Result<T> = std::result::Result<T, ExcelError>;

// ---------------------------------------------------------------------------
// Cell alignment
// ---------------------------------------------------------------------------

/// Horizontal cell alignment.
/// 单元格水平对齐方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellAlignment {
    /// Left-aligned / 左对齐
    Left,
    /// Center-aligned / 居中对齐
    Center,
    /// Right-aligned / 右对齐
    Right,
}

impl CellAlignment {
    /// Convert to OOXML alignment attribute value.
    /// 转换为 OOXML 对齐属性值。
    fn as_str(self) -> &'static str {
        match self {
            CellAlignment::Left => "left",
            CellAlignment::Center => "center",
            CellAlignment::Right => "right",
        }
    }
}

// ---------------------------------------------------------------------------
// Cell style
// ---------------------------------------------------------------------------

/// Excel cell style definition.
/// Excel 单元格样式定义。
#[derive(Debug, Clone)]
pub struct ExcelCellStyle {
    /// Bold text / 粗体文本
    pub bold: bool,
    /// Font size in points / 字体大小（磅）
    pub font_size: f64,
    /// Font color as ARGB hex (e.g. "FF000000" for black) / 字体颜色 ARGB 十六进制
    pub font_color: Option<String>,
    /// Background color as ARGB hex / 背景颜色 ARGB 十六进制
    pub bg_color: Option<String>,
    /// Horizontal alignment / 水平对齐
    pub alignment: CellAlignment,
    /// Show cell border / 显示单元格边框
    pub border: bool,
    /// Number format string (e.g. "#,##0.00") / 数字格式字符串
    pub number_format: Option<String>,
}

impl Default for ExcelCellStyle {
    fn default() -> Self {
        Self {
            bold: false,
            font_size: 11.0,
            font_color: None,
            bg_color: None,
            alignment: CellAlignment::Left,
            border: false,
            number_format: None,
        }
    }
}

impl ExcelCellStyle {
    /// Create a new default cell style.
    /// 创建新的默认单元格样式。
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bold text.
    /// 设置粗体文本。
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// Set font size.
    /// 设置字体大小。
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// Set font color (ARGB hex).
    /// 设置字体颜色（ARGB 十六进制）。
    pub fn font_color(mut self, color: impl Into<String>) -> Self {
        self.font_color = Some(color.into());
        self
    }

    /// Set background color (ARGB hex).
    /// 设置背景颜色（ARGB 十六进制）。
    pub fn bg_color(mut self, color: impl Into<String>) -> Self {
        self.bg_color = Some(color.into());
        self
    }

    /// Set alignment.
    /// 设置对齐方式。
    pub fn alignment(mut self, alignment: CellAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set border visibility.
    /// 设置边框可见性。
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    /// Set number format.
    /// 设置数字格式。
    pub fn number_format(mut self, fmt: impl Into<String>) -> Self {
        self.number_format = Some(fmt.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Cell value
// ---------------------------------------------------------------------------

/// Excel cell value.
/// Excel 单元格值。
#[derive(Debug, Clone)]
pub enum ExcelCell {
    /// Text string / 文本字符串
    Text(String),
    /// Numeric value / 数值
    Number(f64),
    /// Boolean value / 布尔值
    Boolean(bool),
    /// Date value (time since UNIX epoch) / 日期值
    Date(SystemTime),
    /// DateTime value / 日期时间值
    DateTime(SystemTime),
    /// Empty cell / 空单元格
    Empty,
}

impl ExcelCell {
    /// Returns true if the cell is empty.
    /// 如果单元格为空则返回 true。
    pub fn is_empty(&self) -> bool {
        matches!(self, ExcelCell::Empty)
    }

    /// Returns the cell type for style-based cells (shared strings / number format).
    /// 返回基于样式单元格的单元格类型。
    fn style_type_attribute(&self) -> &'static str {
        match self {
            ExcelCell::Boolean(_) => "b",
            ExcelCell::Number(_) | ExcelCell::Date(_) | ExcelCell::DateTime(_) => "n",
            _ => "str",
        }
    }

    /// Returns the value string for style-based cells.
    /// 返回基于样式单元格的值字符串。
    fn style_value(&self) -> Option<String> {
        match self {
            ExcelCell::Text(s) => Some(escape_xml(s)),
            ExcelCell::Number(n) => {
                if n.fract() == 0.0 {
                    Some(format!("{}", *n as i64))
                } else {
                    Some(format!("{}", n))
                }
            }
            ExcelCell::Boolean(b) => Some(if *b {
                "1".to_string()
            } else {
                "0".to_string()
            }),
            ExcelCell::Date(t) | ExcelCell::DateTime(t) => {
                let duration = t
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default();
                let secs = duration.as_secs();
                let days = secs / 86400;
                let days_since_epoch = days as i64 - 25569;
                Some(format!("{}", days_since_epoch + 1))
            }
            ExcelCell::Empty => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Export configuration
// ---------------------------------------------------------------------------

/// Excel export configuration.
/// Excel 导出配置。
#[derive(Debug, Clone)]
pub struct ExcelExportConfig {
    /// Sheet name / 工作表名称
    pub sheet_name: String,
    /// Column headers / 列标题
    pub headers: Vec<String>,
    /// Column widths (in characters) / 列宽（字符数）
    pub column_widths: Vec<f64>,
    /// Enable auto-filter on header row / 在标题行上启用自动筛选
    pub auto_filter: bool,
    /// Freeze the header row / 冻结标题行
    pub freeze_header: bool,
    /// Date format string / 日期格式字符串
    pub date_format: String,
}

impl ExcelExportConfig {
    /// Create a new export configuration with the given sheet name.
    /// 使用给定的工作表名称创建新的导出配置。
    pub fn new(sheet_name: impl Into<String>) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            headers: Vec::new(),
            column_widths: Vec::new(),
            auto_filter: false,
            freeze_header: false,
            date_format: "yyyy-mm-dd".to_string(),
        }
    }

    /// Add a column header.
    /// 添加列标题。
    pub fn header(mut self, name: impl Into<String>) -> Self {
        self.headers.push(name.into());
        self
    }

    /// Add multiple column headers.
    /// 添加多个列标题。
    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set column widths.
    /// 设置列宽。
    pub fn column_widths(mut self, widths: Vec<f64>) -> Self {
        self.column_widths = widths;
        self
    }

    /// Enable or disable auto-filter.
    /// 启用或禁用自动筛选。
    pub fn auto_filter(mut self, enabled: bool) -> Self {
        self.auto_filter = enabled;
        self
    }

    /// Enable or disable header freeze.
    /// 启用或禁用标题冻结。
    pub fn freeze_header(mut self, enabled: bool) -> Self {
        self.freeze_header = enabled;
        self
    }

    /// Set the date format string.
    /// 设置日期格式字符串。
    pub fn date_format(mut self, format: impl Into<String>) -> Self {
        self.date_format = format.into();
        self
    }
}

// ---------------------------------------------------------------------------
// Excel exporter
// ---------------------------------------------------------------------------

/// Excel exporter for generating .xlsx files.
/// 用于生成 .xlsx 文件的 Excel 导出器。
///
/// This exporter builds an OOXML-compliant xlsx file as a ZIP archive
/// containing the required XML parts: `[Content_Types].xml`, `_rels/.rels`,
/// `xl/workbook.xml`, `xl/_rels/workbook.xml.rels`, `xl/styles.xml`,
/// `xl/worksheets/sheet1.xml`.
///
/// 该导出器将符合 OOXML 规范的 xlsx 文件构建为 ZIP 归档，
/// 包含所需的 XML 部分：`[Content_Types].xml`、`_rels/.rels`、
/// `xl/workbook.xml`、`xl/_rels/workbook.xml.rels`、`xl/styles.xml`、
/// `xl/worksheets/sheet1.xml`。
pub struct ExcelExporter {
    config: ExcelExportConfig,
    rows: Vec<Vec<ExcelCell>>,
    header_style: Option<ExcelCellStyle>,
    column_styles: HashMap<usize, ExcelCellStyle>,
}

impl ExcelExporter {
    /// Create a new Excel exporter with the given configuration.
    /// 使用给定配置创建新的 Excel 导出器。
    pub fn new(config: ExcelExportConfig) -> Self {
        Self {
            config,
            rows: Vec::new(),
            header_style: None,
            column_styles: HashMap::new(),
        }
    }

    /// Add a data row to the export.
    /// 向导出添加数据行。
    pub fn add_row(&mut self, cells: Vec<ExcelCell>) {
        self.rows.push(cells);
    }

    /// Set the header row style.
    /// 设置标题行样式。
    pub fn add_header_style(&mut self, style: ExcelCellStyle) {
        self.header_style = Some(style);
    }

    /// Set a style for a specific column.
    /// 设置特定列的样式。
    pub fn set_column_style(&mut self, col: usize, style: ExcelCellStyle) {
        self.column_styles.insert(col, style);
    }

    /// Generate the xlsx file as a byte vector.
    /// 将 xlsx 文件生成为字节向量。
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        // 1. [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(CONTENT_TYPES_XML.as_bytes())?;

        // 2. _rels/.rels
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(RELS_XML.as_bytes())?;

        // 3. xl/workbook.xml
        zip.start_file(
            "xl/workbook.xml",
            options,
        )?;
        zip.write_all(workbook_xml(&self.config).as_bytes())?;

        // 4. xl/_rels/workbook.xml.rels
        zip.start_file(
            "xl/_rels/workbook.xml.rels",
            options,
        )?;
        zip.write_all(WORKBOOK_RELS_XML.as_bytes())?;

        // 5. xl/styles.xml
        zip.start_file("xl/styles.xml", options)?;
        zip.write_all(styles_xml(self).as_bytes())?;

        // 6. xl/worksheets/sheet1.xml
        zip.start_file(
            "xl/worksheets/sheet1.xml",
            options,
        )?;
        zip.write_all(sheet_xml(self).as_bytes())?;

        let cursor = zip.finish()?;
        Ok(cursor.into_inner())
    }

    /// Write the xlsx file to a file path.
    /// 将 xlsx 文件写入文件路径。
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ExcelTable trait
// ---------------------------------------------------------------------------

/// Trait for types that can be auto-exported to Excel.
/// 可自动导出到 Excel 的类型的 trait。
///
/// Implement this trait on your structs to enable direct Excel export
/// without manually converting each field to `ExcelCell`.
///
/// 在你的结构体上实现此 trait 以启用直接 Excel 导出，
/// 无需手动将每个字段转换为 `ExcelCell`。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_response::excel::{ExcelTable, ExcelCell};
///
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// impl ExcelTable for User {
///     fn excel_headers() -> Vec<String> {
///         vec!["Name".into(), "Age".into()]
///     }
///
///     fn excel_row(&self) -> Vec<ExcelCell> {
///         vec![
///             ExcelCell::Text(self.name.clone()),
///             ExcelCell::Number(self.age as f64),
///         ]
///     }
/// }
/// ```
pub trait ExcelTable {
    /// Return the column headers for this type.
    /// 返回此类型的列标题。
    fn excel_headers() -> Vec<String>;

    /// Convert this instance into a row of cells.
    /// 将此实例转换为一行单元格。
    fn excel_row(&self) -> Vec<ExcelCell>;
}

/// Export a slice of `ExcelTable` implementors to xlsx bytes.
/// 将 `ExcelTable` 实现者切片导出为 xlsx 字节。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_response::excel::{ExcelExportConfig, ExcelTable, ExcelCell, export_to_excel};
///
/// let data = vec![User { name: "Alice".into(), age: 30 }];
/// let config = ExcelExportConfig::new("Users").headers(User::excel_headers());
/// let bytes = export_to_excel(&data, config).unwrap();
/// ```
pub fn export_to_excel<T: ExcelTable>(data: &[T], config: ExcelExportConfig) -> Result<Vec<u8>> {
    let mut exporter = ExcelExporter::new(config);
    for item in data {
        exporter.add_row(item.excel_row());
    }
    exporter.to_bytes()
}

// ---------------------------------------------------------------------------
// OOXML XML generation
// ---------------------------------------------------------------------------

/// Column reference string (e.g. "A", "AB", "ZZ").
/// 列引用字符串（如 "A"、"AB"、"ZZ"）。
fn col_ref(col: usize) -> String {
    let mut result = String::new();
    let mut n = col;
    loop {
        let rem = (n % 26) as u8;
        result.insert(0, (b'A' + rem) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    result
}

/// Cell reference string (e.g. "A1", "B3").
/// 单元格引用字符串（如 "A1"、"B3"）。
fn cell_ref(col: usize, row: usize) -> String {
    format!("{}{}", col_ref(col), row + 1)
}

/// Escape XML special characters.
/// 转义 XML 特殊字符。
fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// [Content_Types].xml content.
/// [Content_Types].xml 内容。
const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#;

/// _rels/.rels content.
/// _rels/.rels 内容。
const RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;

/// xl/_rels/workbook.xml.rels content.
/// xl/_rels/workbook.xml.rels 内容。
const WORKBOOK_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

/// Generate xl/workbook.xml.
/// 生成 xl/workbook.xml。
fn workbook_xml(config: &ExcelExportConfig) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="{}" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#,
        escape_xml(&config.sheet_name)
    )
}

/// Generate xl/styles.xml.
/// 生成 xl/styles.xml。
fn styles_xml(exporter: &ExcelExporter) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
"#,
    );

    // Collect unique styles: always have default (xfId=0).
    // header_style -> xfId=1
    // date format -> xfId=2
    // column styles start at xfId=3
    // numFmts start at customId=164 (below 164 are reserved)

    let mut num_fmts_section = String::new();
    let mut fonts_section = String::from(
        r#"<fonts count="1"><font><sz val="11"/><color rgb="FF000000"/><name val="Calibri"/></font></fonts>"#,
    );
    let mut fills_section = String::from(r#"<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>"#);
    let mut borders_section = String::from(
        r#"<borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>"#,
    );

    let mut cell_xfs = String::from(r#"<cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>"#);

    // Track what we've added so we can assign sequential IDs.
    let mut num_fmt_count = 0;
    let mut next_num_fmt_id: u32 = 164; // custom IDs start at 164
    let mut next_font_id: u32 = 1;
    let mut next_fill_id: u32 = 2;
    let mut next_border_id: u32 = 1;
    let mut next_xf_id: u32 = 1;
    let mut num_fmt_id_map: HashMap<String, u32> = HashMap::new(); // format string -> numFmtId

    // Helper to register a number format and return its numFmtId.
    let register_num_fmt = |fmt: &str,
                           num_fmts_section: &mut String,
                           num_fmt_id_map: &mut HashMap<String, u32>,
                           next_num_fmt_id: &mut u32,
                           num_fmt_count: &mut u32| {
        if let Some(&id) = num_fmt_id_map.get(fmt) {
            return id;
        }
        let id = *next_num_fmt_id;
        let _ = write!(
            num_fmts_section,
            "<numFmt numFmtId=\"{}\" formatCode=\"{}\"/>",
            id,
            escape_xml(fmt)
        );
        num_fmt_id_map.insert(fmt.to_string(), id);
        *next_num_fmt_id += 1;
        *num_fmt_count += 1;
        id
    };

    // Helper to register a font and return its fontId.
    let register_font = |style: &ExcelCellStyle,
                         fonts_section: &mut String,
                         next_font_id: &mut u32| {
        let id = *next_font_id;
        let color_attr = style
            .font_color
            .as_deref()
            .map(|c| format!(" rgb=\"{}\"", c))
            .unwrap_or_default();
        let bold_attr = if style.bold { "<b/>" } else { "" };
        let _ = write!(
            fonts_section,
            "<font><sz val=\"{}\"/><color{}/><name val=\"Calibri\"/>{}</font>",
            style.font_size, color_attr, bold_attr
        );
        *next_font_id += 1;
        id
    };

    // Helper to register a fill and return its fillId.
    let register_fill = |style: &ExcelCellStyle,
                         fills_section: &mut String,
                         next_fill_id: &mut u32| {
        if let Some(ref color) = style.bg_color {
            let id = *next_fill_id;
            let _ = write!(
                fills_section,
                "<fill><patternFill patternType=\"solid\"><fgColor rgb=\"{}\"/></patternFill></fill>",
                color
            );
            *next_fill_id += 1;
            Some(id)
        } else {
            None
        }
    };

    // Helper to register a border and return its borderId.
    let register_border = |style: &ExcelCellStyle,
                           borders_section: &mut String,
                           next_border_id: &mut u32| {
        if style.border {
            let id = *next_border_id;
            let _ = write!(
                borders_section,
                "<border><left style=\"thin\"><color auto=\"1\"/></left><right style=\"thin\"><color auto=\"1\"/></right><top style=\"thin\"><color auto=\"1\"/></top><bottom style=\"thin\"><color auto=\"1\"/></bottom><diagonal/></border>"
            );
            *next_border_id += 1;
            Some(id)
        } else {
            None
        }
    };

    // Register header style as xfId=1
    if let Some(ref hs) = exporter.header_style {
        let font_id = register_font(hs, &mut fonts_section, &mut next_font_id);
        let fill_id = register_fill(hs, &mut fills_section, &mut next_fill_id);
        let border_id = register_border(hs, &mut borders_section, &mut next_border_id);

        let num_fmt_id = hs
            .number_format
            .as_deref()
            .map_or(0, |fmt| register_num_fmt(fmt, &mut num_fmts_section, &mut num_fmt_id_map, &mut next_num_fmt_id, &mut num_fmt_count));

        let apply = build_apply_attributes(hs);
        let xf = build_xf(num_fmt_id, font_id, fill_id.unwrap_or(0), border_id.unwrap_or(0), hs.alignment, next_xf_id, &apply);
        next_xf_id += 1;
        cell_xfs.push_str(&xf);
    }

    // Register date format as xfId (if we have date columns or a config date_format)
    {
        let date_fmt = &exporter.config.date_format;
        let num_fmt_id = register_num_fmt(date_fmt, &mut num_fmts_section, &mut num_fmt_id_map, &mut next_num_fmt_id, &mut num_fmt_count);
        let xf = build_xf(num_fmt_id, 0, 0, 0, CellAlignment::Left, next_xf_id, "applyNumberFormat=\"1\"");
        next_xf_id += 1;
        cell_xfs.push_str(&xf);
    }

    // Register column styles
    for &col in exporter.column_styles.keys() {
        if let Some(cs) = exporter.column_styles.get(&col) {
            let font_id = register_font(cs, &mut fonts_section, &mut next_font_id);
            let fill_id = register_fill(cs, &mut fills_section, &mut next_fill_id);
            let border_id = register_border(cs, &mut borders_section, &mut next_border_id);

            let num_fmt_id = cs
                .number_format
                .as_deref()
                .map_or(0, |fmt| register_num_fmt(fmt, &mut num_fmts_section, &mut num_fmt_id_map, &mut next_num_fmt_id, &mut num_fmt_count));

            let apply = build_apply_attributes(cs);
            let xf = build_xf(num_fmt_id, font_id, fill_id.unwrap_or(0), border_id.unwrap_or(0), cs.alignment, next_xf_id, &apply);
            next_xf_id += 1;
            cell_xfs.push_str(&xf);
        }
    }

    // Build final output
    if num_fmt_count > 0 {
        let _ = writeln!(
            xml,
            "<numFmts count=\"{}\">{}</numFmts>",
            num_fmt_count, num_fmts_section
        );
    }

    // Update counts in section headers
    let font_count = next_font_id;
    let fill_count = next_fill_id;
    let border_count = next_border_id;
    let xf_count = next_xf_id;

    let fonts_section = fonts_section.replacen("count=\"1\"", &format!("count=\"{}\"", font_count), 1);
    let fills_section = fills_section.replacen("count=\"2\"", &format!("count=\"{}\"", fill_count), 1);
    let borders_section = borders_section.replacen("count=\"1\"", &format!("count=\"{}\"", border_count), 1);
    let cell_xfs = cell_xfs.replacen("count=\"1\"", &format!("count=\"{}\"", xf_count), 1);

    xml.push_str(&fonts_section);
    xml.push('\n');
    xml.push_str(&fills_section);
    xml.push('\n');
    xml.push_str(&borders_section);
    xml.push('\n');
    xml.push_str(&cell_xfs);
    xml.push('\n');
    xml.push_str("</styleSheet>");
    xml
}

/// Build apply attributes string for a cell Xf.
/// 为单元格 Xf 构建 apply 属性字符串。
fn build_apply_attributes(style: &ExcelCellStyle) -> String {
    let mut attrs = Vec::new();
    #[allow(clippy::float_cmp)]
    if style.bold || style.font_size != 11.0 || style.font_color.is_some() {
        attrs.push("applyFont=\"1\"");
    }
    if style.bg_color.is_some() {
        attrs.push("applyFill=\"1\"");
    }
    if style.border {
        attrs.push("applyBorder=\"1\"");
    }
    if style.alignment != CellAlignment::Left {
        attrs.push("applyAlignment=\"1\"");
    }
    if style.number_format.is_some() {
        attrs.push("applyNumberFormat=\"1\"");
    }
    attrs.join(" ")
}

/// Build a single <xf> element.
/// 构建单个 <xf> 元素。
fn build_xf(
    num_fmt_id: u32,
    font_id: u32,
    fill_id: u32,
    border_id: u32,
    alignment: CellAlignment,
    xf_id: u32,
    apply: &str,
) -> String {
    format!(
        "<xf numFmtId=\"{}\" fontId=\"{}\" fillId=\"{}\" borderId=\"{}\" xfId=\"{}\" {}><alignment horizontal=\"{}\"/></xf>",
        num_fmt_id, font_id, fill_id, border_id, xf_id, apply, alignment.as_str()
    )
}

/// Generate xl/worksheets/sheet1.xml.
/// 生成 xl/worksheets/sheet1.xml。
fn sheet_xml(exporter: &ExcelExporter) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
"#,
    );

    // Determine max column count
    let num_cols = exporter
        .config
        .headers
        .len()
        .max(exporter.rows.iter().map(Vec::len).max().unwrap_or(0));

    // Header row index for auto-filter and freeze pane
    let has_header = !exporter.config.headers.is_empty();
    let last_data_row = if has_header {
        exporter.rows.len() + 1 // 1-based: header is row 1
    } else {
        exporter.rows.len()
    };

    // Auto-filter
    if exporter.config.auto_filter && has_header && num_cols > 0 {
        let start_col = col_ref(0);
        let end_col = col_ref(num_cols - 1);
        let _ = writeln!(
            xml,
            "<autoFilter ref=\"{}1:{}{}\"/>",
            start_col, end_col, last_data_row
        );
    }

    // Freeze pane (freeze header row)
    if exporter.config.freeze_header && has_header {
        // Freeze below row 1, no split column
        let _ = writeln!(xml, "<sheetViews><sheetView tabSelected=\"1\" workbookViewId=\"0\"><pane ySplit=\"1\" topLeftCell=\"A2\" activePane=\"bottomLeft\" state=\"frozen\"/></sheetView></sheetViews>");
    }

    xml.push_str("<sheetData>\n");

    // Style ID mapping:
    // 0 = default
    // 1 = header style (if set)
    // 2 = date format
    // 3+ = column styles (in sorted order by column index)
    let header_xf_id = i32::from(exporter.header_style.is_some());
    let date_xf_id = if exporter.header_style.is_some() { 2 } else { 1 };
    let mut col_style_xf_ids: HashMap<usize, u32> = HashMap::new();
    let mut sorted_cols: Vec<usize> = exporter.column_styles.keys().copied().collect();
    sorted_cols.sort_unstable();
    for (i, &col) in sorted_cols.iter().enumerate() {
        let base_xf = if exporter.header_style.is_some() { 3 } else { 2 };
        col_style_xf_ids.insert(col, base_xf + i as u32);
    }

    // Write header row
    if has_header {
        xml.push_str("<row r=\"1\">\n");
        for (col_idx, header) in exporter.config.headers.iter().enumerate() {
            let ref_str = cell_ref(col_idx, 0);
            let _ = writeln!(
                xml,
                "<c r=\"{}\" s=\"{}\" t=\"inlineStr\"><is><t>{}</t></is></c>",
                ref_str, header_xf_id, escape_xml(header)
            );
        }
        xml.push_str("</row>\n");
    }

    // Write data rows
    for (row_idx, row) in exporter.rows.iter().enumerate() {
        let r = row_idx + 1 + usize::from(has_header); // 1-based, offset by header
        let _ = writeln!(xml, "<row r=\"{}\">", r);
        for (col_idx, cell) in row.iter().enumerate() {
            let ref_str = cell_ref(col_idx, r);
            if cell.is_empty() {
                let _ = write!(xml, "<c r=\"{}\"/>", ref_str);
            } else {
                // Determine style based on column style or cell type
                let xf_id = col_style_xf_ids
                    .get(&col_idx)
                    .copied()
                    .unwrap_or({
                        if matches!(cell, ExcelCell::Date(_) | ExcelCell::DateTime(_)) {
                            date_xf_id
                        } else {
                            0
                        }
                    });

                let _ = write!(
                    xml,
                    "<c r=\"{}\" s=\"{}\" t=\"{}\"><v>{}</v></c>",
                    ref_str,
                    xf_id,
                    cell.style_type_attribute(),
                    cell.style_value().unwrap_or_default()
                );
            }
        }
        xml.push_str("\n</row>\n");
    }

    xml.push_str("</sheetData>\n");

    // Column widths
    if !exporter.config.column_widths.is_empty() {
        xml.push_str("<cols>\n");
        for (i, &width) in exporter.config.column_widths.iter().enumerate() {
            // OOXML width is in character widths (approximately)
            let _ = writeln!(
                xml,
                "<col min=\"{}\" max=\"{}\" width=\"{}\" customWidth=\"1\"/>",
                i + 1,
                i + 1,
                width
            );
        }
        xml.push_str("</cols>\n");
    }

    xml.push_str("</worksheet>");
    xml
}

// ---------------------------------------------------------------------------
// IntoResponse integration
// ---------------------------------------------------------------------------

/// Wrapper type that enables `ExcelExporter` output to be used as an HTTP response.
/// 包装类型，使 `ExcelExporter` 输出可以用作 HTTP 响应。
pub struct Excel(pub ExcelExporter);

impl crate::IntoResponse for Excel {
    fn into_response(self) -> crate::Response {
        match self.0.to_bytes() {
            Ok(bytes) => crate::Response::builder()
                .header("content-type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .header("content-disposition", "attachment; filename=\"export.xlsx\"")
                .body(bytes)
                .unwrap_or_else(|_| crate::Response::new()),
            Err(_) => crate::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to generate Excel file")
                .unwrap_or_else(|_| crate::Response::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn test_col_ref() {
        assert_eq!(col_ref(0), "A");
        assert_eq!(col_ref(1), "B");
        assert_eq!(col_ref(25), "Z");
        assert_eq!(col_ref(26), "AA");
        assert_eq!(col_ref(27), "AB");
        assert_eq!(col_ref(701), "ZZ");
        assert_eq!(col_ref(702), "AAA");
    }

    #[test]
    fn test_cell_ref() {
        assert_eq!(cell_ref(0, 0), "A1");
        assert_eq!(cell_ref(1, 2), "B3");
        assert_eq!(cell_ref(25, 9), "Z10");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("a&b"), "a&amp;b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_excel_cell_empty() {
        assert!(ExcelCell::Empty.is_empty());
        assert!(!ExcelCell::Text("x".into()).is_empty());
    }

    #[test]
    fn test_export_config_builder() {
        let config = ExcelExportConfig::new("Test")
            .header("Name")
            .header("Age")
            .column_widths(vec![20.0, 10.0])
            .auto_filter(true)
            .freeze_header(true);

        assert_eq!(config.sheet_name, "Test");
        assert_eq!(config.headers, vec!["Name", "Age"]);
        assert_eq!(config.column_widths, vec![20.0, 10.0]);
        assert!(config.auto_filter);
        assert!(config.freeze_header);
    }

    #[test]
    fn test_simple_export_to_bytes() {
        let config = ExcelExportConfig::new("Sheet1")
            .header("Name")
            .header("Score");

        let mut exporter = ExcelExporter::new(config);
        exporter.add_row(vec![
            ExcelCell::Text("Alice".into()),
            ExcelCell::Number(95.5),
        ]);
        exporter.add_row(vec![
            ExcelCell::Text("Bob".into()),
            ExcelCell::Number(87.0),
        ]);

        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Verify it's a valid ZIP
        let cursor = Cursor::new(bytes);
        let reader = zip::ZipArchive::new(cursor).unwrap();
        assert!(reader.file_names().any(|n| n == "[Content_Types].xml"));
        assert!(reader.file_names().any(|n| n == "xl/workbook.xml"));
        assert!(reader.file_names().any(|n| n == "xl/worksheets/sheet1.xml"));
        assert!(reader.file_names().any(|n| n == "xl/styles.xml"));
    }

    #[test]
    fn test_export_with_header_style() {
        let config = ExcelExportConfig::new("Users")
            .header("Name")
            .header("Email");

        let mut exporter = ExcelExporter::new(config);
        exporter.add_header_style(
            ExcelCellStyle::new()
                .bold(true)
                .bg_color("FF4472C4")
                .font_color("FFFFFFFF")
                .alignment(CellAlignment::Center)
                .border(true),
        );
        exporter.add_row(vec![
            ExcelCell::Text("Alice".into()),
            ExcelCell::Text("alice@example.com".into()),
        ]);

        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_export_with_date_cell() {
        let config = ExcelExportConfig::new("Dates")
            .header("Name")
            .header("Created");

        let mut exporter = ExcelExporter::new(config);
        exporter.add_row(vec![
            ExcelCell::Text("Alice".into()),
            ExcelCell::Date(UNIX_EPOCH + Duration::from_secs(1704067200)), // 2024-01-01
        ]);

        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_export_with_boolean_cell() {
        let config = ExcelExportConfig::new("Flags")
            .header("Active");

        let mut exporter = ExcelExporter::new(config);
        exporter.add_row(vec![ExcelCell::Boolean(true)]);
        exporter.add_row(vec![ExcelCell::Boolean(false)]);
        exporter.add_row(vec![ExcelCell::Empty]);

        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_export_with_column_styles() {
        let config = ExcelExportConfig::new("Numbers")
            .header("Name")
            .header("Balance");

        let mut exporter = ExcelExporter::new(config);
        exporter.set_column_style(
            1,
            ExcelCellStyle::new()
                .alignment(CellAlignment::Right)
                .number_format("#,##0.00"),
        );
        exporter.add_row(vec![
            ExcelCell::Text("Alice".into()),
            ExcelCell::Number(12345.67),
        ]);

        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_export_to_excel_trait() {
        struct Person {
            name: String,
            age: u32,
        }

        impl ExcelTable for Person {
            fn excel_headers() -> Vec<String> {
                vec!["Name".into(), "Age".into()]
            }

            fn excel_row(&self) -> Vec<ExcelCell> {
                vec![
                    ExcelCell::Text(self.name.clone()),
                    ExcelCell::Number(self.age as f64),
                ]
            }
        }

        let data = vec![
            Person {
                name: "Alice".into(),
                age: 30,
            },
            Person {
                name: "Bob".into(),
                age: 25,
            },
        ];

        let config = ExcelExportConfig::new("People").headers(Person::excel_headers());
        let bytes = export_to_excel(&data, config).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_write_to_file() {
        let config = ExcelExportConfig::new("Test").header("Col1");
        let mut exporter = ExcelExporter::new(config);
        exporter.add_row(vec![ExcelCell::Text("value".into())]);

        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join("hiver_excel_test.xlsx");
        exporter.write_to(&path).unwrap();

        assert!(path.exists());
        assert!(path.metadata().unwrap().len() > 0);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_empty_export() {
        let config = ExcelExportConfig::new("Empty").header("Col1");
        let exporter = ExcelExporter::new(config);
        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_export_no_headers() {
        let config = ExcelExportConfig::new("NoHeaders");
        let mut exporter = ExcelExporter::new(config);
        exporter.add_row(vec![
            ExcelCell::Number(1.0),
            ExcelCell::Number(2.0),
        ]);

        let bytes = exporter.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_cell_alignment() {
        assert_eq!(CellAlignment::Left.as_str(), "left");
        assert_eq!(CellAlignment::Center.as_str(), "center");
        assert_eq!(CellAlignment::Right.as_str(), "right");
    }

    #[test]
    fn test_cell_style_builder() {
        let style = ExcelCellStyle::new()
            .bold(true)
            .font_size(14.0)
            .font_color("FFFF0000")
            .bg_color("FFFFFF00")
            .alignment(CellAlignment::Center)
            .border(true)
            .number_format("#,##0");

        assert!(style.bold);
        assert_eq!(style.font_size, 14.0);
        assert_eq!(style.font_color.as_deref(), Some("FFFF0000"));
        assert_eq!(style.bg_color.as_deref(), Some("FFFFFF00"));
        assert_eq!(style.alignment, CellAlignment::Center);
        assert!(style.border);
        assert_eq!(style.number_format.as_deref(), Some("#,##0"));
    }
}
