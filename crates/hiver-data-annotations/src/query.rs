//! @Query annotation macro
//! @Query 注解宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, parse_macro_input};
use syn::{Result as SynResult, parse::Parse, parse::ParseStream};

/// Parses arguments from @Query annotation
/// 解析 @Query 注解的参数
struct QueryArgs {
    query: String,
}

impl Parse for QueryArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Try to parse a string literal
        // 尝试解析字符串字面量
        let query_lit: LitStr = input.parse()?;

        Ok(QueryArgs {
            query: query_lit.value(),
        })
    }
}

/// Implements #[Query] attribute macro
/// 实现 #[Query] 属性宏
///
/// Specifies a custom SQL query for a repository method
/// 为 repository 方法指定自定义 SQL 查询
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::Query;
/// use hiver_data::Repository;
///
/// trait UserRepository: Repository<User, i64> {
///     #[Query("SELECT * FROM users WHERE username = :username")]
///     async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
/// }
/// ```
pub(crate) fn impl_query(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse query from attribute
    // 从属性中解析查询
    let args = parse_macro_input!(attr as QueryArgs);
    let query_sql = args.query;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the SQL query for this method
            /// 返回此方法的 SQL 查询
            const QUERY: &str = #query_sql;

            #[doc(hidden)]
            fn _hiver_get_query() -> &'static str {
                QUERY
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[Insert] attribute macro
/// 实现 #[Insert] 属性宏
///
/// Marks a method as an insert operation
/// 将方法标记为插入操作
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::Insert;
///
/// trait UserRepository {
///     #[Insert("INSERT INTO users (username, email) VALUES (:username, :email)")]
///     async fn insert_user(&self, username: &str, email: &str) -> Result<u64, Error>;
/// }
/// ```
pub(crate) fn impl_insert(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse INSERT SQL from attribute
    // 从属性中解析 INSERT SQL
    let args = parse_macro_input!(attr as QueryArgs);
    let insert_sql = args.query;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the INSERT SQL for this method
            /// 返回此方法的 INSERT SQL
            const QUERY: &str = #insert_sql;

            #[doc(hidden)]
            fn _hiver_get_insert_sql() -> &'static str {
                QUERY
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[Update] attribute macro
/// 实现 #[Update] 属性宏
///
/// Marks a method as an update operation
/// 将方法标记为更新操作
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::Update;
///
/// trait UserRepository {
///     #[Update("UPDATE users SET email = :email WHERE id = :id")]
///     async fn update_email(&self, id: i64, email: &str) -> Result<u64, Error>;
/// }
/// ```
pub(crate) fn impl_update(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse UPDATE SQL from attribute
    // 从属性中解析 UPDATE SQL
    let args = parse_macro_input!(attr as QueryArgs);
    let update_sql = args.query;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the UPDATE SQL for this method
            /// 返回此方法的 UPDATE SQL
            const QUERY: &str = #update_sql;

            #[doc(hidden)]
            fn _hiver_get_update_sql() -> &'static str {
                QUERY
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[Delete] attribute macro
/// 实现 #[Delete] 属性宏
///
/// Marks a method as a delete operation
/// 将方法标记为删除操作
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_annotations::Delete;
///
/// trait UserRepository {
///     #[Delete("DELETE FROM users WHERE id = :id")]
///     async fn delete_by_id(&self, id: i64) -> Result<u64, Error>;
/// }
/// ```
pub(crate) fn impl_delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse DELETE SQL from attribute
    // 从属性中解析 DELETE SQL
    let args = parse_macro_input!(attr as QueryArgs);
    let delete_sql = args.query;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the DELETE SQL for this method
            /// 返回此方法的 DELETE SQL
            const QUERY: &str = #delete_sql;

            #[doc(hidden)]
            fn _hiver_get_delete_sql() -> &'static str {
                QUERY
            }
        }
    };

    TokenStream::from(expanded)
}
