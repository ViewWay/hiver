//! Security benchmarks
//! 安全模块基准测试
//!
//! # Equivalent to Spring Security / 等价于 Spring Security
//!
//! Measures the performance of JWT token creation, verification, and refresh,
//! as well as password encoding (BCrypt, PBKDF2) throughput.
//!
//! 测量 JWT token 创建、验证和刷新的性能，以及密码编码（BCrypt、PBKDF2）的吞吐量。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::time::Duration as StdDuration;

use hiver_security::{
    Authority, Role,
    JwtAlgorithm, JwtClaims, JwtTokenProvider, JwtUtil,
    PasswordEncoder, BcryptPasswordEncoder, Pbkdf2PasswordEncoder,
};

// ============================================================================
// JWT Token Creation Benchmarks / JWT Token 创建基准测试
// ============================================================================

/// Benchmark: JWT token creation with varying authority counts
/// 不同权限数量下 JWT token 创建的基准测试
fn bench_jwt_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_create");

    // Single authority (common case)
    // 单个权限（常见场景）
    group.bench_function("single_authority", |b| {
        let authorities = vec![Authority::Role(Role::User)];
        b.iter(|| {
            let token = JwtUtil::create_token(
                black_box("user-123"),
                black_box("alice"),
                black_box(&authorities),
            )
            .unwrap();
            black_box(token)
        });
    });

    // Multiple authorities
    // 多个权限
    group.bench_function("multiple_authorities", |b| {
        let authorities = vec![
            Authority::Role(Role::Admin),
            Authority::Role(Role::User),
            Authority::Permission("user:read".to_string()),
            Authority::Permission("user:write".to_string()),
            Authority::Permission("user:delete".to_string()),
        ];
        b.iter(|| {
            let token = JwtUtil::create_token(
                black_box("admin-456"),
                black_box("bob"),
                black_box(&authorities),
            )
            .unwrap();
            black_box(token)
        });
    });

    // Token provider with issuer and audience
    // 带签发者和受众的 Token 提供者
    group.bench_function("provider_with_issuer_audience", |b| {
        let provider = JwtTokenProvider::with_settings("bench-secret-key", 24)
            .with_issuer("hiver-bench")
            .with_audience("hiver-api");
        let authorities = vec![Authority::Role(Role::User)];
        b.iter(|| {
            let token = provider
                .generate_token(
                    black_box("user-123"),
                    black_box("alice"),
                    black_box(&authorities),
                )
                .unwrap();
            black_box(token)
        });
    });

    group.finish();
}

/// Benchmark: JWT token creation with claims builder
/// 使用 claims builder 创建 JWT token 的基准测试
fn bench_jwt_claims_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_create");

    group.bench_function("claims_builder", |b| {
        b.iter(|| {
            let claims = JwtClaims::builder("user-123", "alice")
                .authorities(&[Authority::Role(Role::User)])
                .expiration_hours(24)
                .issuer("hiver-bench")
                .audience("hiver-api")
                .jwt_id("unique-id-123")
                .custom_claim("department", serde_json::Value::String("engineering".to_string()))
                .build();
            black_box(claims)
        });
    });

    group.finish();
}

// ============================================================================
// JWT Token Verification Benchmarks / JWT Token 验证基准测试
// ============================================================================

/// Benchmark: JWT token verification throughput
/// JWT token 验证吞吐量基准测试
fn bench_jwt_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_verify");

    let authorities = vec![Authority::Role(Role::User)];
    let token = JwtUtil::create_token("user-123", "alice", &authorities).unwrap();

    // Verify valid token
    // 验证有效 token
    group.bench_function("verify_valid", |b| {
        b.iter(|| {
            let claims = JwtUtil::verify_token(black_box(&token)).unwrap();
            black_box(claims)
        });
    });

    // Validate via provider
    // 通过 provider 验证
    let provider = JwtTokenProvider::new();
    let provider_token = provider
        .generate_token("user-123", "alice", &authorities)
        .unwrap();

    group.bench_function("provider_validate", |b| {
        b.iter(|| {
            let valid = provider.validate_token(black_box(&provider_token)).unwrap();
            black_box(valid)
        });
    });

    // Decode and validate (full validation)
    // 解码并验证（完整验证）
    let provider_full = JwtTokenProvider::with_settings("bench-secret", 24)
        .with_issuer("hiver-security")
        .with_audience("hiver-api");
    let full_token = provider_full
        .generate_token("user-123", "alice", &authorities)
        .unwrap();

    group.bench_function("decode_and_validate_full", |b| {
        b.iter(|| {
            let claims = provider_full.decode_and_validate(black_box(&full_token)).unwrap();
            black_box(claims)
        });
    });

    // Decode without validation (fast path)
    // 不验证地解码（快速路径）
    group.bench_function("decode_without_validation", |b| {
        b.iter(|| {
            let claims = JwtUtil::decode_without_validation(black_box(&full_token)).unwrap();
            black_box(claims)
        });
    });

    group.finish();
}

