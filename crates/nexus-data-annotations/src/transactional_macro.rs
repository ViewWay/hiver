//! @Transactional attribute macro implementation
//! @Transactional 属性宏实现
//!
//! This macro provides compile-time support for the @Transactional annotation.
//! 此宏为 @Transactional 注解提供编译时支持。

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};
use syn::{parse::ParseStream, Result as SynResult, parse::Parse};

/// Parses @Transactional attributes
/// 解析 @Transactional 属性
struct TransactionalAttrs {
    isolation: Option<syn::Ident>,
    timeout: Option<syn::LitInt>,
    propagation: Option<syn::Ident>,
    read_only: Option<bool>,
    max_retries: Option<syn::LitInt>,
}

impl Parse for TransactionalAttrs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut isolation = None;
        let mut timeout = None;
        let mut propagation = None;
        let mut read_only = None;
        let mut max_retries = None;

        while !input.is_empty() {
            // Parse key = value or key
            // 解析 key = value 或 key
            let key: syn::Ident = input.parse()?;

            if input.peek(syn::token::Eq) {
                // key = value
                input.parse::<syn::token::Eq>()?;

                let lookahead = input.lookahead1();
                if lookahead.peek(syn::LitInt) {
                    let value: syn::LitInt = input.parse()?;
                    if key == "timeout" {
                        timeout = Some(value);
                    } else if key == "max_retries" || key == "maxRetries" {
                        max_retries = Some(value);
                    }
                } else if lookahead.peek(syn::LitBool) {
                    let value: syn::LitBool = input.parse()?;
                    if key == "read_only" || key == "readOnly" {
                        read_only = Some(value.value);
                    }
                } else if lookahead.peek(syn::Ident) {
                    let value: syn::Ident = input.parse()?;
                    if key == "isolation" || key == "isolationLevel" {
                        isolation = Some(value);
                    } else if key == "propagation" {
                        propagation = Some(value);
                    }
                } else {
                    return Err(lookahead.error());
                }
            } else {
                // Just a key, treat as boolean flag
                // 仅仅是 key，视为布尔标志
                if key == "read_only" || key == "readOnly" {
                    read_only = Some(true);
                }
            }

            // Check for comma
            // 检查逗号
            if !input.is_empty() {
                if input.peek(syn::token::Comma) {
                    input.parse::<syn::token::Comma>()?;
                } else {
                    break;
                }
            }
        }

        Ok(TransactionalAttrs {
            isolation,
            timeout,
            propagation,
            read_only,
            max_retries,
        })
    }
}

