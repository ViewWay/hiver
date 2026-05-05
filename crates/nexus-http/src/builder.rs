//! URI Builder for constructing URLs
//! URI构建器，用于构建URL
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `UriComponentsBuilder`
//! - `ServletUriComponentsBuilder`
//! - `MvcUriComponentsBuilder`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::collections::HashMap;
use std::fmt::Write;

/// URI Components Builder
/// URI组件构建器
///
/// A builder for constructing URIs with path variables, query parameters,
/// and other components.
///
/// 用于构建带有路径变量、查询参数和其他组件的URI的构建器。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_http::UriBuilder;
///
/// // Build a simple URI
/// let uri = UriBuilder::new()
///     .scheme("https")
///     .host("example.com")
///     .path("/api/users")
///     .query_param("page", "1")
///     .build();
/// assert_eq!(uri.to_string(), "https://example.com/api/users?page=1");
///
/// // Build with path variables
/// let uri = UriBuilder::new()
///     .scheme("https")
///     .host("example.com")
///     .path_template("/api/users/{id}")
///     .path_var("id", "123")
///     .build();
/// assert_eq!(uri.to_string(), "https://example.com/api/users/123");
///
/// // Build from a base URI
/// let uri = UriBuilder::from_uri("https://example.com/api")
///     .path_segment("users")
///     .path_segment("123")
///     .query_param("details", "true")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct UriBuilder {
    /// URI scheme (e.g., http, https)
    /// URI协议（例如：http、https）
    scheme: Option<String>,

    /// User info (e.g., username:password)
    /// 用户信息（例如：username:password）
    user_info: Option<String>,

    /// Host (e.g., example.com)
    /// 主机（例如：example.com）
    host: Option<String>,

    /// Port number
    /// 端口号
    port: Option<u16>,

    /// Path segments
    /// 路径段
    path: Vec<String>,

    /// Path template (for variable substitution)
    /// 路径模板（用于变量替换）
    path_template: Option<String>,

    /// Path variables
    /// 路径变量
    path_vars: HashMap<String, String>,

    /// Query parameters
    /// 查询参数
    query_params: Vec<(String, String)>,

    /// Fragment
    /// 片段
    fragment: Option<String>,
}

/// Built URI
/// 构建的URI
///
/// Represents a complete URI that can be converted to a string.
/// 表示可以转换为字符串的完整URI。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri {
    /// The complete URI string
    /// 完整的URI字符串
    uri: String,
}

impl Uri {
    /// Get the URI as a string slice
    /// 获取URI的字符串切片
    pub fn as_str(&self) -> &str {
        &self.uri
    }

    /// Get the scheme
    /// 获取协议
    pub fn scheme(&self) -> Option<&str> {
        self.uri.split_once("://").map(|(s, _)| s)
    }

    /// Get the path (without scheme, host, port)
    /// 获取路径（不带协议、主机、端口）
    pub fn path(&self) -> &str {
        // Remove scheme if present
        let after_scheme = if let Some((_, rest)) = self.uri.split_once("://") {
            rest
        } else {
            &self.uri
        };

        // Find the first slash after host
        if let Some(slash_pos) = after_scheme.find('/') {
            let path_and_query = &after_scheme[slash_pos..];
            // Strip query and fragment
            let path_only = path_and_query
                .split_once('?')
                .map_or(path_and_query, |(p, _)| p);
            let path_only = path_only.split_once('#').map_or(path_only, |(p, _)| p);
            if path_only.is_empty() { "/" } else { path_only }
        } else {
            "/"
        }
    }

    /// Get the query string (without leading ?)
    /// 获取查询字符串（不带前导?）
    pub fn query(&self) -> Option<&str> {
        self.uri
            .split_once('?')
            .map(|(_, q)| q.split_once('#').map_or(q, |(query, _)| query))
    }

