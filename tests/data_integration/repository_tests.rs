//! Repository integration tests
//! 仓储集成测试

use async_trait::async_trait;
use hiver_data_commons::{Repository, CrudRepository, Error};
use crate::data_integration::helpers::*;

/// Mock repository for testing
/// 用于测试的模拟仓储
pub struct TestUserRepository;

impl TestUserRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom error type for tests
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Parse error: {0}")]
    Parse(String),
}

impl Into<Error> for TestError {
    fn into(self) -> Error {
        match self {
            TestError::Db(e) => Error::data_integrity_violation(e.to_string()),
            TestError::NotFound => Error::entity_not_found("User", "id"),
            TestError::Parse(s) => Error::InvalidDataAccess(s),
        }
    }
}

/// User entity for repository tests
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: Option<String>,
}

// Implement Sqlx FromRow for User
impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for User {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(User {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            name: row.try_get("name").unwrap_or(None),
        })
    }
}

#[async_trait]
impl Repository<User, i64> for TestUserRepository {
    type Error = TestError;

    async fn save(&self, entity: User) -> Result<User, Self::Error> {
        let pool = get_test_pool().await;

        if entity.id > 0 {
            // Update
            sqlx::query(
                "UPDATE users SET email = ?, name = ? WHERE id = ?"
            )
            .bind(&entity.email)
            .bind(&entity.name)
            .bind(entity.id)
            .execute(pool)
            .await?;
            Ok(entity)
        } else {
            // Insert
            let result = sqlx::query(
                "INSERT INTO users (email, name) VALUES (?, ?)"
            )
            .bind(&entity.email)
            .bind(&entity.name)
            .execute(pool)
            .await?;

            Ok(User {
                id: result.last_insert_rowid(),
                ..entity
            })
        }
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, Self::Error> {
        let pool = get_test_pool().await;

        let row = sqlx::query_as::<_, User>(
            "SELECT id, email, name FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self) -> Result<Vec<User>, Self::Error> {
        let pool = get_test_pool().await;

        let rows = sqlx::query_as::<_, User>(
            "SELECT id, email, name FROM users ORDER BY id"
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> Result<u64, Self::Error> {
        let pool = get_test_pool().await;

        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;

        Ok(row.0 as u64)
    }

    async fn delete_by_id(&self, id: i64) -> Result<(), Self::Error> {
        let pool = get_test_pool().await;
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, entity: User) -> Result<(), Self::Error> {
        self.delete_by_id(entity.id).await
    }

    async fn delete_all(&self) -> Result<(), Self::Error> {
        let pool = get_test_pool().await;
        sqlx::query("DELETE FROM users").execute(pool).await?;
        Ok(())
    }
}

// CrudRepository is a marker trait, so we just need to implement it
impl CrudRepository<User, i64> for TestUserRepository {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_save() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Save new user
        let user = User {
            id: 0,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };

        let saved = repo.save(user).await.unwrap();
        assert!(saved.id > 0);
        assert_eq!(saved.email, "test@example.com");

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_find_by_id() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create a user
        let user = User {
            id: 0,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };
        let saved = repo.save(user).await.unwrap();

        // Find by ID
        let found = repo.find_by_id(saved.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "test@example.com");

        // Find non-existent
        let not_found = repo.find_by_id(9999).await.unwrap();
        assert!(not_found.is_none());

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_exists_by_id() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create a user
        let user = User {
            id: 0,
            email: "test@example.com".to_string(),
            name: None,
        };
        let saved = repo.save(user).await.unwrap();

        // Exists
        assert!(repo.exists_by_id(saved.id).await.unwrap());

        // Not exists
        assert!(!repo.exists_by_id(9999).await.unwrap());

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_find_all() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create multiple users
        for i in 1..=3 {
            let user = User {
                id: 0,
                email: format!("user{}@example.com", i),
                name: Some(format!("User {}", i)),
            };
            repo.save(user).await.unwrap();
        }

        // Find all
        let users = repo.find_all().await.unwrap();
        assert_eq!(users.len(), 3);

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_count() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Count empty
        assert_eq!(repo.count().await.unwrap(), 0);

        // Add users
        for i in 0..3 {
            let user = User {
                id: 0,
                email: format!("count{}@example.com", i),
                name: None,
            };
            repo.save(user).await.unwrap();
        }

        // Count
        assert_eq!(repo.count().await.unwrap(), 3);

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_delete_by_id() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create a user
        let user = User {
            id: 0,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };
        let saved = repo.save(user).await.unwrap();

        // Delete
        repo.delete_by_id(saved.id).await.unwrap();

        // Verify deleted
        assert!(!repo.exists_by_id(saved.id).await.unwrap());

        // Delete non-existent should not error
        repo.delete_by_id(9999).await.unwrap();

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_delete() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create a user
        let user = User {
            id: 0,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };
        let saved = repo.save(user).await.unwrap();

        // Delete entity
        repo.delete(saved).await.unwrap();

        // Verify count is 0
        assert_eq!(repo.count().await.unwrap(), 0);

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_delete_all() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create users
        for i in 0..3 {
            let user = User {
                id: 0,
                email: format!("delete_all{}@example.com", i),
                name: None,
            };
            repo.save(user).await.unwrap();
        }

        // Delete all
        repo.delete_all().await.unwrap();

        // Verify
        assert_eq!(repo.count().await.unwrap(), 0);

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_update() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create a user
        let user = User {
            id: 0,
            email: "test@example.com".to_string(),
            name: Some("Original Name".to_string()),
        };
        let saved = repo.save(user).await.unwrap();

        // Update
        let updated = User {
            name: Some("Updated Name".to_string()),
            ..saved.clone()
        };
        let result = repo.save(updated).await.unwrap();

        assert_eq!(result.id, saved.id);
        assert_eq!(result.name.as_ref().unwrap(), "Updated Name");

        // Verify in DB
        let found = repo.find_by_id(saved.id).await.unwrap().unwrap();
        assert_eq!(found.name.as_ref().unwrap(), "Updated Name");

        cleanup_test_data().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_crud_repository_save_all() {
        init_test_schema().await.unwrap();
        cleanup_test_data().await.unwrap();

        let repo = TestUserRepository::new();

        // Create multiple users
        let users = vec![
            User {
                id: 0,
                email: "user1@example.com".to_string(),
                name: Some("User 1".to_string()),
            },
            User {
                id: 0,
                email: "user2@example.com".to_string(),
                name: Some("User 2".to_string()),
            },
            User {
                id: 0,
                email: "user3@example.com".to_string(),
                name: Some("User 3".to_string()),
            },
        ];

        let saved = repo.save_all(users).await.unwrap();
        assert_eq!(saved.len(), 3);
        assert!(saved[0].id > 0);
        assert!(saved[1].id > 0);
        assert!(saved[2].id > 0);

        cleanup_test_data().await.unwrap();
    }
}
