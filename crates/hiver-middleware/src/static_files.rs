//! Static file serving middleware
//! 静态文件服务中间件
//!
//! This middleware provides static file serving capabilities.
//! 本中间件提供静态文件服务功能。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::fmt::Write as FmtWrite;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use hiver_http::{Body, Error, Request, Response, Result, StatusCode};
use hiver_router::{Middleware, Next};

/// Static file serving configuration
/// 静态文件服务配置
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_middleware::StaticFiles;
/// use std::sync::Arc;
///
/// let static_files = Arc::new(
///     StaticFiles::new("/static", "./public")
///         .with_index_file("index.html")
///         .with_spa_mode(true)
/// );
/// ```
#[derive(Clone)]
pub struct StaticFiles {
    /// URI prefix to serve files from (e.g., "/static")
    /// 服务文件的URI前缀（如 "/static"）
    uri_prefix: String,

    /// File system path to serve files from (e.g., "./public")
    /// 服务文件的文件系统路径（如 "./public"）
    base_path: PathBuf,

    /// Default file to serve when requesting a directory (e.g., "index.html")
    /// 请求目录时服务的默认文件（如 "index.html"）
    index_file: Option<String>,

    /// Enable SPA mode - redirect all paths to index.html
    /// 启用SPA模式 - 将所有路径重定向到 index.html
    spa_mode: bool,

    /// List of allowed file extensions (None = allow all)
    /// 允许的文件扩展名列表（None = 允许所有）
    allowed_extensions: Option<Vec<String>>,

    /// Whether to show directory listings
    /// 是否显示目录列表
    show_listing: bool,

    /// Cache control header (e.g., "public, max-age=3600")
    /// 缓存控制头（如 "public, max-age=3600"）
    cache_control: Option<String>,
}

impl StaticFiles {
    /// Create a new static file handler
    /// 创建新的静态文件处理器
    ///
    /// # Arguments / 参数
    ///
    /// * `uri_prefix` - URI prefix to serve files from (e.g., "/static")
    /// * `base_path` - File system path to serve files from (e.g., "./public")
    pub fn new(uri_prefix: impl Into<String>, base_path: impl Into<PathBuf>) -> Self {
        let uri_prefix = uri_prefix.into();
        Self {
            uri_prefix,
            base_path: base_path.into(),
            index_file: None,
            spa_mode: false,
            allowed_extensions: None,
            show_listing: false,
            cache_control: None,
        }
    }

    /// Set the index file name (e.g., "index.html")
    /// 设置索引文件名（如 "index.html"）
    pub fn with_index_file(mut self, file: impl Into<String>) -> Self {
        self.index_file = Some(file.into());
        self
    }

    /// Enable SPA mode - redirect all paths to index.html
    /// 启用SPA模式 - 将所有路径重定向到 index.html
    pub fn with_spa_mode(mut self, enabled: bool) -> Self {
        self.spa_mode = enabled;
        self
    }

    /// Set allowed file extensions (e.g., vec!["html", "css", "js"])
    /// 设置允许的文件扩展名（如 vec!["html", "css", "js"]）
    pub fn with_allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = Some(extensions);
        self
    }

    /// Enable directory listing
    /// 启用目录列表
    pub fn with_listing(mut self, enabled: bool) -> Self {
        self.show_listing = enabled;
        self
    }

    /// Set cache control header
    /// 设置缓存控制头
    pub fn with_cache_control(mut self, cache_control: impl Into<String>) -> Self {
        self.cache_control = Some(cache_control.into());
        self
    }

    /// Check if a path is allowed
    /// 检查路径是否被允许
    fn is_allowed(&self, path: &Path) -> bool {
        if let Some(ref allowed) = self.allowed_extensions {
            if let Some(ext) = path.extension() {
                allowed.contains(&ext.to_string_lossy().to_string())
            } else {
                false
            }
        } else {
            true
        }
    }

    /// Get content type for a file
    /// 获取文件的内容类型
    fn get_content_type(path: &Path) -> String {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
    }

    /// Serve a file
    /// 服务文件
    fn serve_file(&self, file_path: &Path) -> Result<Response> {
        // Read file contents
        let contents = fs::read(file_path)
            .map_err(|e| Error::internal(format!("Failed to read file: {}", e)))?;

        // Get content type
        let content_type = Self::get_content_type(file_path);

        // Build response
        let mut builder = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", content_type);

        // Add cache control if set
        if let Some(ref cache) = self.cache_control {
            builder = builder.header("cache-control", cache);
        }

        Ok(builder
            .body(Body::from(contents))
            .expect("unexpected error"))
    }

    /// Serve directory listing
    /// 服务目录列表
    fn serve_listing(&self, dir_path: &Path, request_path: &str) -> Result<Response> {
        let entries = fs::read_dir(dir_path)
            .map_err(|e| Error::internal(format!("Failed to read directory: {}", e)))?;

        let mut html = String::from(
            r"<!DOCTYPE html>
<html>
<head>
    <title>Directory Listing</title>
    <style>
        body { font-family: monospace; margin: 2rem; }
        a { color: #0066cc; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .parent { color: #0066cc; }
    </style>
</head>
<body>
    <h1>Index of ",
        );
        html.push_str(request_path);
        html.push_str(
            r"</h1>
    <hr>
    <ul>
",
        );

        // Parent directory link
        if request_path != self.uri_prefix {
            html.push_str(
                r#"        <li><a class="parent" href="../">../</a></li>
"#,
            );
        }

        // Directory entries
        for entry_result in entries {
            let entry = entry_result
                .map_err(|e| Error::internal(format!("Failed to read entry: {}", e)))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().is_ok_and(|ft| ft.is_dir());

            let suffix = if is_dir { "/" } else { "" };
            let _ = write!(
                html,
                r#"        <li><a href="{}{}">{}</a></li>
"#,
                name, suffix, name
            );
        }

        html.push_str(
            r"    </ul>
    <hr>
</body>
</html>
",
        );

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(html))
            .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)))
    }
}

