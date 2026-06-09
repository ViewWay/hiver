//! Procedural macros for hiver-retry
//! hiver-retry 的过程宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Retry attribute macro for automatic retry logic
/// 重试属性宏，用于自动重试逻辑
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Retryable(maxAttempts = 3, backoff = @Backoff(delay = 100))
/// public Service callService() {
///     // ...
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_retry::retry;
///
/// #[retry]
/// async fn fetch_data() -> Result<String, std::io::Error> {
///     Ok("data".to_string())
/// }
///
/// #[retry(max_attempts = 5, backoff = "exponential", initial_delay = 100)]
/// async fn call_api() -> Result<String, reqwest::Error> {
///     Ok(reqwest::get("https://api.example.com").await?.text().await?)
/// }
/// ```
#[proc_macro_attribute]
pub fn retry(args: TokenStream, input: TokenStream) -> TokenStream
{
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse arguments using string parsing
    let args_str = args.to_string();
    let config = parse_retry_config_string(&args_str);

    // Extract function details
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let attrs = &input_fn.attrs;

    // Check if function is async
    let is_async = sig.asyncness.is_some();

    // Get retry configuration values
    let max_attempts = config.max_attempts;
    let initial_delay_ms = config.initial_delay;
    let multiplier = config.multiplier;
    let max_delay_ms = config.max_delay.unwrap_or(30000);
    let backoff_type = &config.backoff;

    // Calculate delay function
    let delay_calc = if backoff_type == "exponential"
    {
        quote! {
            let delay = std::time::Duration::from_millis(
                ((#initial_delay_ms as f64) * (#multiplier).powi(attempt as i32 - 1)).min(#max_delay_ms) as u64
            );
        }
    }
    else
    {
        quote! {
            let delay = std::time::Duration::from_millis(#initial_delay_ms);
        }
    };

    // Generate retry wrapper
    let expanded = if is_async
    {
        quote! {
            #(#attrs)*
            #vis #sig {
                let mut last_error: Option<std::sync::Arc<dyn std::error::Error + Send + Sync>> = None;

                for attempt in 1..=#max_attempts {
                    let result = async move #block .await;

                    match result {
                        Ok(value) => {
                            return Ok(value);
                        }
                        Err(error) => {
                            use std::sync::Arc;
                            let error_arc = Arc::new(error) as Arc<dyn std::error::Error + Send + Sync>;
                            last_error = Some(error_arc);

                            if attempt < #max_attempts {
                                #delay_calc
                                tokio::time::sleep(delay).await;
                            }
                        }
                    }
                }

                Err(last_error.expect("Error should exist after all retries"))
            }
        }
    }
    else
    {
        quote! {
            #(#attrs)*
            #vis #sig {
                let mut last_error: Option<std::sync::Arc<dyn std::error::Error + Send + Sync>> = None;

                for attempt in 1..=#max_attempts {
                    let result = #block;

                    match result {
                        Ok(value) => {
                            return Ok(value);
                        }
                        Err(error) => {
                            use std::sync::Arc;
                            let error_arc = Arc::new(error) as Arc<dyn std::error::Error + Send + Sync>;
                            last_error = Some(error_arc);

                            if attempt < #max_attempts {
                                #delay_calc
                                std::thread::sleep(delay);
                            }
                        }
                    }
                }

                Err(last_error.expect("Error should exist after all retries"))
            }
        }
    };

    TokenStream::from(expanded)
}

/// Recover attribute macro for fallback methods
/// 恢复属性宏，用于降级方法
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Recover
/// public Service fallback(ServiceException e) {
///     // Return default value
/// }
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_retry::recover;
///
/// #[recover]
/// async fn fallback(error: RetryError<MyError>) -> MyData {
///     MyData::default()
/// }
/// ```
#[proc_macro_attribute]
pub fn recover(_args: TokenStream, input: TokenStream) -> TokenStream
{
    // For now, just pass through - recover methods work standalone
    input
}

/// Retry configuration
/// 重试配置
#[derive(Debug, Default)]
struct RetryConfig
{
    /// Maximum number of retry attempts
    /// 最大重试次数
    max_attempts: usize,

    /// Initial delay in milliseconds
    /// 初始延迟（毫秒）
    initial_delay: u64,

    /// Backoff type: "fixed" or "exponential"
    /// 退避类型："fixed" 或 "exponential"
    backoff: String,

    /// Multiplier for exponential backoff
    /// 指数退避倍数
    multiplier: f64,

    /// Maximum delay in milliseconds
    /// 最大延迟（毫秒）
    max_delay: Option<u64>,
}

/// Parse retry configuration from attribute string
/// 从属性字符串解析重试配置
fn parse_retry_config_string(args_str: &str) -> RetryConfig
{
    let mut config = RetryConfig::default();

    for pair in args_str.split(',')
    {
        let pair = pair.trim();
        if let Some((key, value)) = pair.split_once('=')
        {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key
            {
                "max_attempts" =>
                {
                    if let Ok(val) = value.parse::<usize>()
                    {
                        config.max_attempts = val;
                    }
                },
                "initial_delay" =>
                {
                    if let Ok(val) = value.parse::<u64>()
                    {
                        config.initial_delay = val;
                    }
                },
                "backoff" =>
                {
                    config.backoff = value.to_string();
                },
                "multiplier" =>
                {
                    if let Ok(val) = value.parse::<f64>()
                    {
                        config.multiplier = val;
                    }
                },
                "max_delay" =>
                {
                    if let Ok(val) = value.parse::<u64>()
                    {
                        config.max_delay = Some(val);
                    }
                },
                _ =>
                {},
            }
        }
    }

    config
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests;
