//! Data derive macro implementation
//! Data 派生宏实现

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields};

/// Implement #[Data] derive macro
/// 实现 #[Data] 派生宏
///
/// Combines functionality from:
/// 结合以下功能：
/// - AllArgsConstructor (constructor with all fields)
/// - Getter (getter methods)
/// - Setter (setter methods)
/// - With (with_xxx methods for chaining)
pub fn impl_data(input: DeriveInput) -> TokenStream {
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
                "#[Data] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into();
        },
    };

    // Get field names and types
    // 获取字段名和类型
    let field_names: Vec<&Ident> = fields.iter().filter_map(|f| f.ident.as_ref()).collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    // Generate set_ method names using format_ident
    // 使用 format_ident 生成 set_ 方法名
    let setter_method_names: Vec<Ident> = field_names
        .iter()
        .map(|name| format_ident!("set_{}", name))
        .collect();

    // Generate with_ method names using format_ident
    // 使用 format_ident 生成 with_ 方法名
    let with_method_names: Vec<Ident> = field_names
        .iter()
        .map(|name| format_ident!("with_{}", name))
        .collect();

    // Generate constructor: new()
    // 生成构造函数: new()
    let constructor = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #[inline]
            #[doc = "Creates a new instance of the struct with all fields.\n"]
            #[doc = "创建包含所有字段的结构体新实例。"]
            pub fn new(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }
        }
    };

    // Generate getters
    // 生成 getters
    let getters: Vec<TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            quote! {
                #[inline]
                pub fn #name(&self) -> #ty
                where
                    #ty: ::std::clone::Clone,
                {
                    self.#name.clone()
                }
            }
        })
        .collect();

    // Generate setters
    // 生成 setters
    let setters: Vec<TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .zip(setter_method_names.iter())
        .map(|((name, ty), set_name)| {
            quote! {
                #[inline]
                pub fn #set_name(&mut self, #name: #ty) {
                    self.#name = #name;
                }
            }
        })
        .collect();

    // Generate with_ methods (requires Clone)
    // 生成 with_ 方法（需要 Clone）
    let with_methods: Vec<TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .zip(with_method_names.iter())
        .map(|((name, ty), with_name)| {
            quote! {
                #[inline]
                #[doc = concat!("Creates a modified copy with `", stringify!(#name), "` changed.")]
                pub fn #with_name(&self, #name: #ty) -> Self {
                    let mut clone = self.clone();
                    clone.#name = #name;
                    clone
                }
            }
        })
        .collect();

    // Generate Default implementation for Data
    // 为 Data 生成 Default 实现
    let default_impl = quote! {
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

    // Combine all expansions
    // 合并所有展开
    let expanded: TokenStream = quote! {
        #constructor

        #default_impl

        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#getters)*
            #(#setters)*
        }

        impl #impl_generics #struct_name #ty_generics #where_clause
        where
            #struct_name #ty_generics: Clone,
        {
            #(#with_methods)*
        }
    };

    TokenStream::from(expanded)
}
