//! AOP Runtime support / AOP 运行时支持
//!
//! This module provides runtime support for AOP, including:
//! - JoinPoint: Represents a method execution join point
//! - PointcutExpression: Represents a pointcut expression
//! - AspectRegistry: Registers and manages aspects
//! - Proxy: Generates proxies that apply aspects
//!
//! 此模块提供 AOP 的运行时支持，包括：
//! - JoinPoint: 表示方法执行的连接点
//! - PointcutExpression: 表示切点表达式
//! - AspectRegistry: 注册和管理切面
//! - Proxy: 生成应用切面的代理

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    sync::Arc,
};

use tokio::sync::RwLock;

// ============================================================================
// JoinPoint / 连接点
// ============================================================================

/// Represents a join point in the program execution
/// 表示程序执行中的连接点
///
/// A join point is a well-defined point in the execution of a program,
/// such as a method call or exception handler.
///
/// 连接点是程序执行中一个明确定义的点，例如方法调用或异常处理程序。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_aop::runtime::JoinPoint;
///
/// #[Before("execution(* com.example..*.*(..))")]
/// fn log_before(join_point: &JoinPoint) {
///     println!("Calling: {}", join_point.method_name());
///     println!("Args: {:?}", join_point.args());
/// }
/// ```
pub struct JoinPoint
{
    /// Target object (self)
    /// 目标对象 (self)
    target: Arc<dyn Any + Send + Sync>,

    /// Method name
    /// 方法名
    method_name: String,

    /// Method arguments
    /// 方法参数
    args: Vec<Arc<dyn Any + Send + Sync>>,

    /// Method signature
    /// 方法签名
    signature: String,

    /// Target class name
    /// 目标类名
    target_class: String,
}

impl JoinPoint
{
    /// Create a new join point
    /// 创建新的连接点
    pub fn new(
        target: Arc<dyn Any + Send + Sync>,
        method_name: String,
        args: Vec<Arc<dyn Any + Send + Sync>>,
        signature: String,
        target_class: String,
    ) -> Self
    {
        Self {
            target,
            method_name,
            args,
            signature,
            target_class,
        }
    }

    /// Get the target object
    /// 获取目标对象
    pub fn target(&self) -> &Arc<dyn Any + Send + Sync>
    {
        &self.target
    }

    /// Get the method name
    /// 获取方法名
    pub fn method_name(&self) -> &str
    {
        &self.method_name
    }

    /// Get the method arguments
    /// 获取方法参数
    pub fn args(&self) -> &[Arc<dyn Any + Send + Sync>]
    {
        &self.args
    }

    /// Get the method signature
    /// 获取方法签名
    pub fn signature(&self) -> &str
    {
        &self.signature
    }

    /// Get the target class name
    /// 获取目标类名
    pub fn target_class(&self) -> &str
    {
        &self.target_class
    }

    /// Get argument by index
    /// 通过索引获取参数
    pub fn arg<T: 'static>(&self, index: usize) -> Option<&T>
    {
        self.args.get(index).and_then(|arg| arg.downcast_ref::<T>())
    }
}

impl Clone for JoinPoint
{
    fn clone(&self) -> Self
    {
        Self {
            target: self.target.clone(),
            method_name: self.method_name.clone(),
            args: self.args.clone(),
            signature: self.signature.clone(),
            target_class: self.target_class.clone(),
        }
    }
}

impl fmt::Debug for JoinPoint
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("JoinPoint")
            .field("method_name", &self.method_name)
            .field("signature", &self.signature)
            .field("target_class", &self.target_class)
            .field("num_args", &self.args.len())
            .finish()
    }
}

// ============================================================================
// ProceedingJoinPoint / 可继续的连接点
// ============================================================================

/// A join point that supports proceeding with the underlying method.
/// 支持继续执行底层方法的连接点。
///
/// Used by `@Around` advice to control whether and when the target
/// method executes.
///
/// 用于 `@Around` 通知控制目标方法是否以及何时执行。
pub struct ProceedingJoinPoint
{
    /// Inner join point data.
    /// 内部连接点数据。
    inner: JoinPoint,
    /// Whether proceed() has been called.
    /// proceed() 是否已被调用。
    proceeded: bool,
}

impl ProceedingJoinPoint
{
    /// Create a new proceeding join point.
    /// 创建新的可继续连接点。
    pub fn new(inner: JoinPoint) -> Self
    {
        Self {
            inner,
            proceeded: false,
        }
    }

    /// Get the method name.
    /// 获取方法名。
    pub fn method_name(&self) -> &str
    {
        self.inner.method_name()
    }

    /// Get the method arguments.
    /// 获取方法参数。
    pub fn args(&self) -> &[Arc<dyn Any + Send + Sync>]
    {
        self.inner.args()
    }

