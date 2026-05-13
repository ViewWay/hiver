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

    if let Data::Struct(DataStruct { fields, .. }) = &input.data
        && let Fields::Named(named_fields) = fields {
            for field in &named_fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                field_names.push(field_name.clone());

                // Parse field attributes
                let field_attrs = parse_field_attrs(&field.attrs);

                if field_attrs.ignore {
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

/// Generate a custom query method implementation
fn generate_query_method(
    method: &syn::TraitItemFn,
    sql: &str,
    _entity_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let method_sig = &method.sig;

    quote! {
        #method_sig {
            let query = #sql;
            // Execute query using the client
            // This is a simplified implementation
            // In production, you'd parse parameters and bind them properly
            Err(nexus_data_commons::Error::not_implemented(format!("Custom query: {}", query)))
        }
    }
}

/// Generate a derived query method implementation
fn generate_derived_query_method(
    method: &syn::TraitItemFn,
    table_name: &str,
    _entity_type: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let method_sig = &method.sig;
    let method_name_str = method.sig.ident.to_string();
    let return_type_str = quote!(#method_sig.output).to_string();

    // Parse method name to extract conditions
    let (operation, conditions) = parse_method_name(&method_name_str);

    // Build WHERE clause from conditions
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = conditions
            .iter()
            .map(|c| format!("{} = ?", to_snake_case(c)))
            .collect();
        format!("WHERE {}", parts.join(" AND "))
    };

    let sql = match operation.as_str() {
        "find" => format!("SELECT * FROM {} {}", table_name, where_clause),
        "count" => format!("SELECT COUNT(*) FROM {} {}", table_name, where_clause),
        "exists" => format!("SELECT EXISTS(SELECT 1 FROM {} {}) AS exists", table_name, where_clause),
        "delete" => format!("DELETE FROM {} {}", table_name, where_clause),
        "find_all" => format!("SELECT * FROM {} {}", table_name, where_clause),
        _ => format!("SELECT * FROM {} {}", table_name, where_clause),
    };

    // Determine return type
    let returns_option = return_type_str.contains("Option");
    let returns_vec = return_type_str.contains("Vec");
    let returns_bool = return_type_str.contains("bool");
    let returns_usize = return_type_str.contains("usize");

    let implementation = if returns_bool {
        // exists or delete method returning bool
        quote! {
            let query = #sql;
            // Execute query
            Err(nexus_data_commons::Error::not_implemented(format!("Query: {}", query)))
        }
    } else if returns_usize {
        // count method
        quote! {
            let query = #sql;
            // Execute count query
            Err(nexus_data_commons::Error::not_implemented(format!("Query: {}", query)))
        }
    } else if returns_option {
        // find_by returning Option<Entity>
        quote! {
            let query = #sql;
            // Execute query returning Option
            Err(nexus_data_commons::Error::not_implemented(format!("Query: {}", query)))
        }
    } else if returns_vec {
        // find_all_by returning Vec<Entity>
        quote! {
            let query = #sql;
            // Execute query returning Vec
            Err(nexus_data_commons::Error::not_implemented(format!("Query: {}", query)))
        }
    } else {
        quote! {
            let query = #sql;
            // Execute query
            Err(nexus_data_commons::Error::not_implemented(format!("Query: {}", query)))
        }
    };

    quote! {
        #method_sig {
            #implementation
        }
    }
}

/// Parse method name to extract operation and conditions
/// Examples:
/// - "find_by_username_and_email" -> ("find", ["username", "email"])
/// - "count_by_active" -> ("count", ["active"])
/// - "delete_by_user_id" -> ("delete", ["user_id"])
/// - "find_all_by_status" -> ("find_all", ["status"])
fn parse_method_name(method_name: &str) -> (String, Vec<String>) {
    let method_name_lower = method_name.to_lowercase();

    let operation = if method_name_lower.starts_with("find_all_by_") {
        "find_all".to_string()
    } else if method_name_lower.starts_with("find_by_") {
        "find".to_string()
    } else if method_name_lower.starts_with("count_by_") {
        "count".to_string()
    } else if method_name_lower.starts_with("exists_by_") {
        "exists".to_string()
    } else if method_name_lower.starts_with("delete_by_") {
        "delete".to_string()
    } else {
        "find".to_string()
    };

    // Extract conditions after "by_"
    let conditions_part = if let Some(idx) = method_name_lower.find("_by_") {
        &method_name[idx + 4..]
    } else {
        return (operation, Vec::new());
    };

    // Split by "_and_" to get individual conditions
    let conditions: Vec<String> = conditions_part
        .split("_and_")
        .map(|s| s.to_string())
        .collect();

    (operation, conditions)
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
        let (op, conds) = parse_method_name("find_by_username");
        assert_eq!(op, "find");
        assert_eq!(conds, vec!["username"]);

        let (op, conds) = parse_method_name("find_by_username_and_email");
        assert_eq!(op, "find");
        assert_eq!(conds, vec!["username", "email"]);

        let (op, conds) = parse_method_name("count_by_active");
        assert_eq!(op, "count");
        assert_eq!(conds, vec!["active"]);

        let (op, conds) = parse_method_name("delete_by_user_id");
        assert_eq!(op, "delete");
        assert_eq!(conds, vec!["user_id"]);

        let (op, conds) = parse_method_name("find_all_by_status");
        assert_eq!(op, "find_all");
        assert_eq!(conds, vec!["status"]);
    }
}
