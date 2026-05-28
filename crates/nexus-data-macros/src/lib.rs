//! Procedural macros for Nexus data layer
//! Nexus 数据层的过程宏
//!
//! # Overview / 概述
//!
//! This module provides procedural macros for Spring Data-like repository functionality.
//! 本模块为类似Spring Data的Repository功能提供过程宏。
//!
//! # Features / 功能
//!
//! - `#[derive(Model)]` - Automatically implement Model trait
//! - `#[model]` attribute - Configure model metadata
//! - `#[Repository]` - Auto-generate CRUD repository implementation
//! - `#[Query]` - Custom query annotation
//! - Method name parsing - `findByUsernameAndEmail` style queries
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_data_macros::{Model, Repository};
//! use nexus_data_commons::CrudRepository;
//!
//! #[derive(Model, Debug, Clone)]
//! #[model(table = "users")]
//! struct User {
//!     #[model(primary_key)]
//!     id: i64,
//!     name: String,
//!     email: String,
//! }
//!
//! #[Repository]
//! trait UserRepository: CrudRepository<User, i64> {
//!     // Auto-generated from method name: SELECT * FROM users WHERE username = ?
//!     async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
//!
//!     // Auto-generated: SELECT * FROM users WHERE email = ? AND active = ?
//!     async fn find_by_email_and_active(&self, email: &str, active: bool) -> Result<Vec<User>, Error>;
//!
//!     // Custom query with SQL
//!     #[Query("SELECT * FROM users WHERE age > ? ORDER BY created_at DESC")]
//!     async fn find_by_age_greater_than(&self, age: i32) -> Result<Vec<User>, Error>;
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields};
use syn::Type;
use proc_macro2::{Ident, Span, Literal};

/// Derive macro for Model trait
/// Model trait 的 derive 宏
///
/// Automatically implements the Model trait and generates metadata.
/// 自动实现 Model trait 并生成元数据。
///
/// # Attributes / 属性
///
/// ## Struct-level attributes / 结构体级别属性
///
/// - `table = "name"` - Set the table name (default: struct name in snake_case)
/// - `schema = "name"` - Set the schema name (default: "public")
///
/// ## Field-level attributes / 字段级别属性
///
/// - `primary_key` - Mark as primary key
/// - `unique` - Add unique constraint
/// - `nullable` - Allow null values
/// - `default = "expression"` - Set default value (SQL expression)
/// - `max_length = n` - Set maximum length for string types
/// - `ignore` - Exclude from model
/// - `column = "name"` - Set custom column name
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// #[derive(Model)]
/// #[model(table = "users")]
/// struct User {
///     #[model(primary_key)]
///     id: i64,
///
///     #[model(max_length = 255, unique)]
///     email: String,
/// }
/// ```
#[proc_macro_derive(Model, attributes(model))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    model_derive_impl(input)
}

/// Field-level model attributes
#[derive(Debug, Default)]
struct FieldAttrs {
    primary_key: bool,
    unique: bool,
    nullable: bool,
    default: Option<String>,
    max_length: Option<usize>,
    column: Option<String>,
    ignore: bool,
    /// Marks this field as the optimistic-lock version field (@Version equivalent)
    version: bool,
}

/// Struct-level model attributes
#[derive(Debug, Default)]
struct StructAttrs {
    table: Option<String>,
    schema: Option<String>,
}

