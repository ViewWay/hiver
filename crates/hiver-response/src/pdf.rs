//! PDF export module
//! PDF 导出模块
//!
//! # Overview / 概述
//!
//! This module provides basic PDF generation capabilities using a minimal
//! PDF structure. It generates PDF 1.4 compliant files with text content,
//! tables, and basic formatting without requiring an external PDF library.
//!
//! 本模块提供使用最小 PDF 结构的基本 PDF 生成功能。
//! 它生成符合 PDF 1.4 的文件，包含文本内容、表格和基本格式，无需外部 PDF 库。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! - JasperReports
//! - iText
//! - Apache PDFBox
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_response::pdf::{PdfDocument, PdfPage, PdfText};
//!
//! let mut doc = PdfDocument::new("Report");
//! let mut page = PdfPage::a4();
//!
//! page.add_text(PdfText::new("Hello, World!").font_size(24.0).at(72.0, 700.0));
//! doc.add_page(page);
//!
//! let bytes = doc.to_bytes().unwrap();
//! ```

use std::fmt::Write as FmtWrite;
use std::io;
use std::path::Path;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// PDF export error.
/// PDF 导出错误。
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    /// I/O error during write.
    /// 写入过程中的 I/O 错误。
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Formatting error.
    /// 格式化错误。
    #[error("Format error: {0}")]
    Format(String),
}

/// Result type for PDF operations.
/// PDF 操作的结果类型。
pub type Result<T> = std::result::Result<T, PdfError>;

// ---------------------------------------------------------------------------
// PDF text element
// ---------------------------------------------------------------------------

/// Font family for PDF text.
/// PDF 文本的字体族。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum PdfFont {
    /// Helvetica (default sans-serif)
    /// Helvetica（默认无衬线体）
    #[default]
    Helvetica,
    /// Helvetica Bold
    /// Helvetica 粗体
    HelveticaBold,
    /// Courier (monospace)
    /// Courier（等宽字体）
    Courier,
    /// Courier Bold
    /// Courier 粗体
    CourierBold,
    /// Times Roman (serif)
    /// Times Roman（衬线体）
    TimesRoman,
    /// Times Bold
    /// Times 粗体
    TimesBold,
}


impl PdfFont {
    /// Get the PDF internal font name.
    /// 获取 PDF 内部字体名称。
    pub fn pdf_name(&self) -> &'static str {
        match self {
            PdfFont::Helvetica => "Helvetica",
            PdfFont::HelveticaBold => "Helvetica-Bold",
            PdfFont::Courier => "Courier",
            PdfFont::CourierBold => "Courier-Bold",
            PdfFont::TimesRoman => "Times-Roman",
            PdfFont::TimesBold => "Times-Bold",
        }
    }
}

/// A text element on a PDF page.
/// PDF 页面上的文本元素。
#[derive(Debug, Clone)]
pub struct PdfText {
    /// Text content / 文本内容
    pub content: String,
    /// X position in points / X 位置（磅）
    pub x: f64,
    /// Y position in points / Y 位置（磅）
    pub y: f64,
    /// Font size in points / 字体大小（磅）
    pub font_size: f64,
    /// Font family / 字体族
    pub font: PdfFont,
    /// Text color as RGB hex (e.g. "#FF0000" for red) / 文本颜色 RGB 十六进制
    pub color: Option<String>,
}

