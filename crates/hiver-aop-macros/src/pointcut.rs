//! `pointcut` attribute macro
//! `pointcut` 属性宏
//!
//! Like the advice macros, the pointcut macro emits the annotated function
//! unchanged plus a uniquely-named companion `const` recording the expression
//! string. This is valid Rust both at module scope and inside an `impl` block
//! (where a nested `impl` would be illegal).
//! 与通知宏一样，切点宏会原样输出被注解的函数，并附加一个唯一命名的伴生 `const`
//! 记录表达式字符串。它在模块作用域和 `impl` 块内部都是合法的 Rust 代码
//! （在后者中嵌套 `impl` 是非法的）。

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    ItemFn, LitStr, Result as SynResult,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Parses a pointcut expression from the `pointcut` annotation.
/// 从 `pointcut` 注解中解析切点表达式。
struct PointcutExpr
{
    expression: String,
}

impl Parse for PointcutExpr
{
    fn parse(input: ParseStream) -> SynResult<Self>
    {
        // Parse a string literal: `#[pointcut("execution(...")]`
        // 解析字符串字面量：`#[pointcut("execution(...")]`
        let expr_lit: LitStr = input.parse()?;

        Ok(PointcutExpr {
            expression: expr_lit.value(),
        })
    }
}

/// Implements the `#[pointcut]` attribute macro.
/// 实现 `#[pointcut]` 属性宏。
///
/// Defines a reusable pointcut expression that can be referenced by advice
/// methods. The macro records the expression as a companion `const` next to the
/// annotated function.
/// 定义可重用的切点表达式，可被通知方法引用。宏将表达式记录为被注解函数旁的伴生 `const`。
///
/// # Pointcut Designators / 切点指示符
///
/// - **execution** — Method execution join point / 方法执行连接点
/// - **call** — Method call join point / 方法调用连接点
/// - **within** — Limits to within certain types / 限制在特定类型内
/// - **this** — Limit to match bean reference / 限制匹配 bean 引用
/// - **target** — Limit to match target object / 限制匹配目标对象
/// - **args** — Limit to match arguments / 限制匹配参数
/// - **@annotation** — Limit to join points with subject annotation / 限制带有特定注解的连接点
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_aop::pointcut;
///
/// // Define a reusable pointcut
/// // 定义可重用的切点
/// #[pointcut("execution(* *.service.*.*(..))")]
/// fn service_layer() {}
/// ```
///
/// # Complex Expressions / 复杂表达式
///
/// Pointcuts may be combined with `&&` (all must match) or `||` (any may match):
/// 切点可通过 `&&`（全部匹配）或 `||`（任一匹配）组合：
///
/// ```rust,ignore
/// use hiver_aop::pointcut;
///
/// #[pointcut("execution(public * *(..))")]
/// fn public_methods() {}
///
/// #[pointcut("execution(* *..*.*(..)) || within(*.Service)")]
/// fn broad_match() {}
/// ```
pub(crate) fn impl_pointcut(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    // Uniquely-named companion const so multiple pointcut definitions never clash.
    // 唯一命名的伴生 const，使多个切点定义永不冲突。
    let meta_ident = format_ident!("_HIVER_POINTCUT_{}_EXPR", func_name.to_string().to_uppercase());

    let expanded = quote! {
        #input

        /// Hiver AOP pointcut expression recorded by the `pointcut` macro.
        /// 由 `pointcut` 宏记录的 Hiver AOP 切点表达式。
        #[doc(hidden)]
        const #meta_ident: &'static str = #pointcut;
    };

    TokenStream::from(expanded)
}
