//! @Entity and @Table attribute macros
//! @Entity 和 @Table 属性宏

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field, LitStr};

/// Field-level helper attributes recognized and consumed by #[Entity].
/// #[Entity] 识别并消费的字段级辅助属性。
const HELPER_ATTRS: &[&str] = &[
    "Id",
    "GeneratedValue",
    "Column",
    "OneToMany",
    "OneToOne",
    "ManyToOne",
    "ManyToMany",
    "JoinColumn",
    "JoinTable",
    "Transient",
    "PrePersist",
    "PostPersist",
    "PreUpdate",
    "PostUpdate",
    "PreRemove",
    "PostLoad",
    "CreatedDate",
    "LastModifiedDate",
    "CreatedBy",
    "LastModifiedBy",
];

// ============================================================
// Attribute argument parsing utilities
// 属性参数解析工具
// ============================================================

/// Parse key=value pairs from an attribute like #[Attr(key = "val", flag = true)]
/// 从属性中解析 key=value 对，如 #[Attr(key = "val", flag = true)]
fn parse_attr_args(attr: &syn::Attribute) -> Vec<(String, syn::Expr)> {
    let mut args = Vec::new();
    let _ = attr.parse_nested_meta(|meta| {
        if let Some(id) = meta.path.get_ident() {
            let value: syn::Expr = meta.value()?.parse()?;
            args.push((id.to_string(), value));
        }
        Ok(())
    });
    args
}

fn extract_string(args: &[(String, syn::Expr)], key: &str) -> Option<String> {
    args.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = v
        {
            Some(s.value())
        } else {
            None
        }
    })
}

fn extract_bool(args: &[(String, syn::Expr)], key: &str) -> Option<bool> {
    args.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(b),
            ..
        }) = v
        {
            Some(b.value)
        } else {
            None
        }
    })
}

fn extract_usize(args: &[(String, syn::Expr)], key: &str) -> Option<usize> {
    args.iter().find(|(k, _)| k == key).and_then(|(_, v)| {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(i),
            ..
        }) = v
        {
            i.base10_parse::<usize>().ok()
        } else {
            None
        }
    })
}

// ============================================================
// Field metadata extraction
// 字段元数据提取
// ============================================================

struct FieldMeta {
    name: String,
    ty: String,
    is_id: bool,
    id_strategy: Option<String>,
    column_name: Option<String>,
    nullable: bool,
    unique: bool,
    length: Option<usize>,
    is_transient: bool,
    relation_kind: Option<String>,
    relation_target: Option<String>,
    relation_mapped_by: Option<String>,
    join_column: Option<String>,
    join_table: Option<String>,
}

impl FieldMeta {
    fn resolved_column(&self) -> String {
        self.column_name
            .clone()
            .unwrap_or_else(|| self.name.clone())
    }
}