impl PdfText {
    /// Create a new text element.
    /// 创建新的文本元素。
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            x: 72.0,
            y: 720.0,
            font_size: 12.0,
            font: PdfFont::Helvetica,
            color: None,
        }
    }

    /// Set the position (x, y in points from bottom-left).
    /// 设置位置（x, y 以左下角为原点的磅值）。
    pub fn at(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the font size.
    /// 设置字体大小。
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// Set the font family.
    /// 设置字体族。
    pub fn font(mut self, font: PdfFont) -> Self {
        self.font = font;
        self
    }

    /// Set the text color.
    /// 设置文本颜色。
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

// ---------------------------------------------------------------------------
// PDF table element
// ---------------------------------------------------------------------------

/// A table element on a PDF page.
/// PDF 页面上的表格元素。
#[derive(Debug, Clone)]
pub struct PdfTable {
    /// Column headers / 列标题
    pub headers: Vec<String>,
    /// Data rows / 数据行
    pub rows: Vec<Vec<String>>,
    /// X position / X 位置
    pub x: f64,
    /// Y position (top of table) / Y 位置（表格顶部）
    pub y: f64,
    /// Column width in points / 列宽（磅）
    pub col_width: f64,
    /// Row height in points / 行高（磅）
    pub row_height: f64,
    /// Header font size / 标题字体大小
    pub header_font_size: f64,
    /// Data font size / 数据字体大小
    pub data_font_size: f64,
}

impl PdfTable {
    /// Create a new table with headers.
    /// 创建带标题的新表格。
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            x: 72.0,
            y: 700.0,
            col_width: 120.0,
            row_height: 20.0,
            header_font_size: 10.0,
            data_font_size: 9.0,
        }
    }

    /// Add a data row.
    /// 添加数据行。
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    /// Set position.
    /// 设置位置。
    pub fn at(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set column width.
    /// 设置列宽。
    pub fn col_width(mut self, width: f64) -> Self {
        self.col_width = width;
        self
    }

    /// Set row height.
    /// 设置行高。
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }
}

// ---------------------------------------------------------------------------
// PDF line element
// ---------------------------------------------------------------------------

/// A horizontal line element.
/// 水平线元素。
#[derive(Debug, Clone)]
pub struct PdfLine {
    /// X start position / X 起始位置
    pub x: f64,
    /// Y position / Y 位置
    pub y: f64,
    /// Line length / 线长度
    pub length: f64,
    /// Line width in points / 线宽（磅）
    pub width: f64,
}

impl PdfLine {
    /// Create a new horizontal line.
    /// 创建新的水平线。
    pub fn new(x: f64, y: f64, length: f64) -> Self {
        Self {
            x,
            y,
            length,
            width: 0.5,
        }
    }

    /// Set line width.
    /// 设置线宽。
    pub fn width(mut self, w: f64) -> Self {
        self.width = w;
        self
    }
}

// ---------------------------------------------------------------------------
// PDF page
// ---------------------------------------------------------------------------

/// A single page in a PDF document.
/// PDF 文档中的单页。
#[derive(Debug, Clone)]
pub struct PdfPage {
    /// Page width in points / 页面宽度（磅）
    pub width: f64,
    /// Page height in points / 页面高度（磅）
    pub height: f64,
    /// Text elements / 文本元素
    pub texts: Vec<PdfText>,
    /// Table elements / 表格元素
    pub tables: Vec<PdfTable>,
    /// Line elements / 线元素
    pub lines: Vec<PdfLine>,
}

