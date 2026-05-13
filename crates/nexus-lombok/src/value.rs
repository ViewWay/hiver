//! Value derive macro implementation
//! Value 派生宏实现

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{DeriveInput, Data, DataStruct, Fields};

/// Implement #[Value] derive macro
/// 实现 #[Value] 派生宏
///
/// Similar to @Data but for immutable value objects.
/// Similar to @Data but creates immutable value objects.
///
/// Generates:
/// 生成：
/// - Constructor with all fields / 包含所有字段的构造函数
/// - Getter methods / Getter 方法
/// - with_xxx methods (for creating modified copies) / with_xxx 方法（用于创建修改副本）
pub fn impl_value(input: DeriveInput) -> TokenStream {
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
                "#[Value] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into()
        }
    };

    // Get field names and types
    // 获取字段名和类型
    let field_names: Vec<&Ident> = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate with_ method identifiers using format_ident
    // 使用 format_ident 生成 with_ 方法标识符
    let with_method_names: Vec<Ident> = field_names
        .iter()
        .map(|name| format_ident!("with_{}", name))
        .collect();

    // Generate constructor
    // 生成构造函数
    let constructor = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #[inline]
            #[doc = "Creates a new value instance.\n"]
            #[doc = "创建新的值实例。"]
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }
    };

    // Generate getters
    // 生成 getters
    let getters: Vec<TokenStream> = field_names.iter().zip(field_types.iter()).map(|(name, ty)| {
        quote! {
            #[inline]
            pub fn #name(&self) -> #ty
            where
                #ty: ::std::clone::Clone,
            {
                self.#name.clone()
            }
        }
    }).collect();

    // Generate with_methods
    // 生成 with_methods
    let with_methods: Vec<TokenStream> = field_names.iter().zip(field_types.iter()).zip(with_method_names.iter()).map(|((name, ty), with_name)| {
        quote! {
            #[inline]
            #[must_use]
            #[doc = concat!("Creates a modified copy with `", stringify!(#name), "` changed.")]
            pub fn #with_name(&self, #name: #ty) -> Self
            where
                #ty: Clone,
            {
                let mut result = self.clone();
                result.#name = #name;
                result
            }
        }
    }).collect();

    // Combine all expansions
    // 合并所有展开
    quote! {
        #constructor

        impl #impl_generics #struct_name #ty_generics #where_clause
        where
            #struct_name: Clone,
        {
            #(#getters)*

            #(#with_methods)*
        }
    }
}
