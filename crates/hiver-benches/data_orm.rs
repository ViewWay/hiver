//! Data ORM benchmarks
//! 数据 ORM 基准测试
//!
//! # Equivalent to Spring Data / 等价于 Spring Data
//!
//! Measures the performance of ORM query building, SQL generation,
//! model metadata operations, and serialization overhead.
//!
//! 测量 ORM 查询构建、SQL 生成、模型元数据操作和序列化开销的性能。

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::time::Duration as StdDuration;

use hiver_data_orm::{
    Column, ColumnType, JoinType, Model, ModelMeta, OrderBy, OrderDirection, QueryBuilder,
    SqlDialect, WhereClause,
};
use hiver_data_rdbc::QueryParam;

// ============================================================================
// Mock Models / 模拟模型
// ============================================================================

/// User model for benchmarking
/// 用于基准测试的用户模型
#[derive(Debug, Clone)]
struct User;

impl Model for User {
    fn meta() -> ModelMeta {
        let mut meta = ModelMeta::new("users");
        meta.columns
            .push(Column::new("id", ColumnType::I64).primary_key());
        meta.columns.push(Column::new("name", ColumnType::String));
        meta.columns.push(Column::new("email", ColumnType::String));
        meta.columns.push(Column::new("age", ColumnType::I32));
        meta.columns.push(Column::new("active", ColumnType::Bool));
        meta.columns
            .push(Column::new("created_at", ColumnType::Timestamp));
        meta
    }

    fn primary_key(&self) -> hiver_data_orm::prelude::Result<String> {
        Ok("1".to_string())
    }

    fn set_primary_key(&mut self, _value: String) -> hiver_data_orm::prelude::Result<()> {
        Ok(())
    }
}

/// Product model for benchmarking (wider table)
/// 用于基准测试的产品模型（更宽的表）
#[derive(Debug, Clone)]
struct Product;

impl Model for Product {
    fn meta() -> ModelMeta {
        let mut meta = ModelMeta::new("products");
        meta.columns
            .push(Column::new("id", ColumnType::I64).primary_key());
        meta.columns.push(Column::new("name", ColumnType::String));
        meta.columns
            .push(Column::new("description", ColumnType::Text));
        meta.columns.push(Column::new("price", ColumnType::F64));
        meta.columns.push(Column::new("stock", ColumnType::I32));
        meta.columns
            .push(Column::new("category_id", ColumnType::I64));
        meta.columns
            .push(Column::new("sku", ColumnType::String).unique());
        meta.columns.push(Column::new("weight", ColumnType::F64));
        meta.columns.push(Column::new("active", ColumnType::Bool));
        meta.columns
            .push(Column::new("created_at", ColumnType::Timestamp));
        meta
    }

    fn primary_key(&self) -> hiver_data_orm::prelude::Result<String> {
        Ok("1".to_string())
    }

    fn set_primary_key(&mut self, _value: String) -> hiver_data_orm::prelude::Result<()> {
        Ok(())
    }
}

/// Order model for benchmarking (with relationships)
/// 用于基准测试的订单模型（带关系）
#[derive(Debug, Clone)]
struct Order;

impl Model for Order {
    fn meta() -> ModelMeta {
        let mut meta = ModelMeta::new("orders");
        meta.columns
            .push(Column::new("id", ColumnType::I64).primary_key());
        meta.columns.push(Column::new("user_id", ColumnType::I64));
        meta.columns.push(Column::new("total", ColumnType::F64));
        meta.columns.push(Column::new("status", ColumnType::String));
        meta.columns
            .push(Column::new("created_at", ColumnType::Timestamp));
        meta
    }

    fn primary_key(&self) -> hiver_data_orm::prelude::Result<String> {
        Ok("1".to_string())
    }

    fn set_primary_key(&mut self, _value: String) -> hiver_data_orm::prelude::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Query Building Benchmarks / 查询构建基准测试
// ============================================================================

/// Benchmark: Simple SELECT query building
/// 简单 SELECT 查询构建的基准测试
fn bench_query_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_query_build");

    // SELECT * FROM users
    // SELECT * FROM users
    group.bench_function("select_all", |b| {
        b.iter(|| {
            let (sql, params) = QueryBuilder::<User>::new().build();
            black_box((sql, params))
        });
    });

    // SELECT * FROM users WHERE active = true LIMIT 10
    // SELECT * FROM users WHERE active = true LIMIT 10
    group.bench_function("select_where_limit", |b| {
        b.iter(|| {
            let (sql, params) = QueryBuilder::<User>::new()
                .where_("active = ?", &[QueryParam::Bool(true)])
                .limit(10)
                .build();
            black_box((sql, params))
        });
    });

    // SELECT * FROM users WHERE age > ? AND active = ? ORDER BY name ASC
    // SELECT * FROM users WHERE age > ? AND active = ? ORDER BY name ASC
    group.bench_function("select_where_order", |b| {
        b.iter(|| {
            let (sql, params) = QueryBuilder::<User>::new()
                .where_("age > ?", &[QueryParam::I32(18)])
                .where_("active = ?", &[QueryParam::Bool(true)])
                .order_by("name")
                .asc()
                .build();
            black_box((sql, params))
        });
    });

