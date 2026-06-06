//! Hiver Response - Response builders
//! Hiver响应 - 响应构建器
//!
//! # Overview / 概述
//!
//! `hiver-response` provides response builders and types for HTTP responses.
//!
//! `hiver-response` 提供HTTP响应的构建器和类型。

#![warn(missing_docs)]
#![warn(unreachable_pub)]
// Allow expect_used: Response::builder().body() failure is unrecoverable;
// using expect with a descriptive message is intentional.
// 允许 expect_used：Response::builder().body() 失败是不可恢复的；
// 使用带有描述性消息的 expect 是有意的。
#![allow(clippy::expect_used)]

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

pub mod csv;
pub mod excel;
pub mod html;
pub mod json;
pub mod pdf;
pub mod response;
pub mod result;
#[cfg(feature = "sse")]
pub mod sse;
pub mod unified;

// CSV re-exports
pub use csv::{Csv, CsvError, CsvExportConfig, CsvExporter, CsvTable, export_to_csv};
// Excel re-exports
pub use excel::{
    CellAlignment, Excel, ExcelCell, ExcelCellStyle, ExcelError, ExcelExportConfig, ExcelExporter,
    ExcelTable, export_to_excel,
};
pub use html::Html;
pub use json::Json;
// PDF re-exports
pub use pdf::{Pdf, PdfDocument, PdfError, PdfFont, PdfLine, PdfPage, PdfTable, PdfText};
pub use response::{IntoResponse, Response};
pub use result::{PageResult, Result, ResultCode};
// SSE re-exports
#[cfg(feature = "sse")]
pub use sse::{SseEmitter, SseError, SseEvent, SseEventBuilder, SseSender, sse_channel};
pub use unified::{ApiResponse, DefaultResponseAdvice, ResponseAdvice, ResponseResult};