/// Parse field attributes
fn parse_field_attrs(attrs: &[syn::Attribute]) -> FieldAttrs {
    let mut result = FieldAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("model") {
            continue;
        }

        if let Ok(list) = attr.meta.require_list() {
            for nested in list.tokens.to_string().split(',') {
                let nested = nested.trim();
                if nested.is_empty() {
                    continue;
                }

                if nested == "primary_key" {
                    result.primary_key = true;
                } else if nested == "unique" {
                    result.unique = true;
                } else if nested == "nullable" {
                    result.nullable = true;
                } else if nested == "ignore" {
                    result.ignore = true;
                } else if nested == "version" {
                    result.version = true;
                } else if nested.contains('=') {
                    let parts: Vec<&str> = nested.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim().trim_matches('"').trim_matches('\'');

                        match key {
                            "default" => result.default = Some(value.to_string()),
                            "max_length" => {
                                if let Ok(n) = value.parse::<usize>() {
                                    result.max_length = Some(n);
                                }
                            }
                            "column" => result.column = Some(value.to_string()),
                            "table" => { /* handled at struct level */ }
                            "schema" => { /* handled at struct level */ }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    result
}

/// Parse struct attributes
fn parse_struct_attrs(attrs: &[syn::Attribute]) -> StructAttrs {
    let mut result = StructAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("model") {
            continue;
        }

        if let Ok(list) = attr.meta.require_list() {
            let tokens = list.tokens.to_string();
            for pair in tokens.split(',') {
                let pair = pair.trim();
                if pair.contains('=') {
                    let parts: Vec<&str> = pair.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim().trim_matches('"').trim_matches('\'');

                        match key {
                            "table" => result.table = Some(value.to_string()),
                            "schema" => result.schema = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    result
}

/// Field info collected during macro expansion for non-ignored fields.
/// 宏展开期间收集的非忽略字段信息。
struct FieldInfo {
    /// Field identifier
    field_ident: Ident,
    /// Column name (custom or same as field name)
    column_name: String,
    /// Whether this field is nullable
    nullable: bool,
    /// Field type tokens
    field_type: Type,
}

/// Implementation of the Model derive macro
/// Model derive 宏的实现
fn model_derive_impl(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let vis = &input.vis;

    // Parse struct-level attributes
    let struct_attrs = parse_struct_attrs(&input.attrs);

    // Default table name: struct name in snake_case
    let table_name = struct_attrs.table.unwrap_or_else(|| to_snake_case(&struct_name.to_string()));

    // Parse fields and generate column metadata
    let mut column_definitions = Vec::new();
    let mut primary_keys = Vec::new();
    let mut field_names = Vec::new();
    let mut version_field: Option<Ident> = None;
    let mut field_infos: Vec<FieldInfo> = Vec::new();
    let mut ignored_fields: Vec<Ident> = Vec::new();

    if let Data::Struct(DataStruct { fields, .. }) = &input.data
        && let Fields::Named(named_fields) = fields {
            for field in &named_fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                field_names.push(field_name.clone());

                // Parse field attributes
                let field_attrs = parse_field_attrs(&field.attrs);

                if field_attrs.ignore {
                    ignored_fields.push(field_name.clone());
                    continue;
                }

                if field_attrs.primary_key {
                    primary_keys.push(field_name_str.clone());
                }

                if field_attrs.version {
                    version_field = Some(field_name.clone());
                }

                // Get the type
                let field_type = &field.ty;
                let column_type = map_type_to_column_type(field_type);

                // Column name (custom or field name)
                let column_name = field_attrs.column.unwrap_or_else(|| field_name_str.clone());

                field_infos.push(FieldInfo {
                    field_ident: field_name.clone(),
                    column_name: column_name.clone(),
                    nullable: field_attrs.nullable,
                    field_type: field_type.clone(),
                });

                // Build column metadata
                let primary_key_method = if field_attrs.primary_key {
                    quote! { .primary_key() }
                } else {
                    quote! {}
                };

                let unique_method = if field_attrs.unique {
                    quote! { .unique() }
                } else {
                    quote! {}
                };

                let nullable_method = if field_attrs.nullable {
                    quote! { .nullable() }
                } else {
                    quote! {}
                };

                let default_method = if let Some(default) = &field_attrs.default {
                    let default_lit = Literal::string(default);
                    quote! { .with_default(#default_lit) }
                } else {
                    quote! {}
                };

                column_definitions.push(quote! {
                    nexus_data_orm::Column::new(#column_name, #column_type)
                        #primary_key_method
                        #unique_method
                        #nullable_method
                        #default_method
                });
            }
        }

    // Generate primary_key and set_primary_key implementations
    let pk_field = if !primary_keys.is_empty() {
        Ident::new(&primary_keys[0], Span::call_site())
    } else {
        Ident::new("id", Span::call_site())
    };

    // Generate OptimisticLock impl if a version field was detected
    let optimistic_lock_impl = if let Some(ver_field) = &version_field {
        let ver_field_str = ver_field.to_string();
        quote! {
            impl #vis nexus_data_orm::OptimisticLock for #struct_name {
                fn version(&self) -> i64 {
                    self.#ver_field as i64
                }
                fn version_column() -> &'static str {
                    #ver_field_str
                }
            }
        }
    } else {
        quote! {}
    };

    // Build column_names static list
    let column_name_strs: Vec<&str> = field_infos.iter().map(|fi| fi.column_name.as_str()).collect();
    let column_name_count = column_name_strs.len();

    // Build from_row field extractions (non-ignored fields from row, ignored fields from Default)
    let from_row_fields: Vec<proc_macro2::TokenStream> = field_infos.iter().map(|fi| {
        let ident = &fi.field_ident;
        let col = &fi.column_name;
        let ty = &fi.field_type;
        if fi.nullable {
            // Nullable fields use Default (Option<T> defaults to None).
            // Attempt to extract from row; if column is missing/Null, Default applies.
            // nullable 字段使用 Default（Option<T> 默认为 None）。
            quote! {
                #ident: row.get_as::<#ty>(#col).unwrap_or_default()
            }
        } else {
            quote! {
                #ident: row.get_as::<#ty>(#col).map_err(|e| nexus_data_orm::Error::unknown(
                    format!("failed to read column '{}': {}", #col, e)
                ))?
            }
        }
    }).chain(ignored_fields.iter().map(|ident| {
        quote! { #ident: Default::default() }
    })).collect();

    // Build to_row field conversions
    let to_row_fields: Vec<proc_macro2::TokenStream> = field_infos.iter().map(|fi| {
        let ident = &fi.field_ident;
        let col = &fi.column_name;
        quote! {
            (#col, Box::new(self.#ident.clone()) as Box<dyn std::any::Any + Send>)
        }
    }).collect();

    // Generate the Model implementation
    let expanded = quote! {
        // Implement Model trait
        impl #vis nexus_data_orm::Model for #struct_name {
            fn meta() -> nexus_data_orm::ModelMeta {
                nexus_data_orm::ModelMeta::new(#table_name)
                    #(.add_column(#column_definitions))*
            }

            fn primary_key(&self) -> nexus_data_orm::Result<String> {
                Ok(format!("{}", self.#pk_field))
            }

            fn set_primary_key(&mut self, value: String) -> nexus_data_orm::Result<()> {
                self.#pk_field = value.parse().map_err(|e| {
                    nexus_data_orm::Error::validation(format!("Invalid primary key: {}", e))
                })?;
                Ok(())
            }

            /// Get all column names in declaration order.
            /// 获取声明顺序的所有列名。
            fn column_names() -> &'static [&'static str] {
                &[#(#column_name_strs),*]
            }

            /// Get the number of columns.
            /// 获取列数量。
            fn column_count() -> usize {
                #column_name_count
            }

            /// Construct the model from a database row.
            /// 从数据库行构造模型实例。
            fn from_row(row: &nexus_data_rdbc::Row) -> nexus_data_orm::Result<Self> {
                Ok(Self {
                    #(#from_row_fields),*
                })
            }

            /// Convert the model to column-value pairs for INSERT/UPDATE.
            /// 将模型转换为列-值对，用于 INSERT/UPDATE。
            fn to_row(&self) -> Vec<(&'static str, Box<dyn std::any::Any + Send>)> {
                vec![
                    #(#to_row_fields),*
                ]
            }
        }

        #optimistic_lock_impl

        // Add display implementation
        impl std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!(#struct_name), self.#pk_field)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Map Rust type to database column type
/// 将 Rust 类型映射到数据库列类型
fn map_type_to_column_type(ty: &Type) -> proc_macro2::TokenStream {
    let type_str = quote!(#ty).to_string();

    // Remove spaces and generics for matching
    let simplified = type_str
        .replace(" ", "")
        .replace("alloc::string::String", "String")
        .replace("&str", "str");

    

    match simplified.as_str() {
        "i8" => quote!(nexus_data_orm::ColumnType::I8),
        "i16" => quote!(nexus_data_orm::ColumnType::I16),
        "i32" => quote!(nexus_data_orm::ColumnType::I32),
        "i64" => quote!(nexus_data_orm::ColumnType::I64),
        "i128" => quote!(nexus_data_orm::ColumnType::I128),
        "u8" => quote!(nexus_data_orm::ColumnType::U8),
        "u16" => quote!(nexus_data_orm::ColumnType::U16),
        "u32" => quote!(nexus_data_orm::ColumnType::U32),
        "u64" => quote!(nexus_data_orm::ColumnType::U64),
        "f32" => quote!(nexus_data_orm::ColumnType::F32),
        "f64" => quote!(nexus_data_orm::ColumnType::F64),
        "bool" => quote!(nexus_data_orm::ColumnType::Bool),
        "String" | "string" => quote!(nexus_data_orm::ColumnType::String),
        "str" => quote!(nexus_data_orm::ColumnType::String),
        "Vec<u8>" => quote!(nexus_data_orm::ColumnType::Bytes),
        "Uuid" | "uuid" => quote!(nexus_data_orm::ColumnType::Uuid),
        "DateTime" | "chrono::DateTime" => quote!(nexus_data_orm::ColumnType::Timestamp),
        "Date" | "chrono::Date" => quote!(nexus_data_orm::ColumnType::Date),
        "serde_json::Value" => quote!(nexus_data_orm::ColumnType::Json),
        _ => quote!(nexus_data_orm::ColumnType::Text),
    }
}

/// Convert CamelCase/PascalCase to snake_case
/// 将 CamelCase/PascalCase 转换为 snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            // Add underscore if:
            // 1. Not the first character
            // 2. Previous character is lowercase (camelCase boundary)
            // 3. Next character is lowercase and we're in an acronym (XMLHttp -> xml_http)
            let prev_is_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

            if prev_is_lower || (next_is_lower && i > 0 && chars[i - 1].is_uppercase()) {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

// =============================================================================
// Repository Macro Implementation / Repository 宏实现
// =============================================================================

/// Repository attribute macro
/// Repository 属性宏
///
/// Automatically generates CRUD repository implementation similar to Spring Data JPA.
/// 自动生成类似Spring Data JPA的CRUD Repository实现。
///
/// # Features / 功能
///
/// - Auto-implements `CrudRepository` methods
/// - Parses method names to generate queries (e.g., `findByUsernameAndEmail`)
/// - Supports `@Query` annotation for custom SQL
/// - Supports pagination with `PageRequest`
/// - Supports sorting with `Sort`
///
/// # Method Name Patterns / 方法名模式
///
/// | Pattern | SQL Generated | Example |
/// |---------|--------------|---------|
/// | `findByXxx` | `WHERE xxx = ?` | `findByUsername` |
/// | `findByXxxAndYyy` | `WHERE xxx = ? AND yyy = ?` | `findByUsernameAndActive` |
/// | `findByXxxOrYyy` | `WHERE xxx = ? OR yyy = ?` | `findByUsernameOrEmail` |
/// | `findAllByXxx` | `WHERE xxx = ?` | `findAllByActive` |
/// | `countByXxx` | `SELECT COUNT(*) WHERE xxx = ?` | `countByActive` |
/// | `existsByXxx` | `SELECT EXISTS(...)` | `existsByEmail` |
/// | `deleteByXxx` | `DELETE WHERE xxx = ?` | `deleteByUserId` |
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use nexus_data_macros::Repository;
/// use nexus_data_commons::{CrudRepository, Result, Error};
///
/// #[Repository]
/// pub trait UserRepository: CrudRepository<User, i64> {
///     // Auto-generated: SELECT * FROM users WHERE username = ? LIMIT 1
///     async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
///
///     // Auto-generated: SELECT * FROM users WHERE active = ? AND age > ?
///     async fn find_by_active_and_age_greater_than(
///         &self,
///         active: bool,
///         age: i32
///     ) -> Result<Vec<User>, Error>;
///
///     // Custom query
///     #[Query("SELECT * FROM users WHERE email LIKE ?")]
///     async fn search_by_email_pattern(&self, pattern: &str) -> Result<Vec<User>, Error>;
/// }
/// ```
#[proc_macro_attribute]
pub fn repository(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input_trait = parse_macro_input!(input as syn::ItemTrait);
    repository_impl(input_trait)
}

/// Implementation of the Repository macro
fn repository_impl(trait_def: syn::ItemTrait) -> TokenStream {
    let trait_name = &trait_def.ident;
    let vis = &trait_def.vis;

    // Extract entity type and ID type from super traits
    let (entity_type, id_type) = extract_super_traits(&trait_def.supertraits);

    // Extract table name from entity type
    let table_name = to_snake_case(&entity_type.to_string());

    // Generate repository struct
    let struct_name = Ident::new(
        &format!("{}Impl", trait_name),
        trait_name.span()
    );

    // Parse methods and generate implementations
    let mut method_impls = Vec::new();
    let mut query_methods = Vec::new();

    for item in &trait_def.items {
        if let syn::TraitItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();

            // Check for @Query annotation
            let query_sql = extract_query_attr(&method.attrs);

            if let Some(sql) = query_sql {
                // Custom query implementation
                query_methods.push(generate_query_method(method, &sql, &entity_type));
            } else if method_name_str.starts_with("find_by")
                || method_name_str.starts_with("count_by")
                || method_name_str.starts_with("exists_by")
                || method_name_str.starts_with("delete_by")
                || method_name_str.starts_with("find_all_by")
            {
                // Derive query from method name
                method_impls.push(generate_derived_query_method(
                    method,
                    &table_name,
                    &entity_type,
                ));
            }
        }
    }

    // Generate the struct and impl
    let expanded = quote! {
        // Repository struct
        #vis struct #struct_name {
            client: std::sync::Arc<nexus_data_rdbc::SqlxPoolClient>,
            table: &'static str,
        }

        impl #struct_name {
            /// Create a new repository instance
            pub fn new(client: std::sync::Arc<nexus_data_rdbc::SqlxPoolClient>) -> Self {
                Self {
                    client,
                    table: #table_name,
                }
            }

            /// Get the table name
            pub fn table(&self) -> &str {
                self.table
            }

            /// Get the database client
            pub fn client(&self) -> &std::sync::Arc<nexus_data_rdbc::SqlxPoolClient> {
                &self.client
            }
        }

        // Implement CrudRepository using QueryBuilder/ActiveRecord
        #[async_trait::async_trait]
        impl nexus_data_commons::CrudRepository<#entity_type, #id_type> for #struct_name {
            type Error = nexus_data_commons::Error;

            async fn save(&self, entity: #entity_type) -> Result<#entity_type, Self::Error> {
                let pk = entity.primary_key().map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                let sql = format!(
                    "INSERT INTO {} (id) VALUES ({}) ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id",
                    self.table, pk
                );
                self.client.execute_cmd(&sql).await.map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                Ok(entity)
            }

            async fn save_all(&self, entities: Vec<#entity_type>) -> Result<Vec<#entity_type>, Self::Error> {
                let mut results = Vec::new();
                for entity in entities {
                    results.push(self.save(entity).await?);
                }
                Ok(results)
            }

            async fn find_by_id(&self, id: #id_type) -> Result<Option<#entity_type>, Self::Error> {
                let sql = format!("SELECT * FROM {} WHERE id = {}", self.table, id);
                match self.client.fetch_one(&sql).await.map_err(|e| nexus_data_commons::Error::other(e.to_string()))? {
                    Some(row) => row.deserialize().map(Some).map_err(|e| nexus_data_commons::Error::other(e.to_string())),
                    None => Ok(None),
                }
            }

            async fn exists_by_id(&self, id: #id_type) -> Result<bool, Self::Error> {
                Ok(self.find_by_id(id).await?.is_some())
            }

            async fn find_all(&self) -> Result<Vec<#entity_type>, Self::Error> {
                let sql = format!("SELECT * FROM {}", self.table);
                let rows = self.client.fetch_all(&sql).await.map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                let mut results = Vec::with_capacity(rows.len());
                for row in &rows {
                    results.push(row.deserialize().map_err(|e| nexus_data_commons::Error::other(e.to_string()))?);
                }
                Ok(results)
            }

            async fn find_all_by_ids(&self, ids: Vec<#id_type>) -> Result<Vec<#entity_type>, Self::Error> {
                let mut results = Vec::new();
                for id in ids {
                    if let Some(entity) = self.find_by_id(id).await? {
                        results.push(entity);
                    }
                }
                Ok(results)
            }

            async fn count(&self) -> Result<usize, Self::Error> {
                let sql = format!("SELECT COUNT(*) AS cnt FROM {}", self.table);
                let rows = self.client.fetch_all(&sql).await.map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                let cnt = rows.first().and_then(|r| r.get_as::<i64>("cnt").ok()).unwrap_or(0);
                Ok(cnt as usize)
            }

            async fn delete_by_id(&self, id: #id_type) -> Result<bool, Self::Error> {
                let sql = format!("DELETE FROM {} WHERE id = {}", self.table, id);
                let affected = self.client.execute_cmd(&sql).await.map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                Ok(affected > 0)
            }

            async fn delete(&self, entity: #entity_type) -> Result<bool, Self::Error> {
                let pk = entity.primary_key().map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                let sql = format!("DELETE FROM {} WHERE id = {}", self.table, pk);
                let affected = self.client.execute_cmd(&sql).await.map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                Ok(affected > 0)
            }

            async fn delete_all(&self) -> Result<usize, Self::Error> {
                let sql = format!("DELETE FROM {}", self.table);
                let affected = self.client.execute_cmd(&sql).await.map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                Ok(affected as usize)
            }

            async fn delete_all_by_ids(&self, ids: Vec<#id_type>) -> Result<usize, Self::Error> {
                let mut count = 0;
                for id in ids {
                    if self.delete_by_id(id).await? {
                        count += 1;
                    }
                }
                Ok(count)
            }

            async fn delete_all_by_entities(&self, entities: Vec<#entity_type>) -> Result<usize, Self::Error> {
                let mut count = 0;
                for entity in entities {
                    if self.delete(entity).await? {
                        count += 1;
                    }
                }
                Ok(count)
            }
        }

        // Implement the repository trait with custom methods
        #[async_trait::async_trait]
        impl #trait_name for #struct_name {
            #(#method_impls)*
            #(#query_methods)*
        }

        // Constructor for convenience
        impl #struct_name {
            pub fn with_url(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
                use nexus_data_rdbc::{DatabaseConfig, SqlxPoolClient};
                let config = DatabaseConfig::from_url(url)?;
                let client = std::sync::Arc::new(SqlxPoolClient::new(&config.url())?);
                Ok(Self::new(client))
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract entity and ID types from super traits
fn extract_super_traits(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut entity_type = None;
    let mut id_type = None;

    for bound in bounds {
        let bound_str = quote!(#bound).to_string();

        // Look for CrudRepository<Entity, Id>
        if bound_str.contains("CrudRepository") {
            // Extract types from CrudRepository<Entity, Id>
            let parts: Vec<&str> = bound_str.split('<').collect();
            if parts.len() > 1 {
                let inner = parts[1].trim_end_matches('>');
                let type_params: Vec<&str> = inner.split(',').collect();
                if type_params.len() >= 2 {
                    let entity = type_params[0].trim();
                    let id = type_params[1].trim();
                    entity_type = Some(entity.to_string());
                    id_type = Some(id.to_string());
                }
            }
        }
    }

    let entity = entity_type
        .unwrap_or_else(|| "Entity".to_string())
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| quote!(Entity));

    let id = id_type
        .unwrap_or_else(|| "u64".to_string())
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| quote!(u64));

    (entity, id)
}

/// Extract @Query annotation
fn extract_query_attr(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("Query") {
            // Parse #[Query("SELECT ...")]
            if let Ok(list) = attr.meta.require_list() {
                let sql = list.tokens.to_string();
                let sql = sql.trim_matches('"').trim_matches('\'');
                return Some(sql.to_string());
            }
        }
    }
    None
}

/// Convert `?` placeholders to `$N` positional markers.
fn convert_placeholders(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let mut n = 0;
    for ch in sql.chars() {
        if ch == '?' {
            n += 1;
            result.push_str(&format!("${n}"));
        } else {
            result.push(ch);
        }
    }
    result
}

/// Extract typed argument identifiers from a trait method signature (skips &self / self).
fn extract_args(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Vec<proc_macro2::TokenStream> {
    inputs.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(pt) = arg
            && let syn::Pat::Ident(pi) = &*pt.pat {
                let ident = &pi.ident;
                return Some(quote!(#ident));
            }
        None
    }).collect()
}

/// Generate a custom @Query method implementation.
fn generate_query_method(
    method: &syn::TraitItemFn,
    sql: &str,
    entity_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let method_sig = &method.sig;
    let return_type_str = quote!(#method_sig.output).to_string();
    let args = extract_args(&method.sig.inputs);
    let converted_sql = convert_placeholders(sql);

    let returns_option = return_type_str.contains("Option");
    let _returns_vec = return_type_str.contains("Vec");

    let params_build = quote! {
        let params: Vec<nexus_data_rdbc::QueryParam> = vec![
            #(nexus_data_rdbc::QueryParam::from(#args)),*
        ];
    };

    let implementation = if sql.trim().to_uppercase().starts_with("SELECT") {
        if returns_option {
            quote! {
                let sql: &str = #converted_sql;
                #params_build
                let row = self.client.fetch_one_params(sql, &params).await
                    .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                match row {
                    Some(r) => {
                        let entity: #entity_type = r.deserialize()
                            .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                        Ok(Some(entity))
                    }
                    None => Ok(None),
                }
            }
        } else {
            quote! {
                let sql: &str = #converted_sql;
                #params_build
                let rows = self.client.fetch_all_params(sql, &params).await
                    .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                let mut results = Vec::with_capacity(rows.len());
                for row in &rows {
                    results.push(row.deserialize()
                        .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?);
                }
                Ok(results)
            }
        }
    } else {
        quote! {
            let sql: &str = #converted_sql;
            #params_build
            let _affected = self.client.execute_params(sql, &params).await
                .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
            Ok(_affected as usize)
        }
    };

    quote! {
        #method_sig {
            #implementation
        }
    }
}

/// Generate a derived query method implementation from method name conventions.
fn generate_derived_query_method(
    method: &syn::TraitItemFn,
    table_name: &str,
    entity_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let method_sig = &method.sig;
    let method_name_str = method.sig.ident.to_string();
    let return_type_str = quote!(#method_sig.output).to_string();

    let (operation, conditions, order_by, limit) = parse_method_name(&method_name_str);
    let args = extract_args(&method.sig.inputs);

    // Build WHERE clause with $N placeholders
    let mut param_idx = 0usize;
    let where_parts: Vec<String> = conditions.iter().map(|c| {
        let field = to_snake_case(c.field());
        match c {
            QueryCondition::Eq(_) => { param_idx += 1; format!("{} = ${}", field, param_idx) }
            QueryCondition::Like(_) => { param_idx += 1; format!("{} LIKE ${}", field, param_idx) }
            QueryCondition::NotLike(_) => { param_idx += 1; format!("{} NOT LIKE ${}", field, param_idx) }
            QueryCondition::GreaterThan(_) => { param_idx += 1; format!("{} > ${}", field, param_idx) }
            QueryCondition::GreaterThanEqual(_) => { param_idx += 1; format!("{} >= ${}", field, param_idx) }
            QueryCondition::LessThan(_) => { param_idx += 1; format!("{} < ${}", field, param_idx) }
            QueryCondition::LessThanEqual(_) => { param_idx += 1; format!("{} <= ${}", field, param_idx) }
            QueryCondition::In(_) => { param_idx += 1; format!("{} IN (${})", field, param_idx) }
            QueryCondition::Between(_) => { param_idx += 2; format!("{} BETWEEN ${} AND ${}", field, param_idx - 1, param_idx) }
            QueryCondition::IsNull(_) => format!("{} IS NULL", field),
            QueryCondition::IsNotNull(_) => format!("{} IS NOT NULL", field),
            QueryCondition::Not(_) => { param_idx += 1; format!("{} != ${}", field, param_idx) }
        }
    }).collect();

    let where_clause = if where_parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_parts.join(" AND "))
    };

    let order_clause = if order_by.is_empty() {
        String::new()
    } else {
        format!(" ORDER BY {}", order_by.join(", "))
    };

    let limit_clause = limit.map(|n| format!(" LIMIT {}", n)).unwrap_or_default();

    let sql = match operation.as_str() {
        "find" | "find_all" => format!("SELECT * FROM {} {}{}{}", table_name, where_clause, order_clause, limit_clause),
        "count" => format!("SELECT COUNT(*) AS cnt FROM {} {}", table_name, where_clause),
        "exists" => format!("SELECT EXISTS(SELECT 1 FROM {} {}) AS ex", table_name, where_clause),
        "delete" => format!("DELETE FROM {} {}", table_name, where_clause),
        _ => format!("SELECT * FROM {} {}{}{}", table_name, where_clause, order_clause, limit_clause),
    };

    let returns_option = return_type_str.contains("Option");
    let _returns_vec = return_type_str.contains("Vec");
    let returns_bool = return_type_str.contains("bool");
    let _returns_usize = return_type_str.contains("usize") || return_type_str.contains("i64");

    let params_build = quote! {
        let params: Vec<nexus_data_rdbc::QueryParam> = vec![
            #(nexus_data_rdbc::QueryParam::from(#args)),*
        ];
    };

    let implementation = if operation == "delete" {
        if returns_bool {
            quote! {
                let sql: &str = #sql;
                #params_build
                let n = self.client.execute_params(sql, &params).await
                    .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                Ok(n > 0)
            }
        } else {
            quote! {
                let sql: &str = #sql;
                #params_build
                let n = self.client.execute_params(sql, &params).await
                    .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                Ok(n as usize)
            }
        }
    } else if operation == "count" {
        quote! {
            let sql: &str = #sql;
            #params_build
            let rows = self.client.fetch_all_params(sql, &params).await
                .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
            let cnt = rows.first()
                .and_then(|r| r.get_as::<i64>("cnt").ok())
                .unwrap_or(0);
            Ok(cnt as usize)
        }
    } else if operation == "exists" {
        quote! {
            let sql: &str = #sql;
            #params_build
            let rows = self.client.fetch_all_params(sql, &params).await
                .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
            let ex = rows.first()
                .and_then(|r| r.get_as::<bool>("ex").ok())
                .unwrap_or(false);
            Ok(ex)
        }
    } else if returns_option {
        quote! {
            let sql: &str = #sql;
            #params_build
            let row = self.client.fetch_one_params(sql, &params).await
                .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
            match row {
                Some(r) => {
                    let entity: #entity_type = r.deserialize()
                        .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
                    Ok(Some(entity))
                }
                None => Ok(None),
            }
        }
    } else {
        quote! {
            let sql: &str = #sql;
            #params_build
            let rows = self.client.fetch_all_params(sql, &params).await
                .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?;
            let mut results = Vec::with_capacity(rows.len());
            for row in &rows {
                results.push(row.deserialize()
                    .map_err(|e| nexus_data_commons::Error::other(e.to_string()))?);
            }
            Ok(results)
        }
    };

    quote! {
        #method_sig {
            #implementation
        }
    }
}

/// A single condition parsed from a method name.
/// 从方法名解析的单个条件。
#[derive(Debug, Clone)]
enum QueryCondition {
    /// field = ? (default equality)
    Eq(String),
    /// field LIKE ?
    Like(String),
    /// field NOT LIKE ?
    NotLike(String),
    /// field > ?
    GreaterThan(String),
    /// field >= ?
    GreaterThanEqual(String),
    /// field < ?
    LessThan(String),
    /// field <= ?
    LessThanEqual(String),
    /// field IN (?)
    In(String),
    /// field BETWEEN ? AND ?
    Between(String),
    /// field IS NULL
    IsNull(String),
    /// field IS NOT NULL
    IsNotNull(String),
    /// field != ?
    Not(String),
}

impl QueryCondition {
    fn field(&self) -> &str {
        match self {
            Self::Eq(f) | Self::Like(f) | Self::NotLike(f)
            | Self::GreaterThan(f) | Self::GreaterThanEqual(f)
            | Self::LessThan(f) | Self::LessThanEqual(f)
            | Self::In(f) | Self::Between(f)
            | Self::IsNull(f) | Self::IsNotNull(f)
            | Self::Not(f) => f,
        }
    }
}

/// Parse method name to extract operation, conditions, and query structure.
/// 解析方法名以提取操作、条件和查询结构。
///
/// Supported patterns / 支持的模式:
/// - `find_by_username` → WHERE username = $1
/// - `find_by_username_and_email` → WHERE username = $1 AND email = $2
/// - `find_by_username_or_email` → WHERE username = $1 OR email = $2
/// - `find_by_age_greater_than` → WHERE age > $1
/// - `find_by_name_like` → WHERE name LIKE $1
/// - `find_by_status_in` → WHERE status IN ($1)
/// - `find_by_age_between` → WHERE age BETWEEN $1 AND $2
/// - `find_by_deleted_is_null` → WHERE deleted IS NULL
/// - `count_by_active` → SELECT COUNT(*) ... WHERE active = $1
/// - `find_all_by_status_order_by_name` → ... ORDER BY name
/// - `find_by_active_limit_10` → ... LIMIT 10
fn parse_method_name(method_name: &str) -> (String, Vec<QueryCondition>, Vec<String>, Option<usize>) {
    let method_name_lower = method_name.to_lowercase();

    let (operation, rest) = if method_name_lower.starts_with("find_all_by_") {
        ("find_all", &method_name[12..])
    } else if method_name_lower.starts_with("find_by_") {
        ("find", &method_name[8..])
    } else if method_name_lower.starts_with("count_by_") {
        ("count", &method_name[9..])
    } else if method_name_lower.starts_with("exists_by_") {
        ("exists", &method_name[10..])
    } else if method_name_lower.starts_with("delete_by_") {
        ("delete", &method_name[10..])
    } else {
        return ("find".to_string(), Vec::new(), Vec::new(), None);
    };

    let rest_lower = rest.to_lowercase();

    // Extract LIMIT if present
    let limit = if let Some(idx) = rest_lower.find("_limit_") {
        let after = &rest[idx + 7..];
        after.parse::<usize>().ok()
    } else {
        None
    };

    // Strip _limit_N suffix and _order_by_... suffix for condition parsing
    let conditions_part = rest_lower
        .split("_order_by_").next().unwrap_or(&rest_lower)
        .split("_limit_").next().unwrap_or(&rest_lower);

    // Split by _and_ and _or_ to get condition segments
    let conditions = parse_conditions(conditions_part);

    // Extract ORDER BY fields
    let order_by = if let Some(idx) = rest_lower.find("_order_by_") {
        let order_part = rest_lower[idx + 10..]
            .split("_limit_").next().unwrap_or("");
        order_part.split('_').filter(|s| !s.is_empty()).map(String::from).collect()
    } else {
        Vec::new()
    };

    (operation.to_string(), conditions, order_by, limit)
}

/// Parse condition segments from a method name fragment.
fn parse_conditions(part: &str) -> Vec<QueryCondition> {
    let mut conditions = Vec::new();
    let segments = split_conditions(part);

    for seg in &segments {
        let seg_lower = seg.to_lowercase();
        let seg_str = seg_lower.as_str();

        if seg_str.ends_with("_greater_than_equal") {
            let field = &seg_str[..seg_str.len() - 20];
            if !field.is_empty() { conditions.push(QueryCondition::GreaterThanEqual(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_less_than_equal") {
            if !field.is_empty() { conditions.push(QueryCondition::LessThanEqual(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_greater_than") {
            if !field.is_empty() { conditions.push(QueryCondition::GreaterThan(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_less_than") {
            if !field.is_empty() { conditions.push(QueryCondition::LessThan(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_not_like") {
            if !field.is_empty() { conditions.push(QueryCondition::NotLike(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_like") {
            if !field.is_empty() { conditions.push(QueryCondition::Like(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_is_not_null") {
            if !field.is_empty() { conditions.push(QueryCondition::IsNotNull(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_is_null") {
            if !field.is_empty() { conditions.push(QueryCondition::IsNull(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_in") {
            if !field.is_empty() { conditions.push(QueryCondition::In(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_between") {
            if !field.is_empty() { conditions.push(QueryCondition::Between(field.to_string())); }
        } else if let Some(field) = seg_str.strip_suffix("_not") {
            if !field.is_empty() { conditions.push(QueryCondition::Not(field.to_string())); }
        } else if !seg_str.is_empty() {
            conditions.push(QueryCondition::Eq(seg_str.to_string()));
        }
    }

    conditions
}

/// Split a method name fragment by _and_ and _or_ separators.
fn split_conditions(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if i + 5 <= len && &s[i..i+5] == "_and_" {
            if !current.is_empty() { result.push(std::mem::take(&mut current)); }
            i += 5;
        } else if i + 4 <= len && &s[i..i+4] == "_or_" {
            if !current.is_empty() { result.push(std::mem::take(&mut current)); }
            i += 4;
        } else {
            current.push(bytes[i] as char);
            i += 1;
        }
    }
    if !current.is_empty() { result.push(current); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
        assert_eq!(to_snake_case("user123"), "user123");
    }

    #[test]
    fn test_parse_method_name() {
        let (op, conds, order, limit) = parse_method_name("find_by_username");
        assert_eq!(op, "find");
        assert_eq!(conds.len(), 1);
        assert!(matches!(&conds[0], QueryCondition::Eq(f) if f == "username"));
        assert!(order.is_empty());
        assert!(limit.is_none());

        let (op, conds, _, _) = parse_method_name("find_by_username_and_email");
        assert_eq!(op, "find");
        assert_eq!(conds.len(), 2);

        let (op, conds, _, _) = parse_method_name("count_by_active");
        assert_eq!(op, "count");
        assert_eq!(conds.len(), 1);

        let (op, conds, _, _) = parse_method_name("delete_by_user_id");
        assert_eq!(op, "delete");
        assert_eq!(conds.len(), 1);

        let (op, conds, _, _) = parse_method_name("find_all_by_status");
        assert_eq!(op, "find_all");
        assert_eq!(conds.len(), 1);
    }

    #[test]
    fn test_parse_method_name_operators() {
        let (op, conds, _, _) = parse_method_name("find_by_age_greater_than");
        assert_eq!(op, "find");
        assert!(matches!(&conds[0], QueryCondition::GreaterThan(f) if f == "age"));

        let (op, conds, _, _) = parse_method_name("find_by_name_like");
        assert!(matches!(&conds[0], QueryCondition::Like(f) if f == "name"));

        let (op, conds, _, _) = parse_method_name("find_by_deleted_is_null");
        assert!(matches!(&conds[0], QueryCondition::IsNull(f) if f == "deleted"));

        let (_, conds, order, limit) = parse_method_name("find_by_active_order_by_name_limit_10");
        assert!(matches!(&conds[0], QueryCondition::Eq(f) if f == "active"));
        assert_eq!(order, vec!["name"]);
        assert_eq!(limit, Some(10));
    }
}
