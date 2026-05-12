//! Procedural macros for nexus-events
//! nexus-events的过程宏

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Type};

/// Event listener attribute macro
/// 事件监听器属性宏
///
/// # Usage / 用法
///
/// ```rust,ignore
/// struct MyListener;
///
/// impl MyListener {
///     #[EventListener]
///     async fn on_event(&self, event: MyEvent) {
///         // Handle event
///     }
///
///     #[EventListener(order = 10)]
///     async fn on_priority_event(&self, event: PriorityEvent) {
///         // Handle with priority
///     }
/// }
/// ```
///
/// # Parameters / 参数
///
/// - `order`: Execution order (lower = higher priority), default 0
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn EventListener(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse arguments (simple key=value parsing)
    let args_str = args.to_string();
    let mut order: i32 = 0;

    for pair in args_str.split(',') {
        let pair = pair.trim();
        if let Some((key, value)) = pair.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            if key == "order"
                && let Ok(val) = value.parse::<i32>() {
                    order = val;
                }
        }
    }

    // Extract function details
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let _fn_output = &input_fn.sig.output;

    // Find event type (first parameter after self)
    let event_type = find_event_type(fn_inputs);

    // Check if function is async
    let is_async = input_fn.sig.asyncness.is_some();

    // Generate the implementation
    let expanded = if is_async {
        generate_async_listener(&input_fn, fn_name, fn_inputs, _fn_output, event_type, order)
    } else {
        generate_sync_listener(&input_fn, fn_name, fn_inputs, _fn_output, event_type, order)
    };

    TokenStream::from(expanded)
}

/// Transactional event listener attribute macro
/// 事务事件监听器属性宏
///
/// # Usage / 用法
///
/// ```rust,ignore
/// struct MyListener;
///
/// impl MyListener {
///     #[TransactionalEventListener(phase = "after_commit")]
///     async fn on_after_commit(&self, event: DataUpdatedEvent) {
///         // Handle after commit
///     }
/// }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn TransactionalEventListener(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse arguments
    let args_str = args.to_string();
    let mut phase = String::from("after_commit");
    let mut order: i32 = 0;

    for pair in args_str.split(',') {
        let pair = pair.trim();
        if let Some((key, value)) = pair.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            if key == "phase" {
                phase = value.to_string();
            } else if key == "order"
                && let Ok(val) = value.parse::<i32>() {
                    order = val;
                }
        }
    }

    // Extract function details
    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let _fn_output = &input_fn.sig.output;

    // Find event type
    let event_type = find_event_type(fn_inputs);

    // Check if function is async
    let is_async = input_fn.sig.asyncness.is_some();

    // Generate the implementation with transactional support
    let expanded = if is_async {
        generate_transactional_listener_async(&input_fn, fn_name, fn_inputs, _fn_output, event_type, order, phase)
    } else {
        generate_transactional_listener_sync(&input_fn, fn_name, fn_inputs, _fn_output, event_type, order, phase)
    };

    TokenStream::from(expanded)
}

/// Find the event type from function parameters
/// 从函数参数中查找事件类型
fn find_event_type(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> Option<Type> {
    for arg in inputs {
        if let FnArg::Typed(pat) = arg
            && let Type::Path(type_path) = &*pat.ty
                && let Some(segment) = type_path.path.segments.first()
                    && segment.ident != "Self" && segment.ident != "self" {
                        return Some(Type::Path(type_path.clone()));
                    }
    }
    None
}

/// Generate async event listener implementation
/// 生成异步事件监听器实现
fn generate_async_listener(
    input_fn: &ItemFn,
    fn_name: &Ident,
    _fn_inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    _fn_output: &syn::ReturnType,
    event_type: Option<Type>,
    order: i32,
) -> proc_macro2::TokenStream {
    if let Some(evt) = event_type {
        quote! {
            // Original function (unchanged)
            #input_fn

            // Note: The macro preserves the original function.
            // Manual registration using ListenerFn or AsyncListenerFn is recommended.
            impl #fn_name {
                pub const ORDER: i32 = #order;
                pub const EVENT_TYPE: &str = stringify!(#evt);
            }
        }
    } else {
        quote! {
            compile_error!("EventListener function must have an event parameter");
            #input_fn
        }
    }
}

/// Generate sync event listener implementation
/// 生成同步事件监听器实现
fn generate_sync_listener(
    input_fn: &ItemFn,
    fn_name: &Ident,
    _fn_inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    _fn_output: &syn::ReturnType,
    event_type: Option<Type>,
    order: i32,
) -> proc_macro2::TokenStream {
    if let Some(evt) = event_type {
        quote! {
            // Original function (unchanged)
            #input_fn

            // Note: For sync listeners, users should manually register using ListenerFn
            impl #fn_name {
                pub const ORDER: i32 = #order;
                pub const EVENT_TYPE: &str = stringify!(#evt);
            }
        }
    } else {
        quote! {
            compile_error!("EventListener function must have an event parameter");
            #input_fn
        }
    }
}

/// Generate transactional event listener implementation (async)
/// 生成事务事件监听器实现（异步）
fn generate_transactional_listener_async(
    input_fn: &ItemFn,
    fn_name: &Ident,
    _fn_inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    _fn_output: &syn::ReturnType,
    event_type: Option<Type>,
    order: i32,
    _phase: String,
) -> proc_macro2::TokenStream {
    if let Some(_evt) = event_type {
        quote! {
            // Original function (unchanged)
            #input_fn

            // Transactional listeners are registered with phase information
            impl #fn_name {
                pub const ORDER: i32 = #order;
                pub const PHASE: &str = #_phase;
            }
        }
    } else {
        quote! {
            compile_error!("TransactionalEventListener function must have an event parameter");
            #input_fn
        }
    }
}

/// Generate transactional event listener implementation (sync)
/// 生成事务事件监听器实现（同步）
fn generate_transactional_listener_sync(
    input_fn: &ItemFn,
    fn_name: &Ident,
    _fn_inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    _fn_output: &syn::ReturnType,
    event_type: Option<Type>,
    order: i32,
    _phase: String,
) -> proc_macro2::TokenStream {
    if let Some(_evt) = event_type {
        quote! {
            // Original function (unchanged)
            #input_fn

            // Transactional listeners are registered with phase information
            impl #fn_name {
                pub const ORDER: i32 = #order;
                pub const PHASE: &str = #_phase;
            }
        }
    } else {
        quote! {
            compile_error!("TransactionalEventListener function must have an event parameter");
            #input_fn
        }
    }
}


#[cfg(test)]
mod tests;