impl PdfPage {
    /// Standard US Letter size (8.5" x 11").
    /// 标准 US Letter 尺寸（8.5" x 11"）。
    pub fn letter() -> Self {
        Self {
            width: 612.0,
            height: 792.0,
            texts: Vec::new(),
            tables: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Standard A4 size (210mm x 297mm).
    /// 标准 A4 尺寸（210mm x 297mm）。
    pub fn a4() -> Self {
        Self {
            width: 595.28,
            height: 841.89,
            texts: Vec::new(),
            tables: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Create a custom-sized page.
    /// 创建自定义尺寸页面。
    pub fn custom(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            texts: Vec::new(),
            tables: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Add a text element.
    /// 添加文本元素。
    pub fn add_text(&mut self, text: PdfText) {
        self.texts.push(text);
    }

    /// Add a table element.
    /// 添加表格元素。
    pub fn add_table(&mut self, table: PdfTable) {
        self.tables.push(table);
    }

    /// Add a line element.
    /// 添加线元素。
    pub fn add_line(&mut self, line: PdfLine) {
        self.lines.push(line);
    }
}

// ---------------------------------------------------------------------------
// PDF document
// ---------------------------------------------------------------------------

/// A PDF document containing one or more pages.
/// 包含一页或多页的 PDF 文档。
///
/// Equivalent to Spring's JasperReports `JasperPrint` or iText `PdfDocument`.
/// 等价于 Spring 的 JasperReports `JasperPrint` 或 iText `PdfDocument`。
pub struct PdfDocument {
    /// Document title / 文档标题
    pub title: String,
    /// Document author / 文档作者
    pub author: String,
    /// Pages / 页面
    pub pages: Vec<PdfPage>,
}

impl PdfDocument {
    /// Create a new PDF document with a title.
    /// 创建带标题的新 PDF 文档。
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            author: String::new(),
            pages: Vec::new(),
        }
    }

    /// Set the document author.
    /// 设置文档作者。
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Add a page to the document.
    /// 向文档添加页面。
    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }

    /// Returns the number of pages.
    /// 返回页数。
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Generate the PDF as bytes.
    /// 将 PDF 生成为字节。
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut pdf = PdfBuilder::new();
        pdf.set_title(&self.title);
        if !self.author.is_empty() {
            pdf.set_author(&self.author);
        }
        for page in &self.pages {
            pdf.add_page(page);
        }
        pdf.build()
    }

    /// Write the PDF to a file.
    /// 将 PDF 写入文件。
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PDF builder (low-level)
// ---------------------------------------------------------------------------

/// Internal PDF structure builder.
/// 内部 PDF 结构构建器。
struct PdfBuilder {
    title: String,
    author: String,
    pages: Vec<PdfPage>,
}

impl PdfBuilder {
    fn new() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            pages: Vec::new(),
        }
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn set_author(&mut self, author: &str) {
        self.author = author.to_string();
    }

    fn add_page(&mut self, page: &PdfPage) {
        self.pages.push(page.clone());
    }

    /// Build the PDF document as bytes.
    /// 将 PDF 文档构建为字节。
    fn build(&self) -> Result<Vec<u8>> {
        let mut buf = String::new();
        let mut offsets: Vec<usize> = Vec::new();

        // Header
        buf.push_str("%PDF-1.4\n");
        // Binary comment to mark as binary PDF
        buf.push_str("%\u{00E2}\u{00E3}\u{00CF}\u{00D3}\n");

        // Collect fonts used
        let fonts = collect_fonts(&self.pages);
        let font_names: Vec<&str> = fonts.iter().map(PdfFont::pdf_name).collect();
        let unique_fonts: Vec<&str> = {
            let mut seen = std::collections::HashSet::new();
            font_names.into_iter().filter(|n| seen.insert(*n)).collect()
        };

        // Object 1: Catalog
        offsets.push(buf.len());
        buf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Object 2: Pages (placeholder count)
        offsets.push(buf.len());
        let page_count = self.pages.len();
        let _ = write!(
            buf,
            "2 0 obj\n<< /Type /Pages /Kids [{} ] /Count {} >>\nendobj\n",
            (0..page_count)
                .map(|i| {
                    // Page objects start at 3 + num_fonts
                    let base = 3 + unique_fonts.len();
                    format!("{} 0 R", base + i * 2)
                })
                .collect::<Vec<String>>()
                .join(" "),
            page_count
        );

        // Object 3+: Font objects
        for (i, font_name) in unique_fonts.iter().enumerate() {
            offsets.push(buf.len());
            let _ = write!(
                buf,
                "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /{} >>\nendobj\n",
                3 + i,
                font_name
            );
        }

        // Page objects: each page needs a Page dict + Content stream
        let base_obj = 3 + unique_fonts.len();
        for (page_idx, page) in self.pages.iter().enumerate() {
            let page_obj = base_obj + page_idx * 2;
            let content_obj = page_obj + 1;

            // Build content stream
            let stream = build_page_stream(page, &unique_fonts);

            // Page object
            offsets.push(buf.len());
            let font_refs: Vec<String> = unique_fonts
                .iter()
                .enumerate()
                .map(|(i, _)| format!("/F{} {} 0 R", i, 3 + i))
                .collect();
            let _ = write!(
                buf,
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R /Resources << /Font << {} >> >> >>\nendobj\n",
                page_obj,
                page.width,
                page.height,
                content_obj,
                font_refs.join(" ")
            );

            // Content stream
            offsets.push(buf.len());
            let _ = write!(
                buf,
                "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
                content_obj,
                stream.len(),
                stream
            );
        }

        // Info object
        let info_obj = base_obj + self.pages.len() * 2;
        offsets.push(buf.len());
        let _ = write!(
            buf,
            "{} 0 obj\n<< /Title ({}) /Author ({}) /Producer (Hiver PDF) >>\nendobj\n",
            info_obj,
            escape_pdf_string(&self.title),
            escape_pdf_string(&self.author)
        );

        // Cross-reference table
        let xref_offset = buf.len();
        buf.push_str("xref\n");
        let total_objects = info_obj + 1;
        let _ = writeln!(buf, "0 {}", total_objects);
        buf.push_str("0000000000 65535 f \n");
        for offset in &offsets {
            let _ = writeln!(buf, "{:010} 00000 n ", offset);
        }

        // Trailer
        let _ = write!(
            buf,
            "trailer\n<< /Size {} /Root 1 0 R /Info {} 0 R >>\n",
            total_objects, info_obj
        );
        let _ = write!(buf, "startxref\n{}\n%%EOF\n", xref_offset);

        Ok(buf.into_bytes())
    }
}

/// Collect all unique fonts used across pages.
/// 收集所有页面中使用的唯一字体。
fn collect_fonts(pages: &[PdfPage]) -> Vec<PdfFont> {
    let mut fonts = Vec::new();
    for page in pages {
        for text in &page.texts {
            if !fonts.contains(&text.font) {
                fonts.push(text.font);
            }
        }
        for _table in &page.tables {
            // Tables use Helvetica by default
            if !fonts.contains(&PdfFont::Helvetica) {
                fonts.push(PdfFont::Helvetica);
            }
            if !fonts.contains(&PdfFont::HelveticaBold) {
                fonts.push(PdfFont::HelveticaBold);
            }
        }
    }
    if fonts.is_empty() {
        fonts.push(PdfFont::Helvetica);
    }
    fonts
}

/// Find the font index in the unique fonts list.
/// 在唯一字体列表中查找字体索引。
#[allow(clippy::trivially_copy_pass_by_ref)]
fn font_index(font: &PdfFont, unique_fonts: &[&str]) -> usize {
    let name = font.pdf_name();
    unique_fonts.iter().position(|f| *f == name).unwrap_or(0)
}

/// Build the content stream for a page.
/// 为页面构建内容流。
#[allow(clippy::cast_precision_loss)]
fn build_page_stream(page: &PdfPage, unique_fonts: &[&str]) -> String {
    let mut stream = String::new();

    // Render lines
    for line in &page.lines {
        let _ =
            writeln!(stream, "{} {} 0 0 {} {} re S", line.width / 2.0, line.x, line.length, line.y);
    }

    // Render text elements
    for text in &page.texts {
        let fi = font_index(&text.font, unique_fonts);

        if let Some(ref color) = text.color {
            let (r, g, b) = parse_hex_color(color);
            let _ = writeln!(stream, "{:.3} {:.3} {:.3} rg", r, g, b);
        }

        let _ = write!(
            stream,
            "BT\n/F{} {:.1} Tf\n{:.1} {:.1} Td\n({}) Tj\nET\n",
            fi,
            text.font_size,
            text.x,
            text.y,
            escape_pdf_string(&text.content)
        );
    }

    // Render tables
    for table in &page.tables {
        // Header row
        let header_fi = font_index(&PdfFont::HelveticaBold, unique_fonts);
        let data_fi = font_index(&PdfFont::Helvetica, unique_fonts);

        // Draw header background
        let header_height = table.row_height;
        let total_width = table.col_width * table.headers.len() as f64;
        let _ = write!(
            stream,
            "0.9 0.9 0.9 rg\n{} {} {} {} re f\n",
            table.x,
            table.y - header_height,
            total_width,
            header_height
        );

        // Header text
        for (i, header) in table.headers.iter().enumerate() {
            let x = table.x + i as f64 * table.col_width + 4.0;
            let y = table.y - header_height + 6.0;
            let _ = write!(
                stream,
                "0 0 0 rg\nBT\n/F{} {:.1} Tf\n{:.1} {:.1} Td\n({}) Tj\nET\n",
                header_fi,
                table.header_font_size,
                x,
                y,
                escape_pdf_string(header)
            );
        }

        // Header underline
        let _ = write!(
            stream,
            "0 0 0 RG\n0.5 w\n{} {} m\n{} {} l S\n",
            table.x,
            table.y - header_height,
            table.x + total_width,
            table.y - header_height
        );

        // Data rows
        for (row_idx, row) in table.rows.iter().enumerate() {
            let row_y = table.y - header_height - (row_idx + 1) as f64 * table.row_height;

            for (col_idx, cell) in row.iter().enumerate() {
                let x = table.x + col_idx as f64 * table.col_width + 4.0;
                let y = row_y + 6.0;
                let _ = write!(
                    stream,
                    "BT\n/F{} {:.1} Tf\n{:.1} {:.1} Td\n({}) Tj\nET\n",
                    data_fi,
                    table.data_font_size,
                    x,
                    y,
                    escape_pdf_string(cell)
                );
            }

            // Row separator line
            let _ = write!(
                stream,
                "0.8 0.8 0.8 RG\n0.3 w\n{} {} m\n{} {} l S\n",
                table.x,
                row_y,
                table.x + total_width,
                row_y
            );
        }
    }

    stream
}

/// Parse a hex color string (e.g. "#FF0000") into (r, g, b) floats in [0, 1].
/// 将十六进制颜色字符串（如 "#FF0000"）解析为 [0, 1] 范围的 (r, g, b) 浮点数。
fn parse_hex_color(color: &str) -> (f64, f64, f64) {
    let hex = color.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
        (r, g, b)
    } else {
        (0.0, 0.0, 0.0)
    }
}

