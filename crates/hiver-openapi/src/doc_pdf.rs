//! API documentation generator (Markdown / HTML)
//! API 文档生成器（Markdown / HTML）
//!
//! Generates structured documentation from an `OpenAPI` 3.0 specification
//! in Markdown or printable HTML format (which can be exported to PDF).
//! 从 `OpenAPI` 3.0 规范生成结构化文档，支持 Markdown 或可打印 HTML 格式（可导出为 PDF）。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_openapi::doc_pdf::ApiDocPdf;
//!
//! let doc = ApiDocPdf::from_openapi(&openapi);
//! let md = doc.to_markdown();
//! let html = doc.to_html();
//! ```

use std::{collections::HashMap, fmt::Write};

use serde::{Deserialize, Serialize};

use crate::openapi::OpenApi;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// API documentation document
/// API 文档文档
#[derive(Debug, Clone)]
pub struct ApiDocPdf
{
    /// Document title
    /// 文档标题
    pub title: String,

    /// API version
    /// API 版本
    pub version: String,

    /// Description
    /// 描述
    pub description: Option<String>,

    /// Sections (one per tag / group)
    /// 章节（每个标签 / 分组一个）
    pub sections: Vec<DocSection>,

    /// Contact information
    /// 联系信息
    pub contact: Option<ContactDoc>,

    /// License information
    /// 许可证信息
    pub license: Option<LicenseDoc>,
}

/// Document section (grouped by tag)
/// 文档章节（按标签分组）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSection
{
    /// Section title
    /// 章节标题
    pub title: String,

    /// Section description
    /// 章节描述
    pub description: Option<String>,

    /// Endpoints in this section
    /// 本章节的端点
    pub endpoints: Vec<EndpointDoc>,
}

/// Endpoint documentation
/// 端点文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDoc
{
    /// HTTP method
    /// HTTP 方法
    pub method: String,

    /// Path
    /// 路径
    pub path: String,

    /// Summary
    /// 摘要
    pub summary: Option<String>,

    /// Description
    /// 描述
    pub description: Option<String>,

    /// Operation ID
    /// 操作 ID
    pub operation_id: Option<String>,

    /// Parameters
    /// 参数
    pub parameters: Vec<ParamDoc>,

    /// Request body description / schema
    /// 请求体描述 / 模式
    pub request_body: Option<String>,

    /// Responses
    /// 响应
    pub responses: Vec<ResponseDoc>,

    /// Deprecated flag
    /// 是否已弃用
    pub deprecated: bool,
}

/// Parameter documentation
/// 参数文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDoc
{
    /// Parameter name
    /// 参数名
    pub name: String,

    /// Location (path, query, header, cookie)
    /// 位置（path, query, header, cookie）
    pub location: String,

    /// Description
    /// 描述
    pub description: Option<String>,

    /// Required flag
    /// 是否必需
    pub required: bool,

    /// Type
    /// 类型
    pub type_: Option<String>,
}

/// Response documentation
/// 响应文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDoc
{
    /// Status code
    /// 状态码
    pub code: String,

    /// Description
    /// 描述
    pub description: String,
}

/// Contact documentation
/// 联系文档
#[derive(Debug, Clone)]
pub struct ContactDoc
{
    /// Name
    /// 名称
    pub name: Option<String>,

    /// Email
    /// 邮箱
    pub email: Option<String>,

    /// URL
    /// URL
    pub url: Option<String>,
}

/// License documentation
/// 许可证文档
#[derive(Debug, Clone)]
pub struct LicenseDoc
{
    /// License name
    /// 许可证名称
    pub name: String,

    /// License URL
    /// 许可证 URL
    pub url: Option<String>,
}

// ---------------------------------------------------------------------------
// ApiDocPdf methods
// ---------------------------------------------------------------------------

