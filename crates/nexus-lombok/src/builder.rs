//! Builder derive macro implementation
//! Builder 派生宏实现

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, DataStruct, Fields};

/// Implement #[Builder] derive macro
/// 实现 #[Builder] 派生宏
///
/// Generates a builder pattern for the struct.
/// 为结构体生成 builder 模式。
pub fn impl_builder(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;
    let builder_name = format!("{}Builder", struct_name);
    let builder_ident = syn::Ident::new(&builder_name, struct_name.span());

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
                "#[Builder] can only be used on structs with named fields",
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

    // Generate Builder struct
    // 生成 Builder 结构体
    let builder_struct = quote! {
        #[derive(Default, Debug)]
        #[doc = concat!("Builder for `", stringify!(#struct_name), "`")]
        pub struct #builder_ident #impl_generics #ty_generics #where_clause {
            #(
                #field_names: Option<#field_types>,
            )*
        }
    };

    // Generate builder methods on original struct
    // 在原结构体上生成 builder 方法
    let builder_method = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #[inline]
            #[doc = "Creates a new builder for this struct.\n"]
            #[doc = "为此结构体创建新的 builder。"]
            pub fn builder() -> #builder_ident #ty_generics {
                #builder_ident::default()
            }
        }
    };

    // Generate setter methods on Builder
    // 生成 Builder 上的 setter 方法
    let builder_setters = field_names.iter().zip(field_types.iter()).map(|(name, ty)| {
        quote! {
            #[inline]
            pub fn #name(mut self, #name: #ty) -> Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    // Generate build method
    // 生成 build 方法
    let build_method = quote! {
        impl #impl_generics #builder_ident #ty_generics #where_clause {
            #[inline]
            #[doc = "Builds the struct.\n"]
            #[doc = "构建结构体。"]
            pub fn build(self) -> Result<#struct_name #ty_generics, String> {
                Ok(#struct_name {
                    #(
                        #field_names: self.#field_names
                            .ok_or_else(|| concat!(stringify!(#field_names), " is required").to_string())?,
                    )*
                })
            }
        }
    };

    // Combine all expansions
    // 合并所有展开
    let expanded = quote! {
        #builder_struct

        #builder_method

        impl #impl_generics #builder_ident #ty_generics #where_clause {
            #(#builder_setters)*
        }

        #build_method
    };

    TokenStream::from(expanded)
}