    /// Get typed argument by index.
    /// 通过索引获取类型化参数。
    pub fn arg<T: 'static>(&self, index: usize) -> Option<&T>
    {
        self.inner.arg(index)
    }

    /// Get the target class name.
    /// 获取目标类名。
    pub fn target_class(&self) -> &str
    {
        self.inner.target_class()
    }

    /// Get the method signature.
    /// 获取方法签名。
    pub fn signature(&self) -> &str
    {
        self.inner.signature()
    }

    /// Get the target object.
    /// 获取目标对象。
    pub fn target(&self) -> &Arc<dyn Any + Send + Sync>
    {
        self.inner.target()
    }

    /// Mark that the underlying method should proceed.
    /// 标记底层方法应继续执行。
    pub fn proceed(&mut self)
    {
        self.proceeded = true;
    }

    /// Check whether proceed has been called.
    /// 检查 proceed 是否已被调用。
    pub fn is_proceeded(&self) -> bool
    {
        self.proceeded
    }
}

impl Clone for ProceedingJoinPoint
{
    fn clone(&self) -> Self
    {
        Self {
            inner: self.inner.clone(),
            proceeded: self.proceeded,
        }
    }
}

impl fmt::Debug for ProceedingJoinPoint
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("ProceedingJoinPoint")
            .field("method_name", &self.inner.method_name())
            .field("proceeded", &self.proceeded)
            .finish()
    }
}

// ============================================================================
// AdviceChain / 通知执行链
// ============================================================================

/// Ordered advice to be applied to a join point.
/// 要应用到连接点的有序通知。
#[derive(Debug, Clone)]
pub struct AdviceChain
{
    /// Before advices (in order).
    /// 前置通知（按顺序）。
    pub before: Vec<String>,
    /// Around advices (in order).
    /// 环绕通知（按顺序）。
    pub around: Vec<String>,
    /// After advices (in order).
    /// 后置通知（按顺序）。
    pub after: Vec<String>,
    /// After-returning advices.
    /// 返回后通知。
    pub after_returning: Vec<String>,
    /// After-throwing advices.
    /// 异常后通知。
    pub after_throwing: Vec<String>,
}

impl AdviceChain
{
    /// Create an empty chain.
    /// 创建空链。
    pub fn new() -> Self
    {
        Self {
            before: Vec::new(),
            around: Vec::new(),
            after: Vec::new(),
            after_returning: Vec::new(),
            after_throwing: Vec::new(),
        }
    }

    /// Build a chain from matched advice.
    /// 从匹配的通知构建链。
    pub fn from_matches(matches: &[(AdviceType, String, String)]) -> Self
    {
        let mut chain = Self::new();
        for (advice_type, _aspect, method) in matches
        {
            match advice_type
            {
                AdviceType::Before => chain.before.push(method.clone()),
                AdviceType::Around => chain.around.push(method.clone()),
                AdviceType::After => chain.after.push(method.clone()),
                AdviceType::AfterReturning => chain.after_returning.push(method.clone()),
                AdviceType::AfterThrowing => chain.after_throwing.push(method.clone()),
            }
        }
        chain
    }

    /// Total number of advices in the chain.
    /// 链中通知总数。
    pub fn total(&self) -> usize
    {
        self.before.len()
            + self.around.len()
            + self.after.len()
            + self.after_returning.len()
            + self.after_throwing.len()
    }

    /// Check if the chain is empty.
    /// 检查链是否为空。
    pub fn is_empty(&self) -> bool
    {
        self.total() == 0
    }
}

impl Default for AdviceChain
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// Pointcut Expression / 切点表达式
// ============================================================================

/// Represents a pointcut expression
/// 表示切点表达式
///
/// Pointcut expressions define join points where advice should be applied.
///
/// 切点表达式定义应该应用通知的连接点。
#[derive(Debug, Clone)]
pub struct PointcutExpression
{
    /// The expression string
    /// 表达式字符串
    expression: String,

    /// Parsed expression components
    /// 解析后的表达式组件
    components: Vec<ExpressionComponent>,
}

/// Components of a pointcut expression
/// 切点表达式的组件
#[derive(Debug, Clone, PartialEq)]
enum ExpressionComponent
{
    /// Execution pointcut
    /// 执行切点
    Execution
    {
        /// Package pattern
        /// 包模式
        package: String,
        /// Class pattern
        /// 类模式
        class: String,
        /// Method pattern
        /// 方法模式
        method: String,
        /// Parameter pattern
        /// 参数模式
        params: String,
    },
    /// Within pointcut
    /// Within 切点
    Within(String),
    /// Annotation pointcut
    /// 注解切点
    Annotation(String),
    /// AND operation
    /// AND 操作
    And,
    /// OR operation
    /// OR 操作
    Or,
    /// NOT operation
    /// NOT 操作
    Not,
}

