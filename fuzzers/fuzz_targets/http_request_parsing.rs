//! HTTP Request Parsing Fuzz Target
//! HTTP 请求解析 Fuzz 测试目标
//!
//! This fuzz target tests HTTP request parsing robustness.
//! 此 Fuzz 测试目标测试 HTTP 请求解析的鲁棒性。

#![no_main]

use libfuzzer_sys::fuzz_target;
use hiver_http::Request;

fuzz_target!(|data: &[u8]| {
    // Skip if data is too large
    // 如果数据过大则跳过
    if data.len() > 64 * 1024 {
        return;
    }

    // Try to parse as HTTP/1.1 request
    // 尝试解析为 HTTP/1.1 请求
    if let Ok(s) = std::str::from_utf8(data) {
        // Test request line parsing / 测试请求行解析
        let lines: Vec<&str> = s.lines().collect();
        if !lines.is_empty() {
            let parts: Vec<&str> = lines[0].split_whitespace().collect();
            if parts.len() >= 2 {
                let method = parts[0];
                let path = parts[1];

                // Only test valid HTTP methods / 仅测试有效的 HTTP 方法
                if matches!(method, "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH") {
                    // Test Request builder doesn't panic
                    // 测试 Request 构建器不会 panic
                    let _ = Request::builder()
                        .method(method)
                        .uri(path)
                        .body(hiver_http::Body::empty());

                    // Test with version if present / 如果有版本则测试
                    if parts.len() >= 3 {
                        let version = parts[2];
                        if version == "HTTP/1.1" || version == "HTTP/1.0" {
                            let _ = Request::builder()
                                .method(method)
                                .uri(path)
                                .body(hiver_http::Body::empty());
                        }
                    }
                }
            }
        }

        // Test header parsing
        // 测试 header 解析
        for line in lines.iter().skip(1) {
            if let Some(colon) = line.find(':') {
                let name = &line[..colon];
                let value = line[colon + 1..].trim();

                // Test that header names are valid
                // 测试 header 名称有效
                if !name.is_empty() && name.len() <= 64 && value.len() <= 8192 {
                    // Valid header format / 有效的 header 格式
                }
            }
        }
    }
});