impl ApiDocPdf
{
    /// Generate documentation from an `OpenApi` specification
    /// 从 `OpenApi` 规范生成文档
    pub fn from_openapi(openapi: &OpenApi) -> Self
    {
        let mut tag_sections: HashMap<String, DocSection> = HashMap::new();

        // Pre-populate sections from tags
        // 从标签预填充章节
        for tag in &openapi.tags
        {
            tag_sections.insert(tag.name.clone(), DocSection {
                title: tag.name.clone(),
                description: tag.description.clone(),
                endpoints: Vec::new(),
            });
        }

        // Walk paths and operations
        // 遍历路径和操作
        for (path_str, path_item) in &openapi.paths
        {
            let ops: Vec<(&str, &crate::Operation)> = vec![
                ("GET", path_item.get.as_ref()),
                ("POST", path_item.post.as_ref()),
                ("PUT", path_item.put.as_ref()),
                ("DELETE", path_item.delete.as_ref()),
                ("PATCH", path_item.patch.as_ref()),
                ("HEAD", path_item.head.as_ref()),
                ("OPTIONS", path_item.options.as_ref()),
                ("TRACE", path_item.trace.as_ref()),
            ]
            .into_iter()
            .filter_map(|(m, o)| o.map(|op| (m, op)))
            .collect();

            for (method, op) in ops
            {
                let params: Vec<ParamDoc> = path_item
                    .parameters
                    .iter()
                    .chain(op.parameters.iter())
                    .map(|p| ParamDoc {
                        name: p.name.clone(),
                        location: format!("{:?}", p.location).to_lowercase(),
                        description: p.description.clone(),
                        required: p.required,
                        type_: p.schema.as_ref().map(|s| {
                            s.schema_type.as_ref().map_or_else(
                                || "any".to_string(),
                                |t| format!("{:?}", t).to_lowercase(),
                            )
                        }),
                    })
                    .collect();

                let mut responses: Vec<ResponseDoc> = op
                    .responses
                    .iter()
                    .map(|(code, resp)| ResponseDoc {
                        code: code.clone(),
                        description: resp.description.clone(),
                    })
                    .collect();
                responses.sort_by(|a, b| {
                    let na: u16 = a.code.parse().unwrap_or(999);
                    let nb: u16 = b.code.parse().unwrap_or(999);
                    na.cmp(&nb)
                });

                let request_body = op.request_body.as_ref().map(|body| {
                    let mut desc_parts = Vec::new();
                    if let Some(d) = &body.description
                    {
                        desc_parts.push(d.clone());
                    }
                    for (ct, media) in &body.content
                    {
                        desc_parts.push(format!("Content-Type: {}", ct));
                        if let Some(schema) = &media.schema
                        {
                            desc_parts.push(format!("Schema: {}", describe_schema(schema)));
                        }
                    }
                    desc_parts.join("\n")
                });

                let endpoint = EndpointDoc {
                    method: method.to_string(),
                    path: path_str.clone(),
                    summary: op.summary.clone(),
                    description: op.description.clone(),
                    operation_id: op.operation_id.clone(),
                    parameters: params,
                    request_body,
                    responses,
                    deprecated: op.deprecated,
                };

                let tag = op
                    .tags
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());

                tag_sections
                    .entry(tag.clone())
                    .or_insert_with(|| DocSection {
                        title: tag.clone(),
                        description: None,
                        endpoints: Vec::new(),
                    })
                    .endpoints
                    .push(endpoint);
            }
        }

        // Build ordered sections
        // 构建有序章节
        let mut sections = Vec::new();
        for tag in &openapi.tags
        {
            if let Some(section) = tag_sections.remove(&tag.name)
            {
                sections.push(section);
            }
        }
        // Remaining sections not in tag order
        // 不在标签顺序中的剩余章节
        for (_, section) in tag_sections
        {
            sections.push(section);
        }

