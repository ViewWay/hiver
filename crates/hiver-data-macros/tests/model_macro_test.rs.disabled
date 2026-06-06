//! Integration tests for the Model derive macro
//! Model derive 宏的集成测试

use hiver_data_orm::Model;

#[test]
fn test_basic_model_derive() {
    #[derive(Model, Debug, Clone)]
    #[model(table = "test_users")]
    struct TestUser {
        #[model(primary_key)]
        id: i64,

        #[model(max_length = 255, unique)]
        email: String,

        #[model(nullable)]
        name: Option<String>,

        #[model(default = "now()")]
        created_at: String,
    }

    // Test that meta() returns the correct table name
    let meta = TestUser::meta();
    assert_eq!(meta.table_name(), "test_users");

    // Test that columns are properly generated
    assert!(!meta.columns.is_empty());

    // Find the id column
    let id_col = meta.columns.iter().find(|c| c.name == "id").unwrap();
    assert!(id_col.is_primary_key);

    // Find the email column
    let email_col = meta.columns.iter().find(|c| c.name == "email").unwrap();
    assert!(email_col.is_unique);

    // Find the name column
    let name_col = meta.columns.iter().find(|c| c.name == "name").unwrap();
    assert!(name_col.is_nullable);
}

#[test]
fn test_default_table_name() {
    #[derive(Model, Debug, Clone)]
    struct UserProfile {
        #[model(primary_key)]
        id: i64,
        name: String,
    }

    // Should auto-generate table name as "user_profile"
    let meta = UserProfile::meta();
    assert_eq!(meta.table_name(), "user_profile");
}

#[test]
fn test_custom_column_name() {
    #[derive(Model, Debug, Clone)]
    struct TestEntity {
        #[model(primary_key, column = "entity_id")]
        id: i64,

        #[model(column = "user_email")]
        email: String,
    }

    let meta = TestEntity::meta();

    // Check that custom column names are used
    let id_col = meta.columns.iter().find(|c| c.name == "entity_id").unwrap();
    assert!(id_col.is_primary_key);

    let email_col = meta.columns.iter().find(|c| c.name == "user_email");
    assert!(email_col.is_some());
}

#[test]
fn test_primary_key_methods() {
    #[derive(Model, Debug, Clone)]
    struct TestModel {
        #[model(primary_key)]
        id: i64,
        value: String,
    }

    let model = TestModel {
        id: 42,
        value: "test".to_string(),
    };

    // Test primary_key() method
    let pk = model.primary_key().unwrap();
    assert_eq!(pk, "42");

    // Test Display implementation
    let display_str = format!("{}", model);
    assert!(display_str.contains("TestModel"));
    assert!(display_str.contains("42"));
}

#[test]
fn test_set_primary_key() {
    #[derive(Model, Debug, Clone)]
    struct TestModel {
        #[model(primary_key)]
        id: i64,
        value: String,
    }

    let mut model = TestModel {
        id: 0,
        value: "test".to_string(),
    };

    // Test set_primary_key() method
    model.set_primary_key("123".to_string()).unwrap();
    assert_eq!(model.id, 123);
}

#[test]
fn test_multiple_attributes() {
    #[derive(Model, Debug, Clone)]
    #[model(table = "products")]
    struct Product {
        #[model(primary_key)]
        id: i64,

        #[model(max_length = 100, unique)]
        sku: String,

        #[model(nullable, default = "0")]
        stock: i32,

        #[model(max_length = 500)]
        description: String,

        #[model(column = "is_active")]
        active: bool,
    }

    let meta = Product::meta();
    assert_eq!(meta.table_name(), "products");

    // Check all columns are present
    assert_eq!(meta.columns.len(), 5);

    // Verify specific column properties
    let sku_col = meta.columns.iter().find(|c| c.name == "sku").unwrap();
    assert!(sku_col.is_unique);

    let stock_col = meta.columns.iter().find(|c| c.name == "stock").unwrap();
    assert!(stock_col.is_nullable);

    let active_col = meta.columns.iter().find(|c| c.name == "is_active");
    assert!(active_col.is_some());
}

#[test]
fn test_ignore_field() {
    #[derive(Model, Debug, Clone)]
    struct TestModel {
        #[model(primary_key)]
        id: i64,
        name: String,

        #[model(ignore)]
        internal_field: String,
    }

    let meta = TestModel::meta();

    // The ignored field should not be in columns
    assert_eq!(meta.columns.len(), 2);

    // Check that 'internal_field' is not present
    let internal_col = meta.columns.iter().find(|c| c.name == "internal_field");
    assert!(internal_col.is_none());
}
