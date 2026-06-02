//! Setter derive macro implementation
//! Setter 派生宏实现

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields};

/// Implement #[Setter] derive macro
/// 实现 #[Setter] 派生宏
pub fn impl_setter(input: DeriveInput) -> TokenStream {
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
                "#[Setter] can only be used on structs with named fields",
            )
            .to_compile_error()
            .into();
        },
    };

    // Check for chain attribute
    // 检查 chain 属性
    let has_chain_attr = input.attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "setter")
    });

    // Collect fields that should not be skipped
    // 收集不应跳过的字段
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut setter_method_names = Vec::new();

    for field in fields {
        let Some(field_name) = field.ident.as_ref() else {
            continue;
        };
        let field_type = &field.ty;

        // Check if field should be skipped
        // 检查是否应跳过字段
        let should_skip = field.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .iter()
                .any(|seg| seg.ident == "set" || seg.ident == "skip")
        });

        if should_skip {
            continue;
        }

        field_names.push(field_name);
        field_types.push(field_type);

        // If not using chain mode, generate set_ method name
        // 如果不使用 chain 模式，生成 set_ 方法名
        if !has_chain_attr {
            let set_name = format_ident!("set_{}", field_name);
            setter_method_names.push(set_name);
        }
    }

    // Generate setter methods for each field
    // 为每个字段生成 setter 方法
    let setters: Vec<TokenStream> = if has_chain_attr {
        // Return &mut Self for chaining
        // 返回 &mut Self 以支持链式调用
        field_names
            .iter()
            .zip(field_types.iter())
            .map(|(name, ty)| {
                quote! {
                    #[inline]
                    pub fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = #name;
                        self
                    }
                }
            })
            .collect()
    } else {
        // Return ()
        // 返回 ()
        field_names
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
            .collect()
    };

    let expanded: TokenStream = quote! {
        impl #struct_name {
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}