        Self {
            title: openapi.info.title.clone(),
            version: openapi.info.version.clone(),
            description: openapi.info.description.clone(),
            sections,
            contact: openapi.info.contact.as_ref().map(|c| ContactDoc {
                name: c.name.clone(),
                email: c.email.clone(),
                url: c.url.clone(),
            }),
            license: openapi.info.license.as_ref().map(|l| LicenseDoc {
                name: l.name.clone(),
                url: l.url.clone(),
            }),
        }
    }

    /// Generate Markdown documentation
    /// 生成 Markdown 文档
    pub fn to_markdown(&self) -> String
    {
        let mut md = String::new();

        // Title
        // 标题
        let _ = writeln!(md, "# {}\n", self.title);
        let _ = writeln!(md, "**Version:** {}\n", self.version);

        if let Some(desc) = &self.description
        {
            let _ = writeln!(md, "{}\n", desc);
        }

        // Contact & License
        // 联系 & 许可证
        if let Some(contact) = &self.contact
        {
            md.push_str("## Contact\n\n");
            if let Some(name) = &contact.name
            {
                let _ = writeln!(md, "- **Name:** {}", name);
            }
            if let Some(email) = &contact.email
            {
                let _ = writeln!(md, "- **Email:** {}", email);
            }
            if let Some(url) = &contact.url
            {
                let _ = writeln!(md, "- **URL:** {}", url);
            }
            md.push('\n');
        }

        if let Some(license) = &self.license
        {
            let _ = writeln!(md, "## License\n\n{}\n", license.name);
            if let Some(url) = &license.url
            {
                let _ = writeln!(md, "[License URL]({})\n", url);
            }
        }

        // Table of Contents
        // 目录
        md.push_str("## Table of Contents\n\n");
        for (i, section) in self.sections.iter().enumerate()
        {
            let _ = writeln!(md, "{}. {}", i + 1, section.title);
        }
        md.push('\n');

        // Sections
        // 章节
        for section in &self.sections
        {
            let _ = writeln!(md, "## {}\n", section.title);
            if let Some(desc) = &section.description
            {
                let _ = writeln!(md, "{}\n", desc);
            }

            for endpoint in &section.endpoints
            {
                let _ = writeln!(md, "### {} `{}`\n", endpoint.method, endpoint.path);

                if endpoint.deprecated
                {
                    md.push_str("> **DEPRECATED**\n>\n");
                }

                if let Some(summary) = &endpoint.summary
                {
                    let _ = writeln!(md, "**{}**\n", summary);
                }
                if let Some(desc) = &endpoint.description
                {
                    let _ = writeln!(md, "{}\n", desc);
                }
                if let Some(op_id) = &endpoint.operation_id
                {
                    let _ = writeln!(md, "**Operation ID:** `{}`\n", op_id);
                }

                // Parameters
                // 参数
                if !endpoint.parameters.is_empty()
                {
                    md.push_str("#### Parameters\n\n");
                    md.push_str("| Name | In | Required | Type | Description |\n");
                    md.push_str("|------|----|----------|------|-------------|\n");
                    for p in &endpoint.parameters
                    {
                        let desc = p.description.as_deref().unwrap_or("-");
                        let type_ = p.type_.as_deref().unwrap_or("-");
                        let _ = writeln!(
                            md,
                            "| `{}` | {} | {} | {} | {} |\n",
                            p.name, p.location, p.required, type_, desc
                        );
                    }
                    md.push('\n');
                }

                // Request body
                // 请求体
                if let Some(body) = &endpoint.request_body
                {
                    md.push_str("#### Request Body\n\n");
                    md.push_str("```text\n");
                    md.push_str(body);
                    md.push_str("\n```\n\n");
                }

                // Responses
                // 响应
                if !endpoint.responses.is_empty()
                {
                    md.push_str("#### Responses\n\n");
                    md.push_str("| Code | Description |\n");
                    md.push_str("|------|-------------|\n");
                    for r in &endpoint.responses
                    {
                        let _ = writeln!(md, "| `{}` | {} |", r.code, r.description);
                    }
                    md.push('\n');
                }
            }
        }

        md
    }

    /// Generate printable HTML documentation (can be exported to PDF)
    /// 生成可打印 HTML 文档（可导出为 PDF）
    pub fn to_html(&self) -> String
    {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n<head>\n");
        let _ = writeln!(html, "  <title>{}</title>", escape_html(&self.title));
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("  <style>\n");
        html.push_str(STYLESHEET);
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");

        // Header
        // 头部
        let _ =
            writeln!(html, "  <div class=\"header\">\n    <h1>{}</h1>\n", escape_html(&self.title));
        let _ =
            writeln!(html, "    <p class=\"version\">Version: {}</p>", escape_html(&self.version));
        html.push_str("  </div>\n");

        // Description
        // 描述
        if let Some(desc) = &self.description
        {
            let _ =
                writeln!(html, "  <div class=\"description\"><p>{}</p></div>", escape_html(desc));
        }

        // Table of Contents
        // 目录
        html.push_str("  <div class=\"toc\">\n    <h2>Table of Contents</h2>\n    <ol>\n");
        for section in &self.sections
        {
            let _ = writeln!(
                html,
                "      <li><a href=\"#section-{}\">{}</a></li>\n",
                slug(&section.title),
                escape_html(&section.title)
            );
        }
        html.push_str("    </ol>\n  </div>\n");

        // Sections
        // 章节
        for section in &self.sections
        {
            let _ = writeln!(
                html,
                "  <div class=\"section\" id=\"section-{}\">\n    <h2>{}</h2>\n",
                slug(&section.title),
                escape_html(&section.title)
            );

            if let Some(desc) = &section.description
            {
                let _ = writeln!(html, "    <p class=\"section-desc\">{}</p>", escape_html(desc));
            }

            for endpoint in &section.endpoints
            {
                let method_class = endpoint.method.to_lowercase();
                html.push_str("    <div class=\"endpoint\">\n");
                let _ = writeln!(
                    html,
                    "      <h3><span class=\"method method-{}\">{}</span> <code>{}</code></h3>\n",
                    method_class,
                    endpoint.method,
                    escape_html(&endpoint.path)
                );

                if endpoint.deprecated
                {
                    html.push_str("      <p class=\"deprecated\">DEPRECATED</p>\n");
                }

                if let Some(summary) = &endpoint.summary
                {
                    let _ = writeln!(
                        html,
                        "      <p class=\"summary\"><strong>{}</strong></p>",
                        escape_html(summary)
                    );
                }
                if let Some(desc) = &endpoint.description
                {
                    let _ = writeln!(
                        html,
                        "      <p class=\"endpoint-desc\">{}</p>",
                        escape_html(desc)
                    );
                }

                // Parameters table
                // 参数表格
                if !endpoint.parameters.is_empty()
                {
                    html.push_str("      <table class=\"params\">\n");
                    html.push_str(
                        "        <thead><tr><th>Name</th><th>In</th><th>Required</th><th>Type</\
                         th><th>Description</th></tr></thead>\n",
                    );
                    html.push_str("        <tbody>\n");
                    for p in &endpoint.parameters
                    {
                        let desc = escape_html(p.description.as_deref().unwrap_or("-"));
                        let type_ = escape_html(p.type_.as_deref().unwrap_or("-"));
                        let req = if p.required { "Yes" } else { "No" };
                        let _ = writeln!(
                            html,
                            "          <tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</\
                             td><td>{}</td></tr>\n",
                            escape_html(&p.name),
                            p.location,
                            req,
                            type_,
                            desc
                        );
                    }
                    html.push_str("        </tbody>\n      </table>\n");
                }

                // Request body
                // 请求体
                if let Some(body) = &endpoint.request_body
                {
                    html.push_str("      <div class=\"request-body\">\n");
                    html.push_str("        <h4>Request Body</h4>\n");
                    let _ = writeln!(html, "        <pre>{}</pre>", escape_html(body));
                    html.push_str("      </div>\n");
                }

                // Responses table
                // 响应表格
                if !endpoint.responses.is_empty()
                {
                    html.push_str("      <table class=\"responses\">\n");
                    html.push_str(
                        "        <thead><tr><th>Code</th><th>Description</th></tr></thead>\n",
                    );
                    html.push_str("        <tbody>\n");
                    for r in &endpoint.responses
                    {
                        let _ = writeln!(
                            html,
                            "          <tr><td><code>{}</code></td><td>{}</td></tr>\n",
                            escape_html(&r.code),
                            escape_html(&r.description)
                        );
                    }
                    html.push_str("        </tbody>\n      </table>\n");
                }

                html.push_str("    </div>\n");
            }

            html.push_str("  </div>\n");
        }

        // Footer
        // 页脚
        html.push_str("  <div class=\"footer\">\n");
        let _ = writeln!(
            html,
            "    <p>Generated by Hiver OpenAPI | API Documentation v{}</p>",
            escape_html(&self.version)
        );
        html.push_str("  </div>\n");

        html.push_str("</body>\n</html>\n");
        html
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Describe a schema as a human-readable string
/// 将模式描述为可读字符串
fn describe_schema(schema: &crate::Schema) -> String
{
    if let Some(ref_) = &schema.ref_
    {
        return ref_.clone();
    }
    match schema.schema_type.as_ref()
    {
        Some(t) => format!("{:?}", t).to_lowercase(),
        None => "object".to_string(),
    }
}

/// Escape HTML special characters
/// 转义 HTML 特殊字符
fn escape_html(s: &str) -> String
{
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Create a URL-safe slug from text
/// 从文本创建 URL 安全的 slug
fn slug(text: &str) -> String
{
    text.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric()
            {
                c.to_ascii_lowercase()
            }
            else
            {
                '-'
            }
        })
        .collect()
}