    group.finish();
}

/// Benchmark: Complex query building (joins, groups, pagination)
/// 复杂查询构建的基准测试（JOIN、GROUP BY、分页）
fn bench_query_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_query_build");

    // JOIN query: orders + users
    // JOIN 查询：orders + users
    group.bench_function("select_join", |b| {
        b.iter(|| {
            let (sql, params) = QueryBuilder::<Order>::new()
                .join(JoinType::Inner, "users", "orders.user_id = users.id")
                .where_("orders.status = ?", &[QueryParam::Text("completed".into())])
                .order_by("created_at")
                .desc()
                .limit(50)
                .build();
            black_box((sql, params))
        });
    });

    // Multi-join with select and group by
    // 多 JOIN 带 SELECT 和 GROUP BY
    group.bench_function("select_multi_join_group", |b| {
        b.iter(|| {
            let (sql, params) = QueryBuilder::<Order>::new()
                .select(&[
                    "user_id",
                    "COUNT(*) as order_count",
                    "SUM(total) as total_spent",
                ])
                .join(JoinType::Inner, "users", "orders.user_id = users.id")
                .join(JoinType::Left, "products", "orders.id = products.id")
                .where_("orders.status = ?", &[QueryParam::Text("completed".into())])
                .group_by(&["user_id"])
                .order_by("total_spent")
                .desc()
                .limit(100)
                .build();
            black_box((sql, params))
        });
    });

    // Paginated query with offset
    // 带偏移的分页查询
    group.bench_function("select_paginated", |b| {
        b.iter(|| {
            let (sql, params) = QueryBuilder::<Product>::new()
                .where_("active = ?", &[QueryParam::Bool(true)])
                .where_("category_id = ?", &[QueryParam::I64(5)])
                .order_by("price")
                .asc()
                .limit(20)
                .offset(40)
                .build();
            black_box((sql, params))
        });
    });

    group.finish();
}