/// Benchmark: JWT verification at scale
/// 大规模 JWT 验证的基准测试
fn bench_jwt_verify_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_verify_scale");

    let authorities = vec![Authority::Role(Role::User)];
    let provider = JwtTokenProvider::new();

    for count in [10usize, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_verify", count),
            count,
            |b, &count| {
                let tokens: Vec<String> = (0..count)
                    .map(|i| {
                        provider
                            .generate_token(
                                &format!("user-{}", i),
                                &format!("name-{}", i),
                                &authorities,
                            )
                            .unwrap()
                    })
                    .collect();
                b.iter(|| {
                    for token in &tokens {
                        let _ = provider.validate_token(black_box(token)).unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// JWT Algorithm Comparison / JWT 算法对比
// ============================================================================

/// Benchmark: JWT operations across different HMAC algorithms
/// 不同 HMAC 算法下的 JWT 操作基准测试
fn bench_jwt_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_algorithm");

    let authorities = vec![Authority::Role(Role::User)];

    for algo in [JwtAlgorithm::Hs256, JwtAlgorithm::Hs384, JwtAlgorithm::Hs512] {
        let algo_name = format!("{:?}", algo).to_lowercase();

        let provider = JwtTokenProvider::with_settings("bench-secret-key", 24)
            .with_algorithm(algo);

        group.bench_function(&format!("create_{}", algo_name), |b| {
            b.iter(|| {
                let token = provider
                    .generate_token(
                        black_box("user-123"),
                        black_box("alice"),
                        black_box(&authorities),
                    )
                    .unwrap();
                black_box(token)
            });
        });

        let token = provider
            .generate_token("user-123", "alice", &authorities)
            .unwrap();

        group.bench_function(&format!("verify_{}", algo_name), |b| {
            b.iter(|| {
                let claims = provider.decode_and_validate(black_box(&token)).unwrap();
                black_box(claims)
            });
        });
    }

    group.finish();
}

// ============================================================================
// JWT Token Refresh Benchmarks / JWT Token 刷新基准测试
// ============================================================================

/// Benchmark: JWT token refresh operations
/// JWT token 刷新操作的基准测试
fn bench_jwt_refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_refresh");

    let authorities = vec![Authority::Role(Role::User)];

    // Refresh token
    // 刷新 token
    let token = JwtUtil::create_token("user-123", "alice", &authorities).unwrap();
    group.bench_function("refresh_token", |b| {
        // Each iteration creates a fresh token to avoid expiry issues
        // 每次迭代创建新 token 以避免过期问题
        let fresh_token = JwtUtil::create_token("user-123", "alice", &authorities).unwrap();
        b.iter(|| {
            let new_token = JwtUtil::refresh_token(black_box(&fresh_token)).unwrap();
            black_box(new_token)
        });
    });

    // Refresh if needed (no refresh needed)
    // 条件刷新（无需刷新）
    group.bench_function("refresh_if_needed_noop", |b| {
        let fresh_token = JwtUtil::create_token("user-123", "alice", &authorities).unwrap();
        b.iter(|| {
            let (returned, was_refreshed) =
                JwtUtil::refresh_if_needed(black_box(&fresh_token), 3600).unwrap();
            black_box((returned, was_refreshed))
        });
    });

    group.finish();
}

// ============================================================================
// Password Encoding Benchmarks / 密码编码基准测试
// ============================================================================

/// Benchmark: BCrypt password encoding throughput at different cost factors
/// 不同成本因子下 BCrypt 密码编码吞吐量的基准测试
///
/// BCrypt is intentionally slow — this benchmark helps choose the right cost.
/// BCrypt 是故意设计为慢速的 — 此基准测试帮助选择合适的成本因子。
fn bench_bcrypt_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_bcrypt_encode");

    for cost in [4u32, 8, 10, 12].iter() {
        group.bench_function(&format!("cost_{}", cost), |b| {
            let encoder = BcryptPasswordEncoder::with_cost(*cost);
            b.iter(|| {
                let hash = encoder.encode(black_box("password123"));
                black_box(hash)
            });
        });
    }

    group.finish();
}

/// Benchmark: BCrypt password verification throughput
/// BCrypt 密码验证吞吐量的基准测试
fn bench_bcrypt_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_bcrypt_verify");

    for cost in [4u32, 8, 10, 12].iter() {
        group.bench_function(&format!("cost_{}", cost), |b| {
            let encoder = BcryptPasswordEncoder::with_cost(*cost);
            let hash = encoder.encode("password123");
            b.iter(|| {
                let matches = encoder.matches(black_box("password123"), black_box(&hash));
                black_box(matches)
            });
        });
    }

    group.finish();
}

/// Benchmark: PBKDF2 password encoding throughput
/// PBKDF2 密码编码吞吐量的基准测试
fn bench_pbkdf2_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_pbkdf2_encode");

    for iterations in [1_000u32, 10_000, 100_000].iter() {
        group.bench_function(&format!("iterations_{}", iterations), |b| {
            let encoder = Pbkdf2PasswordEncoder::with_iterations(*iterations);
            b.iter(|| {
                let hash = encoder.encode(black_box("password123"));
                black_box(hash)
            });
        });
    }

    group.finish();
}

