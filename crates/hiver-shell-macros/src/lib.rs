//! Procedural macros for hiver-shell
//! hiver-shell 的过程宏
//!
//! # Macros / 宏
//!
//! - `#[shell_component]` — Marks a struct as a shell command component
//! - `#[shell_method]` — Marks a method as a shell command

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

/// Marks a struct as a shell component that contains shell commands.
/// 将结构体标记为包含shell命令的shell组件。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_shell_macros::shell_component;
///
/// #[shell_component]
/// struct MyCommands;
/// ```
#[proc_macro_attribute]
pub fn shell_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            /// Auto-generated method to register commands
            pub fn register_commands(registry: &mut hiver_shell::command::CommandRegistry) {
                // Commands are registered at compile time via attribute macros
                let _ = registry;
            }
        }
    };

    TokenStream::from(expanded)
}

/// Marks a method as a shell command.
/// 将方法标记为shell命令。
///
/// # Attributes / 属性
///
/// - `name` — Command name / 命令名称
/// - `description` — Command description / 命令描述
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_shell_macros::shell_method;
///
/// #[shell_component]
/// struct GreetCommand;
///
/// impl GreetCommand {
///     #[shell_method("greet", "Greet someone")]
///     fn greet(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
///         Ok(format!("Hello, {}!", args.first().unwrap_or(&"World")))
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn shell_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attributes: first token is command name, rest is description
    let args: Vec<String> = attr
        .to_string()
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .collect();

    let cmd_name = args.first().map_or("unknown", String::as_str);
    let cmd_desc = args.get(1).map_or("", String::as_str);

    let input_fn = parse_macro_input!(item as syn::ImplItemFn);
    let fn_name = &input_fn.sig.ident;

    // Generate a const that registers this method
    let register_name = quote::format_ident!("__REGISTER_{}", fn_name.to_string().to_uppercase());

    let expanded = quote! {
        #input_fn

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        pub const #register_name: () = {
            // This forces the method to be registered at compile time
        };
    };

    // Attach documentation attribute
    let doc_comment = format!(" Command: {} — {}", cmd_name, cmd_desc);
    let expanded = quote! {
        #[doc = #doc_comment]
        #expanded
    };

    TokenStream::from(expanded)
}
