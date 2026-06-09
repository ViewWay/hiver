//! MySQL integration tests using testcontainers.
//! 使用 testcontainers 的 MySQL 集成测试。
//!
//! These tests require Docker to be running.
//! 这些测试需要 Docker 正在运行。
//!
//! Run with: cargo test --features integration-tests --test mysql_integration

use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, MySqlPool, Row, mysql::MySqlPoolOptions};
use testcontainers::{GenericImage, core::IntoContainerPort, runners::AsyncRunner};

/// Test product entity for CRUD operations.
/// 用于 CRUD 操作的测试产品实体。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct TestProduct {
    id: i64,
    name: String,
    description: Option<String>,
    price: Option<f64>,
    stock: i32,
    is_available: bool,
}

/// Helper: start a MySQL container and return pool + container.
/// 辅助函数：启动 MySQL 容器并返回连接池和容器。
async fn setup_mysql() -> (MySqlPool, testcontainers::ContainerAsync<GenericImage>) {
    let container = GenericImage::new("mysql", "8.0")
        .with_env_var("MYSQL_ROOT_PASSWORD", "testpass")
        .with_env_var("MYSQL_DATABASE", "testdb")
        .with_env_var("MYSQL_USER", "testuser")
        .with_env_var("MYSQL_PASSWORD", "testpass")
        .with_wait_for(testcontainers::core::WaitFor::Seconds(15))
        .with_mapped_port(3306, 3306.tcp())
        .start()
        .await
        .expect("Failed to start MySQL container");

    let host_port = container
        .get_host_port_ipv4(3306.tcp())
        .await
        .expect("Failed to get MySQL port");

    let connection_string = format!("mysql://testuser:testpass@127.0.0.1:{host_port}/testdb");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&connection_string)
        .await
        .expect("Failed to connect to MySQL");

    (pool, container)
}

/// Helper: create the test_products table.
/// 辅助函数：创建 test_products 表。
async fn create_test_table(pool: &MySqlPool) {
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS test_products (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            price DOUBLE,
            stock INT NOT NULL DEFAULT 0,
            is_available BOOLEAN NOT NULL DEFAULT TRUE
        )
        "#,
    )
    .await
    .expect("Failed to create test_products table");
}

/// Helper: seed initial test data.
/// 辅助函数：插入初始测试数据。
async fn seed_test_data(pool: &MySqlPool) {
    sqlx::query(
        r#"
        INSERT IGNORE INTO test_products (name, description, price, stock, is_available)
        VALUES
            ('Widget', 'A useful widget', 9.99, 100, TRUE),
            ('Gadget', 'A fancy gadget', 24.99, 50, TRUE),
            ('Doohickey', 'A mysterious doohickey', 4.99, 0, FALSE)
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed test data");
}

// ============================================================
// Test 1: MySQL container starts and accepts connections
// 测试 1：MySQL 容器启动并接受连接
// ============================================================
#[tokio::test]
async fn test_mysql_container_connectivity() {
    let (pool, _container) = setup_mysql().await;

    let row: (String,) = sqlx::query_as("SELECT VERSION()")
        .fetch_one(&pool)
        .await
        .expect("Failed to query MySQL version");

    assert!(row.0.contains("8.0"), "Expected MySQL 8.0 version string, got: {}", row.0);
}

// ============================================================
// Test 2: Create table with various column types
// 测试 2：创建包含多种列类型的表
// ============================================================
#[tokio::test]
async fn test_mysql_create_table() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;

    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'testdb' AND \
         table_name = 'test_products'",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check table existence");

    assert_eq!(row.0, 1, "test_products table should exist");
}

// ============================================================
// Test 3: Insert and select a single row
// 测试 3：插入并查询单行数据
// ============================================================
#[tokio::test]
async fn test_mysql_insert_and_select() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;

    sqlx::query(
        "INSERT INTO test_products (name, description, price, stock, is_available) VALUES (?, ?, \
         ?, ?, ?)",
    )
    .bind("TestItem")
    .bind("A test item")
    .bind(19.99)
    .bind(200)
    .bind(true)
    .execute(&pool)
    .await
    .expect("Failed to insert product");

    let product: TestProduct = sqlx::query_as("SELECT * FROM test_products WHERE name = ?")
        .bind("TestItem")
        .fetch_one(&pool)
        .await
        .expect("Failed to select product");

    assert_eq!(product.name, "TestItem");
    assert_eq!(product.description, Some("A test item".to_string()));
    assert_eq!(product.stock, 200);
}

// ============================================================
// Test 4: Update existing rows
// 测试 4：更新已有行
// ============================================================
#[tokio::test]
async fn test_mysql_update() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    sqlx::query("UPDATE test_products SET price = ? WHERE name = ?")
        .bind(12.99)
        .bind("Widget")
        .execute(&pool)
        .await
        .expect("Failed to update product");

    let product: TestProduct = sqlx::query_as("SELECT * FROM test_products WHERE name = ?")
        .bind("Widget")
        .fetch_one(&pool)
        .await
        .expect("Failed to select product after update");

    let price = product.price.expect("Price should not be NULL");
    assert!((price - 12.99).abs() < 0.01, "Price should be 12.99, got {price}");
}

