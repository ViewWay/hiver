//! Getter derive macro implementation
//! Getter 派生宏实现

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields};

/// Implement #[Getter] derive macro
/// 实现 #[Getter] 派生宏
pub fn impl_getter(input: DeriveInput) -> TokenStream {
    let struct_name = &input.ident;

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
                "#[Getter] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into();
        },
    };

    // Generate getter methods for each field
    // 为每个字段生成 getter 方法
    let getters = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref()?;
        let field_type = &field.ty;

        // Check if field should be skipped (has #[get(skip)])
        // 检查是否应跳过字段（有 #[get(skip)] 属性）
        let should_skip = field.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .iter()
                .any(|seg| seg.ident == "get" || seg.ident == "skip")
        });

        if should_skip {
            return None;
        }

        Some(quote! {
            #[inline]
            pub fn #field_name(&self) -> #field_type
            where
                #field_type: ::std::clone::Clone,
            {
                self.#field_name.clone()
            }
        })
    });

    let expanded = quote! {
        impl #struct_name {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}
