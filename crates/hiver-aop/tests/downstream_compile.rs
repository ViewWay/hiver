//! Downstream compile regression tests / 下游编译回归测试
//!
//! These tests use [`trybuild`] to compile small programs as if they were
//! written in a *downstream* crate that depends on `hiver-aop`. They guard
//! against the regression documented in issues #60 and #61, where the advice
//! and pointcut macros generated an invalid `impl #func_name { ... }` that:
//!
//! 1. Failed inside an `impl` block with `error: implementation is not supported in traits or
//!    impls`; and
//! 2. Failed on a free function with `error[E0573]: expected type, found function`.
//!
//! 这些测试使用 [`trybuild`]，以"下游 crate 依赖 `hiver-aop`"的方式编译小程序，
//! 用于防范 issue #60 和 #61 中记录的回归：通知与切点宏曾生成非法的
//! `impl #func_name { ... }`，导致：
//!
//! 1. 在 `impl` 块内报 `error: implementation is not supported in traits or impls`；
//! 2. 在自由函数上报 `error[E0573]: expected type, found function`。
//!
//! Run with: `cargo test -p hiver-aop --test downstream_compile`
//!
//! [`trybuild`]: https://docs.rs/trybuild

#[allow(
    clippy::indexing_slicing,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
#[test]
fn downstream_compile_tests()
{
    let t = trybuild::TestCases::new();

    // These fixtures MUST compile after the fix.
    // 这些夹具在修复后必须能编译通过。
    t.pass("tests/ui/free_fn_advice.rs");
    t.pass("tests/ui/impl_block_advice.rs");
    t.pass("tests/ui/uppercase_aliases.rs");

    // Documents the compile error when a non-existent macro name is imported.
    // 记录导入不存在的宏名时的编译错误。
    t.compile_fail("tests/ui/wrong_macro_name_fail.rs");
}
