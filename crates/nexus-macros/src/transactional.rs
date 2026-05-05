//! Transactional macro implementation
//! Transactional宏实现
//!
//! This module provides the #[transactional] procedural macro for automatic transaction management.
//! 本模块提供#[transactional]过程宏用于自动事务管理。

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ItemFn, parse_macro_input};

/// #[transactional] macro implementation
/// #[transactional]宏实现
///
/// This is the actual implementation of the transactional macro.
/// The public wrapper is in lib.rs with the #[`proc_macro_attribute`] tag.
/// 这是transactional宏的实际实现。
/// 公共包装器在lib.rs中，带有#[`proc_macro_attribute`]标签。
///
/// Equivalent to Spring's @Transactional annotation.
/// 等价于Spring的@Transactional注解。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// #[transactional]
/// async fn create_user(user: User) -> Result<User> {
///     // Transaction logic
/// }
///
/// #[transactional(propagation = "REQUIRES_NEW", isolation = "SERIALIZABLE")]
/// async fn transfer_money(from: u64, to: u64, amount: f64) -> Result<()> {
///     // Transaction logic
/// }
/// ```
pub(crate) fn transactional_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute if not empty
    // 解析属性（如果不为空）
    let options = if attr.is_empty() {
        TransactionalOptionsParsed::default()
    } else {
        parse_transactional_options_attr(attr)
    };

    let function = parse_macro_input!(item as ItemFn);

    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;
    let fn_async = &function.sig.asyncness;
    let fn_inputs = &function.sig.inputs;
    let fn_output = &function.sig.output;
    let fn_block = &function.block;
    let fn_attrs = &function.attrs;

    // Build propagation configuration
    // 构建传播配置
    let propagation_config = if let Some(prop) = &options.propagation {
        quote! { definition = #prop; }
    } else {
        quote! {}
    };

    // Build isolation configuration
    // 构建隔离级别配置
    let isolation_config = if let Some(iso) = &options.isolation {
        quote! { definition = #iso; }
    } else {
        quote! {}
    };

    // Build timeout configuration
    // 构建超时配置
    let timeout_config = if let Some(timeout) = &options.timeout_secs {
        quote! { definition = #timeout; }
    } else {
        quote! {}
    };

    // Build read_only configuration
    // 构建只读配置
    let readonly_config = if options.read_only {
        quote! { definition = definition.read_only(true); }
    } else {
        quote! {}
    };

    // Generate transaction wrapper code
    // 生成事务包装代码
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            use nexus_tx::{TransactionManager, TransactionDefinition, Propagation, IsolationLevel};

            // Get transaction manager via OnceLock lazy initialization
            // 通过 OnceLock 懒加载获取事务管理器
            static TX_MANAGER: ::std::sync::OnceLock<::std::sync::Arc<dyn TransactionManager>> = ::std::sync::OnceLock::new();
            let tx_mgr = TX_MANAGER.get_or_init(|| {
                // In a full implementation, this would resolve from the DI container
                // For now, we use a placeholder that panics at runtime if actually called
                // 在完整实现中，这将从DI容器解析。
                // 目前使用占位符，如果实际调用会在运行时panic。
                ::std::sync::Arc::new(nexus_tx::NoopTransactionManager)
            });

            // Create transaction definition
            // 创建事务定义
            let mut definition = TransactionDefinition::new(stringify!(#fn_name));

            #propagation_config
            #isolation_config
            #timeout_config
            #readonly_config

            // Begin transaction
            // 开始事务
            let status = match tx_mgr.begin(&definition).await {
                Ok(s) => s,
                Err(e) => return Err(e.into()),
            };

            // Execute the original function
            // 执行原函数
            let result = async move {
                #fn_block
            }.await;

            // Commit or rollback based on result
            // 根据结果提交或回滚
            match &result {
                Ok(_) => {
                    if let Err(e) = tx_mgr.commit(status).await {
                        return Err(e.into());
                    }
                }
                Err(_err) => {
                    // Simplified: always rollback on error
                    // 简化版：错误时总是回滚
                    let _ = tx_mgr.rollback(status).await;
                }
            }

            result
        }
    };

    TokenStream::from(expanded)
}

/// Parsed transactional options
/// 解析的transactional选项
#[derive(Default)]
struct TransactionalOptionsParsed {
    propagation: Option<Expr>,
    isolation: Option<Expr>,
    timeout_secs: Option<Expr>,
    read_only: bool,
}

/// Parse transactional options from attribute token stream
/// 从属性token流解析transactional选项
fn parse_transactional_options_attr(attr: TokenStream) -> TransactionalOptionsParsed {
    let mut options = TransactionalOptionsParsed::default();

    // Parse the attribute as a comma-separated list of name=value pairs
    // 将属性解析为逗号分隔的 name=value 对列表
    let attr_str = attr.to_string();

    // Simple parsing - split by comma, then by '='
    // 简单解析 - 先按逗号分割，再按等号分割
    for pair in attr_str.split(',') {
        let pair = pair.trim();
        if let Some((key, value)) = pair.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match key {
                "propagation" => {
                    let prop_expr = match value {
                        "REQUIRED" => quote! { .propagation(Propagation::Required) },
                        "REQUIRES_NEW" => quote! { .propagation(Propagation::RequiresNew) },
                        "SUPPORTS" => quote! { .propagation(Propagation::Supports) },
                        "NOT_SUPPORTED" => quote! { .propagation(Propagation::NotSupported) },
                        "MANDATORY" => quote! { .propagation(Propagation::Mandatory) },
                        "NEVER" => quote! { .propagation(Propagation::Never) },
                        "NESTED" => quote! { .propagation(Propagation::Nested) },
                        _ => quote! {},
                    };
                    options.propagation = Some(syn::parse2(prop_expr).expect("unexpected error"));
                },
                "isolation" => {
                    let iso_expr = match value {
                        "READ_UNCOMMITTED" => {
                            quote! { .isolation(IsolationLevel::ReadUncommitted) }
                        },
                        "READ_COMMITTED" => quote! { .isolation(IsolationLevel::ReadCommitted) },
                        "REPEATABLE_READ" => quote! { .isolation(IsolationLevel::RepeatableRead) },
                        "SERIALIZABLE" => quote! { .isolation(IsolationLevel::Serializable) },
                        _ => quote! {},
                    };
                    options.isolation = Some(syn::parse2(iso_expr).expect("unexpected error"));
                },
                "timeout" | "timeout_secs" => {
                    if let Ok(timeout_val) = value.parse::<u64>() {
                        let timeout_expr = quote! { .timeout_secs(#timeout_val) };
                        options.timeout_secs = Some(syn::parse2(timeout_expr).expect("unexpected error"));
                    }
                },
                "read_only" => {
                    options.read_only = value == "true" || value == "1";
                },
                _ => {},
            }
        }
    }

    options
}