// ============================================================
// Test 5: Delete rows
// 测试 5：删除行
// ============================================================
#[tokio::test]
async fn test_mysql_delete() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    sqlx::query("DELETE FROM test_products WHERE name = ?")
        .bind("Gadget")
        .execute(&pool)
        .await
        .expect("Failed to delete product");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM test_products")
        .fetch_one(&pool)
        .await
        .expect("Failed to count products");
    assert_eq!(count.0, 2);

    let result = sqlx::query_as::<_, TestProduct>("SELECT * FROM test_products WHERE name = ?")
        .bind("Gadget")
        .fetch_optional(&pool)
        .await
        .expect("Failed to check deleted product");
    assert!(result.is_none());
}

// ============================================================
// Test 6: Transaction commit
// 测试 6：事务提交
// ============================================================
#[tokio::test]
async fn test_mysql_transaction_commit() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;

    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    sqlx::query(
        "INSERT INTO test_products (name, description, price, stock, is_available) VALUES (?, ?, \
         ?, ?, ?)",
    )
    .bind("tx_product")
    .bind("Created in a transaction")
    .bind(5.00)
    .bind(10)
    .bind(true)
    .execute(&mut *tx)
    .await
    .expect("Failed to insert in transaction");

    tx.commit().await.expect("Failed to commit transaction");

    let product: TestProduct = sqlx::query_as("SELECT * FROM test_products WHERE name = ?")
        .bind("tx_product")
        .fetch_one(&pool)
        .await
        .expect("Failed to select committed product");

    assert_eq!(product.name, "tx_product");
}

// ============================================================
// Test 7: Transaction rollback
// 测试 7：事务回滚
// ============================================================
#[tokio::test]
async fn test_mysql_transaction_rollback() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;

    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    sqlx::query("INSERT INTO test_products (name, stock, is_available) VALUES (?, ?, ?)")
        .bind("rollback_product")
        .bind(999)
        .bind(false)
        .execute(&mut *tx)
        .await
        .expect("Failed to insert in transaction");

    tx.rollback().await.expect("Failed to rollback transaction");

    let result = sqlx::query_as::<_, TestProduct>("SELECT * FROM test_products WHERE name = ?")
        .bind("rollback_product")
        .fetch_optional(&pool)
        .await
        .expect("Failed to check rolled-back product");
    assert!(result.is_none(), "Rolled-back row should not exist");
}

// ============================================================
// Test 8: Pagination with LIMIT/OFFSET
// 测试 8：使用 LIMIT/OFFSET 分页
// ============================================================
#[tokio::test]
async fn test_mysql_pagination() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    let page1: Vec<TestProduct> =
        sqlx::query_as("SELECT * FROM test_products ORDER BY id ASC LIMIT 2 OFFSET 0")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch page 1");
    assert_eq!(page1.len(), 2);

    let page2: Vec<TestProduct> =
        sqlx::query_as("SELECT * FROM test_products ORDER BY id ASC LIMIT 2 OFFSET 2")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch page 2");
    assert_eq!(page2.len(), 1);
}

// ============================================================
// Test 9: Aggregation queries
// 测试 9：聚合查询
// ============================================================
#[tokio::test]
async fn test_mysql_aggregation() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    let stats: (i64, Option<f64>) =
        sqlx::query_as("SELECT COUNT(*) as cnt, AVG(price) as avg_price FROM test_products")
            .fetch_one(&pool)
            .await
            .expect("Failed to run aggregation query");

    assert_eq!(stats.0, 3, "Should have 3 rows");
    // AVG of 9.99, 24.99, 4.99 = 13.3233...
    let avg = stats.1.expect("Average should not be NULL");
    assert!((avg - 13.32).abs() < 1.0, "Average price should be ~13.32, got {avg}");
}

// ============================================================
// Test 10: Auto-increment ID generation
// 测试 10：自增 ID 生成
// ============================================================
#[tokio::test]
async fn test_mysql_auto_increment() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;

    sqlx::query("INSERT INTO test_products (name, stock, is_available) VALUES (?, ?, ?)")
        .bind("first")
        .bind(1)
        .bind(true)
        .execute(&pool)
        .await
        .expect("Failed to insert first product");

    sqlx::query("INSERT INTO test_products (name, stock, is_available) VALUES (?, ?, ?)")
        .bind("second")
        .bind(2)
        .bind(true)
        .execute(&pool)
        .await
        .expect("Failed to insert second product");

    let products: Vec<TestProduct> = sqlx::query_as("SELECT * FROM test_products ORDER BY id ASC")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch products");

    assert!(products[0].id < products[1].id, "IDs should be ascending");
}

// ============================================================
// Test 11: LIKE pattern matching
// 测试 11：LIKE 模式匹配
// ============================================================
#[tokio::test]
async fn test_mysql_like_pattern() {
    let (pool, _container) = setup_mysql().await;
    create_test_table(&pool).await;
    seed_test_data(&pool).await;

    let results: Vec<TestProduct> = sqlx::query_as("SELECT * FROM test_products WHERE name LIKE ?")
        .bind("%get%")
        .fetch_all(&pool)
        .await
        .expect("Failed to run LIKE query");

    // Should match "Widget" and "Gadget"
    assert_eq!(results.len(), 2, "LIKE '%get%' should match Widget and Gadget");
}

// ============================================================
// Test 12: Concurrent queries via connection pool
// 测试 12：通过连接池执行并发查询
// ============================================================
#[tokio::test]
async fn test_mysql_concurrent_queries() {
    let (pool, _container) = setup_mysql().await;

    let mut handles = Vec::new();
    for i in 0..10 {
        let pool_clone = pool.clone();
        handles.push(tokio::spawn(async move {
            let row: (i32,) = sqlx::query_as("SELECT ? AS val")
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