impl<S> Middleware<S> for StaticFiles
where
    S: Send + Sync + 'static,
{
    fn call(
        &self,
        req: Request,
        state: Arc<S>,
        next: Next<S>,
    ) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> {
        let uri_prefix = self.uri_prefix.clone();
        let base_path = self.base_path.clone();
        let index_file = self.index_file.clone();
        let spa_mode = self.spa_mode;
        let show_listing = self.show_listing;
        let allowed_extensions = self.allowed_extensions.clone();
        let cache_control = self.cache_control.clone();

        Box::pin(async move {
            let path = req.path();

            // Check if path starts with URI prefix
            if !path.starts_with(&uri_prefix) {
                return next.call(req, state).await;
            }

            // Strip URI prefix to get relative path
            let relative_path = path[uri_prefix.len()..].trim_start_matches('/');

            // Build full file path
            let mut file_path = base_path.clone();
            if !relative_path.is_empty() {
                file_path.push(relative_path);
            }

            // Check for path traversal attacks
            if file_path.canonicalize().is_ok_and(|p| {
                !p.starts_with(
                    base_path
                        .canonicalize()
                        .unwrap_or_else(|_| base_path.clone()),
                )
            }) {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("Access denied"))
                    .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)));
            }

            // Check if file exists
            if !file_path.exists() {
                // SPA mode: serve index.html for non-existent files
                if spa_mode && let Some(ref index) = index_file {
                    file_path = base_path.clone();
                    file_path.push(index);
                    if file_path.exists() {
                        return Self {
                            uri_prefix,
                            base_path,
                            index_file,
                            spa_mode,
                            allowed_extensions,
                            show_listing,
                            cache_control,
                        }
                        .serve_file(&file_path);
                    }
                }
                return next.call(req, state).await;
            }

            // Check if it's a directory
            if file_path.is_dir() {
                // Try to serve index file
                if let Some(ref index) = index_file {
                    let index_path = file_path.join(index);
                    if index_path.exists() {
                        return Self {
                            uri_prefix,
                            base_path,
                            index_file,
                            spa_mode,
                            allowed_extensions,
                            show_listing,
                            cache_control,
                        }
                        .serve_file(&index_path);
                    }
                }

                // Serve directory listing if enabled
                if show_listing {
                    return Self {
                        uri_prefix,
                        base_path,
                        index_file,
                        spa_mode,
                        allowed_extensions,
                        show_listing,
                        cache_control,
                    }
                    .serve_listing(&file_path, path);
                }

                // Otherwise, try next
                return next.call(req, state).await;
            }

            // Check file extension
            let this = Self {
                uri_prefix,
                base_path,
                index_file,
                spa_mode,
                allowed_extensions,
                show_listing,
                cache_control,
            };

            if !this.is_allowed(&file_path) {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("File type not allowed"))
                    .unwrap_or_else(|_| Response::new(StatusCode::INTERNAL_SERVER_ERROR)));
            }

            // Serve the file
            this.serve_file(&file_path)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_files_creation() {
        let static_files = StaticFiles::new("/static", "./public");
        assert_eq!(static_files.uri_prefix, "/static");
        assert_eq!(static_files.base_path, PathBuf::from("./public"));
        assert!(static_files.index_file.is_none());
        assert!(!static_files.spa_mode);
    }

    #[test]
    fn test_static_files_builder() {
        let static_files = StaticFiles::new("/assets", "./dist")
            .with_index_file("index.html")
            .with_spa_mode(true)
            .with_allowed_extensions(vec!["html".into(), "css".into(), "js".into()])
            .with_listing(false)
            .with_cache_control("public, max-age=3600");

        assert_eq!(static_files.uri_prefix, "/assets");
        assert_eq!(static_files.index_file, Some("index.html".to_string()));
        assert!(static_files.spa_mode);
        assert!(static_files.allowed_extensions.is_some());
        assert!(!static_files.show_listing);
        assert_eq!(static_files.cache_control, Some("public, max-age=3600".to_string()));
    }

    #[test]
    fn test_is_allowed_with_no_restrictions() {
        let static_files = StaticFiles::new("/static", "./public");
        assert!(static_files.is_allowed(Path::new("test.html")));
        assert!(static_files.is_allowed(Path::new("test.exe")));
    }

    #[test]
    fn test_is_allowed_with_restrictions() {
        let static_files = StaticFiles::new("/static", "./public")
            .with_allowed_extensions(vec!["html".into(), "css".into()]);
        assert!(static_files.is_allowed(Path::new("test.html")));
        assert!(static_files.is_allowed(Path::new("test.css")));
        assert!(!static_files.is_allowed(Path::new("test.exe")));
        assert!(!static_files.is_allowed(Path::new("test")));
    }

    #[test]
    fn test_get_content_type() {
        assert_eq!(StaticFiles::get_content_type(Path::new("test.html")), "text/html");
        assert_eq!(StaticFiles::get_content_type(Path::new("test.css")), "text/css");
        assert_eq!(StaticFiles::get_content_type(Path::new("test.js")), "text/javascript");
        assert_eq!(StaticFiles::get_content_type(Path::new("test.json")), "application/json");
        assert_eq!(StaticFiles::get_content_type(Path::new("test.png")), "image/png");
    }
}