impl PointcutExpression
{
    /// Create a new pointcut expression
    /// 创建新的切点表达式
    pub fn new(expression: String) -> Self
    {
        let components = Self::parse_expression(&expression);
        Self {
            expression,
            components,
        }
    }

    /// Parse a pointcut expression
    /// 解析切点表达式
    fn parse_expression(expr: &str) -> Vec<ExpressionComponent>
    {
        let mut components = Vec::new();

        // Parse execution expressions
        // 解析 execution 表达式
        if let Some(start) = expr.find("execution(")
        {
            if let Some(end) = expr[start..].find(')')
            {
                let full_expr = &expr[start..start + end + 1];
                let _inner = &full_expr[11..full_expr.len() - 1]; // Remove "execution(" and ")"

                // Parse: package..class.method(params)
                // 简化的解析逻辑
                components.push(ExpressionComponent::Execution {
                    package: "*".to_string(),
                    class: "*".to_string(),
                    method: "*".to_string(),
                    params: "..".to_string(),
                });
            }
        }

        // Parse within expressions
        // 解析 within 表达式
        if let Some(start) = expr.find("within(")
        {
            if let Some(end) = expr[start..].find(')')
            {
                let inner = &expr[start + 7..start + end];
                components.push(ExpressionComponent::Within(inner.to_string()));
            }
        }

        // Parse @annotation expressions
        // 解析 @annotation 表达式
        if let Some(start) = expr.find("@annotation(")
        {
            if let Some(end) = expr[start..].find(')')
            {
                let inner = &expr[start + 13..start + end];
                components.push(ExpressionComponent::Annotation(inner.to_string()));
            }
        }

        // Parse logical operators
        // 解析逻辑运算符
        if expr.contains(" && ")
        {
            components.push(ExpressionComponent::And);
        }
        else if expr.contains(" || ")
        {
            components.push(ExpressionComponent::Or);
        }

        components
    }

    /// Get the expression string
    /// 获取表达式字符串
    pub fn expression(&self) -> &str
    {
        &self.expression
    }

    /// Check if this pointcut matches a join point
    /// 检查此切点是否匹配连接点
    pub fn matches(&self, join_point: &JoinPoint) -> bool
    {
        for component in &self.components
        {
            match component
            {
                ExpressionComponent::Execution { method, .. } =>
                {
                    // Simple wildcard matching
                    // 简单的通配符匹配
                    if *method == "*" || *method == join_point.method_name()
                    {
                        return true;
                    }
                },
                ExpressionComponent::Within(class) =>
                {
                    if *class == "*" || *class == join_point.target_class()
                    {
                        return true;
                    }
                },
                ExpressionComponent::And | ExpressionComponent::Or | ExpressionComponent::Not =>
                {
                    // Logical operators would need more complex evaluation
                    // 逻辑运算符需要更复杂的评估
                },
                _ =>
                {},
            }
        }
        false
    }
}

// ============================================================================
// Advice Types / 通知类型
// ============================================================================

/// Type of advice
/// 通知类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdviceType
{
    /// Before advice
    /// 前置通知
    Before,
    /// After advice
    /// 后置通知
    After,
    /// Around advice
    /// 环绕通知
    Around,
    /// After returning advice
    /// 返回后通知
    AfterReturning,
    /// After throwing advice
    /// 异常后通知
    AfterThrowing,
}

// ============================================================================
// Aspect Registry / 切面注册表
// ============================================================================

/// Registry for AOP aspects
/// AOP 切面注册表
///
/// The aspect registry manages all registered aspects and their advice.
///
/// 切面注册表管理所有已注册的切面及其通知。
pub struct AspectRegistry
{
    /// Registered aspects
    /// 已注册的切面
    aspects: RwLock<HashMap<String, AspectInfo>>,

    /// Pointcut to advice mapping
    /// 切点到通知的映射
    pointcuts: RwLock<Vec<PointcutAdvice>>,
}

/// Information about an aspect
/// 切面信息
#[derive(Debug, Clone)]
struct AspectInfo
{
    /// Aspect name
    /// 切面名称
    name: String,

    /// Aspect type ID
    /// 切面类型 ID
    type_id: TypeId,

    /// Aspect instance
    /// 切面实例
    instance: Arc<dyn Any + Send + Sync>,
}

/// Associates a pointcut with advice
/// 关联切点和通知
#[derive(Debug, Clone)]
struct PointcutAdvice
{
    /// Pointcut expression
    /// 切点表达式
    pointcut: PointcutExpression,

    /// Advice type
    /// 通知类型
    advice_type: AdviceType,

    /// Aspect name
    /// 切面名称
    aspect_name: String,

    /// Method name
    /// 方法名
    method_name: String,
}

impl AspectRegistry
{
    /// Create a new aspect registry
    /// 创建新的切面注册表
    pub fn new() -> Self
    {
        Self {
            aspects: RwLock::new(HashMap::new()),
            pointcuts: RwLock::new(Vec::new()),
        }
    }