/// Inline CSS stylesheet for generated HTML
/// 生成的 HTML 内联 CSS 样式表
const STYLESHEET: &str = r#"
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
      max-width: 960px;
      margin: 0 auto;
      padding: 40px 20px;
      color: #24292e;
      line-height: 1.6;
    }
    .header {
      border-bottom: 2px solid #e1e4e8;
      padding-bottom: 16px;
      margin-bottom: 24px;
    }
    .header h1 { margin: 0; }
    .version { color: #586069; margin-top: 4px; }
    .description { margin-bottom: 24px; }
    .toc { background: #f6f8fa; border: 1px solid #e1e4e8; border-radius: 6px; padding: 16px 24px; margin-bottom: 32px; }
    .toc h2 { margin-top: 0; font-size: 1.2em; }
    .section { margin-bottom: 32px; }
    .section h2 { border-bottom: 1px solid #e1e4e8; padding-bottom: 8px; }
    .endpoint { margin: 16px 0; padding: 12px 16px; background: #ffffff; border: 1px solid #e1e4e8; border-radius: 6px; }
    .method { display: inline-block; padding: 2px 8px; border-radius: 3px; color: #fff; font-size: 0.85em; font-weight: bold; min-width: 60px; text-align: center; }
    .method-get    { background: #61affe; }
    .method-post   { background: #49cc90; }
    .method-put    { background: #fca130; }
    .method-delete { background: #f93e3e; }
    .method-patch  { background: #50e3c2; }
    .method-head   { background: #9012fe; }
    .method-options{ background: #0d5aa7; }
    .method-trace  { background: #6e7681; }
    .deprecated { color: #d73a49; font-weight: bold; }
    table { width: 100%; border-collapse: collapse; margin: 8px 0; }
    th, td { text-align: left; padding: 6px 12px; border-bottom: 1px solid #e1e4e8; }
    th { background: #f6f8fa; font-weight: 600; }
    code { background: #f6f8fa; padding: 2px 6px; border-radius: 3px; font-size: 0.9em; }
    pre { background: #f6f8fa; border: 1px solid #e1e4e8; border-radius: 6px; padding: 12px; overflow-x: auto; }
    .footer { margin-top: 40px; padding-top: 16px; border-top: 1px solid #e1e4e8; color: #586069; font-size: 0.85em; }
    .request-body h4 { margin-bottom: 4px; }
"#;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;
    use crate::{
        OpenApiBuilder, Operation, Parameter, PathItem, RequestBody, Response, operation::MediaType,
    };

    #[test]
    fn test_api_doc_pdf_from_openapi()
    {
        let openapi = OpenApiBuilder::new()
            .title("Pet Store API")
            .version("1.0.0")
            .description("A sample pet store API")
            .add_path(
                "/pets",
                PathItem::new().get(
                    Operation::new()
                        .summary("List all pets")
                        .add_tag("pets")
                        .add_parameter(Parameter::query("limit").description("Max results"))
                        .add_response("200", Response::ok("A list of pets"))
                        .add_response("401", Response::unauthorized("Unauthorized")),
                ),
            )
            .add_path(
                "/pets/{id}",
                PathItem::new().get(
                    Operation::new()
                        .summary("Get pet by ID")
                        .add_tag("pets")
                        .add_parameter(Parameter::path("id").description("Pet ID"))
                        .add_response("200", Response::ok("Pet found"))
                        .add_response("404", Response::not_found("Pet not found")),
                ),
            )
            .build();

        let doc = ApiDocPdf::from_openapi(&openapi);

        assert_eq!(doc.title, "Pet Store API");
        assert_eq!(doc.version, "1.0.0");
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].title, "pets");
        assert_eq!(doc.sections[0].endpoints.len(), 2);
    }

    #[test]
    fn test_to_markdown()
    {
        let openapi = OpenApiBuilder::new()
            .title("Test API")
            .version("2.0.0")
            .add_path(
                "/health",
                PathItem::new().get(
                    Operation::new()
                        .summary("Health check")
                        .add_response("200", Response::ok("OK")),
                ),
            )
            .build();

        let doc = ApiDocPdf::from_openapi(&openapi);
        let md = doc.to_markdown();

        assert!(md.contains("# Test API"));
        assert!(md.contains("**Version:** 2.0.0"));
        assert!(md.contains("### GET `/health`"));
        assert!(md.contains("Health check"));
        assert!(md.contains("`200`"));
    }

    #[test]
    fn test_to_html()
    {
        let openapi = OpenApiBuilder::new()
            .title("Test API")
            .version("1.0.0")
            .add_path(
                "/users",
                PathItem::new().post(
                    Operation::new()
                        .summary("Create user")
                        .add_tag("users")
                        .add_parameter(Parameter::path("id"))
                        .request_body(
                            RequestBody::new()
                                .description("New user")
                                .add_content("application/json", MediaType::new()),
                        )
                        .add_response("201", Response::created("User created")),
                ),
            )
            .build();

        let doc = ApiDocPdf::from_openapi(&openapi);
        let html = doc.to_html();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test API</title>"));
        assert!(html.contains("method-post"));
        assert!(html.contains("/users"));
        assert!(html.contains("Request Body"));
        assert!(html.contains("<code>201</code>"));
    }

    #[test]
    fn test_html_method_colors()
    {
        let openapi = OpenApiBuilder::new()
            .title("Color Test")
            .version("1.0.0")
            .add_path(
                "/items/{id}",
                PathItem::new()
                    .get(Operation::new().summary("GET").add_tag("items"))
                    .put(Operation::new().summary("PUT").add_tag("items"))
                    .delete(Operation::new().summary("DELETE").add_tag("items"))
                    .patch(Operation::new().summary("PATCH").add_tag("items")),
            )
            .build();

        let doc = ApiDocPdf::from_openapi(&openapi);
        let html = doc.to_html();

        assert!(html.contains("method-get"));
        assert!(html.contains("method-put"));
        assert!(html.contains("method-delete"));
        assert!(html.contains("method-patch"));
    }

    #[test]
    fn test_deprecated_endpoint()
    {
        let openapi = OpenApiBuilder::new()
            .title("Deprecated Test")
            .version("1.0.0")
            .add_path(
                "/old",
                PathItem::new().get(
                    Operation::new()
                        .summary("Old endpoint")
                        .deprecated(true)
                        .add_response("200", Response::ok("OK")),
                ),
            )
            .build();

        let doc = ApiDocPdf::from_openapi(&openapi);
        let md = doc.to_markdown();
        let html = doc.to_html();

        assert!(md.contains("DEPRECATED"));
        assert!(html.contains("deprecated"));
    }

    #[test]
    fn test_escape_html()
    {
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;"
        );
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("quotes\"here"), "quotes&quot;here");
    }

    #[test]
    fn test_slug()
    {
        assert_eq!(slug("Users API"), "users-api");
        assert_eq!(slug("Pet Store!"), "pet-store-");
    }

    #[test]
    fn test_responses_sorted_by_code()
    {
        let openapi = OpenApiBuilder::new()
            .title("Sort Test")
            .version("1.0.0")
            .add_path(
                "/x",
                PathItem::new().get(
                    Operation::new()
                        .add_response("500", Response::internal_error("Server error"))
                        .add_response("200", Response::ok("OK"))
                        .add_response("404", Response::not_found("Not found")),
                ),
            )
            .build();

        let doc = ApiDocPdf::from_openapi(&openapi);
        let responses = &doc.sections[0].endpoints[0].responses;
        assert_eq!(responses[0].code, "200");
        assert_eq!(responses[1].code, "404");
        assert_eq!(responses[2].code, "500");
    }
}
