//! PostgreSQL integration tests using testcontainers.
//! 使用 testcontainers 的 PostgreSQL 集成测试。
//!
//! These tests require Docker to be running.
//! 这些测试需要 Docker 正在运行。
//!
//! Run with: cargo test --features integration-tests --test postgres_integration

use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Row, postgres::PgPoolOptions};
use testcontainers::{core::IntoContainerPort, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

/// Test user entity for CRUD operations.
/// 用于 CRUD 操作的测试用户实体。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct TestUser {
    id: i64,
    username: String,
    email: String,
    age: Option<i32>,
    is_active: bool,
    created_at: Option<chrono::NaiveDateTime>,
}

/// Helper: start a PostgreSQL container and return pool + container.
/// 辅助函数：启动 PostgreSQL 容器并返回连接池和容器。
async fn setup_postgres() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .with_user("testuser")
        .with_password("testpass")
        .with_db_name("testdb")
        .start()
        .await
        .expect("Failed to start PostgreSQL container");

    let host_port = container
        .get_host_port_ipv4(5432.tcp())
        .await
        .expect("Failed to get PostgreSQL port");

    let connection_string = format!("postgres://testuser:testpass@127.0.0.1:{host_port}/testdb");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&connection_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    (pool, container)
}

/// Helper: create the test_users table.
/// 辅助函数：创建 test_users 表。
async fn create_test_table(pool: &PgPool) {
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS test_users (
            id BIGSERIAL PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            email VARCHAR(255) NOT NULL,
            age INTEGER,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .await
    .expect("Failed to create test_users table");
}

/// Helper: seed initial test data.
/// 辅助函数：插入初始测试数据。
async fn seed_test_data(pool: &PgPool) {
    sqlx::query(
        r#"
        INSERT INTO test_users (username, email, age, is_active)
        VALUES
            ('alice', 'alice@example.com', 30, TRUE),
            ('bob', 'bob@example.com', 25, TRUE),
            ('charlie', 'charlie@example.com', 35, FALSE)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed test data");
}

// ============================================================
// Test 1: PostgreSQL container starts and accepts connections
// 测试 1：PostgreSQL 容器启动并接受连接
// ============================================================
#[tokio::test]
async fn test_postgres_container_connectivity() {
    let (pool, _container) = setup_postgres().await;

    let row: (String,) = sqlx::query_as("SELECT version()")
        .fetch_one(&pool)
        .await
        .expect("Failed to query PostgreSQL version");

    assert!(
        row.0.contains("PostgreSQL"),
        "Expected PostgreSQL version string, got: {}",
        row.0
    );
}

// ============================================================
// Test 2: Create table with various column types
// 测试 2：创建包含多种列类型的表
// ============================================================
#[tokio::test]
async fn test_postgres_create_table() {
    let (pool, _container) = setup_postgres().await;

    create_test_table(&pool).await;

    // Verify the table exists
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'test_users'",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check table existence");

    assert_eq!(row.0, 1, "test_users table should exist");
}

// ============================================================
// Test 3: Insert a single row and read it back
// 测试 3：插入单行数据并读回
// ============================================================
#[tokio::test]
async fn test_postgres_insert_and_select() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;

    sqlx::query("INSERT INTO test_users (username, email, age, is_active) VALUES ($1, $2, $3, $4)")
        .bind("testuser")
        .bind("testuser@example.com")
        .bind(42)
        .bind(true)
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    let user: TestUser = sqlx::query_as("SELECT * FROM test_users WHERE username = $1")
        .bind("testuser")
        .fetch_one(&pool)
        .await
        .expect("Failed to select user");

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "testuser@example.com");
    assert_eq!(user.age, Some(42));
    assert!(user.is_active);
}

// ============================================================
// Test 4: Update existing rows
// 测试 4：更新已有行
// ============================================================
#[tokio::test]
async fn test_postgres_update() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    sqlx::query("UPDATE test_users SET email = $1 WHERE username = $2")
        .bind("alice_new@example.com")
        .bind("alice")
        .execute(&pool)
        .await
        .expect("Failed to update user");

    let user: TestUser = sqlx::query_as("SELECT * FROM test_users WHERE username = $1")
        .bind("alice")
        .fetch_one(&pool)
        .await
        .expect("Failed to select user after update");

    assert_eq!(user.email, "alice_new@example.com");
}

// ============================================================
// Test 5: Delete rows
// 测试 5：删除行
// ============================================================
#[tokio::test]
async fn test_postgres_delete() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    let before_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count before delete");
    assert_eq!(before_count.0, 3);

    sqlx::query("DELETE FROM test_users WHERE username = $1")
        .bind("bob")
        .execute(&pool)
        .await
        .expect("Failed to delete user");

    let after_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count after delete");
    assert_eq!(after_count.0, 2);

    // Verify bob is gone
    let result = sqlx::query_as::<_, TestUser>("SELECT * FROM test_users WHERE username = $1")
        .bind("bob")
        .fetch_optional(&pool)
        .await
        .expect("Failed to check deleted user");
    assert!(result.is_none());
}