/// Benchmark: Query building throughput at scale
/// 大规模查询构建吞吐量的基准测试
fn bench_query_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_query_throughput");

    for n_wheres in [1usize, 3, 5, 10].iter() {
        group.throughput(Throughput::Elements(*n_wheres as u64));
        group.bench_with_input(
            BenchmarkId::new("where_clauses", n_wheres),
            n_wheres,
            |b, &n_wheres| {
                b.iter(|| {
                    let mut builder = QueryBuilder::<User>::new();
                    for i in 0..n_wheres {
                        builder = builder
                            .where_(&format!("field{} > ?", i), &[QueryParam::I32(i as i32)]);
                    }
                    let (sql, params) = builder.build();
                    black_box((sql, params))
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Model Metadata Benchmarks / 模型元数据基准测试
// ============================================================================

/// Benchmark: Model metadata creation and inspection
/// 模型元数据创建和检查的基准测试
fn bench_model_meta(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_model_meta");

    group.bench_function("user_meta_creation", |b| {
        b.iter(|| {
            let meta = User::meta();
            black_box(&meta)
        });
    });

    group.bench_function("product_meta_creation", |b| {
        b.iter(|| {
            let meta = Product::meta();
            black_box(&meta)
        });
    });

    group.bench_function("table_name", |b| {
        b.iter(|| {
            let name = User::table_name();
            black_box(name)
        });
    });

    group.finish();
}

/// Benchmark: Column type to SQL conversion across dialects
/// 列类型在不同 SQL 方言间的转换基准测试
fn bench_column_type_sql(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_column_sql");

    let types = vec![
        ("bool", ColumnType::Bool),
        ("i32", ColumnType::I32),
        ("i64", ColumnType::I64),
        ("string", ColumnType::String),
        ("text", ColumnType::Text),
        ("timestamp", ColumnType::Timestamp),
        ("json", ColumnType::Json),
    ];

    let dialects = vec![
        ("postgresql", SqlDialect::PostgreSQL),
        ("mysql", SqlDialect::MySQL),
        ("sqlite", SqlDialect::SQLite),
    ];

    for (type_name, col_type) in &types {
        for (dialect_name, dialect) in &dialects {
            group.bench_function(&format!("{}_{}", type_name, dialect_name), |b| {
                b.iter(|| {
                    let sql_type = col_type.as_sql(*dialect);
                    black_box(sql_type)
                });
            });
        }
    }

    group.finish();
}

// ============================================================================
// SQL Generation Benchmarks / SQL 生成基准测试
// ============================================================================

/// Benchmark: to_sql (inline values) vs build (parameterized)
/// to_sql（内联值）与 build（参数化）的基准测试对比
fn bench_sql_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_sql_gen");

    group.bench_function("build_parameterized", |b| {
        b.iter(|| {
            let (sql, params) = QueryBuilder::<User>::new()
                .where_("age > ?", &[QueryParam::I32(18)])
                .where_("active = ?", &[QueryParam::Bool(true)])
                .where_("name LIKE ?", &[QueryParam::Text("%alice%".into())])
                .order_by("created_at")
                .desc()
                .limit(20)
                .build();
            black_box((sql, params))
        });
    });

    group.bench_function("to_sql_inline", |b| {
        b.iter(|| {
            let sql = QueryBuilder::<User>::new()
                .where_("age > ?", &[QueryParam::I32(18)])
                .where_("active = ?", &[QueryParam::Bool(true)])
                .where_("name LIKE ?", &[QueryParam::Text("%alice%".into())])
                .order_by("created_at")
                .desc()
                .limit(20)
                .to_sql();
            black_box(sql)
        });
    });

    group.finish();
}

/// Benchmark: CRUD SQL construction patterns
/// CRUD SQL 构建模式的基准测试
fn bench_crud_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_crud_sql");

    // INSERT pattern (via raw SQL string building)
    // INSERT 模式（通过原始 SQL 字符串构建）
    group.bench_function("insert_sql_build", |b| {
        b.iter(|| {
            let cols = ["name", "email", "age", "active"];
            let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("${i}")).collect();
            let sql = format!(
                "INSERT INTO users ({}) VALUES ({}) RETURNING *",
                cols.join(", "),
                placeholders.join(", ")
            );
            black_box(sql)
        });
    });

    // UPDATE pattern
    // UPDATE 模式
    group.bench_function("update_sql_build", |b| {
        b.iter(|| {
            let cols = ["name", "email", "age", "active"];
            let set_parts: Vec<String> = cols
                .iter()
                .enumerate()
                .map(|(i, col)| format!("{} = ${}", col, i + 1))
                .collect();
            let sql = format!(
                "UPDATE users SET {} WHERE id = ${} RETURNING *",
                set_parts.join(", "),
                cols.len() + 1
            );
            black_box(sql)
        });
    });

    // DELETE pattern
    // DELETE 模式
    group.bench_function("delete_sql_build", |b| {
        b.iter(|| {
            let sql = "DELETE FROM users WHERE id = $1".to_string();
            black_box(sql)
        });
    });

    // SELECT by ID pattern
    // 按 ID SELECT 模式
    group.bench_function("select_by_id_sql_build", |b| {
        b.iter(|| {
            let sql = "SELECT * FROM users WHERE id = $1 LIMIT 1".to_string();
            black_box(sql)
        });
    });

    // COUNT pattern
    // COUNT 模式
    group.bench_function("count_sql_build", |b| {
        b.iter(|| {
            let sql = "SELECT COUNT(*) AS cnt FROM users".to_string();
            black_box(sql)
        });
    });

    group.finish();
}

// ============================================================================
// Serialization Overhead / 序列化开销
// ============================================================================

/// Benchmark: JSON serialization overhead for ORM-like objects
/// ORM 类对象的 JSON 序列化开销基准测试
fn bench_orm_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("orm_serialization");

    // Serialize a user-like struct
    // 序列化用户类结构体
    #[derive(serde::Serialize)]
    struct UserRow {
        id: i64,
        name: String,
        email: String,
        age: i32,
        active: bool,
    }

    let user = UserRow {
        id: 42,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        active: true,
    };

    group.bench_function("serialize_user", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&user)).unwrap();
            black_box(json)
        });
    });

    group.bench_function("serialize_user_pretty", |b| {
        b.iter(|| {
            let json = serde_json::to_string_pretty(black_box(&user)).unwrap();
            black_box(json)
        });
    });

    // Deserialize user from JSON
    // 从 JSON 反序列化用户
    let user_json = serde_json::to_string(&user).unwrap();
    group.bench_function("deserialize_user", |b| {
        b.iter(|| {
            let u: UserRow = serde_json::from_str(black_box(&user_json)).unwrap();
            black_box(u)
        });
    });

    // Serialize a list of 100 users
    // 序列化 100 个用户的列表
    let users: Vec<UserRow> = (0..100)
        .map(|i| UserRow {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@example.com", i),
            age: 20 + (i % 50) as i32,
            active: i % 3 != 0,
        })
        .collect();

    group.throughput(Throughput::Elements(100));
    group.bench_function("serialize_100_users", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&users)).unwrap();
            black_box(json)
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
    name = orm_query;
    config = configure_criterion();
    targets =
        bench_query_simple,
        bench_query_complex,
        bench_query_throughput,
}

criterion_group! {
    name = orm_meta;
    config = configure_criterion();
    targets =
        bench_model_meta,
        bench_column_type_sql,
}

criterion_group! {
    name = orm_sql;
    config = configure_criterion();
    targets =
        bench_sql_generation,
        bench_crud_patterns,
}

criterion_group! {
    name = orm_serialize;
    config = configure_criterion();
    targets = bench_orm_serialization,
}

criterion_main!(orm_query, orm_meta, orm_sql, orm_serialize,);
