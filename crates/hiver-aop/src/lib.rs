//! # Hiver AOP
//!
//! Spring AOP style aspects for Hiver framework.
//! Hiver框架的Spring AOP风格切面支持。
//!
//! ## Macros (re-exported from `hiver-aop-macros`)
//!
//! - **`#[Aspect]`** - Marks a struct as an aspect
//! - **`@Before`** - Before advice
//! - **`@After`** - After advice
//! - **`@Around`** - Around advice
//! - **`@AfterReturning`** - After-returning advice
//! - **`@AfterThrowing`** - After-throwing advice
//! - **`@Pointcut`** - Pointcut definition
//!
//! ## Runtime types
//!
//! - `JoinPoint` - Method execution join point
//! - `ProceedingJoinPoint` - Proceedable join point for around advice
//! - `AspectRegistry` - Aspect registration and management
//! - `AdviceChain` - Advice execution chain
//!
//! ## Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_aop::{Aspect, Before, After, Around};
//!
//! #[Aspect]
//! struct LoggingAspect;
//!
//! impl LoggingAspect {
//!     #[Before("execution(* com.example..*.*(..))")]
//!     fn log_before(&self, join_point: &JoinPoint) {
//!         println!("Entering: {}", join_point.method_name());
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

// Re-export procedural macros from hiver-aop-macros
pub use hiver_aop_macros::{
    after, after_returning, after_throwing, around, aspect, before, pointcut,
};

pub mod runtime;

#[cfg(test)]
mod tests;

// Re-export runtime types at crate root for convenience
pub use runtime::{
    AdviceChain, AdviceType, AspectRegistry, JoinPoint, PointcutExpression, ProceedingJoinPoint,
    global_registry,
};