    /// Get the fragment (without leading #)
    /// 获取片段（不带前导#）
    pub fn fragment(&self) -> Option<&str> {
        self.uri.split_once('#').map(|(_, f)| f)
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl UriBuilder {
    /// Create a new URI builder
    /// 创建新的URI构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a URI builder from an existing URI string
    /// 从现有URI字符串创建URI构建器
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let builder = UriBuilder::from_uri("https://example.com/api?foo=bar#section");
    /// ```
    pub fn from_uri(uri: &str) -> Self {
        let mut builder = Self::new();

        // Parse scheme
        if let Some((scheme, rest)) = uri.split_once("://") {
            builder = builder.scheme(scheme);
            let mut remaining = rest;

            // Parse user info if present
            if let Some(at_pos) = remaining.find('@') {
                let before_host = &remaining[..at_pos];
                let (user_info, _host_part) =
                    before_host.split_once(':').unwrap_or((before_host, ""));
                builder = builder.user_info(user_info);
                remaining = &remaining[at_pos + 1..];
            }

            // Parse host and port
            let (host_part, path_part) = remaining.split_once('/').unwrap_or((remaining, ""));
            if let Some((host, port_str)) = host_part.split_once(':') {
                builder = builder.host(host);
                if let Ok(port) = port_str.parse::<u16>() {
                    builder = builder.port(port);
                }
            } else {
                builder = builder.host(host_part);
            }

            let remaining = if path_part.is_empty() { "" } else { path_part };

            // Parse fragment first
            let fragment_result = remaining.split_once('#');
            if let Some((before_fragment, frag)) = fragment_result {
                builder = builder.fragment(frag);
                let to_parse = before_fragment;

                // Then parse query from before fragment
                if let Some((path, q)) = to_parse.split_once('?') {
                    if !path.is_empty() {
                        builder.path = path
                            .split('/')
                            .filter(|s: &&str| !s.is_empty())
                            .map(std::string::ToString::to_string)
                            .collect();
                    }
                    for pair in q.split('&') {
                        let parts: Vec<&str> = pair.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            builder = builder.query_param(parts[0], parts[1]);
                        }
                    }
                } else if !to_parse.is_empty() {
                    builder.path = to_parse
                        .split('/')
                        .filter(|s: &&str| !s.is_empty())
                        .map(std::string::ToString::to_string)
                        .collect();
                }
            } else {
                // No fragment, parse query from remaining
                if let Some((path, q)) = remaining.split_once('?') {
                    if !path.is_empty() {
                        builder.path = path
                            .split('/')
                            .filter(|s: &&str| !s.is_empty())
                            .map(std::string::ToString::to_string)
                            .collect();
                    }
                    for pair in q.split('&') {
                        let parts: Vec<&str> = pair.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            builder = builder.query_param(parts[0], parts[1]);
                        }
                    }
                } else if !remaining.is_empty() {
                    builder.path = remaining
                        .split('/')
                        .filter(|s: &&str| !s.is_empty())
                        .map(std::string::ToString::to_string)
                        .collect();
                }
            }
        } else {
            // No scheme, treat as path
            builder.path = uri
                .split('/')
                .filter(|s: &&str| !s.is_empty())
                .map(std::string::ToString::to_string)
                .collect();
        }

        builder
    }

