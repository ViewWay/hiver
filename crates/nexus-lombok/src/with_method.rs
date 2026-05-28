//! With derive macro implementation
//! With 派生宏实现

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Data, DataStruct, Fields};

/// Implement #[With] derive macro
/// 实现 #[With] 派生宏
///
/// Generates with_xxx methods for creating modified copies of struct instances.
/// 生成用于创建结构体实例修改副本的 with_xxx 方法。
///
/// Note: The struct must implement Clone for this to work.
/// 注意：结构体必须实现 Clone 才能使用此宏。
pub fn impl_with(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract fields from struct
    // 从结构体中提取字段
    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "#[With] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    };

    // Collect fields that should not be skipped
    // 收集不应跳过的字段
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut with_method_names = Vec::new();

    for field in fields {
        let field_name = match field.ident.as_ref() {
            Some(name) => name,
            None => continue,
        };
        let field_type = &field.ty;

        // Check if field should be skipped
        // 检查是否应跳过字段
        let should_skip = field.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .iter()
                .any(|seg| seg.ident == "with" || seg.ident == "skip")
        });

        if should_skip {
            continue;
        }

        // Generate with_ method name using format_ident
        // 使用 format_ident 生成 with_ 方法名
        let with_name = format_ident!("with_{}", field_name);

        field_names.push(field_name);
        field_types.push(field_type);
        with_method_names.push(with_name);
    }

    // Generate with_ methods
    // 生成 with_ 方法
    let with_methods: Vec<TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .zip(with_method_names.iter())
        .map(|((name, ty), with_name)| {
            quote! {
                #[inline]
                #[must_use]
                pub fn #with_name(&self, #name: #ty) -> Self
                where
                    Self: Clone,
                {
                    let mut clone = self.clone();
                    clone.#name = #name;
                    clone
                }
            }
        })
        .collect();

    // Add Clone bound requirement
    // 添加 Clone bound 要求
    let where_clause = if where_clause.is_some() {
        quote! { #where_clause }
    } else {
        quote! { where #struct_name #ty_generics: Clone }
    };

    let expanded: TokenStream = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#with_methods)*
        }
    };

    TokenStream::from(expanded)
}
