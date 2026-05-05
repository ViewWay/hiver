//! Entity annotation macro
//! @Entity 派生宏实现

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Represents attributes on struct or fields
/// 表示结构体或字段上的属性
#[derive(Debug, FromDeriveInput)]
struct EntityArgs {
    table: Option<String>,
}

/// Implements #[Entity] derive macro
/// 实现 #[Entity] 派生宏
///
/// Marks a struct as a JPA Entity
/// 将结构体标记为 JPA 实体
///
/// # Example / 示例
///
/// ```rust
/// use nexus_data_annotations::Entity;
///
/// #[Entity]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
/// ```
pub(crate) fn impl_entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // Parse Entity attributes if any
    // 解析 Entity 属性（如果有）
    let _attrs = EntityArgs::from_derive_input(&input);

    // Generate implementation
    // 生成实现
    let expanded = quote! {
        #input

        impl #name {
            /// Returns the table name for this entity
            /// 返回此实体的表名
            pub fn table_name() -> &'static str {
                stringify!(#name)
            }

            /// Returns all field names of this entity
            /// 返回此实体的所有字段名
            pub fn field_names() -> &'static [&'static str] {
                &[]  // Placeholder, actual implementation would extract fields
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[Table] attribute macro
/// 实现 #[Table] 属性宏
///
/// Specifies the database table for the entity
/// 指定实体的数据库表
///
/// # Example / 示例
///
/// ```rust
/// use nexus_data_annotations::Table;
///
/// #[Table(name = "users")]
/// pub struct User {
///     pub id: i64,
///     pub username: String,
/// }
/// ```
pub(crate) fn impl_table(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // Parse table name from attribute
    // 从属性中解析表名
    let table_name = if attr.is_empty() {
        // Default to struct name in lowercase
        // 默认为小写的结构体名
        name.to_string().to_lowercase()
    } else {
        // Parse custom table name
        // 解析自定义表名
        attr.to_string().trim_matches('"').to_string()
    };

    let expanded = quote! {
        #input

        impl #name {
            /// Returns the database table name
            /// 返回数据库表名
            pub fn table_name() -> &'static str {
                #table_name
            }
        }
    };

    TokenStream::from(expanded)
}
