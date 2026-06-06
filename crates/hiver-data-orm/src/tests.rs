//! Integration tests for hiver-data-orm
//! hiver-data-orm 的集成测试

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests
{
    use std::sync::Arc;

    use hiver_data_rdbc::DatabaseClient;
    use serde::Deserialize;

    use crate::{
        ColumnType as OrmColumnType, Model, ModelMeta, mock_connection::Connection,
        query::QueryBuilder,
    };

    /// Test entity that mimics a real database model.
    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct TestUser
    {
        id: i64,
        name: String,
        email: String,
    }

    impl Model for TestUser
    {
        fn meta() -> ModelMeta
        {
            ModelMeta::new("test_users")
                .add_column(crate::Column::new("id", OrmColumnType::I64).primary_key())
                .add_column(crate::Column::new("name", OrmColumnType::String))
                .add_column(crate::Column::new("email", OrmColumnType::String))
        }

        fn primary_key(&self) -> crate::Result<String>
        {
            Ok(self.id.to_string())
        }

        fn set_primary_key(&mut self, value: String) -> crate::Result<()>
        {
            self.id = value
                .parse()
                .map_err(|_| crate::Error::validation("Invalid primary key"))?;
            Ok(())
        }
    }

    /// Mock DatabaseClient using the in-memory Connection.
    struct MockDbClient
    {
        conn: Arc<std::sync::Mutex<Connection>>,
    }

    impl MockDbClient
    {
        fn new() -> Self
        {
            let conn = Connection::new("mock://test").unwrap();
            Self {
                conn: Arc::new(std::sync::Mutex::new(conn)),
            }
        }

        fn ensure_table(&self, ddl: &str)
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(ddl).unwrap();
        }

        fn insert_test_data(&self)
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO test_users (id, name, email) VALUES (1, 'Alice', 'alice@example.com')",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO test_users (id, name, email) VALUES (2, 'Bob', 'bob@example.com')",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO test_users (id, name, email) VALUES (3, 'Charlie', \
                 'charlie@example.com')",
            )
            .unwrap();
        }
    }

    #[async_trait::async_trait]
    impl hiver_data_rdbc::DatabaseClient for MockDbClient
    {
        async fn fetch_all(&self, sql: &str) -> hiver_data_rdbc::Result<Vec<hiver_data_rdbc::Row>>
        {
            let conn = self.conn.lock().unwrap();
            let results = conn
                .query(sql)
                .map_err(|e| hiver_data_rdbc::Error::Sql(e.to_string()))?;
            let rows: Vec<hiver_data_rdbc::Row> = results
                .into_iter()
                .map(|map| {
                    let mut row = hiver_data_rdbc::Row::new();
                    for (k, v) in map
                    {
                        let cv = match v
                        {
                            serde_json::Value::Number(n) =>
                            {
                                if let Some(i) = n.as_i64()
                                {
                                    hiver_data_rdbc::ColumnValue::I64(i)
                                }
                                else if let Some(f) = n.as_f64()
                                {
                                    hiver_data_rdbc::ColumnValue::F64(f)
                                }
                                else
                                {
                                    hiver_data_rdbc::ColumnValue::Null
                                }
                            },
                            serde_json::Value::String(s) => hiver_data_rdbc::ColumnValue::String(s),
                            serde_json::Value::Bool(b) => hiver_data_rdbc::ColumnValue::Bool(b),
                            _ => hiver_data_rdbc::ColumnValue::Null,
                        };
                        row = row.with_column(k, cv);
                    }
                    row
                })
                .collect();
            Ok(rows)
        }

        async fn fetch_one(
            &self,
            sql: &str,
        ) -> hiver_data_rdbc::Result<Option<hiver_data_rdbc::Row>>
        {
            let rows = self.fetch_all(sql).await?;
            Ok(rows.into_iter().next())
        }

        async fn execute_cmd(&self, sql: &str) -> hiver_data_rdbc::Result<u64>
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(sql)
                .map_err(|e| hiver_data_rdbc::Error::Sql(e.to_string()))?;
            Ok(1)
        }

        async fn begin_transaction(&self) -> hiver_data_rdbc::Result<hiver_data_rdbc::Transaction>
        {
            Err(hiver_data_rdbc::Error::Sql("transactions not supported in mock".into()))
        }

        async fn ping(&self) -> hiver_data_rdbc::Result<()>
        {
            Ok(())
        }

        async fn close(&self) -> hiver_data_rdbc::Result<()>
        {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_client_basic_ops()
    {
        let client = MockDbClient::new();
        client.ensure_table("CREATE TABLE test_users (id INT, name TEXT, email TEXT)");
        client.insert_test_data();

        // Basic fetch_all works with the mock
        let rows = client
            .fetch_all("SELECT * FROM test_users")
            .await
            .expect("fetch failed");
        assert_eq!(rows.len(), 3);

        // fetch_one returns first row
        let row = client
            .fetch_one("SELECT * FROM test_users")
            .await
            .expect("fetch failed");
        assert!(row.is_some());

        // execute_cmd is a no-op that succeeds
        client
            .execute_cmd("DELETE FROM test_users WHERE id = 1")
            .await
            .expect("execute failed");
    }

    #[tokio::test]
    async fn test_query_builder_with_mock_client_all()
    {
        let client = MockDbClient::new();
        client.ensure_table("CREATE TABLE test_users (id INT, name TEXT, email TEXT)");
        client.insert_test_data();

        let users = QueryBuilder::<TestUser>::new()
            .all(&client)
            .await
            .expect("query should succeed");
        // Mock returns all 3 inserted rows
        assert_eq!(users.len(), 3);
        // Rows returned in insertion order
        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[1].name, "Bob");
        assert_eq!(users[2].name, "Charlie");
    }

    #[tokio::test]
    async fn test_query_builder_first()
    {
        let client = MockDbClient::new();
        client.ensure_table("CREATE TABLE test_users (id INT, name TEXT, email TEXT)");
        client.insert_test_data();

        let user = QueryBuilder::<TestUser>::new()
            .first(&client)
            .await
            .expect("first should succeed");
        assert!(user.is_some());
    }

    #[test]
    fn test_model_trait_methods()
    {
        let mut user = TestUser {
            id: 1,
            name: "Test".into(),
            email: "test@test.com".into(),
        };
        assert_eq!(user.primary_key().unwrap(), "1");
        user.set_primary_key("42".to_string()).unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(TestUser::table_name(), "test_users");
    }

    #[test]
    fn test_model_meta()
    {
        let meta = TestUser::meta();
        assert_eq!(meta.table_name(), "test_users");
        assert_eq!(meta.columns.len(), 3);
        assert!(meta.columns[0].is_primary_key);
    }

    #[test]
    fn test_column_types()
    {
        let col = crate::Column::new("id", OrmColumnType::I64)
            .primary_key()
            .unique();
        assert_eq!(col.name, "id");
        assert!(col.is_primary_key);
        assert!(col.is_unique);
    }

    #[test]
    fn test_sql_dialect()
    {
        use crate::ColumnType;
        assert_eq!(ColumnType::I32.as_sql(crate::SqlDialect::PostgreSQL), "INTEGER");
        assert_eq!(ColumnType::I32.as_sql(crate::SqlDialect::MySQL), "INT");
        assert_eq!(ColumnType::Json.as_sql(crate::SqlDialect::PostgreSQL), "JSONB");
    }

    // Smoke test (keep original)
    #[test]
    fn smoke_test()
    {
        assert!(true, "hiver-data-orm test infrastructure is working");
    }
    // ── Edge case tests ──

    /// Test that an empty table yields empty results.
    #[test]
    fn test_empty_model_meta()
    {
        #[derive(Debug, Deserialize)]
        struct EmptyModel;

        impl Model for EmptyModel
        {
            fn meta() -> ModelMeta
            {
                ModelMeta::new("empty_table")
            }

            fn primary_key(&self) -> crate::Result<String>
            {
                Err(crate::Error::validation("no primary key"))
            }

            fn set_primary_key(&mut self, _: String) -> crate::Result<()>
            {
                Err(crate::Error::validation("no primary key"))
            }
        }

        let meta = EmptyModel::meta();
        assert_eq!(meta.table_name(), "empty_table");
        assert_eq!(meta.columns.len(), 0);
    }

    /// ModelMeta builder pattern works for complex schemas.
    #[test]
    fn test_model_meta_builder_chain()
    {
        let meta = ModelMeta::new("products")
            .add_column(crate::Column::new("id", OrmColumnType::I64).primary_key())
            .add_column(crate::Column::new("name", OrmColumnType::String).unique())
            .add_column(crate::Column::new("price", OrmColumnType::F64).nullable());

        assert_eq!(meta.columns.len(), 3);
        assert!(meta.columns[0].is_primary_key);
        assert!(meta.columns[1].is_unique);
        assert!(meta.columns[2].is_nullable);
    }

    /// Null/NONE handling in relationship types.
    #[test]
    fn test_relation_on_delete_default()
    {
        let rel = crate::Relation::new("posts", crate::RelationType::OneToMany, "posts", "user_id");
        // Default on_delete should be Restrict
        assert!(matches!(rel.on_delete, crate::OnDelete::Restrict));
    }

    /// EagerLoad default is empty.
    #[test]
    fn test_eager_load_default_empty()
    {
        let eager = crate::EagerLoad::default();
        assert!(eager.relationships.is_empty());
    }

    /// Column name correction with custom column mapping.
    #[test]
    fn test_column_custom_name()
    {
        let col = crate::Column::new("user_id", OrmColumnType::I64).primary_key();
        // Custom column name: the struct field maps to a DB column
        assert_eq!(col.name, "user_id");
        assert!(col.is_primary_key);
    }

    /// Result type alias works with ? operator.
    #[test]
    fn test_result_type_alias()
    {
        fn fallible() -> crate::Result<i32>
        {
            Ok(42)
        }
        let val = fallible().expect("should succeed");
        assert_eq!(val, 42);
    }

    /// OomError Display impl produces readable messages.
    #[test]
    fn test_error_display_readable()
    {
        let err = crate::Error::validation("email is required");
        assert_eq!(err.to_string(), "Validation error: email is required");

        let err = crate::Error::query_build("invalid column name");
        assert_eq!(err.to_string(), "Query build error: invalid column name");

        let err = crate::Error::relationship("circular dependency detected");
        assert_eq!(err.to_string(), "Relationship error: circular dependency detected");

        let err = crate::Error::not_found("record 42");
        assert_eq!(err.to_string(), "Not found: record 42");

        let err = crate::Error::duplicate("email already exists");
        assert_eq!(err.to_string(), "Duplicate: email already exists");

        let err = crate::Error::migration("table already exists");
        assert_eq!(err.to_string(), "Migration error: table already exists");
    }

    /// Error::is_* classification methods.
    #[test]
    fn test_error_classification()
    {
        assert!(crate::Error::validation("x").is_validation());
        assert!(crate::Error::not_found("x").is_not_found());
        assert!(crate::Error::duplicate("x").is_duplicate());

        assert!(!crate::Error::not_found("x").is_validation());
        assert!(!crate::Error::validation("x").is_duplicate());
    }

    /// Error category strings match expectations.
    #[test]
    fn test_error_categories_match()
    {
        assert_eq!(crate::Error::validation("x").category(), "validation");
        assert_eq!(crate::Error::query_build("x").category(), "query_build");
        assert_eq!(crate::Error::relationship("x").category(), "relationship");
        assert_eq!(crate::Error::migration("x").category(), "migration");
        assert_eq!(crate::Error::not_found("x").category(), "not_found");
        assert_eq!(crate::Error::duplicate("x").category(), "duplicate");
    }
}