/// Escape special characters in PDF string literals.
/// 转义 PDF 字符串字面量中的特殊字符。
fn escape_pdf_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c as u32 > 127 => {
                let _ = write!(out, "\\{:03o}", c as u32);
            },
            c => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// IntoResponse integration
// ---------------------------------------------------------------------------

/// Wrapper type that enables PDF output to be used as an HTTP response.
/// 包装类型，使 PDF 输出可以用作 HTTP 响应。
pub struct Pdf(pub PdfDocument);

impl crate::IntoResponse for Pdf {
    fn into_response(self) -> crate::Response {
        match self.0.to_bytes() {
            Ok(bytes) => crate::Response::builder()
                .header("content-type", "application/pdf")
                .header("content-disposition", "attachment; filename=\"export.pdf\"")
                .body(bytes)
                .unwrap_or_else(|_| crate::Response::new()),
            Err(_) => crate::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to generate PDF")
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

    #[test]
    fn test_pdf_text_builder() {
        let text = PdfText::new("Hello")
            .at(100.0, 700.0)
            .font_size(16.0)
            .font(PdfFont::Courier)
            .color("#FF0000");

        assert_eq!(text.content, "Hello");
        assert_eq!(text.x, 100.0);
        assert_eq!(text.y, 700.0);
        assert_eq!(text.font_size, 16.0);
        assert_eq!(text.font, PdfFont::Courier);
        assert_eq!(text.color.as_deref(), Some("#FF0000"));
    }

    #[test]
    fn test_pdf_table_builder() {
        let mut table = PdfTable::new(vec!["Name".into(), "Age".into()])
            .at(72.0, 700.0)
            .col_width(100.0)
            .row_height(18.0);

        table.add_row(vec!["Alice".into(), "30".into()]);
        table.add_row(vec!["Bob".into(), "25".into()]);

        assert_eq!(table.headers.len(), 2);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_pdf_page_a4() {
        let page = PdfPage::a4();
        assert_eq!(page.width, 595.28);
        assert_eq!(page.height, 841.89);
    }

    #[test]
    fn test_pdf_page_letter() {
        let page = PdfPage::letter();
        assert_eq!(page.width, 612.0);
        assert_eq!(page.height, 792.0);
    }

    #[test]
    fn test_pdf_document_simple() {
        let mut doc = PdfDocument::new("Test Report").author("Hiver");
        let mut page = PdfPage::a4();
        page.add_text(PdfText::new("Hello, World!").at(72.0, 760.0));
        doc.add_page(page);

        let bytes = doc.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Verify PDF header
        let header = String::from_utf8_lossy(&bytes[..8]);
        assert!(header.starts_with("%PDF-1.4"));
    }

    #[test]
    fn test_pdf_document_with_table() {
        let mut doc = PdfDocument::new("Table Report");
        let mut page = PdfPage::a4();

        let mut table = PdfTable::new(vec!["ID".into(), "Name".into()])
            .col_width(80.0)
            .row_height(20.0);
        table.add_row(vec!["1".into(), "Alice".into()]);
        table.add_row(vec!["2".into(), "Bob".into()]);

        page.add_table(table);
        doc.add_page(page);

        let bytes = doc.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_pdf_multi_page() {
        let mut doc = PdfDocument::new("Multi-page");

        for i in 0..3 {
            let mut page = PdfPage::a4();
            page.add_text(PdfText::new(format!("Page {}", i + 1)).at(72.0, 760.0));
            doc.add_page(page);
        }

        assert_eq!(doc.page_count(), 3);
        let bytes = doc.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_pdf_write_to_file() {
        let mut doc = PdfDocument::new("File Test");
        let mut page = PdfPage::a4();
        page.add_text(PdfText::new("File output test").at(72.0, 760.0));
        doc.add_page(page);

        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join("hiver_pdf_test.pdf");
        doc.write_to(&path).unwrap();

        assert!(path.exists());
        assert!(path.metadata().unwrap().len() > 0);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_escape_pdf_string() {
        assert_eq!(escape_pdf_string("hello"), "hello");
        assert_eq!(escape_pdf_string("a(b)c"), "a\\(b\\)c");
        assert_eq!(escape_pdf_string("a\\b"), "a\\\\b");
        assert_eq!(escape_pdf_string("a\nb"), "a\\nb");
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FF0000"), (1.0, 0.0, 0.0));
        assert_eq!(parse_hex_color("#00FF00"), (0.0, 1.0, 0.0));
        assert_eq!(parse_hex_color("#0000FF"), (0.0, 0.0, 1.0));
        assert_eq!(parse_hex_color("#000000"), (0.0, 0.0, 0.0));
        assert_eq!(parse_hex_color("invalid"), (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_font_pdf_name() {
        assert_eq!(PdfFont::Helvetica.pdf_name(), "Helvetica");
        assert_eq!(PdfFont::HelveticaBold.pdf_name(), "Helvetica-Bold");
        assert_eq!(PdfFont::Courier.pdf_name(), "Courier");
        assert_eq!(PdfFont::TimesRoman.pdf_name(), "Times-Roman");
    }

    #[test]
    fn test_pdf_line() {
        let line = PdfLine::new(72.0, 700.0, 200.0).width(1.0);
        assert_eq!(line.x, 72.0);
        assert_eq!(line.y, 700.0);
        assert_eq!(line.length, 200.0);
        assert_eq!(line.width, 1.0);
    }

    #[test]
    fn test_pdf_empty_document() {
        let doc = PdfDocument::new("Empty");
        assert_eq!(doc.page_count(), 0);
        // Should still generate valid PDF
        let bytes = doc.to_bytes().unwrap();
        assert!(!bytes.is_empty());
        let header = String::from_utf8_lossy(&bytes[..8]);
        assert!(header.starts_with("%PDF-1.4"));
    }
}