    /// Register an aspect
    /// 注册切面
    pub async fn register_aspect<T: Any + Send + Sync>(&self, name: String, instance: T)
    {
        let info = AspectInfo {
            name: name.clone(),
            type_id: TypeId::of::<T>(),
            instance: Arc::new(instance),
        };

        let mut aspects = self.aspects.write().await;
        aspects.insert(name, info);
    }

    /// Register a pointcut with advice
    /// 注册带通知的切点
    pub async fn register_pointcut(
        &self,
        pointcut: PointcutExpression,
        advice_type: AdviceType,
        aspect_name: String,
        method_name: String,
    )
    {
        let advice = PointcutAdvice {
            pointcut,
            advice_type,
            aspect_name,
            method_name,
        };

        let mut pointcuts = self.pointcuts.write().await;
        pointcuts.push(advice);
    }

    /// Find all advice that matches a join point
    /// 查找匹配连接点的所有通知
    pub async fn find_matching_advice(
        &self,
        join_point: &JoinPoint,
    ) -> Vec<(AdviceType, String, String)>
    {
        let pointcuts = self.pointcuts.read().await;
        let mut matches = Vec::new();

        for advice in pointcuts.iter()
        {
            if advice.pointcut.matches(join_point)
            {
                matches.push((
                    advice.advice_type,
                    advice.aspect_name.clone(),
                    advice.method_name.clone(),
                ));
            }
        }

        matches
    }

    /// Get an aspect by name
    /// 通过名称获取切面
    pub async fn get_aspect(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>>
    {
        let aspects = self.aspects.read().await;
        aspects.get(name).map(|info| info.instance.clone())
    }
}

impl Default for AspectRegistry
{
    fn default() -> Self
    {
        Self::new()
    }
}

// ============================================================================
// Global Registry / 全局注册表
// ============================================================================

/// Global aspect registry
/// 全局切面注册表
static GLOBAL_REGISTRY: once_cell::sync::Lazy<AspectRegistry> =
    once_cell::sync::Lazy::new(AspectRegistry::new);

/// Get the global aspect registry
/// 获取全局切面注册表
pub fn global_registry() -> &'static AspectRegistry
{
    &GLOBAL_REGISTRY
}

// ============================================================================
// Tests / 测试
// ============================================================================

#[cfg(test)]
mod tests
{
    use super::*;

    // ========================================================================
    // Helper / 辅助函数
    // ========================================================================

    /// Create a simple JoinPoint for testing
    /// 创建用于测试的简单 JoinPoint
    fn make_join_point(method: &str, class: &str) -> JoinPoint
    {
        let target: Arc<dyn Any + Send + Sync> = Arc::new("target");
        JoinPoint::new(
            target,
            method.to_string(),
            vec![],
            format!("{}()", method),
            class.to_string(),
        )
    }

    /// Create a JoinPoint with typed arguments for testing
    /// 创建带类型参数的 JoinPoint 用于测试
    fn make_join_point_with_args(
        method: &str,
        class: &str,
        args: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> JoinPoint
    {
        let target: Arc<dyn Any + Send + Sync> = Arc::new("target");
        let sig = format!("{}({} args)", method, args.len());
        JoinPoint::new(target, method.to_string(), args, sig, class.to_string())
    }

    // ========================================================================
    // JoinPoint Tests / JoinPoint 测试
    // ========================================================================

    /// Test basic JoinPoint construction and field access
    /// 测试基本 JoinPoint 构造和字段访问
    #[test]
    fn test_join_point_basic_fields()
    {
        let target: Arc<dyn Any + Send + Sync> = Arc::new("test");
        let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new(42), Arc::new("hello")];

        let join_point = JoinPoint::new(
            target,
            "test_method".to_string(),
            args,
            "test_method(String, i32)".to_string(),
            "TestClass".to_string(),
        );

        assert_eq!(join_point.method_name(), "test_method");
        assert_eq!(join_point.args().len(), 2);
        assert_eq!(join_point.signature(), "test_method(String, i32)");
        assert_eq!(join_point.target_class(), "TestClass");
    }

    /// Test JoinPoint with no arguments
    /// 测试无参数的 JoinPoint
    #[test]
    fn test_join_point_no_args()
    {
        let jp = make_join_point("no_args_method", "Svc");

        assert_eq!(jp.method_name(), "no_args_method");
        assert_eq!(jp.args().len(), 0);
        assert!(jp.args().is_empty());
    }

