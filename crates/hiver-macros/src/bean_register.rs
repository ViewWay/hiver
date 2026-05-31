//! Shared helpers for generating `inventory::submit!` bean registration code.
//! 生成 `inventory::submit!` Bean 注册代码的共享辅助模块。
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Fields, ItemStruct, Type};

/// Extract inner type `T` from `Arc<T>` / `std::sync::Arc<T>`.
fn extract_arc_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let seg = type_path.path.segments.last()?;
    if seg.ident != "Arc" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

fn field_is_autowired(field: &Field) -> bool {
    field.attrs.iter().any(|a| {
        a.path().is_ident("autowired")
            || a.path()
                .get_ident()
                .is_some_and(|i| i == "Autowired")
    })
}

fn inject_type_for_field(field: &Field) -> Option<&Type> {
    if let Some(inner) = extract_arc_inner(&field.ty) {
        return Some(inner);
    }
    if field_is_autowired(field) {
        return Some(&field.ty);
    }
    None
}

/// Generate inventory bean registration for a struct stereotype (`#[service]`, etc.).
fn extract_condition_fn(
    input: &ItemStruct,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    use quote::format_ident;
    let cond_fn = format_ident!("__hiver_condition_{}", input.ident);

    for attr in &input.attrs {
        if attr.path().is_ident("conditional_on_property") {
            let mut key = String::new();
            let mut value: Option<String> = None;
            let _ = attr.parse_nested_meta(|meta| {
                if (meta.path.is_ident("name") || meta.path.is_ident("key"))
                    && let Ok(lit) = meta.value()
                        && let Ok(s) = lit.parse::<syn::LitStr>() {
                            key = s.value();
                        }
                if (meta.path.is_ident("having_value") || meta.path.is_ident("value"))
                    && let Ok(lit) = meta.value()
                        && let Ok(s) = lit.parse::<syn::LitStr>() {
                            value = Some(s.value());
                        }
                Ok(())
            });
            if !key.is_empty() {
                if let Some(ref v) = value {
                    let def = quote! {
                        fn #cond_fn(ctx: &::hiver_starter::core::ApplicationContext) -> bool {
                            ctx.get_property(#key).as_deref() == Some(#v)
                        }
                    };
                    return (def, quote! { #cond_fn });
                }
                let def = quote! {
                    fn #cond_fn(ctx: &::hiver_starter::core::ApplicationContext) -> bool {
                        ctx.get_property(#key).is_some()
                    }
                };
                return (def, quote! { #cond_fn });
            }
        }
    }

    // @ConditionalOnMissingBean — register only if no bean of the given type is in the container
    for attr in &input.attrs {
        if attr.path().is_ident("conditional_on_missing_bean") {
            let mut target_type: Option<Type> = None;
            let _ = attr.parse_nested_meta(|meta| {
                let path = meta.path.clone();
                target_type = Some(Type::Path(syn::TypePath {
                    qself: None,
                    path,
                }));
                Ok(())
            });
            if let Some(ty) = target_type {
                let def = quote! {
                    fn #cond_fn(ctx: &::hiver_starter::core::ApplicationContext) -> bool {
                        !ctx.has_bean::<#ty>()
                    }
                };
                return (def, quote! { #cond_fn });
            }
            // No type specified — condition on self
            let struct_name = &input.ident;
            let def = quote! {
                fn #cond_fn(ctx: &::hiver_starter::core::ApplicationContext) -> bool {
                    !ctx.has_bean::<#struct_name>()
                }
            };
            return (def, quote! { #cond_fn });
        }
    }

    (
        quote! {},
        quote! { ::hiver_starter::core::registry::always_true },
    )
}

pub fn generate_bean_registration(
    input: &ItemStruct,
    scope: TokenStream2,
) -> TokenStream2 {
    let struct_name = &input.ident;
    let factory_fn = format_ident!("__hiver_factory_{}", struct_name);
    let deps_static = format_ident!("__NEXUS_DEPS_{}", struct_name.to_string().to_uppercase());
    let bean_name_lit = struct_name.to_string();
    let bean_name_camel = to_camel_case(&bean_name_lit);
    let (condition_def, condition_fn) = extract_condition_fn(input);

    let Fields::Named(fields) = &input.fields else {
        return quote! {};
    };

    let mut dep_type_fns: Vec<TokenStream2> = Vec::new();
    let mut field_inits: Vec<TokenStream2> = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().expect("Fields::Named always have idents");
        let field_ty = &field.ty;

        if let Some(dep_ty) = inject_type_for_field(field) {
            dep_type_fns.push(quote! { || ::std::any::TypeId::of::<#dep_ty>() });
            if extract_arc_inner(field_ty).is_some() {
                field_inits.push(quote! {
                    #field_name: ctx.get_required_bean::<#dep_ty>()
                        .unwrap_or_else(|e| panic!("failed to inject {}: {e}", stringify!(#dep_ty)))
                });
            } else {
                field_inits.push(quote! {
                    #field_name: ctx.get_required_bean::<#dep_ty>()
                        .map(|arc| (*arc).clone())
                        .unwrap_or_else(|e| panic!("failed to inject {}: {e}", stringify!(#dep_ty)))
                });
            }
        } else if let Type::Path(p) = field_ty {
            if p.path.segments.last().is_some_and(|s| s.ident == "String") {
                field_inits.push(quote! { #field_name: ::std::string::String::new() });
            } else {
                field_inits.push(quote! {
                    #field_name: ::std::default::Default::default()
                });
            }
        } else {
            field_inits.push(quote! {
                #field_name: ::std::default::Default::default()
            });
        }
    }

    let dep_array = if dep_type_fns.is_empty() {
        quote! { &[] as &[fn() -> ::std::any::TypeId] }
    } else {
        quote! {
            {
                static #deps_static: &[fn() -> ::std::any::TypeId] = &[#(#dep_type_fns),*];
                #deps_static
            }
        }
    };

    quote! {
        fn #factory_fn(ctx: &::hiver_starter::core::ApplicationContext) -> ::std::boxed::Box<dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync> {
            ::std::boxed::Box::new(::std::sync::Arc::new(#struct_name {
                #(#field_inits),*
            }))
        }

        #condition_def

        ::inventory::submit! {
            ::hiver_starter::core::registry::BeanDescriptor {
                name: #bean_name_camel,
                type_id: || ::std::any::TypeId::of::<#struct_name>(),
                scope: #scope,
                factory: #factory_fn,
                dep_type_ids: #dep_array,
                condition: #condition_fn,
            }
        }
    }
}

fn to_camel_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}
