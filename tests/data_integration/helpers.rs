//! Integration test helpers
//! 集成测试辅助工具

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tokio::sync::OnceCell;

/// Test database pool (using OnceCell for thread-safe lazy initialization)
/// 测试数据库连接池（使用 OnceCell 进行线程安全的延迟初始化）
static TEST_POOL: OnceCell<SqlitePool> = OnceCell::const_new();

/// Get or create the test database pool
/// 获取或创建测试数据库连接池
pub async fn get_test_pool() -> &'static SqlitePool {
    TEST_POOL
        .get_or_init(|| async {
            // Create new pool with shared in-memory database
            // Using file:memdb?mode=memory&cache=shared allows sharing across connections
            let options = SqliteConnectOptions::new()
                .filename("file:memdb?mode=memory&cache=shared")
                .create_if_missing(true)
                .shared_cache(true);

            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(options)
                .await
                .expect("Failed to create test database pool");

            // Initialize schema
            init_test_schema_with_pool(&pool).await.unwrap();

            pool
        })
        .await
}

/// Initialize the test database schema with given pool
/// 使用给定连接池初始化测试数据库模式
async fn init_test_schema_with_pool(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create test tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL UNIQUE,
            name TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            published BOOLEAN DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Initialize the test database schema
/// 初始化测试数据库模式
pub async fn init_test_schema() -> Result<(), sqlx::Error> {
    // Already initialized in get_test_pool
    let _pool = get_test_pool().await;
    Ok(())
}

/// Clean up test data
/// 清理测试数据
pub async fn cleanup_test_data() -> Result<(), sqlx::Error> {
    let pool = get_test_pool().await;

    // Clean up posts if table exists (ignore errors)
    let _ = sqlx::query("DELETE FROM posts").execute(pool).await;

    // Clean up users if table exists (ignore errors)
    let _ = sqlx::query("DELETE FROM users").execute(pool).await;

    Ok(())
}

/// Insert a test user
/// 插入测试用户
pub async fn insert_test_user(email: &str, name: Option<&str>) -> Result<i64, sqlx::Error> {
    let pool = get_test_pool().await;

    let result = sqlx::query("INSERT INTO users (email, name) VALUES (?, ?)")
        .bind(email)
        .bind(name)
        .execute(pool)
        .await?;

    Ok(result.last_insert_rowid())
}

/// Find a user by email (returns tuple)
/// 通过邮箱查找用户（返回元组）
pub async fn find_user_by_email(
    email: &str,
) -> Result<Option<(i64, String, Option<String>)>, sqlx::Error> {
    let pool = get_test_pool().await;

    let row = sqlx::query_as::<_, (i64, String, Option<String>)>(
        "SELECT id, email, name FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .ok();

    Ok(row)
}

/// Count users
/// 统计用户数量
pub async fn count_users() -> Result<i64, sqlx::Error> {
    let pool = get_test_pool().await;

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_setup_database() {
        cleanup_test_data().await.unwrap();

        // Insert a test user
        let id = insert_test_user("test@example.com", Some("Test User"))
            .await
            .unwrap();
        assert!(id > 0);

        // Find the user
        let user = find_user_by_email("test@example.com").await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.as_ref().unwrap().1, "test@example.com");

        // Count users
        let count = count_users().await.unwrap();
        assert_eq!(count, 1);

        // Cleanup
        cleanup_test_data().await.unwrap();

        // Verify cleanup
        let count = count_users().await.unwrap();
        assert_eq!(count, 0);
    }
}