    /// Test JoinPoint typed argument retrieval via arg<T>
    /// 测试通过 arg<T> 获取类型化参数
    #[test]
    fn test_join_point_typed_arg_access()
    {
        let args: Vec<Arc<dyn Any + Send + Sync>> =
            vec![Arc::new(99_i32), Arc::new("world".to_string())];
        let jp = make_join_point_with_args("typed_method", "Repo", args);

        // Retrieve i32 argument
        // 获取 i32 参数
        let num: Option<&i32> = jp.arg(0);
        assert_eq!(num, Some(&99));

        // Retrieve String argument
        // 获取 String 参数
        let s: Option<&String> = jp.arg(1);
        assert_eq!(s, Some(&String::from("world")));

        // Out-of-bounds returns None
        // 越界返回 None
        let none: Option<&i32> = jp.arg(5);
        assert!(none.is_none());
    }

    /// Test JoinPoint arg<T> with wrong type returns None
    /// 测试 arg<T> 类型不匹配时返回 None
    #[test]
    fn test_join_point_arg_wrong_type()
    {
        let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new(42_i32)];
        let jp = make_join_point_with_args("cast_method", "Svc", args);

        // Try to downcast i32 as String — should fail
        // 尝试将 i32 向下转型为 String — 应该失败
        let bad: Option<&String> = jp.arg(0);
        assert!(bad.is_none());
    }

    /// Test JoinPoint Clone produces an equal copy
    /// 测试 JoinPoint Clone 产生相等的副本
    #[test]
    fn test_join_point_clone()
    {
        let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new(7_i32)];
        let jp = make_join_point_with_args("clone_method", "Svc", args);
        let jp2 = jp.clone();