/// Extract metadata from a struct field by reading helper attributes.
/// 通过读取辅助属性从结构体字段提取元数据。
fn extract_field_meta(field: &Field) -> Option<FieldMeta> {
    let name = field.ident.as_ref()?.to_string();
    let ty = quote!(#field.ty).to_string().replace(' ', "");
    let mut meta = FieldMeta {
        name,
        ty,
        is_id: false,
        id_strategy: None,
        column_name: None,
        nullable: true,
        unique: false,
        length: None,
        is_transient: false,
        relation_kind: None,
        relation_target: None,
        relation_mapped_by: None,
        join_column: None,
        join_table: None,
    };

    for attr in &field.attrs {
        let id = match attr.path().get_ident() {
            Some(id) => id.to_string(),
            None => continue,
        };

        match id.as_str() {
            "Id" => {
                meta.is_id = true;
                meta.nullable = false;
            }
            "GeneratedValue" => {
                let args = parse_attr_args(attr);
                meta.id_strategy =
                    Some(extract_string(&args, "strategy").unwrap_or_else(|| "AUTO".into()));
            }
            "Column" => {
                let args = parse_attr_args(attr);
                meta.column_name = extract_string(&args, "name");
                if let Some(n) = extract_bool(&args, "nullable") {
                    meta.nullable = n;
                }
                if let Some(u) = extract_bool(&args, "unique") {
                    meta.unique = u;
                }
                meta.length = extract_usize(&args, "length");
            }
            "Transient" => {
                meta.is_transient = true;
            }
            "OneToMany" => {
                let args = parse_attr_args(attr);
                meta.relation_kind = Some("one_to_many".to_string());
                meta.relation_target = extract_string(&args, "target_entity");
                meta.relation_mapped_by = extract_string(&args, "mapped_by");
            }
            "OneToOne" => {
                let args = parse_attr_args(attr);
                meta.relation_kind = Some("one_to_one".to_string());
                meta.relation_target = extract_string(&args, "target_entity");
                meta.relation_mapped_by = extract_string(&args, "mapped_by");
            }
            "ManyToOne" => {
                let args = parse_attr_args(attr);
                meta.relation_kind = Some("many_to_one".to_string());
                meta.relation_target = extract_string(&args, "target_entity");
                meta.relation_mapped_by = extract_string(&args, "mapped_by");
            }
            "ManyToMany" => {
                let args = parse_attr_args(attr);
                meta.relation_kind = Some("many_to_many".to_string());
                meta.relation_target = extract_string(&args, "target_entity");
                meta.relation_mapped_by = extract_string(&args, "mapped_by");
            }
            "JoinColumn" => {
                let args = parse_attr_args(attr);
                meta.join_column = extract_string(&args, "name");
            }
            "JoinTable" => {
                let args = parse_attr_args(attr);
                meta.join_table = extract_string(&args, "name");
            }
            _ => {}
        }
    }

    Some(meta)
}

/// Convert CamelCase to snake_case for default table names.
/// 将 CamelCase 转换为 snake_case 用于默认表名。
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

// ============================================================
// #[Entity] attribute macro
// ============================================================

/// Implements #[Entity] attribute macro.
/// 实现 #[Entity] 属性宏。
///
/// Parses struct fields, recognizes helper attributes (`#[Id]`, `#[Column]`,
/// `#[GeneratedValue]`, `#[OneToMany]`, etc.), strips them from the output,
/// and generates entity metadata methods.
///
/// # Generated methods / 生成的方法
///
/// - `table_name() -> &'static str`
/// - `field_names() -> &'static [&'static str]`
/// - `field_count() -> usize`
/// - `column_name_for(field: &str) -> &'static str`
/// - `is_nullable(field: &str) -> bool`
/// - `is_unique(field: &str) -> bool`
/// - `id_field_name() -> Option<&'static str>`
/// - `id_generation_strategy() -> Option<&'static str>`
/// - `relations() -> &'static [(&'static str, &'static str, Option<&'static str>)]`
pub(crate) fn impl_entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // Extract table name from #[Table(name = "xxx")] if present
    let table_name_str = input
        .attrs
        .iter()
        .find_map(|attr| {
            let id = attr.path().get_ident()?;
            if id != "Table" {
                return None;
            }
            let args = parse_attr_args(attr);
            extract_string(&args, "name")
        })
        .unwrap_or_else(|| to_snake_case(&name.to_string()));

    let table_name = LitStr::new(&table_name_str, Span::call_site());

    // Extract field metadata
    let fields: Vec<FieldMeta> = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(f) => f
                .named
                .iter()
                .filter_map(extract_field_meta)
                .filter(|m| !m.is_transient)
                .collect(),
            _ => {
                return syn::Error::new_spanned(
                    name,
                    "Entity only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "Entity can only be applied to structs")
                .to_compile_error()
                .into();
        }
    };

    if fields.is_empty() {
        return syn::Error::new_spanned(
            name,
            "Entity struct must have at least one non-transient field",
        )
        .to_compile_error()
        .into();
    }

    // Strip helper attrs from struct fields
    if let syn::Data::Struct(data) = &mut input.data
        && let syn::Fields::Named(named) = &mut data.fields {
            for field in &mut named.named {
                field.attrs.retain(|attr| {
                    attr.path()
                        .get_ident()
                        .is_none_or(|id| !HELPER_ATTRS.contains(&id.to_string().as_str()))
                });
            }
        }

    // Strip #[Table] from struct attrs (Entity handles it)
    input.attrs.retain(|attr| {
        attr.path()
            .get_ident()
            .is_none_or(|id| id != "Table")
    });

    // -- Build generated code --
    let field_name_lits: Vec<LitStr> = fields
        .iter()
        .map(|f| LitStr::new(&f.name, Span::call_site()))
        .collect();
    let field_count = fields.len();

    let column_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let fname = LitStr::new(&f.name, Span::call_site());
            let cname = LitStr::new(&f.resolved_column(), Span::call_site());
            quote! { #fname => #cname }
        })
        .collect();

    let nullable_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let fname = LitStr::new(&f.name, Span::call_site());
            let n = f.nullable;
            quote! { #fname => #n }
        })
        .collect();

    let unique_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let fname = LitStr::new(&f.name, Span::call_site());
            let u = f.unique;
            quote! { #fname => #u }
        })
        .collect();

    let id_field_code = fields
        .iter()
        .find(|f| f.is_id)
        .map(|f| {
            let lit = LitStr::new(&f.name, Span::call_site());
            quote! { Some(#lit) }
        })
        .unwrap_or(quote! { None });

    let id_strategy_code = fields
        .iter()
        .find(|f| f.is_id)
        .and_then(|f| f.id_strategy.clone())
        .map(|s| {
            let lit = LitStr::new(&s, Span::call_site());
            quote! { Some(#lit) }
        })
        .unwrap_or(quote! { None });

    let relation_entries: Vec<_> = fields
        .iter()
        .filter(|f| f.relation_kind.is_some())
        .map(|f| {
            let field = LitStr::new(&f.name, Span::call_site());
            let kind = LitStr::new(f.relation_kind.as_ref().unwrap(), Span::call_site());
            if let Some(t) = &f.relation_target {
                let target = LitStr::new(t, Span::call_site());
                quote! { (#field, #kind, Some(#target)) }
            } else { quote! { (#field, #kind, None) } }
        })
        .collect();

    let field_type_lits: Vec<LitStr> = fields
        .iter()
        .map(|f| LitStr::new(&f.ty, Span::call_site()))
        .collect();

    let join_info_entries: Vec<_> = fields
        .iter()
        .filter(|f| f.join_column.is_some() || f.join_table.is_some())
        .map(|f| {
            let field = LitStr::new(&f.name, Span::call_site());
            let jc = f.join_column.as_deref().map(|s| LitStr::new(s, Span::call_site()));
            let jt = f.join_table.as_deref().map(|s| LitStr::new(s, Span::call_site()));
            let jc_code = jc.map(|l| quote! { Some(#l) }).unwrap_or(quote! { None });
            let jt_code = jt.map(|l| quote! { Some(#l) }).unwrap_or(quote! { None });
            quote! { (#field, #jc_code, #jt_code) }
        })
        .collect();

    let length_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let fname = LitStr::new(&f.name, Span::call_site());
            if let Some(l) = f.length { quote! { #fname => Some(#l) } } else { quote! { #fname => None } }
        })
        .collect();

    let expanded = quote! {
        #input

        impl #name {
            /// Returns the database table name.
            /// 返回数据库表名。
            #[inline]
            pub fn table_name() -> &'static str {
                #table_name
            }

            /// Returns all non-transient field names.
            /// 返回所有非瞬态字段名。
            #[inline]
            pub fn field_names() -> &'static [&'static str] {
                &[#(#field_name_lits),*]
            }

            /// Returns all non-transient field type strings.
            /// 返回所有非瞬态字段的类型字符串。
            #[inline]
            pub fn field_types() -> &'static [&'static str] {
                &[#(#field_type_lits),*]
            }

            /// Returns the number of persistent fields.
            /// 返回持久化字段数量。
            #[inline]
            pub fn field_count() -> usize {
                #field_count
            }

            /// Returns the database column name for a given field name.
            /// Returns the field name itself if no explicit column mapping exists.
            /// 返回给定字段名对应的数据库列名。
            /// 如果没有显式列映射，则返回字段名本身。
            pub fn column_name_for(field: &str) -> &str {
                match field {
                    #(#column_arms,)*
                    _ => field,
                }
            }

            /// Returns whether a field is nullable.
            /// 返回字段是否可为 null。
            pub fn is_nullable(field: &str) -> bool {
                match field {
                    #(#nullable_arms,)*
                    _ => true,
                }
            }

            /// Returns whether a field has a unique constraint.
            /// 返回字段是否有唯一约束。
            pub fn is_unique(field: &str) -> bool {
                match field {
                    #(#unique_arms,)*
                    _ => false,
                }
            }

            /// Returns the column length for a field, if specified.
            /// 返回字段的列长度（如果指定）。
            pub fn length_for(field: &str) -> Option<usize> {
                match field {
                    #(#length_arms,)*
                    _ => None,
                }
            }

            /// Returns the ID field name.
            /// 返回 ID 字段名。
            #[inline]
            pub fn id_field_name() -> Option<&'static str> {
                #id_field_code
            }

            /// Returns the ID generation strategy.
            /// 返回 ID 生成策略。
            #[inline]
            pub fn id_generation_strategy() -> Option<&'static str> {
                #id_strategy_code
            }

            /// Returns relation descriptors: (field_name, relation_kind, target_entity).
            /// 返回关系描述符：(字段名, 关系类型, 目标实体)。
            #[inline]
            pub fn relations() -> &'static [(&'static str, &'static str, Option<&'static str>)] {
                &[#(#relation_entries),*]
            }

            /// Returns join info: (field_name, join_column, join_table).
            /// 返回连接信息：(字段名, 连接列名, 连接表名)。
            #[inline]
            pub fn join_info() -> &'static [(&'static str, Option<&'static str>, Option<&'static str>)] {
                &[#(#join_info_entries),*]
            }
        }
    };

    TokenStream::from(expanded)
}

// ============================================================
// #[Table] attribute macro (standalone)
// ============================================================

/// Implements #[Table] attribute macro for standalone use.
/// 实现 #[Table] 属性宏，用于独立使用。
///
/// When used alongside `#[Entity]`, the Entity macro consumes #[Table]
/// and generates `table_name()` automatically. This standalone version
/// is for cases where only #[Table] is used.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn impl_table(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let table_name = if attr.is_empty() {
        to_snake_case(&name.to_string())
    } else {
        let attr_str = attr.to_string();
        if let Some(eq_pos) = attr_str.find('=') {
            attr_str[eq_pos + 1..]
                .trim()
                .trim_matches('"')
                .to_string()
        } else {
            attr_str.trim_matches('"').to_string()
        }
    };

    let table_name_lit = LitStr::new(&table_name, Span::call_site());

    let expanded = quote! {
        #input

        impl #name {
            /// Returns the database table name.
            /// 返回数据库表名。
            pub fn table_name() -> &'static str {
                #table_name_lit
            }
        }
    };

    TokenStream::from(expanded)
}
