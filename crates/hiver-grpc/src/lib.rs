//! Nexus gRPC — gRPC server and client support for the Nexus framework.
//! Nexus gRPC — Nexus框架的gRPC服务端和客户端支持。
//!
//! # Description / 描述
//!
//! Provides tonic-based gRPC server and client infrastructure similar to
//! Spring Cloud gRPC, including interceptors, metadata propagation,
//! and service registration helpers.
//!
//! 基于 tonic 提供类似 Spring Cloud gRPC 的服务端/客户端基础设施，
//! 包括拦截器、元数据传播和服务注册助手。
//!
//! # Example / 示例
//! ```rust,ignore
//! use hiver_grpc::server::GrpcServer;
//!
//! let server = GrpcServer::builder()
//!     .port(50051)
//!     .add_service(MyServiceServer::new(MyServiceImpl))
//!     .build();
//! server.serve().await?;
//! ```

#![warn(missing_docs)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

pub mod client;
pub mod error;
pub mod health;
pub mod interceptor;
pub mod metadata;
pub mod retry;
pub mod server;
pub mod tls;

pub use error::GrpcError;
pub use retry::RetryPolicy;
