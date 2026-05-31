# Hiver Security Audit Report
# Hiver 安全审计报告

**Date / 日期**: 2026-05-30
**Status / 状态**: In Progress / 进行中
**Phase / 阶段**: Phase 8 - Data Layer / 数据层
**Last Audit Tool / 上次审计工具**: `cargo audit` (RUSTSEC database)

## Summary / 摘要

This document tracks security vulnerabilities found and fixed during Phase 7 production readiness.
本文档记录 Phase 7 生产就绪期间发现并修复的安全漏洞。

## Vulnerabilities Fixed / 已修复的漏洞

### 1. JWT Authentication Middleware API Compatibility (Bug #024)
**Status**: Fixed / 已修复
**Commit**: 572679b

- Rewrote `JwtAuthenticationMiddleware` to match current `Middleware` trait API
- Removed `async_trait` in favor of `Pin<Box<dyn Future>>` return type
- Fixed `Error` enum usage for unauthorized and internal server errors

### 2. RSA Marvin Attack Vulnerability (jsonwebtoken path)
**Status**: Fixed / 已修复
**Commit**: beb5606
**Advisory**: RUSTSEC-2023-0071

- Removed `rust_crypto` feature from `jsonwebtoken` dependency
- Switched to default crypto backend to eliminate RSA dependency path

### 3. ruint Unsoundness Vulnerability
**Status**: Fixed / 已修复
**Commit**: beb5606
**Advisory**: RUSTSEC-2025-0137

- Updated `ruint` from 1.16.0 to 1.17.2
- Updated `alloy` dependencies from 1.4 to 1.5

## Remaining Vulnerabilities / 剩余漏洞

### 1. hickory-proto CPU Exhaustion (NEW 2026-05-30)
**Status**: Awaiting Upstream Fix / 等待上游修复
**Advisory**: RUSTSEC-2026-0119
**Severity**: Medium
**Date**: 2026-05-01

**Dependency Tree**:
```
hickory-proto 0.24.4
└── trust-dns-resolver / hickory-dns
```

**Impact**: O(n²) name compression in DNS message encoding can cause CPU exhaustion.
This is a transitive dependency.

**Mitigation**: Update hickory-proto when upstream releases fix. No direct Hiver code affected.

### 2. rustls-webpki Certificate Validation Issues (NEW 2026-05-30)
**Status**: Awaiting Upstream Fix / 等待上游修复
**Advisories**: RUSTSEC-2026-0104, RUSTSEC-2026-0098, RUSTSEC-2026-0099
**Severity**: Medium-High
**Date**: 2026-04-14/22

**Dependency Tree**:
```
rustls-webpki 0.101.7
└── rustls / tokio-rustls / tungstenite
```

**Impact**: Three issues — reachable panic in CRL parsing, incorrect URI name constraint acceptance, wildcard name constraint bypass.

**Mitigation**: Update rustls and transitive dependencies when fixed versions are available.

### 3. tokio-tar File Smuggling (NEW 2026-05-30)
**Status**: Awaiting Upstream Fix / 等待上游修复
**Advisory**: RUSTSEC-2025-0111
**Severity**: Medium
**Date**: 2025-10-21

**Dependency Tree**:
```
tokio-tar 0.3.1
└── transitive dependency
```

**Impact**: PAX extended headers parsed incorrectly, allows file smuggling (path traversal).

**Mitigation**: Update tokio-tar or switch to alternative tar library when fix available.

### 4. RSA Marvin Attack (sqlx-mysql path)
**Status**: Awaiting Upstream Fix / 等待上游修复
**Advisory**: RUSTSEC-2023-0071
**Severity**: Medium (5.9)

**Dependency Tree**:
```
rsa 0.9.10
└── sqlx-mysql 0.8.6
    └── sqlx 0.8.6
```

**Impact**: The RSA Marvin Attack vulnerability affects MySQL database connections via the `sqlx` crate.
This is a transitive dependency and cannot be fixed directly in Hiver.

**Mitigation**:
- Use PostgreSQL instead of MySQL where possible
- Monitor for `sqlx` updates that address this vulnerability
- Consider using alternative MySQL libraries without RSA dependency

### 2. Unmaintained Dependencies (Warnings)
**Status**: Monitor / 监控中

| Crate | Version | Advisory | Impact |
|-------|---------|----------|--------|
| opentelemetry-jaeger | 0.22.0 | RUSTSEC-2025-0123 | Tracing / 可观测性 |
| rustls-pemfile | 1.0.4 | RUSTSEC-2025-0134 | TLS (transitive) |
| unic-char-property | 0.9.0 | RUSTSEC-2025-0081 | Unicode (tera) |
| unic-char-range | 0.9.0 | RUSTSEC-2025-0075 | Unicode (tera) |
| unic-common | 0.9.0 | RUSTSEC-2025-0080 | Unicode (tera) |
| unic-segment | 0.9.0 | RUSTSEC-2025-0074 | Unicode (tera) |
| unic-ucd-segment | 0.9.0 | RUSTSEC-2025-0104 | Unicode (tera) |
| unic-ucd-version | 0.9.0 | RUSTSEC-2025-0098 | Unicode (tera) |
| derivative | 2.2.0 | RUSTSEC-2024-0388 | Macros (alloy transitive) |
| paste | 1.0.15 | RUSTSEC-2024-0436 | Macros (alloy transitive) |

**Action Plan**:
- Monitor for updates to these dependencies
- Consider alternatives for `opentelemetry-jaeger` (use OTLP instead)
- Consider alternatives for `tera` template engine if unmaintained status persists

## Security Best Practices Implemented / 已实施的安全最佳实践

1. **JWT Authentication**: Proper token validation and error handling
2. **Password Hashing**: BCrypt with salt for password storage
3. **CORS**: Configurable CORS middleware
4. **Rate Limiting**: Built-in rate limiting middleware
5. **CSRF Protection**: CSRF token middleware (optional)
6. **Input Validation**: Request extractors with validation
7. **SQL Injection**: Parameterized queries via sqlx/sea-orm
8. **Secret Management**: Environment-based configuration

## Next Steps / 下一步

1. [ ] Update `rustls` / `tokio-rustls` to fix 3 rustls-webpki CVEs (RUSTSEC-2026-0104/0098/0099)
2. [ ] Update `hickory-proto` to fix CPU exhaustion (RUSTSEC-2026-0119)
3. [ ] Update or replace `tokio-tar` to fix file smuggling (RUSTSEC-2025-0111)
4. [ ] Monitor for `sqlx` updates addressing RSA vulnerability (RUSTSEC-2023-0071)
5. [ ] Replace `opentelemetry-jaeger` with `opentelemetry-otlp`
6. [ ] Consider alternative template engines to `tera`
7. [ ] Run fuzzing tests regularly (`cargo fuzz`)
8. [ ] Set up continuous security scanning in CI/CD (`cargo audit` in GitHub Actions)

## References / 参考

- [RustSec Advisory Database](https://rustsec.org/)
- [cargo-audit](https://github.com/RustSec/cargo-audit)
- [OWASP Rust Security](https://owasp.org/www-project-rust-security/)
