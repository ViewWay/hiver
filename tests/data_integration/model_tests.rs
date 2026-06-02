//! Model integration tests
//! 模型集成测试

use hiver_data_orm::Model;

/// Test User model with derive macro
#[derive(Model, Debug, Clone, PartialEq)]
#[model(table = "test_users")]
struct UserModel {
    #[model(primary_key)]
    id: i64,

    #[model(max_length = 255, unique)]
    email: String,

    #[model(nullable)]
    name: Option<String>,

    #[model(default = "CURRENT_TIMESTAMP")]
    created_at: String,
}

/// Test Post model with relationships
#[derive(Model, Debug, Clone, PartialEq)]
#[model(table = "test_posts")]
struct PostModel {
    #[model(primary_key)]
    id: i64,

    #[model(column = "user_id")]
    user_id: i64,

    #[model(max_length = 200)]
    title: String,

    #[model(nullable)]
    content: Option<String>,

    published: bool,

    #[model(default = "0")]
    view_count: i32,
}

/// Test Product model with multiple attributes
#[derive(Model, Debug, Clone, PartialEq)]
#[model(table = "products")]
struct ProductModel {
    #[model(primary_key)]
    id: i64,

    #[model(max_length = 100, unique)]
    sku: String,

    #[model(max_length = 255)]
    name: String,

    #[model(nullable)]
    description: Option<String>,

    #[model(default = "0")]
    stock: i32,

    #[model(default = "0.0")]
    price: f64,

    #[model(column = "is_active")]
    active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_meta_table_name() {
        let meta = UserModel::meta();
        assert_eq!(meta.table_name(), "test_users");
    }

    #[test]
    fn test_model_meta_columns() {
        let meta = UserModel::meta();
        assert_eq!(meta.columns.len(), 4);

        // Check column names
        let column_names: Vec<&str> = meta.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(column_names.contains(&"id"));
        assert!(column_names.contains(&"email"));
        assert!(column_names.contains(&"name"));
        assert!(column_names.contains(&"created_at"));
    }

    #[test]
    fn test_model_primary_key_column() {
        let meta = UserModel::meta();

        let id_column = meta
            .columns
            .iter()
            .find(|c| c.name == "id")
            .expect("id column not found");

        assert!(id_column.is_primary_key);
    }

    #[test]
    fn test_model_unique_column() {
        let meta = UserModel::meta();

        let email_column = meta
            .columns
            .iter()
            .find(|c| c.name == "email")
            .expect("email column not found");

        assert!(email_column.is_unique);
    }

    #[test]
    fn test_model_nullable_column() {
        let meta = UserModel::meta();

        let name_column = meta
            .columns
            .iter()
            .find(|c| c.name == "name")
            .expect("name column not found");

        assert!(name_column.is_nullable);
    }

    #[test]
    fn test_model_default_value() {
        let meta = UserModel::meta();

        let created_at_column = meta
            .columns
            .iter()
            .find(|c| c.name == "created_at")
            .expect("created_at column not found");

        assert_eq!(created_at_column.default.as_ref().unwrap(), "CURRENT_TIMESTAMP");
    }

    #[test]
    fn test_model_custom_column_name() {
        let meta = PostModel::meta();

        // Check that user_id column exists with custom name
        let user_id_column = meta
            .columns
            .iter()
            .find(|c| c.name == "user_id")
            .expect("user_id column not found");

        assert!(!user_id_column.is_nullable);
    }

    #[test]
    fn test_model_auto_table_name() {
        let meta = ProductModel::meta();

        // Should auto-generate table name from struct name
        // ProductModel -> product_model
        assert_eq!(meta.table_name(), "products");
    }

    #[test]
    fn test_model_max_length() {
        let meta = ProductModel::meta();

        let sku_column = meta
            .columns
            .iter()
            .find(|c| c.name == "sku")
            .expect("sku column not found");

        assert!(sku_column.is_unique);
    }

    #[test]
    fn test_model_multiple_attributes() {
        let meta = ProductModel::meta();
        assert_eq!(meta.columns.len(), 7);

        // Verify stock column has default
        let stock_column = meta
            .columns
            .iter()
            .find(|c| c.name == "stock")
            .expect("stock column not found");

        assert_eq!(stock_column.default.as_ref().unwrap(), "0");

        // Verify price column has default
        let price_column = meta
            .columns
            .iter()
            .find(|c| c.name == "price")
            .expect("price column not found");

        assert_eq!(price_column.default.as_ref().unwrap(), "0.0");

        // Verify active column with custom name
        let active_column = meta.columns.iter().find(|c| c.name == "is_active");

        assert!(active_column.is_some());
    }

    #[test]
    fn test_model_primary_key_method() {
        let user = UserModel {
            id: 123,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            created_at: "2024-01-01".to_string(),
        };

        let pk = user.primary_key().unwrap();
        assert_eq!(pk, "123");
    }

    #[test]
    fn test_model_set_primary_key() {
        let mut user = UserModel {
            id: 0,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            created_at: "2024-01-01".to_string(),
        };

        user.set_primary_key("456".to_string()).unwrap();
        assert_eq!(user.id, 456);
    }

    #[test]
    fn test_model_display() {
        let user = UserModel {
            id: 123,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            created_at: "2024-01-01".to_string(),
        };

        let display_str = format!("{}", user);
        assert!(display_str.contains("UserModel"));
        assert!(display_str.contains("123"));
    }

    #[test]
    fn test_model_validation_default() {
        let user = UserModel {
            id: 123,
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            created_at: "2024-01-01".to_string(),
        };

        // Default validation should pass
        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_post_model_columns() {
        let meta = PostModel::meta();

        assert_eq!(meta.table_name(), "test_posts");

        // Check title has max_length
        let title_column = meta
            .columns
            .iter()
            .find(|c| c.name == "title")
            .expect("title column not found");

        assert!(!title_column.is_nullable);

        // Check content is nullable
        let content_column = meta
            .columns
            .iter()
            .find(|c| c.name == "content")
            .expect("content column not found");

        assert!(content_column.is_nullable);
    }
}