// ============================================================
// Test 6: Transaction commit
// 测试 6：事务提交
// ============================================================
#[tokio::test]
async fn test_postgres_transaction_commit() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;

    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    sqlx::query("INSERT INTO test_users (username, email, age, is_active) VALUES ($1, $2, $3, $4)")
        .bind("tx_user")
        .bind("tx@example.com")
        .bind(20)
        .bind(true)
        .execute(&mut *tx)
        .await
        .expect("Failed to insert in transaction");

    tx.commit().await.expect("Failed to commit transaction");

    let user: TestUser = sqlx::query_as("SELECT * FROM test_users WHERE username = $1")
        .bind("tx_user")
        .fetch_one(&pool)
        .await
        .expect("Failed to select committed user");

    assert_eq!(user.username, "tx_user");
}

// ============================================================
// Test 7: Transaction rollback
// 测试 7：事务回滚
// ============================================================
#[tokio::test]
async fn test_postgres_transaction_rollback() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;

    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    sqlx::query("INSERT INTO test_users (username, email, age, is_active) VALUES ($1, $2, $3, $4)")
        .bind("rollback_user")
        .bind("rollback@example.com")
        .bind(99)
        .bind(false)
        .execute(&mut *tx)
        .await
        .expect("Failed to insert in transaction");

    tx.rollback().await.expect("Failed to rollback transaction");

    let result = sqlx::query_as::<_, TestUser>("SELECT * FROM test_users WHERE username = $1")
        .bind("rollback_user")
        .fetch_optional(&pool)
        .await
        .expect("Failed to check rolled-back user");
    assert!(result.is_none(), "Rolled-back row should not exist");
}

// ============================================================
// Test 8: Pagination with LIMIT/OFFSET
// 测试 8：使用 LIMIT/OFFSET 分页
// ============================================================
#[tokio::test]
async fn test_postgres_pagination() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    // Page 1: first 2 results
    let page1: Vec<TestUser> =
        sqlx::query_as("SELECT * FROM test_users ORDER BY id ASC LIMIT 2 OFFSET 0")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch page 1");
    assert_eq!(page1.len(), 2);

    // Page 2: remaining result
    let page2: Vec<TestUser> =
        sqlx::query_as("SELECT * FROM test_users ORDER BY id ASC LIMIT 2 OFFSET 2")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch page 2");
    assert_eq!(page2.len(), 1);
}

// ============================================================
// Test 9: Aggregation queries (COUNT, AVG, MAX)
// 测试 9：聚合查询（COUNT, AVG, MAX）
// ============================================================
#[tokio::test]
async fn test_postgres_aggregation() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    let stats: (i64, Option<i32>, Option<i32>) = sqlx::query_as(
        "SELECT COUNT(*) as cnt, AVG(age) as avg_age, MAX(age) as max_age FROM test_users",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to run aggregation query");

    assert_eq!(stats.0, 3, "Should have 3 rows");
    // AVG of 30, 25, 35 = 30
    let avg = stats.1.expect("Average should not be NULL");
    assert!((avg - 30).abs() < 2, "Average age should be ~30, got {avg}");
    assert_eq!(stats.2, Some(35), "Max age should be 35");
}

// ============================================================
// Test 10: Unique constraint violation
// 测试 10：唯一约束冲突
// ============================================================
#[tokio::test]
async fn test_postgres_unique_constraint() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;

    sqlx::query("INSERT INTO test_users (username, email, age, is_active) VALUES ($1, $2, $3, $4)")
        .bind("unique_user")
        .bind("unique@example.com")
        .bind(30)
        .bind(true)
        .execute(&pool)
        .await
        .expect("Failed to insert first user");

    // Attempting to insert a duplicate username should fail
    let result = sqlx::query(
        "INSERT INTO test_users (username, email, age, is_active) VALUES ($1, $2, $3, $4)",
    )
    .bind("unique_user")
    .bind("other@example.com")
    .bind(40)
    .bind(true)
    .execute(&pool)
    .await;

    assert!(result.is_err(), "Duplicate username should violate unique constraint");
}

// ============================================================
// Test 11: NULL handling in queries
// 测试 11：查询中的 NULL 处理
// ============================================================
#[tokio::test]
async fn test_postgres_null_handling() {
    let (pool, _container) = setup_postgres().await;
    create_test_table(&pool).await;

    // Insert user with NULL age
    sqlx::query("INSERT INTO test_users (username, email, is_active) VALUES ($1, $2, $3)")
        .bind("null_age_user")
        .bind("nullage@example.com")
        .bind(true)
        .execute(&pool)
        .await
        .expect("Failed to insert user with NULL age");

    let user: TestUser = sqlx::query_as("SELECT * FROM test_users WHERE username = $1")
        .bind("null_age_user")
        .fetch_one(&pool)
        .await
        .expect("Failed to select user");

    assert_eq!(user.age, None, "Age should be NULL");
    assert!(user.is_active);
}

// ============================================================
// Test 12: Connection pool manages multiple connections
// 测试 12：连接池管理多个连接
// ============================================================
#[tokio::test]
async fn test_postgres_connection_pool() {
    let (pool, _container) = setup_postgres().await;

    // Spawn multiple concurrent queries to exercise the pool
    let mut handles = Vec::new();
    for i in 0..10 {
        let pool_clone = pool.clone();
        handles.push(tokio::spawn(async move {
            let row: (i32,) = sqlx::query_as("SELECT $1::int AS val")
                .bind(i)
                .fetch_one(&pool_clone)
                .await;
            row
        }));
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    for (i, result) in results.into_iter().enumerate() {
        let row = result.expect("Query should succeed");
        assert_eq!(row.0, i as i32, "Concurrent query {i} should return correct value");
    }
}
