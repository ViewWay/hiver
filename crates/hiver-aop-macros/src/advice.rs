//! Advice attribute macros (@Before, @After, @Around, @AfterReturning, @AfterThrowing)
//! 通知属性宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemFn, LitStr, Result as SynResult,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Parses pointcut expression from advice annotation
/// 解析通知注解中的切点表达式
struct PointcutExpr {
    expression: String,
}

impl Parse for PointcutExpr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Try to parse a string literal
        // 尝试解析字符串字面量
        let expr_lit: LitStr = input.parse()?;

        Ok(PointcutExpr {
            expression: expr_lit.value(),
        })
    }
}

/// Implements #[Before] attribute macro
/// 实现 #[Before] 属性宏
///
/// Marks a method as before advice (executed before the join point)
/// 将方法标记为前置通知（在连接点之前执行）
///
/// # Pointcut Expressions / 切点表达式
///
/// Common patterns / 常用模式:
/// - `execution(* com.example..*.*(..))` - All methods in com.example package
/// - `execution(* com.example.Service.*(..))` - All methods in Service class
/// - `execution(public * *(..))` - All public methods
/// - `execution(@org.annotation.Transactional * *(..))` - Methods with @Transactional
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Before;
///
/// #[Before("execution(* com.example..*.*(..))")]
/// fn log_before(&self, join_point: &JoinPoint) {
///     println!("Entering: {}", join_point.method_name());
/// }
/// ```
pub(crate) fn impl_before(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse pointcut expression from attribute
    // 从属性中解析切点表达式
    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the pointcut expression for this advice
            /// 返回此通知的切点表达式
            const POINTCUT: &str = #pointcut;

            /// Returns the advice type
            /// 返回通知类型
            const ADVICE_TYPE: &str = "before";

            #[doc(hidden)]
            fn _hiver_get_pointcut() -> &'static str {
                POINTCUT
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[After] attribute macro
/// 实现 #[After] 属性宏
///
/// Marks a method as after advice (executed after the join point)
/// 将方法标记为后置通知（在连接点之后执行）
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::After;
///
/// #[After("execution(* com.example..*.*(..))")]
/// fn log_after(&self, join_point: &JoinPoint) {
///     println!("Exiting: {}", join_point.method_name());
/// }
/// ```
pub(crate) fn impl_after(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse pointcut expression from attribute
    // 从属性中解析切点表达式
    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the pointcut expression for this advice
            /// 返回此通知的切点表达式
            const POINTCUT: &str = #pointcut;

            /// Returns the advice type
            /// 返回通知类型
            const ADVICE_TYPE: &str = "after";

            #[doc(hidden)]
            fn _hiver_get_pointcut() -> &'static str {
                POINTCUT
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[Around] attribute macro
/// 实现 #[Around] 属性宏
///
/// Marks a method as around advice (wraps the join point execution)
/// 将方法标记为环绕通知（包装连接点的执行）
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Around;
///
/// #[Around("execution(* com.example..*.*(..))")]
/// fn log_around(&self, join_point: JoinPoint) -> Result<(), Error> {
///     println!("Before: {}", join_point.method_name());
///     let result = join_point.proceed()?;
///     println!("After: {}", join_point.method_name());
///     Ok(result)
/// }
/// ```
pub(crate) fn impl_around(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse pointcut expression from attribute
    // 从属性中解析切点表达式
    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the pointcut expression for this advice
            /// 返回此通知的切点表达式
            const POINTCUT: &str = #pointcut;

            /// Returns the advice type
            /// 返回通知类型
            const ADVICE_TYPE: &str = "around";

            #[doc(hidden)]
            fn _hiver_get_pointcut() -> &'static str {
                POINTCUT
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[AfterReturning] attribute macro
/// 实现 #[AfterReturning] 属性宏
///
/// Marks a method as after-returning advice (executed after the join point returns successfully)
/// 将方法标记为返回后通知（在连接点成功返回后执行）
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::AfterReturning;
///
/// #[AfterReturning("execution(* com.example..*.*(..))")]
/// fn log_success(&self, join_point: &JoinPoint, result: &ReturnValue) {
///     println!("Method returned successfully: {}", join_point.method_name());
/// }
/// ```
pub(crate) fn impl_after_returning(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the pointcut expression for this advice
            /// 返回此通知的切点表达式
            const POINTCUT: &str = #pointcut;

            /// Returns the advice type
            /// 返回通知类型
            const ADVICE_TYPE: &str = "after_returning";

            #[doc(hidden)]
            fn _hiver_get_pointcut() -> &'static str {
                POINTCUT
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implements #[AfterThrowing] attribute macro
/// 实现 #[AfterThrowing] 属性宏
///
/// Marks a method as after-throwing advice (executed when the join point throws an error)
/// 将方法标记为异常后通知（在连接点抛出异常时执行）
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::AfterThrowing;
///
/// #[AfterThrowing("execution(* com.example..*.*(..))")]
/// fn log_error(&self, join_point: &JoinPoint, error: &Error) {
///     eprintln!("Method threw error: {} - {}", join_point.method_name(), error);
/// }
/// ```
pub(crate) fn impl_after_throwing(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the pointcut expression for this advice
            /// 返回此通知的切点表达式
            const POINTCUT: &str = #pointcut;

            /// Returns the advice type
            /// 返回通知类型
            const ADVICE_TYPE: &str = "after_throwing";

            #[doc(hidden)]
            fn _hiver_get_pointcut() -> &'static str {
                POINTCUT
            }
        }
    };

    TokenStream::from(expanded)
}