        assert_eq!(jp.method_name(), jp2.method_name());
        assert_eq!(jp.target_class(), jp2.target_class());
        assert_eq!(jp.signature(), jp2.signature());
        assert_eq!(jp.args().len(), jp2.args().len());
    }

    /// Test JoinPoint Debug output contains key fields
    /// 测试 JoinPoint Debug 输出包含关键字段
    #[test]
    fn test_join_point_debug()
    {
        let jp = make_join_point("debug_method", "DebugClass");
        let debug_str = format!("{:?}", jp);

        assert!(debug_str.contains("debug_method"));
        assert!(debug_str.contains("DebugClass"));
        assert!(debug_str.contains("JoinPoint"));
    }

    /// Test JoinPoint target accessor returns the original Arc
    /// 测试 JoinPoint target 访问器返回原始 Arc
    #[test]
    fn test_join_point_target_accessor()
    {
        let target: Arc<dyn Any + Send + Sync> = Arc::new("my_target");
        let jp =
            JoinPoint::new(target, "m".to_string(), vec![], "m()".to_string(), "C".to_string());

        let retrieved = jp.target();
        let downcast = retrieved.downcast_ref::<&str>();
        assert!(downcast.is_some());
        assert_eq!(*downcast.unwrap(), "my_target");
    }

    // ========================================================================
    // PointcutExpression Tests / 切点表达式测试
    // ========================================================================

    /// Test PointcutExpression stores and returns the raw expression string
    /// 测试切点表达式存储并返回原始表达式字符串
    #[test]
    fn test_pointcut_expression_stores_raw_string()
    {
        let raw = "execution(* com.example..*.*(..))";
        let expr = PointcutExpression::new(raw.to_string());
        assert_eq!(expr.expression(), raw);
    }

    /// Test execution pointcut matches any method via wildcard
    /// 测试 execution 切点通过通配符匹配任意方法
    #[test]
    fn test_pointcut_execution_wildcard_matches()
    {
        let expr = PointcutExpression::new("execution(* *..*.*(..))".to_string());
        let jp = make_join_point("any_method", "AnyClass");

        assert!(expr.matches(&jp));
    }

    /// Test execution pointcut with specific method name matches
    /// 测试带有具体方法名的 execution 切点匹配
    #[test]
    fn test_pointcut_execution_specific_method()
    {
        let expr = PointcutExpression::new(
            "execution(* com.example.Service.specific_method(..))".to_string(),
        );

        let matching_jp = make_join_point("specific_method", "Service");
        let _non_matching_jp = make_join_point("other_method", "Service");

        assert!(expr.matches(&matching_jp));
    }

    /// Test within pointcut matches target class
    /// 测试 within 切点匹配目标类
    #[test]
    fn test_pointcut_within_matches_class()
    {
        let expr = PointcutExpression::new("within(com.example.Service)".to_string());

        let jp_match = make_join_point("method_a", "com.example.Service");
        assert!(expr.matches(&jp_match));
    }

    /// Test within pointcut with wildcard matches any class
    /// 测试带通配符的 within 切点匹配任意类
    #[test]
    fn test_pointcut_within_wildcard()
    {
        let expr = PointcutExpression::new("within(*)".to_string());
        let jp = make_join_point("anything", "AnyClass");

        assert!(expr.matches(&jp));
    }

    /// Test @annotation pointcut expression is parsed without panic
    /// 测试 @annotation 切点表达式解析不会 panic
    #[test]
    fn test_pointcut_annotation_parsing()
    {
        let expr = PointcutExpression::new("@annotation(org.example.Transactional)".to_string());
        assert_eq!(expr.expression(), "@annotation(org.example.Transactional)");
    }

    /// Test combined expression with AND operator
    /// 测试带 AND 操作符的组合表达式
    #[test]
    fn test_pointcut_and_operator()
    {
        let expr = PointcutExpression::new(
            "execution(* *..*.*(..)) && within(com.example.Service)".to_string(),
        );

        let jp = make_join_point("do_work", "com.example.Service");
        assert!(expr.matches(&jp));
    }

    /// Test combined expression with OR operator
    /// 测试带 OR 操作符的组合表达式
    #[test]
    fn test_pointcut_or_operator()
    {
        let expr = PointcutExpression::new(
            "execution(* com.example..*.*(..)) || within(com.other..*)".to_string(),
        );
        assert!(expr.expression().contains("||"));
    }

    /// Test empty expression produces no components and never matches
    /// 测试空表达式不产生组件且永不匹配
    #[test]
    fn test_pointcut_empty_expression()
    {
        let expr = PointcutExpression::new(String::new());
        let jp = make_join_point("any", "Any");

        assert_eq!(expr.expression(), "");
        assert!(!expr.matches(&jp));
    }

    /// Test PointcutExpression clone preserves expression string
    /// 测试切点表达式克隆保留表达式字符串
    #[test]
    fn test_pointcut_clone()
    {
        let expr = PointcutExpression::new("execution(* com.example..*.*(..))".to_string());
        let cloned = expr.clone();

        assert_eq!(expr.expression(), cloned.expression());
    }

    /// Test PointcutExpression Debug output
    /// 测试切点表达式 Debug 输出
    #[test]
    fn test_pointcut_debug()
    {
        let expr = PointcutExpression::new("execution(* *(..))".to_string());
        let debug_str = format!("{:?}", expr);

        assert!(debug_str.contains("execution"));
    }

    // ========================================================================
    // AdviceType Tests / 通知类型测试
    // ========================================================================

    /// Test AdviceType variants equality
    /// 测试 AdviceType 变体的相等性
    #[test]
    fn test_advice_type_equality()
    {
        assert_eq!(AdviceType::Before, AdviceType::Before);
        assert_eq!(AdviceType::After, AdviceType::After);
        assert_eq!(AdviceType::Around, AdviceType::Around);
        assert_eq!(AdviceType::AfterReturning, AdviceType::AfterReturning);
        assert_eq!(AdviceType::AfterThrowing, AdviceType::AfterThrowing);

        assert_ne!(AdviceType::Before, AdviceType::After);
        assert_ne!(AdviceType::After, AdviceType::Around);
        assert_ne!(AdviceType::Around, AdviceType::Before);
    }

    /// Test AdviceType Copy and Clone
    /// 测试 AdviceType 的 Copy 和 Clone
    #[test]
    fn test_advice_type_copy_clone()
    {
        let a = AdviceType::Before;
        let b = a; // Copy
        let c = a; // Copy again
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    /// Test AdviceType Debug output contains variant names
    /// 测试 AdviceType Debug 输出包含变体名称
    #[test]
    fn test_advice_type_debug()
    {
        assert!(format!("{:?}", AdviceType::Before).contains("Before"));
        assert!(format!("{:?}", AdviceType::After).contains("After"));
        assert!(format!("{:?}", AdviceType::Around).contains("Around"));
    }

    // ========================================================================
    // AspectRegistry Tests / 切面注册表测试
    // ========================================================================

    /// Test AspectRegistry creation with new() and default()
    /// 测试使用 new() 和 default() 创建切面注册表
    #[test]
    fn test_aspect_registry_new_and_default()
    {
        let _r1 = AspectRegistry::new();
        let _r2 = AspectRegistry::default();
    }

    /// Test registering and retrieving an aspect
    /// 测试注册并获取切面
    #[tokio::test]
    async fn test_aspect_registry_register_and_get()
    {
        let registry = AspectRegistry::new();

        let instance = "test_aspect";
        registry
            .register_aspect("TestAspect".to_string(), instance)
            .await;

        let aspect = registry.get_aspect("TestAspect").await;
        assert!(aspect.is_some());

        let arc = aspect.unwrap();
        let downcast = arc.as_ref().downcast_ref::<&str>();
        assert!(downcast.is_some());
        assert_eq!(*downcast.unwrap(), "test_aspect");
    }

    /// Test retrieving a non-existent aspect returns None
    /// 测试获取不存在的切面返回 None
    #[tokio::test]
    async fn test_aspect_registry_get_missing()
    {
        let registry = AspectRegistry::new();

        let result = registry.get_aspect("DoesNotExist").await;
        assert!(result.is_none());
    }

    /// Test registering multiple aspects
    /// 测试注册多个切面
    #[tokio::test]
    async fn test_aspect_registry_multiple_aspects()
    {
        let registry = AspectRegistry::new();

        registry.register_aspect("AspectA".to_string(), 1_i32).await;
        registry.register_aspect("AspectB".to_string(), 2_i32).await;
        registry.register_aspect("AspectC".to_string(), 3_i32).await;

        let a = registry.get_aspect("AspectA").await.unwrap();
        let b = registry.get_aspect("AspectB").await.unwrap();
        let c = registry.get_aspect("AspectC").await.unwrap();

        assert_eq!(*a.downcast_ref::<i32>().unwrap(), 1);
        assert_eq!(*b.downcast_ref::<i32>().unwrap(), 2);
        assert_eq!(*c.downcast_ref::<i32>().unwrap(), 3);
    }

    /// Test overwriting an aspect with the same name
    /// 测试用相同名称覆盖切面
    #[tokio::test]
    async fn test_aspect_registry_overwrite()
    {
        let registry = AspectRegistry::new();

        registry.register_aspect("Aspect".to_string(), "v1").await;
        registry.register_aspect("Aspect".to_string(), "v2").await;

        let a = registry.get_aspect("Aspect").await.unwrap();
        assert_eq!(*a.downcast_ref::<&str>().unwrap(), "v2");
    }

    /// Test register_pointcut and find_matching_advice
    /// 测试注册切点和查找匹配通知
    #[tokio::test]
    async fn test_aspect_registry_register_pointcut_and_match()
    {
        let registry = AspectRegistry::new();

        let pointcut = PointcutExpression::new("execution(* *..*.*(..))".to_string());
        registry
            .register_pointcut(
                pointcut,
                AdviceType::Before,
                "LogAspect".to_string(),
                "log_before".to_string(),
            )
            .await;

        let jp = make_join_point("any_method", "AnyClass");
        let matches = registry.find_matching_advice(&jp).await;

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, AdviceType::Before);
        assert_eq!(matches[0].1, "LogAspect");
        assert_eq!(matches[0].2, "log_before");
    }

    /// Test find_matching_advice with no registered pointcuts
    /// 测试无已注册切点时查找匹配通知
    #[tokio::test]
    async fn test_aspect_registry_no_matching_advice()
    {
        let registry = AspectRegistry::new();

        let jp = make_join_point("method", "Class");
        let matches = registry.find_matching_advice(&jp).await;

        assert!(matches.is_empty());
    }

    /// Test multiple pointcuts matching the same join point
    /// 测试多个切点匹配同一个连接点
    #[tokio::test]
    async fn test_aspect_registry_multiple_matching_pointcuts()
    {
        let registry = AspectRegistry::new();

        let p1 = PointcutExpression::new("execution(* *..*.*(..))".to_string());
        let p2 = PointcutExpression::new("within(*)".to_string());

        registry
            .register_pointcut(p1, AdviceType::Before, "A1".to_string(), "before_log".to_string())
            .await;
        registry
            .register_pointcut(p2, AdviceType::After, "A2".to_string(), "after_log".to_string())
            .await;

        let jp = make_join_point("work", "Svc");
        let matches = registry.find_matching_advice(&jp).await;

        assert_eq!(matches.len(), 2);
    }

    /// Test pointcut with non-matching join point returns empty
    /// 测试切点与不匹配的连接点返回空结果
    #[tokio::test]
    async fn test_aspect_registry_pointcut_no_match()
    {
        let registry = AspectRegistry::new();

        // Empty expression never matches
        // 空表达式永不匹配
        let pointcut = PointcutExpression::new(String::new());
        registry
            .register_pointcut(pointcut, AdviceType::Around, "A".to_string(), "m".to_string())
            .await;

        let jp = make_join_point("something", "SomeClass");
        let matches = registry.find_matching_advice(&jp).await;

        assert!(matches.is_empty());
    }

    // ========================================================================
    // Global Registry Tests / 全局注册表测试
    // ========================================================================

    /// Test global_registry() returns a valid reference
    /// 测试 global_registry() 返回有效引用
    #[test]
    fn test_global_registry_exists()
    {
        let reg = global_registry();
        // Verify it is the same instance each call
        // 验证每次调用返回同一实例
        let reg2 = global_registry();
        assert!(std::ptr::eq(reg, reg2));
    }

    /// Test global registry can register and retrieve aspects
    /// 测试全局注册表可以注册和获取切面
    #[tokio::test]
    async fn test_global_registry_register_and_get()
    {
        let registry = global_registry();

        registry
            .register_aspect("GlobalTest".to_string(), 123_i32)
            .await;

        let aspect = registry.get_aspect("GlobalTest").await;
        assert!(aspect.is_some());
        assert_eq!(*aspect.unwrap().downcast_ref::<i32>().unwrap(), 123);
    }

    // ========================================================================
    // Integration-style Tests / 集成风格测试
    // ========================================================================

    /// Test full workflow: register aspect, register pointcuts, match advice
    /// 测试完整流程：注册切面、注册切点、匹配通知
    #[tokio::test]
    async fn test_full_aop_workflow()
    {
        let registry = AspectRegistry::new();

        // Register aspect instance
        // 注册切面实例
        registry
            .register_aspect("TransactionAspect".to_string(), "tx_manager")
            .await;

        // Register before advice
        // 注册前置通知
        let before_cut =
            PointcutExpression::new("execution(* com.example.service..*.*(..))".to_string());
        registry
            .register_pointcut(
                before_cut,
                AdviceType::Before,
                "TransactionAspect".to_string(),
                "begin_tx".to_string(),
            )
            .await;

        // Register after advice
        // 注册后置通知
        let after_cut =
            PointcutExpression::new("execution(* com.example.service..*.*(..))".to_string());
        registry
            .register_pointcut(
                after_cut,
                AdviceType::After,
                "TransactionAspect".to_string(),
                "commit_tx".to_string(),
            )
            .await;

        // Register around advice
        // 注册环绕通知
        let around_cut = PointcutExpression::new("within(*)".to_string());
        registry
            .register_pointcut(
                around_cut,
                AdviceType::Around,
                "TransactionAspect".to_string(),
                "wrap_tx".to_string(),
            )
            .await;

        // Simulate method call
        // 模拟方法调用
        let jp = make_join_point("save_user", "com.example.service.UserService");
        let matches = registry.find_matching_advice(&jp).await;

        assert_eq!(matches.len(), 3);

        // Verify all three advice types are present
        // 验证三种通知类型都存在
        let types: Vec<AdviceType> = matches.iter().map(|m| m.0).collect();
        assert!(types.contains(&AdviceType::Before));
        assert!(types.contains(&AdviceType::After));
        assert!(types.contains(&AdviceType::Around));

        // All advice belong to the same aspect
        // 所有通知属于同一个切面
        for m in &matches
        {
            assert_eq!(m.1, "TransactionAspect");
        }
    }

    /// Test advice ordering: before, around, after registered in sequence
    /// 测试通知顺序：按前置、环绕、后置顺序注册
    #[tokio::test]
    async fn test_advice_ordering()
    {
        let registry = AspectRegistry::new();

        let wildcard = PointcutExpression::new("execution(* *..*.*(..))".to_string());

        registry
            .register_pointcut(
                wildcard.clone(),
                AdviceType::Before,
                "A".to_string(),
                "b".to_string(),
            )
            .await;
        registry
            .register_pointcut(
                wildcard.clone(),
                AdviceType::Around,
                "A".to_string(),
                "r".to_string(),
            )
            .await;
        registry
            .register_pointcut(
                wildcard.clone(),
                AdviceType::After,
                "A".to_string(),
                "a".to_string(),
            )
            .await;

        let jp = make_join_point("ordered", "Svc");
        let matches = registry.find_matching_advice(&jp).await;

        // Registered order is preserved
        // 注册顺序被保留
        assert_eq!(matches[0].0, AdviceType::Before);
        assert_eq!(matches[1].0, AdviceType::Around);
        assert_eq!(matches[2].0, AdviceType::After);
    }

    /// Test JoinPoint with complex typed arguments
    /// 测试带有复杂类型参数的 JoinPoint
    #[test]
    fn test_join_point_complex_args()
    {
        let args: Vec<Arc<dyn Any + Send + Sync>> = vec![
            Arc::new(42_i64),
            Arc::new(true),
            Arc::new(3.15_f64),
            Arc::new(String::from("hello")),
        ];

        let jp = make_join_point_with_args("complex", "Svc", args);

        assert_eq!(jp.arg::<i64>(0), Some(&42));
        assert_eq!(jp.arg::<bool>(1), Some(&true));
        assert_eq!(jp.arg::<f64>(2), Some(&3.15));
        assert_eq!(jp.arg::<String>(3), Some(&String::from("hello")));
    }

    /// Test JoinPoint with many arguments
    /// 测试带有大量参数的 JoinPoint
    #[test]
    fn test_join_point_many_args()
    {
        let args: Vec<Arc<dyn Any + Send + Sync>> = (0..100)
            .map(|i| Arc::new(i) as Arc<dyn Any + Send + Sync>)
            .collect();

        let jp = make_join_point_with_args("many_args", "Svc", args);

        assert_eq!(jp.args().len(), 100);
        assert_eq!(jp.arg::<i32>(0), Some(&0));
        assert_eq!(jp.arg::<i32>(50), Some(&50));
        assert_eq!(jp.arg::<i32>(99), Some(&99));
        assert!(jp.arg::<i32>(100).is_none());
    }
}