/// Implements #[Transactional] attribute macro
/// 实现 #[Transactional] 属性宏
///
/// Marks a function or method to be executed within a transaction.
/// 将函数或方法标记为在事务中执行。
///
/// # Attributes / 属性
///
/// - `isolation` - Transaction isolation level (default: Default)
///   事务隔离级别（默认：Default）
/// - `timeout` - Transaction timeout in seconds (default: 30)
///   事务超时时间（秒，默认：30）
/// - `propagation` - Transaction propagation behavior (default: Required)
///   事务传播行为（默认：Required）
/// - `read_only` - Whether transaction is read-only (default: false)
///   事务是否只读（默认：false）
/// - `max_retries` - Max retry attempts (default: 3)
///   最大重试次数（默认：3）
///
/// # Isolation Levels / 隔离级别
///
/// - `Default` - Use database default
/// - `ReadUncommitted` - Lowest isolation
/// - `ReadCommitted` - Prevents dirty reads
/// - `RepeatableRead` - Prevents non-repeatable reads
/// - `Serializable` - Highest isolation
///
/// # Propagation Behaviors / 传播行为
///
/// - `Required` - Support current, create new if none (default)
/// - `Supports` - Support current, non-transactional if none
/// - `Mandatory` - Support current, error if none
/// - `RequiresNew` - Always create new
/// - `NotSupported` - Non-transactional, suspend current
/// - `Never` - Non-transactional, error if exists
/// - `Nested` - Nested transaction if exists
///
/// # Example / 示例
///
/// ```rust,ignore
/// use nexus_data_annotations::Transactional;
/// use nexus_data_annotations::transactional::{IsolationLevel, Propagation};
///
/// // Default configuration
/// // 默认配置
/// #[Transactional]
/// async fn create_user(&self, user: User) -> Result<(), Error> {
///     // Automatically executed in a transaction
///     // 自动在事务中执行
///     Ok(())
/// }
///
/// // Custom isolation level
/// // 自定义隔离级别
/// #[Transactional(isolation = ReadCommitted)]
/// async fn transfer_funds(&self, from: i64, to: i64, amount: i64) -> Result<(), Error> {
///     // Executed with READ COMMITTED isolation
///     // 使用 READ COMMITTED 隔离级别执行
///     Ok(())
/// }
///
/// // Multiple attributes
/// // 多个属性
/// #[Transactional(
///     isolation = Serializable,
///     timeout = 60,
///     propagation = RequiresNew,
///     read_only = false,
///     max_retries = 5
/// )]
/// async fn critical_operation(&self) -> Result<(), Error> {
///     // Highly configured transaction
///     // 高度配置的事务
///     Ok(())
/// }
/// ```
pub(crate) fn impl_transactional(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let _func_name = &input.sig.ident;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    // Parse transactional attributes
    // 解析事务属性
    let transactional_attrs = parse_macro_input!(attr as TransactionalAttrs);

    // Build configuration
    // 构建配置
    let isolation = transactional_attrs.isolation;
    let timeout = transactional_attrs.timeout;
    let propagation = transactional_attrs.propagation;
    let read_only = transactional_attrs.read_only;
    let max_retries = transactional_attrs.max_retries;

    // Generate config builder code
    // 生成配置构建器代码
    let mut config_builder = quote! {
        ::nexus_data_annotations::transactional::TransactionalConfig::new()
    };

    if let Some(iso) = isolation {
        let _iso_str = format!("{:?}", iso).to_uppercase();
        config_builder = quote! {
            #config_builder.isolation(::nexus_data_annotations::transactional::IsolationLevel::#iso)
        };
    }

    if let Some(to) = timeout {
        let to_value = to.base10_parse::<u64>().unwrap_or(30);
        config_builder = quote! {
            #config_builder.timeout(#to_value)
        };
    }

    if let Some(prop) = propagation {
        config_builder = quote! {
            #config_builder.propagation(::nexus_data_annotations::transactional::Propagation::#prop)
        };
    }

    if let Some(ro) = read_only {
        config_builder = quote! {
            #config_builder.read_only(#ro)
        };
    }

    if let Some(mr) = max_retries {
        let mr_value = mr.base10_parse::<u32>().unwrap_or(3);
        config_builder = quote! {
            #config_builder.max_retries(#mr_value)
        };
    }

    // Extract function parameters for the wrapper
    // 为包装器提取函数参数
    let _inputs = &sig.inputs;
    let _output = &sig.output;
    let generics = &sig.generics;

    // Generate wrapper function
    // 生成包装器函数
    let expanded = quote! {
        #(#attrs)*
        #vis #sig #generics {
            // Get transactional executor from context or create default
            // 从上下文获取事务执行器或创建默认值
            let executor = ::nexus_data_annotations::transactional::get_transactional_executor();

            // Build transactional configuration
            // 构建事务配置
            let config = #config_builder;

            // Execute the function within a transaction
            // 在事务中执行函数
            let result = executor.execute(config, || async move {
                #block
            }).await;

            result
        }
    };

    TokenStream::from(expanded)
}

/// Get the transactional executor
/// 获取事务执行器
///
/// This is a helper function that should be replaced with actual
/// executor retrieval from a dependency injection container.
///
/// 这是一个辅助函数，应该被从依赖注入容器中获取实际执行器的代码替换。
#[doc(hidden)]
pub(crate) fn get_transactional_executor() -> crate::transactional::TransactionalExecutor
{
    // This is a placeholder - actual implementation would get the executor
    // from a dependency injection container or global context
    // 这是一个占位符 - 实际实现会从依赖注入容器或全局上下文获取执行器

    // For now, create a dummy executor that just returns an error
    // 现在创建一个虚拟执行器，只返回错误
    todo!("Transactional executor not yet integrated with dependency injection")
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    #[test]
    fn test_parse_transactional_attrs_empty() {
        let input = quote! {};
        let attrs: TransactionalAttrs = syn::parse2(input).unwrap();
        assert!(attrs.isolation.is_none());
        assert!(attrs.timeout.is_none());
        assert!(attrs.propagation.is_none());
        assert!(attrs.read_only.is_none());
        assert!(attrs.max_retries.is_none());
    }

    #[test]
    fn test_parse_transactional_attrs_isolation() {
        let input = quote! { isolation = ReadCommitted };
        let attrs: TransactionalAttrs = syn::parse2(input).unwrap();
        assert_eq!(attrs.isolation.unwrap().to_string(), "ReadCommitted");
    }

    #[test]
    fn test_parse_transactional_attrs_multiple() {
        let input = quote! {
            isolation = Serializable,
            timeout = 60,
            propagation = RequiresNew,
            read_only = true,
            max_retries = 5
        };
        let attrs: TransactionalAttrs = syn::parse2(input).unwrap();
        assert_eq!(attrs.isolation.unwrap().to_string(), "Serializable");
        assert_eq!(attrs.timeout.unwrap().base10_parse::<u64>().unwrap(), 60);
        assert_eq!(attrs.propagation.unwrap().to_string(), "RequiresNew");
        assert_eq!(attrs.read_only.unwrap(), true);
        assert_eq!(attrs.max_retries.unwrap().base10_parse::<u32>().unwrap(), 5);
    }

    #[test]
    fn test_parse_transactional_attrs_read_only_flag() {
        let input = quote! { read_only };
        let attrs: TransactionalAttrs = syn::parse2(input).unwrap();
        assert_eq!(attrs.read_only.unwrap(), true);
    }
}
