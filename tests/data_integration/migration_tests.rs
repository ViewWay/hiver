//! Migration integration tests
//! 迁移集成测试

use crate::data_integration::helpers::*;
use hiver_data_orm::migrations::Migration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_migration_creation() {
        let migration = Migration::new("001_create_users")
            .up("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT);")
            .down("DROP TABLE users;");

        assert_eq!(migration.name, "001_create_users");
        assert_eq!(migration.version, "001");
        assert!(!migration.up_sql.is_empty());
        assert!(!migration.down_sql.is_empty());
    }

    #[test]
    #[ignore]
    fn test_migration_validation() {
        let valid_migration = Migration::new("001_test")
            .up("CREATE TABLE test (id INTEGER);")
            .down("DROP TABLE test;");
        assert!(valid_migration.validate().is_ok());

        let invalid_up = Migration::new("002_invalid").down("DROP TABLE test;");
        assert!(invalid_up.validate().is_err());

        let invalid_down = Migration::new("003_invalid").up("CREATE TABLE test (id INTEGER);");
        assert!(invalid_down.validate().is_err());
    }

    #[test]
    #[ignore]
    fn test_migration_with_description() {
        let migration = Migration::new("001_test")
            .description("Create test table")
            .up("CREATE TABLE test (id INTEGER);")
            .down("DROP TABLE test;");

        assert_eq!(migration.description, "Create test table");
    }

    #[test]
    #[ignore]
    fn test_migration_with_attributes() {
        let migration = Migration::new("001_test")
            .attribute("author", "Hiver")
            .attribute("category", "schema")
            .up("CREATE TABLE test (id INTEGER);")
            .down("DROP TABLE test;");

        assert_eq!(migration.attributes.get("author"), Some(&"Hiver".to_string()));
        assert_eq!(migration.attributes.get("category"), Some(&"schema".to_string()));
    }

    #[test]
    #[ignore]
    fn test_migration_version_extraction() {
        let m1 = Migration::new("001_create_users");
        assert_eq!(m1.version, "001");

        let m2 = Migration::new("V002__create_posts");
        assert_eq!(m2.version, "V002");

        let m3 = Migration::new("no_prefix");
        assert_eq!(m3.version, "no_prefix");
    }

    #[tokio::test]
    #[ignore]
    async fn test_migration_execution() {
        init_test_schema().await.unwrap();

        // Tables should exist
        let pool = get_test_pool().await;

        // Check users table exists
        let result =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
                .fetch_optional(pool)
                .await
                .unwrap();

        assert!(result.is_some(), "users table should exist");

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_insert_with_schema() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        // Insert test data
        let id = insert_test_user("migration@test.com", Some("Migration Test"))
            .await
            .unwrap();
        assert!(id > 0);

        // Verify data
        let user = find_user_by_email("migration@test.com").await.unwrap();
        assert!(user.is_some());
        // user is (id, email, name) tuple
        assert_eq!(user.unwrap().2, Some("Migration Test".to_string()));

        cleanup_test_data().await.unwrap();
    }
}
