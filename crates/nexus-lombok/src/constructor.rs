//! Constructor derive macros implementation
//! 构造函数派生宏实现

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, DataStruct, Fields};

/// Implement #[AllArgsConstructor] derive macro
/// 实现 #[AllArgsConstructor] 派生宏
pub fn impl_all_args(input: DeriveInput) -> TokenStream {
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
                "#[AllArgsConstructor] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    };

    // Get field names and types
    // 获取字段名和类型
    let field_names: Vec<_> = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate constructor with default name "new"
    // 生成名为 "new" 的构造函数
    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #[inline]
            #[doc = "Creates a new instance with all fields.\n"]
            #[doc = "使用所有字段创建新实例。"]
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implement #[NoArgsConstructor] derive macro
/// 实现 #[NoArgsConstructor] 派生宏
pub fn impl_no_args(input: DeriveInput) -> TokenStream {
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
                "#[NoArgsConstructor] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    };

    // Get field names and types
    // 获取字段名和类型
    let field_names: Vec<_> = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate constructor with Default::default() for each field
    // 为每个字段生成使用 Default::default() 的构造函数
    let constructor_expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #[inline]
            #[doc = "Creates a new instance with default values.\n"]
            #[doc = "使用默认值创建新实例。"]
            pub fn new() -> Self
            where
                Self: Default,
            {
                Self::default()
            }
        }
    };

    // Generate Default implementation
    // 生成 Default 实现
    let default_expanded = quote! {
        impl #impl_generics Default for #struct_name #ty_generics #where_clause
        where
            #(#field_types: Default,)*
        {
            #[inline]
            fn default() -> Self {
                Self {
                    #(#field_names: Default::default()),*
                }
            }
        }
    };

    let expanded = quote! {
        #constructor_expanded
        #default_expanded
    };

    TokenStream::from(expanded)
}
