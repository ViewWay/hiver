//! @Pointcut attribute macro
//! @Pointcut 属性宏

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, parse_macro_input};
use syn::{Result as SynResult, parse::Parse, parse::ParseStream};

/// Parses pointcut expression from @Pointcut annotation
/// 解析 @Pointcut 注解中的切点表达式
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

/// Implements #[Pointcut] attribute macro
/// 实现 #[Pointcut] 属性宏
///
/// Defines a reusable pointcut expression that can be referenced by advice methods
/// 定义可重用的切点表达式，可以被通知方法引用
///
/// # Pointcut Designators / 切点指示符
///
/// - **execution** - Method execution join point
///   方法执行连接点
/// - **call** - Method call join point
///   方法调用连接点
/// - **within** - Limits to within certain types
///   限制在特定类型内
/// - **this** - Limit to match bean reference
///   限制匹配 bean 引用
/// - **target** - Limit to match target object
///   限制匹配目标对象
/// - **args** - Limit to match arguments
///   限制匹配参数
/// - **@annotation** - Limit to join points with subject annotation
///   限制带有特定注解的连接点
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::Pointcut;
///
/// // Define a reusable pointcut
/// // 定义可重用的切点
/// #[Pointcut("execution(* com.example.service.*.*(..))")]
/// fn service_layer() -> PointcutExpression {
///     // Returns a reusable pointcut for all service layer methods
/// }
///
/// // Use the pointcut in advice
/// // 在通知中使用切点
/// #[Before("service_layer()")]
/// fn log_service_methods(&self, join_point: &JoinPoint) {
///     println!("Calling service method: {}", join_point.method_name());
/// }
/// ```
///
/// # Complex Expressions / 复杂表达式
///
/// ```rust,no_run,ignore
/// use hiver_aop::Pointcut;
///
/// // Combine multiple pointcuts
/// // 组合多个切点
/// #[Pointcut("execution(public * *(..))")]
/// fn public_methods() -> PointcutExpression {}
///
/// #[Pointcut("execution(* com.example..*.*(..))")]
/// fn in_package() -> PointcutExpression {}
///
/// // AND combination
/// // AND 组合
/// #[Pointcut("public_methods() && in_package()")]
/// fn public_methods_in_package() -> PointcutExpression {}
///
/// // OR combination
/// // OR 组合
/// #[Pointcut("execution(* com.example.Service.*(..)) || execution(* com.example.Repository.*(..))")]
/// fn service_or_repository() -> PointcutExpression {}
/// ```
pub(crate) fn impl_pointcut(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let func_name = &input.sig.ident;

    // Parse pointcut expression from attribute
    // 从属性中解析切点表达式
    let args = parse_macro_input!(attr as PointcutExpr);
    let pointcut = args.expression;

    let expanded = quote! {
        #input

        impl #func_name {
            /// Returns the pointcut expression
            /// 返回切点表达式
            const EXPRESSION: &str = #pointcut;

            #[doc(hidden)]
            fn _hiver_get_expression() -> &'static str {
                EXPRESSION
            }
        }
    };

    TokenStream::from(expanded)
}
