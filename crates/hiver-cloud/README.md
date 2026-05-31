# hiver-cloud

[![Crates.io](https://img.shields.io/crates/v/hiver-cloud)](https://crates.io/crates/hiver-cloud)
[![Documentation](https://docs.rs/hiver-cloud/badge.svg)](https://docs.rs/hiver-cloud)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Spring Cloud equivalent features for Hiver Framework
> 
> Nexus框架的Spring Cloud等价功能

---

## 📋 Overview / 概述

`hiver-cloud` provides cloud-native features including service discovery, configuration management, API gateway, and load balancing, similar to Spring Cloud.

`hiver-cloud` 提供云原生功能，包括服务发现、配置管理、API网关和负载均衡，类似于Spring Cloud。

**Key Features** / **核心特性**:
- ✅ **Service Discovery** - Eureka, Consul, etcd
- ✅ **Config Server** - Distributed configuration
- ✅ **API Gateway** - Request routing and filtering
- ✅ **Load Balancer** - Client-side load balancing
- ✅ **Circuit Breaker** - Resilience patterns

---

## ✨ Modules / 模块

| Module | Spring Cloud Equivalent | Description | Status |
|--------|------------------------|-------------|--------|
| **discovery** | `@EnableDiscoveryClient` | Service discovery | 🔄 Phase 4 |
| **config** | `@EnableConfigServer` | Config server | 🔄 Phase 4 |
| **gateway** | `@EnableGateway` | API gateway | 🔄 Phase 4 |
| **load_balancer** | `LoadBalancerClient` | Load balancing | 🔄 Phase 4 |
| **circuit_breaker** | `@EnableCircuitBreaker` | Circuit breaker | 🔄 Phase 4 |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-cloud = "0.1.0-alpha"
```

### Service Discovery / 服务发现

```rust
use hiver_cloud::discovery::{ServiceRegistry, ServiceInstance};

// Register service / 注册服务
let registry = ServiceRegistry::new("http://eureka:8761")?;
registry.register(ServiceInstance {
    service_id: "user-service".to_string(),
    host: "127.0.0.1".to_string(),
    port: 8080,
}).await?;

// Discover service / 发现服务
let instances = registry.discover("user-service").await?;
let instance = instances.first().unwrap();
```

### API Gateway / API网关

```rust
use hiver_cloud::gateway::{Gateway, GatewayRoute};

let gateway = Gateway::builder()
    .route(GatewayRoute::new("/api/users/**")
        .uri("http://user-service")
        .filter(AddRequestHeader::new("X-Gateway", "nexus")))
    .route(GatewayRoute::new("/api/orders/**")
        .uri("http://order-service"))
    .build();

gateway.start().await?;
```

---

## 🚦 Roadmap / 路线图

### Phase 4: Core Cloud Features 🔄 (In Progress / 进行中)
- [ ] Service discovery
- [ ] Config server
- [ ] API gateway
- [ ] Load balancer

### Phase 5: Advanced Features 📋 (Planned / 计划中)
- [ ] Distributed tracing
- [ ] Service mesh integration
- [ ] Cloud-native patterns

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-cloud](https://docs.rs/hiver-cloud)

---

**Built with ❤️ for cloud-native applications**

**为云原生应用程序构建 ❤️**
