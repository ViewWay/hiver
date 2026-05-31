# hiver-resilience

[![Crates.io](https://img.shields.io/crates/v/hiver-resilience)](https://crates.io/crates/hiver-resilience)
[![Documentation](https://docs.rs/hiver-resilience/badge.svg)](https://docs.rs/hiver-resilience)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> High availability patterns for Hiver Framework
> 
> Nexus框架的高可用模式

---

## 📋 Overview / 概述

`hiver-resilience` provides high availability patterns to make your applications resilient to failures, similar to Spring Cloud's resilience patterns.

`hiver-resilience` 提供高可用模式，使您的应用程序能够抵御故障，类似于Spring Cloud的弹性模式。

**Key Features** / **核心特性**:
- ✅ **Circuit Breaker** - Fail fast when service is down
- ✅ **Rate Limiting** - Control request rate
- ✅ **Retry** - Automatic retry with backoff
- ✅ **Timeout** - Request timeout handling
- ✅ **Service Discovery** - Dynamic service discovery

---

## ✨ Resilience Patterns / 弹性模式

| Pattern | Spring Equivalent | Description | Status |
|---------|------------------|-------------|--------|
| **Circuit Breaker** | `@CircuitBreaker`, Resilience4j | Fail fast on errors | 🔄 Phase 4 |
| **Rate Limiter** | `@RateLimiter`, Resilience4j | Limit request rate | 🔄 Phase 4 |
| **Retry** | `@Retry`, Resilience4j | Retry failed requests | 🔄 Phase 4 |
| **Timeout** | `@Timeout`, Resilience4j | Request timeout | 🔄 Phase 4 |
| **Service Discovery** | Eureka, Consul | Service registry | 🔄 Phase 4 |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-resilience = "0.1.0-alpha"
```

### Basic Usage / 基本用法

```rust
use hiver_resilience::{CircuitBreaker, RateLimiter, RetryPolicy};

// Circuit breaker / 熔断器
let circuit = CircuitBreaker::new(CircuitBreakerConfig {
    error_threshold: 0.5,  // Open after 50% errors
    timeout: Duration::from_secs(60),
});

// Rate limiter / 限流器
let limiter = RateLimiter::new(100);  // 100 requests per second

// Retry policy / 重试策略
let retry = RetryPolicy::exponential_backoff(3, Duration::from_secs(1));
```

---

## 📖 Pattern Details / 模式详情

### Circuit Breaker / 熔断器

Protect against cascading failures:

防止级联故障：

```rust
use hiver_resilience::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

// Create circuit breaker / 创建熔断器
let config = CircuitBreakerConfig::builder()
    .error_threshold(0.5)              // Open after 50% errors
    .min_requests(10)                   // Need 10 requests before opening
    .timeout(Duration::from_secs(60))   // Stay open for 60s
    .half_open_max_calls(3)             // Test with 3 calls in half-open
    .build();

let circuit = CircuitBreaker::new(config);

// Use with async operation / 与异步操作一起使用
async fn call_external_api() -> Result<String, Error> {
    circuit.call(|| async {
        // External API call / 外部API调用
        http_client.get("https://api.example.com/data").await
    }).await
}
```

**Circuit States** / **熔断器状态**:

```
┌─────────────────────────────────────────────────────────────┐
│              Circuit Breaker State Machine                  │
│              熔断器状态机                                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  CLOSED (正常)                                               │
│  ┌────────┐                                                 │
│  │ Requests pass through / 请求通过                        │
│  │ Monitor errors / 监控错误                                │
│  └────┬───┘                                                 │
│       │ Error rate > threshold / 错误率 > 阈值              │
│       ▼                                                      │
│  OPEN (打开)                                                │
│  ┌────────┐                                                 │
│  │ Fail fast / 快速失败                                     │
│  │ No requests pass / 无请求通过                           │
│  └────┬───┘                                                 │
│       │ After timeout / 超时后                              │
│       ▼                                                      │
│  HALF-OPEN (半开)                                           │
│  ┌────────┐                                                 │
│  │ Test recovery / 测试恢复                                 │
│  │ Allow limited requests / 允许有限请求                    │
│  └────┬───┘                                                 │
│       │ Success / 成功                                      │
│       └──────► CLOSED                                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Configuration** / **配置**:

```rust
let config = CircuitBreakerConfig::builder()
    // Error threshold / 错误阈值
    .error_threshold(0.5)              // 50% error rate triggers open
    
    // Minimum requests / 最小请求数
    .min_requests(10)                  // Need 10 requests before evaluating
    
    // Timeout / 超时
    .timeout(Duration::from_secs(60))  // Stay open for 60 seconds
    
    // Half-open settings / 半开设置
    .half_open_max_calls(3)           // Test with 3 calls
    .half_open_success_threshold(2)    // Need 2 successes to close
    
    // Sliding window / 滑动窗口
    .sliding_window_size(100)          // Last 100 requests
    .sliding_window_min_calls(10)      // Minimum calls for evaluation
    
    .build();
```

---

### Rate Limiter / 限流器

Control request rate:

控制请求速率：

```rust
use hiver_resilience::{RateLimiter, RateLimiterType};

// Token bucket rate limiter / Token bucket限流器
let limiter = RateLimiter::builder()
    .rate(100)                         // 100 requests per second
    .capacity(200)                     // Burst up to 200
    .limiter_type(RateLimiterType::TokenBucket)
    .build();

// Use with async operation / 与异步操作一起使用
async fn process_request() -> Result<Response, Error> {
    limiter.acquire().await?;  // Wait for permit / 等待许可
    
    // Process request / 处理请求
    handle_request().await
}
```

**Rate Limiter Types** / **限流器类型**:

| Type | Description | Use Case |
|------|-------------|----------|
| **TokenBucket** | Fixed rate with burst capacity | API rate limiting |
| **LeakyBucket** | Smooth rate limiting | Traffic shaping |
| **SlidingWindow** | Time-based window | Request throttling |

**Token Bucket** / **Token Bucket**:
```
┌─────────────────────────────────────────────────────────────┐
│                    Token Bucket                              │
│                    Token桶                                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Bucket Capacity: 200 tokens / 桶容量：200 tokens           │
│  Refill Rate: 100 tokens/second / 补充速率：100 tokens/秒   │
│                                                              │
│  ┌──────────────────────────────────────────────┐           │
│  │ ████████████████████░░░░░░░░░░░░░░░░░░░░░░  │           │
│  │ 120 tokens available / 120 tokens可用       │           │
│  └──────────────────────────────────────────────┘           │
│                                                              │
│  Request arrives → Consume 1 token → Process              │
│  请求到达 → 消耗1个token → 处理                              │
│                                                              │
│  If bucket empty → Wait for refill / 如果桶空 → 等待补充    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Per-Client Rate Limiting** / **每客户端限流**:

```rust
use std::collections::HashMap;
use std::sync::Arc;

struct PerClientLimiter {
    limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
}

impl PerClientLimiter {
    async fn acquire(&self, client_id: &str) -> Result<(), Error> {
        let limiter = self.get_or_create(client_id).await;
        limiter.acquire().await
    }
}
```

---

### Retry Policy / 重试策略

Automatic retry with backoff:

带退避的自动重试：

```rust
use hiver_resilience::{RetryPolicy, retry};
use std::time::Duration;

// Exponential backoff / 指数退避
let policy = RetryPolicy::exponential_backoff(
    3,                              // Max 3 retries
    Duration::from_secs(1),          // Initial delay 1s
);

// Use with async operation / 与异步操作一起使用
async fn call_api() -> Result<String, Error> {
    retry(policy, || async {
        http_client.get("https://api.example.com").await
    }).await
}

// Custom retry policy / 自定义重试策略
let policy = RetryPolicy::builder()
    .max_attempts(5)
    .initial_delay(Duration::from_millis(100))
    .max_delay(Duration::from_secs(10))
    .multiplier(2.0)                // Exponential backoff
    .jitter(true)                   // Add randomness
    .retry_on(|error| {
        // Retry on network errors / 网络错误时重试
        matches!(error, Error::Network(_))
    })
    .build();
```

**Retry Strategies** / **重试策略**:

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Fixed** | Constant delay | Simple retries |
| **Exponential** | Exponential backoff | Network calls |
| **Linear** | Linear increase | Rate-limited APIs |
| **Custom** | User-defined | Special cases |

**Exponential Backoff** / **指数退避**:
```
Attempt 1: Wait 1s
Attempt 2: Wait 2s
Attempt 3: Wait 4s
Attempt 4: Wait 8s
...
Max delay: 10s
```

---

### Timeout / 超时

Enforce request timeouts:

强制执行请求超时：

```rust
use hiver_resilience::Timeout;
use std::time::Duration;

// Create timeout / 创建超时
let timeout = Timeout::new(Duration::from_secs(5));

// Use with async operation / 与异步操作一起使用
async fn call_slow_api() -> Result<String, Error> {
    timeout.timeout(|| async {
        slow_api_call().await
    }).await
}
```

---

### Service Discovery / 服务发现

Dynamic service discovery:

动态服务发现：

```rust
use hiver_resilience::discovery::{ServiceRegistry, ServiceInstance};

// Register service / 注册服务
let registry = ServiceRegistry::new();
registry.register(ServiceInstance {
    service_id: "user-service".to_string(),
    host: "127.0.0.1".to_string(),
    port: 8080,
    metadata: HashMap::new(),
}).await?;

// Discover service / 发现服务
let instances = registry.discover("user-service").await?;
let instance = instances.first().unwrap();

// Use instance / 使用实例
let url = format!("http://{}:{}", instance.host, instance.port);
```

---

## 🎯 Combining Patterns / 组合模式

Combine multiple resilience patterns:

组合多个弹性模式：

```rust
use hiver_resilience::{CircuitBreaker, RateLimiter, RetryPolicy};

// Create resilience wrapper / 创建弹性包装器
struct ResilientClient {
    circuit: CircuitBreaker,
    limiter: RateLimiter,
    retry: RetryPolicy,
}

impl ResilientClient {
    async fn call(&self, request: Request) -> Result<Response, Error> {
        // 1. Rate limit / 限流
        self.limiter.acquire().await?;
        
        // 2. Circuit breaker / 熔断器
        let result = self.circuit.call(|| async {
            // 3. Retry / 重试
            retry(self.retry.clone(), || async {
                http_client.send(request.clone()).await
            }).await
        }).await;
        
        result
    }
}
```

**Execution Order** / **执行顺序**:
1. **Rate Limiter** - Check rate limit / 检查速率限制
2. **Circuit Breaker** - Check circuit state / 检查熔断器状态
3. **Retry** - Retry on failure / 失败时重试
4. **Timeout** - Enforce timeout / 强制执行超时

---

## ⚡ Performance / 性能

### Overhead / 开销

| Pattern | Overhead | Notes |
|---------|----------|-------|
| **Circuit Breaker** | < 1µs | State check only |
| **Rate Limiter** | 1-10µs | Token bucket update |
| **Retry** | Variable | Depends on retry count |
| **Timeout** | < 1µs | Timer check |

### Best Practices / 最佳实践

```rust
// ✅ Good: Use circuit breaker for external calls / 好：对外部调用使用熔断器
let circuit = CircuitBreaker::new(config);
circuit.call(|| external_api_call()).await

// ✅ Good: Rate limit per client / 好：每客户端限流
let limiter = PerClientLimiter::new(100);  // 100 req/s per client

// ✅ Good: Retry with exponential backoff / 好：指数退避重试
let retry = RetryPolicy::exponential_backoff(3, Duration::from_secs(1));

// ❌ Avoid: Too many retries / 避免：重试次数过多
let retry = RetryPolicy::exponential_backoff(10, Duration::from_secs(1));  // Too many!
```

---

## 🧪 Testing / 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker() {
        let circuit = CircuitBreaker::new(CircuitBreakerConfig::default());
        
        // Simulate failures / 模拟失败
        for _ in 0..10 {
            let _ = circuit.call(|| async {
                Err(Error::internal("Service down"))
            }).await;
        }
        
        // Circuit should be open / 熔断器应该打开
        assert_eq!(circuit.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(10);  // 10 req/s
        
        // First 10 should succeed / 前10个应该成功
        for _ in 0..10 {
            assert!(limiter.acquire().await.is_ok());
        }
        
        // 11th should wait / 第11个应该等待
        let start = Instant::now();
        limiter.acquire().await.unwrap();
        assert!(start.elapsed() >= Duration::from_millis(100));
    }
}
```

---

## 🚦 Roadmap / 路线图

### Phase 4: Core Patterns 🔄 (In Progress / 进行中)
- [ ] Circuit breaker implementation
- [ ] Rate limiter implementation
- [ ] Retry policy implementation
- [ ] Timeout implementation
- [ ] Service discovery

### Phase 5: Advanced Features 📋 (Planned / 计划中)
- [ ] Bulkhead pattern
- [ ] Fallback handlers
- [ ] Metrics integration
- [ ] Distributed rate limiting

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-resilience](https://docs.rs/hiver-resilience)
- **Book**: [Resilience Guide](../../docs/book/src/advanced/resilience.md)
- **Examples**: [examples/](../../examples/)

---

## 🤝 Contributing / 贡献

We welcome contributions! Please see:

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Design Spec](../../docs/design-spec.md)
- [GitHub Issues](https://github.com/hiver-framework/nexus/issues)

---

## 📄 License / 许可证

Licensed under Apache License 2.0. See [LICENSE](../../LICENSE) for details.

---

## 🙏 Acknowledgments / 致谢

Nexus Resilience is inspired by:

- **[Resilience4j](https://github.com/resilience4j/resilience4j)** - Java resilience patterns
- **[Spring Cloud Circuit Breaker](https://spring.io/projects/spring-cloud-circuitbreaker)** - Spring resilience
- **[Hystrix](https://github.com/Netflix/Hystrix)** - Original circuit breaker pattern

---

**Built with ❤️ for high availability**

**为高可用性构建 ❤️**
