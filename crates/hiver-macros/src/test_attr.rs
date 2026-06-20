//! Implementation of the `#[hiver::test]` attribute macro.
//! `#[hiver::test]` 属性宏的实现。
//!
//! Transforms an `async fn` test body into a synchronous `#[test]` fn that
//! drives it on `hiver_runtime::Runtime::block_on`, so the test's async code
//! runs against the framework's reactor instead of tokio's.
//!
//! 将 `async fn` 测试体转换为同步的 `#[test]` fn,经由
//! `hiver_runtime::Runtime::block_on` 驱动,使测试的异步代码运行在框架自身的
//! reactor 上而非 tokio 的。

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, ReturnType, Signature, parse_macro_input};

/// Entry point for `#[hiver::test]`.
/// `#[hiver::test]` 的入口。
///
/// Input: an `async fn` (the test body). Output: a `#[test] fn` that wraps the
/// body in `hiver_runtime::Runtime::new()?.block_on(async move { ... })?`.
///
/// 输入:一个 `async fn`（测试体）。输出:一个 `#[test] fn`,把测试体包裹进
/// `hiver_runtime::Runtime::new()?.block_on(async move { ... })?`。
pub fn test_impl(item: TokenStream) -> TokenStream
{
    let mut item_fn = parse_macro_input!(item as ItemFn);

    // Validate that the input is an async fn.
    // 校验输入是 async fn。
    let Signature { asyncness, .. } = &item_fn.sig;
    if asyncness.is_none()
    {
        return syn::Error::new_spanned(
            &item_fn.sig.fn_token,
            "#[hiver::test] must be applied to an async fn / #[hiver::test] 必须应用于 async fn",
        )
        .to_compile_error()
        .into();
    }

    // Strip `async` and `-> ReturnType` from the signature: the wrapper is
    // synchronous and returns ().
    // 从签名移除 `async` 与 `-> ReturnType`:wrapper 是同步的且返回 ()。
    item_fn.sig.asyncness = None;
    let has_explicit_unit_return =
        matches!(&item_fn.sig.output, ReturnType::Default | ReturnType::Type(_, _));
    let _ = has_explicit_unit_return;
    // Force the output type to () regardless of what the async fn returned — the
    // block_on result is unwrapped and discarded.
    // 无论 async fn 返回什么,强制输出类型为 () —— block_on 的结果被 unwrap 后丢弃。
    item_fn.sig.output = ReturnType::Default;

    // The original async fn body becomes the body of an async block driven by
    // block_on. We keep the original block (statements) verbatim.
    // 原 async fn 的 body 成为经 block_on 驱动的 async 块的 body。原样保留
    // 原始语句块。
    let original_block = item_fn.block;

    // Build the new body: create a runtime, drive the async block, unwrap.
    // 构建新 body:创建 runtime,驱动 async 块,unwrap。
    let new_block = quote! {
        {
            let mut __hiver_rt = match hiver_runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => panic!("#[hiver::test]: failed to create runtime: {e}"),
            };
            let __hiver_result = match __hiver_rt.block_on(async move #original_block) {
                Ok(v) => v,
                Err(e) => panic!("#[hiver::test]: block_on failed: {e}"),
            };
            #[allow(clippy::let_unit_value)]
            let _ = __hiver_result;
        }
    };

    // Wrap as a #[test] fn.
    // 包裹为 #[test] fn。
    let test_attr = quote! { #[::core::prelude::v1::test] };
    let sig = &item_fn.sig;
    let vis = &item_fn.vis;

    // Preserve other attributes (e.g. #[ignore], #[should_panic]) but drop the
    // leading #[test]-like ones we replace.
    // 保留其它属性（如 #[ignore]、#[should_panic]）,但丢弃我们替换的 #[test] 类属性。
    let mut attrs = item_fn.attrs.clone();
    // Remove any existing #[test] / #[tokio::test] to avoid double-registration.
    // 移除既有的 #[test] / #[tokio::test] 以避免重复注册。
    attrs.retain(|a| {
        let s = a.path().to_token_stream().to_string();
        !(s == "test" || s == "tokio :: test" || s == "core :: prelude :: v1 :: test")
    });

    let _ = format_ident!("_"); // keep format_ident import used

    let output = quote! {
        #test_attr
        #(#attrs)*
        #vis #sig #new_block
    };

    output.into()
}

use quote::ToTokens;