/// Benchmark: PBKDF2 password verification throughput
/// PBKDF2 密码验证吞吐量的基准测试
fn bench_pbkdf2_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_pbkdf2_verify");

    for iterations in [1_000u32, 10_000, 100_000].iter() {
        group.bench_function(&format!("iterations_{}", iterations), |b| {
            let encoder = Pbkdf2PasswordEncoder::with_iterations(*iterations);
            let hash = encoder.encode("password123");
            b.iter(|| {
                let matches = encoder.matches(black_box("password123"), black_box(&hash));
                black_box(matches)
            });
        });
    }

    group.finish();
}

// ============================================================================
// JWT Claims Inspection / JWT 声明检查
// ============================================================================

/// Benchmark: Claims authority checking overhead
/// 声明权限检查开销的基准测试
fn bench_claims_inspection(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_claims_inspect");

    let authorities = vec![
        Authority::Role(Role::Admin),
        Authority::Role(Role::User),
        Authority::Permission("user:read".to_string()),
        Authority::Permission("user:write".to_string()),
    ];
    let token = JwtUtil::create_token("user-123", "alice", &authorities).unwrap();
    let claims = JwtUtil::verify_token(&token).unwrap();

    group.bench_function("has_role", |b| {
        b.iter(|| {
            let has = claims.has_role(black_box(&Role::Admin));
            black_box(has)
        });
    });

    group.bench_function("has_authority", |b| {
        b.iter(|| {
            let has = claims.has_authority(black_box(&Authority::Permission("user:read".to_string())));
            black_box(has)
        });
    });

    group.bench_function("get_authorities", |b| {
        b.iter(|| {
            let auths = claims.get_authorities();
            black_box(auths)
        });
    });

    group.bench_function("is_expired", |b| {
        b.iter(|| {
            let expired = claims.is_expired();
            black_box(expired)
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Main / Criterion 主函数
// ============================================================================

fn configure_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(StdDuration::from_secs(5))
        .sample_size(100)
        .warm_up_time(StdDuration::from_secs(1))
}

criterion_group! {
    name = jwt_create;
    config = configure_criterion();
    targets =
        bench_jwt_create,
        bench_jwt_claims_builder,
}

criterion_group! {
    name = jwt_verify;
    config = configure_criterion();
    targets =
        bench_jwt_verify,
        bench_jwt_verify_scale,
}

criterion_group! {
    name = jwt_algo;
    config = configure_criterion();
    targets = bench_jwt_algorithms,
}

criterion_group! {
    name = jwt_refresh;
    config = configure_criterion();
    targets = bench_jwt_refresh,
}

criterion_group! {
    name = password_bcrypt;
    config = configure_criterion();
    targets =
        bench_bcrypt_encode,
        bench_bcrypt_verify,
}

criterion_group! {
    name = password_pbkdf2;
    config = configure_criterion();
    targets =
        bench_pbkdf2_encode,
        bench_pbkdf2_verify,
}

criterion_group! {
    name = jwt_inspect;
    config = configure_criterion();
    targets = bench_claims_inspection,
}

criterion_main!(
    jwt_create,
    jwt_verify,
    jwt_algo,
    jwt_refresh,
    password_bcrypt,
    password_pbkdf2,
    jwt_inspect,
);
