#![allow(dead_code)]

//! Nexus Resilience Example / Nexus弹性示例
//!
//! Demonstrates high availability patterns for backend applications.
//! 演示后端应用的高可用模式。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - `CircuitBreaker` → Resilience4j `CircuitBreaker`
//! - `RateLimiter` → Resilience4j `RateLimiter`
//! - `RetryPolicy` → Resilience4j Retry
//! - `ServiceDiscovery` → Spring Cloud Service Discovery

use hiver_resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
    RetryPolicy,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nexus Resilience Example / Nexus弹性示例 ===\n");

    // 1. Circuit Breaker / 熔断器
    println!("1. Circuit Breaker / 熔断器 (类似 Resilience4j)");
    println!("---");
    circuit_breaker_example().await;
    println!();

    // 2. Retry Policy / 重试策略
    println!("2. Retry Policy / 重试策略");
    println!("---");
    retry_policy_example();
    println!();

    // 3. Combined Patterns / 组合模式
    println!("3. Combined Patterns / 组合模式");
    println!("---");
    combined_patterns_example();
    println!();

    println!("=== Example Complete / 示例完成 ===");
    Ok(())
}

/// Circuit breaker example / 熔断器示例
///
/// Demonstrates circuit breaker pattern for fault tolerance.
/// 演示用于容错的熔断器模式。
///
/// Equivalent to Resilience4j's `CircuitBreaker`.
/// 等价于 Resilience4j 的熔断器。
async fn circuit_breaker_example() {
    println!("  Creating circuit breaker:");
    println!("    Failure threshold: 5 failures");
    println!("    Success threshold: 3 successes");
    println!("    Timeout: 30 seconds");
    println!("    Half-open timeout: 10 seconds");

    let config = CircuitBreakerConfig::new();

    let breaker = Arc::new(CircuitBreaker::new("user-service", config));

    println!();
    println!("  Circuit states:");
    println!("    CLOSED    -> Requests pass through (normal operation)");
    println!("    OPEN      -> Requests fail immediately (circuit tripped)");
    println!("    HALF_OPEN -> Testing if service has recovered");
    println!();
    println!("  Current state: {:?}", breaker.state());
    println!();
    println!("  Spring equivalent:");
    println!("    @CircuitBreaker(name = \"userService\", ");
    println!("        fallbackMethod = \"getUserFallback\")");

    // Simulate some calls / 模拟一些调用
    println!();
    println!("  Simulating calls:");
    for i in 1..=5 {
        if breaker.is_request_permitted() {
            println!("    Call {}: Allowed - State: {:?}", i, breaker.state());
        } else {
            println!("    Call {}: Blocked - Circuit is OPEN", i);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!();
    println!("  Circuit breakers protect services from cascading failures.");
    println!("  熔断器保护服务免受级联故障。");
}

/// Retry policy example / 重试策略示例
///
/// Demonstrates retry logic with backoff strategies.
/// 演示带退避策略的重试逻辑。
fn retry_policy_example() {
    println!("  Backoff strategies:");
    println!();

    // Fixed delay / 固定延迟
    println!("  1. Fixed Delay (固定延迟):");
    let _fixed_retry = RetryPolicy::new();
    println!("     -> Retry after fixed delay, up to max attempts");

    // Exponential backoff / 指数退避
    println!();
    println!("  2. Exponential Backoff (指数退避):");
    println!("     -> 100ms, 200ms, 400ms, 800ms, 5s (max)");
    println!("     -> Each retry waits longer than the last");

    // Linear backoff / 线性退避
    println!();
    println!("  3. Linear Backoff (线性退避):");
    println!("     -> 100ms, 150ms, 200ms, 2s (max)");
    println!("     -> Delay increases by fixed amount each time");

    println!();
    println!("  Spring equivalent:");
    println!("    @Retryable(retryFor = {{ SQLException.class }}, ");
    println!("        backoff = @Backoff(delay = 100, multiplier = 2))");
}

/// Combined patterns example / 组合模式示例
///
/// Demonstrates using multiple resilience patterns together.
/// 演示组合使用多种弹性模式。
fn combined_patterns_example() {
    println!("  Combining patterns for production-grade resilience:");
    println!();
    println!("  API call with:");
    println!("    1. Circuit Breaker - Fails fast when service is down");
    println!("    2. Retry - Handles transient failures");
    println!("    3. Timeout - Prevents hanging requests");
    println!();
    println!("  Spring Boot + Resilience4j equivalent:");
    println!("    @CircuitBreaker");
    println!("    @Retry");
    println!("    @TimeLimiter");
    println!("    public User getUser(String id) {{ ... }}");
}

// ============================================================================
// Backend API Examples / 后端API示例
// ============================================================================

/// Example: Protected API endpoint with resilience
/// 示例：带弹性保护的API端点
///
/// Demonstrates how to protect an external API call.
/// 演示如何保护外部API调用。
struct UserService {
    circuit_breaker: Arc<CircuitBreaker>,
}

impl UserService {
    fn new() -> Self {
        let cb_config = CircuitBreakerConfig::new();

        Self {
            circuit_breaker: Arc::new(CircuitBreaker::new("user-api", cb_config)),
        }
    }

    /// Get user with resilience protection / 获取用户（带弹性保护）
    ///
    /// This demonstrates a production-ready service call with:
    /// - Circuit breaking to fail fast when service is down
    ///
    /// Spring equivalent:
    /// ```java
    /// @CircuitBreaker(name = "userService", fallbackMethod = "getUserFallback")
    /// public User getUser(String id) {
    ///     return restTemplate.getForObject("/users/" + id, User.class);
    /// }
    ///
    /// public User getUserFallback(String id, Exception e) {
    ///     return new User(id, "Unknown", "unknown@example.com");
    /// }
    /// ```
    fn get_user_protected(&self, _id: &str) -> Result<String, String> {
        // Check circuit breaker / 检查熔断器
        let state = self.circuit_breaker.state();
        if state == CircuitState::Open {
            return Err("Circuit breaker is OPEN - service unavailable".to_string());
        }

        // In production, would make actual API call here
        // 在生产环境中，这里会进行实际的API调用
        Ok(r#"{"id": "1", "name": "Alice", "email": "alice@example.com"}"#.to_string())
    }
}