    /// Set the scheme (e.g., http, https)
    /// 设置协议（例如：http、https）
    pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
        self.scheme = Some(scheme.into());
        self
    }

    /// Set the user info (e.g., "username:password")
    /// 设置用户信息（例如："username:password"）
    pub fn user_info(mut self, user_info: impl Into<String>) -> Self {
        self.user_info = Some(user_info.into());
        self
    }

    /// Set the host
    /// 设置主机
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the port
    /// 设置端口
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the path
    /// 设置路径
    pub fn path(mut self, path: impl Into<String>) -> Self {
        let path_str = path.into();
        self.path = path_str
            .split('/')
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
            .collect();
        self
    }

    /// Add a path segment
    /// 添加路径段
    pub fn path_segment(mut self, segment: impl Into<String>) -> Self {
        self.path.push(segment.into());
        self
    }

    /// Add multiple path segments
    /// 添加多个路径段
    pub fn path_segments(mut self, segments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for segment in segments {
            self.path.push(segment.into());
        }
        self
    }

    /// Set the path template for variable substitution
    /// 设置路径模板用于变量替换
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// let uri = UriBuilder::new()
    ///     .scheme("https")
    ///     .host("example.com")
    ///     .path_template("/api/users/{id}")
    ///     .path_var("id", "123")
    ///     .build();
    /// // Result: "https://example.com/api/users/123"
    /// ```
    pub fn path_template(mut self, template: impl Into<String>) -> Self {
        self.path_template = Some(template.into());
        self
    }

    /// Add a path variable for substitution in the path template
    /// 添加路径变量用于在路径模板中替换
    pub fn path_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.path_vars.insert(name.into(), value.into());
        self
    }

    /// Add multiple path variables
    /// 添加多个路径变量
    pub fn path_vars(
        mut self,
        vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (name, value) in vars {
            self.path_vars.insert(name.into(), value.into());
        }
        self
    }

    /// Add a query parameter
    /// 添加查询参数
    pub fn query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.push((name.into(), value.into()));
        self
    }

    /// Add multiple query parameters
    /// 添加多个查询参数
    pub fn query_params(
        mut self,
        params: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (name, value) in params {
            self.query_params.push((name.into(), value.into()));
        }
        self
    }

    /// Set the fragment
    /// 设置片段
    pub fn fragment(mut self, fragment: impl Into<String>) -> Self {
        self.fragment = Some(fragment.into());
        self
    }

    /// Build the URI
    /// 构建URI
    pub fn build(self) -> Uri {
        let mut result = String::new();

        // Scheme
        if let Some(scheme) = &self.scheme {
            write!(result, "{}://", scheme).unwrap();
        }

        // User info
        if let Some(user_info) = &self.user_info {
            write!(result, "{}@", user_info).unwrap();
        }

        // Host
        if let Some(host) = &self.host {
            write!(result, "{}", host).unwrap();
        }

        // Port (only if non-standard for the scheme)
        if let Some(port) = self.port {
            let standard_port = match self.scheme.as_deref() {
                Some("http") => Some(80),
                Some("https") => Some(443),
                Some("ftp") => Some(21),
                _ => None,
            };
            if standard_port != Some(port) {
                write!(result, ":{}", port).unwrap();
            }
        }

        // Path
        let path = if let Some(template) = &self.path_template {
            // Substitute variables in template
            let mut substituted = template.clone();
            for (name, value) in &self.path_vars {
                substituted = substituted.replace(&format!("{{{}}}", name), value);
            }
            substituted
        } else {
            format!("/{}", self.path.join("/"))
        };

        // Add to result, handling empty path
        if !path.is_empty() && path != "/" {
            // Remove leading slash if we already have scheme://host
            let path_to_add = if self.scheme.is_some() && path.starts_with('/') {
                &path[1..]
            } else {
                &path
            };
            // Ensure leading slash after host
            if self.host.is_some() && !result.ends_with('/') && !path_to_add.starts_with('/') {
                result.push('/');
            }
            result.push_str(path_to_add);
        } else if self.host.is_some() {
            result.push('/');
        }

        // Query string
        if !self.query_params.is_empty() {
            result.push('?');
            for (i, (name, value)) in self.query_params.iter().enumerate() {
                if i > 0 {
                    result.push('&');
                }
                write!(result, "{}={}", name, value).unwrap();
            }
        }

        // Fragment
        if let Some(fragment) = &self.fragment {
            write!(result, "#{}", fragment).unwrap();
        }

        Uri { uri: result }
    }

    /// Replace all path variables at once
    /// 一次性替换所有路径变量
    ///
    /// This is a convenience method for setting all path variables from a `HashMap`.
    /// `这是用于从HashMap一次性设置所有路径变量的便捷方法`。
    pub fn replace_path_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.path_vars = vars;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_builder_simple() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path("/api/users")
            .build();

        assert_eq!(uri.as_str(), "https://example.com/api/users");
    }

    #[test]
    fn test_uri_builder_with_query() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path("/api/users")
            .query_param("page", "1")
            .query_param("limit", "10")
            .build();

        assert_eq!(uri.as_str(), "https://example.com/api/users?page=1&limit=10");
    }

    #[test]
    fn test_uri_builder_with_port() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .port(8443)
            .path("/api")
            .build();

        assert_eq!(uri.as_str(), "https://example.com:8443/api");
    }

    #[test]
    fn test_uri_builder_standard_port() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .port(443)
            .path("/api")
            .build();

        assert_eq!(uri.as_str(), "https://example.com/api");
    }

    #[test]
    fn test_uri_builder_with_fragment() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path("/api")
            .fragment("section")
            .build();

        assert_eq!(uri.as_str(), "https://example.com/api#section");
    }

    #[test]
    fn test_uri_builder_path_template() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path_template("/api/users/{id}")
            .path_var("id", "123")
            .build();

        assert_eq!(uri.as_str(), "https://example.com/api/users/123");
    }

    #[test]
    fn test_uri_builder_path_segments() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path_segment("api")
            .path_segment("users")
            .path_segment("123")
            .build();

        assert_eq!(uri.as_str(), "https://example.com/api/users/123");
    }

    #[test]
    fn test_uri_builder_from_uri() {
        let uri_str = "https://example.com:8443/api/users?page=1#section";
        let uri = UriBuilder::from_uri(uri_str).build();

        assert_eq!(uri.as_str(), uri_str);
    }

    #[test]
    fn test_uri_get_scheme() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path("/api")
            .build();

        assert_eq!(uri.scheme(), Some("https"));
    }

    #[test]
    fn test_uri_get_path() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path("/api/users")
            .query_param("page", "1")
            .fragment("section")
            .build();

        assert_eq!(uri.path(), "/api/users");
    }

    #[test]
    fn test_uri_get_query() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path("/api")
            .query_param("page", "1")
            .query_param("limit", "10")
            .fragment("section")
            .build();

        assert_eq!(uri.query(), Some("page=1&limit=10"));
    }

    #[test]
    fn test_uri_get_fragment() {
        let uri = UriBuilder::new()
            .scheme("https")
            .host("example.com")
            .path("/api")
            .fragment("section")
            .build();

        assert_eq!(uri.fragment(), Some("section"));
    }

    #[test]
    fn test_uri_builder_path_only() {
        let uri = UriBuilder::new().path("/api/users").build();

        assert_eq!(uri.as_str(), "/api/users");
    }
}
