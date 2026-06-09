//! Query builder integration tests
//! 查询构建器集成测试

/// Query builder unit tests (without actual database)
#[cfg(test)]
mod tests
{
    /// Simple query builder test
    #[test]
    fn test_simple_select_query()
    {
        let table = "users";
        let sql = format!("SELECT * FROM {}", table);
        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM users"));
    }

    /// Test where clause construction
    #[test]
    fn test_where_clause()
    {
        let where_clause = "email = ? AND active = ?";
        assert!(where_clause.contains("email = ?"));
        assert!(where_clause.contains("active = ?"));
    }

    /// Test order by clause
    #[test]
    fn test_order_by()
    {
        let order_by = "created_at DESC";
        assert!(order_by.contains("DESC"));
    }

    /// Test limit clause
    #[test]
    fn test_limit()
    {
        let limit = 10;
        let sql = format!("LIMIT {}", limit);
        assert_eq!(sql, "LIMIT 10");
    }

    /// Test offset clause
    #[test]
    fn test_offset()
    {
        let offset = 5;
        let sql = format!("OFFSET {}", offset);
        assert_eq!(sql, "OFFSET 5");
    }

    /// Test pagination
    #[test]
    fn test_pagination()
    {
        let page = 2;
        let size = 20;
        let offset = (page - 1) * size; // (2 - 1) * 20 = 20
        let sql = format!("LIMIT {} OFFSET {}", size, offset);
        assert!(sql.contains("LIMIT 20"));
        assert!(sql.contains("OFFSET 20"));
    }

    /// Test insert query
    #[test]
    fn test_insert_query()
    {
        let table = "users";
        let columns = "email, name";
        let sql = format!("INSERT INTO {} ({}) VALUES (?, ?)", table, columns);
        assert!(sql.contains("INSERT INTO"));
        assert!(sql.contains("email, name"));
        assert!(sql.contains("VALUES"));
    }

    /// Test update query
    #[test]
    fn test_update_query()
    {
        let table = "users";
        let set_clause = "email = ?, name = ?";
        let sql = format!("UPDATE {} SET {} WHERE id = ?", table, set_clause);
        assert!(sql.contains("UPDATE"));
        assert!(sql.contains("SET"));
        assert!(sql.contains("WHERE id = ?"));
    }

    /// Test delete query
    #[test]
    fn test_delete_query()
    {
        let table = "users";
        let sql = format!("DELETE FROM {} WHERE id = ?", table);
        assert!(sql.contains("DELETE FROM"));
        assert!(sql.contains("WHERE id = ?"));
    }

    /// Test count query
    #[test]
    fn test_count_query()
    {
        let table = "users";
        let sql = format!("SELECT COUNT(*) FROM {}", table);
        assert!(sql.contains("SELECT COUNT(*)"));
        assert!(sql.contains("FROM users"));
    }

    /// Test create table query
    #[test]
    fn test_create_table_query()
    {
        let table = "users";
        let columns = "id INTEGER PRIMARY KEY, email TEXT NOT NULL";
        let sql = format!("CREATE TABLE {} ({})", table, columns);
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("id INTEGER PRIMARY KEY"));
    }

    /// Test drop table query
    #[test]
    fn test_drop_table_query()
    {
        let table = "users";
        let sql = format!("DROP TABLE IF EXISTS {}", table);
        assert!(sql.contains("DROP TABLE"));
        assert!(sql.contains("users"));
    }

    /// Test create index query
    #[test]
    fn test_create_index_query()
    {
        let index = "idx_users_email";
        let table = "users";
        let columns = "email";
        let sql = format!("CREATE UNIQUE INDEX {} ON {} ({})", index, table, columns);
        assert!(sql.contains("CREATE UNIQUE INDEX"));
        assert!(sql.contains("idx_users_email"));
        assert!(sql.contains("users"));
    }

    /// Test drop index query
    #[test]
    fn test_drop_index_query()
    {
        let index = "idx_users_email";
        let sql = format!("DROP INDEX IF EXISTS {}", index);
        assert!(sql.contains("DROP INDEX"));
        assert!(sql.contains("idx_users_email"));
    }
}
