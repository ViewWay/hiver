//! HTTP connection context
//! HTTP 连接上下文

use super::{MAX_BUFFER_SIZE, MAX_HEADER_SIZE};

/// HTTP version
/// HTTP 版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum HttpVersion {
    /// HTTP/1.0
    Http10,
    /// HTTP/1.1
    #[default]
    Http11,
}

impl HttpVersion {
    /// Get the version string
    /// 获取版本字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
        }
    }

    /// Check if keep-alive is default for this version
    /// 检查此版本是否默认启用 keep-alive
    pub fn default_keep_alive(&self) -> bool {
        matches!(self, HttpVersion::Http11)
    }
}


/// Connection context for HTTP parsing/encoding
/// HTTP 解析/编码的连接上下文
#[derive(Debug, Clone)]
pub struct ConnectionContext {
    /// HTTP version
    version: HttpVersion,
    /// Maximum header size
    max_header_size: usize,
    /// Maximum buffer size
    max_buffer_size: usize,
    /// Keep-alive enabled
    keep_alive: bool,
}

impl Default for ConnectionContext {
    fn default() -> Self {
        Self {
            version: HttpVersion::Http11,
            max_header_size: MAX_HEADER_SIZE,
            max_buffer_size: MAX_BUFFER_SIZE,
            keep_alive: true,
        }
    }
}

impl ConnectionContext {
    /// Create a new connection context
    /// 创建新的连接上下文
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the HTTP version
    /// 获取 HTTP 版本
    pub fn version(&self) -> HttpVersion {
        self.version
    }

    /// Set the HTTP version
    /// 设置 HTTP 版本
    pub fn set_version(&mut self, version: HttpVersion) {
        self.version = version;
    }

    /// Get the maximum header size
    /// 获取最大头部大小
    pub fn max_header_size(&self) -> usize {
        self.max_header_size
    }

    /// Set the maximum header size
    /// 设置最大头部大小
    pub fn set_max_header_size(&mut self, size: usize) {
        self.max_header_size = size;
    }

    /// Get the maximum buffer size
    /// 获取最大缓冲区大小
    pub fn max_buffer_size(&self) -> usize {
        self.max_buffer_size
    }

    /// Set the maximum buffer size
    /// 设置最大缓冲区大小
    pub fn set_max_buffer_size(&mut self, size: usize) {
        self.max_buffer_size = size;
    }

    /// Check if keep-alive is enabled
    /// 检查是否启用 keep-alive
    pub fn keep_alive(&self) -> bool {
        self.keep_alive
    }

    /// Set keep-alive
    /// 设置 keep-alive
    pub fn set_keep_alive(&mut self, keep_alive: bool) {
        self.keep_alive = keep_alive;
    }

    /// Update keep-alive based on connection header
    /// 根据 connection 头更新 keep-alive
    pub fn update_keep_alive_from_header(&mut self, connection_header: Option<&str>) {
        if let Some(conn) = connection_header {
            let conn_lower = conn.to_lowercase();
            // "close" always disables keep-alive
            if conn_lower.contains("close") {
                self.keep_alive = false;
            }
            // "keep-alive" enables it (only matters for HTTP/1.0)
            else if conn_lower.contains("keep-alive") {
                self.keep_alive = true;
            }
        } else {
            // Default based on version
            self.keep_alive = self.version.default_keep_alive();
        }
    }
}
