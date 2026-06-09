//! Testcontainers integration — Docker-managed containers for integration tests.
//! Testcontainers 集成 — 用于集成测试的 Docker 管理容器。
//!
//! # Description / 描述
//!
//! Provides type-safe wrappers around `testcontainers` and `testcontainers-modules`
//! so Hiver integration tests can start real Postgres, Redis, and Kafka containers
//! with a Spring Boot-like fluent API.
//!
//! 围绕 testcontainers 和 testcontainers-modules 提供类型安全的包装器，
//! 使 Hiver 集成测试可以使用类似 Spring Boot 的流畅 API 启动真实的
//! Postgres、Redis 和 Kafka 容器。
//!
//! # Example / 示例
//! ```rust,ignore
//! use hiver_test::containers::PostgresContainer;
//!
//! #[tokio::test]
//! async fn test_with_postgres() {
//!     let pg = PostgresContainer::start().await;
//!     let url = pg.connection_url();
//!     // use url to configure your DatabaseClient
//! }
//! ```

#![allow(dead_code)]
use testcontainers::ContainerAsync;
use testcontainers_modules::{kafka::Kafka, postgres::Postgres, redis::Redis};
use tracing::info;

// ─────────────────────────────────────────────────────────────────────────────
// PostgreSQL container
// ─────────────────────────────────────────────────────────────────────────────

/// A running PostgreSQL test container.
/// 正在运行的 PostgreSQL 测试容器。
///
/// Equivalent to Spring's `@Testcontainers` + `@Container` with `PostgreSQLContainer`.
/// 等价于 Spring 的 @Testcontainers + @Container 与 PostgreSQLContainer。
///
/// The container is automatically stopped when this struct is dropped.
/// 此结构体被丢弃时，容器会自动停止。
pub struct PostgresContainer {
    container: ContainerAsync<Postgres>,
    host: String,
    port: u16,
}

impl PostgresContainer {
    /// Start a PostgreSQL container with the default image and credentials.
    /// 使用默认镜像和凭据启动 PostgreSQL 容器。
    ///
    /// Default: `postgres:16-alpine`, user=`postgres`, pass=`postgres`, db=`postgres`
    /// 默认值：postgres:16-alpine，user=postgres，pass=postgres，db=postgres
    pub async fn start() -> Self {
        Self::with_options("postgres", "postgres", "postgres").await
    }

    /// Start with explicit credentials.
    /// 使用显式凭据启动。
    pub async fn with_options(user: &str, password: &str, db: &str) -> Self {
        let image = Postgres::default()
            .with_user(user)
            .with_password(password)
            .with_db_name(db);
        let container = testcontainers::runners::AsyncRunner::start(image)
            .await
            .expect("PostgreSQL container failed to start");
        let host = container.get_host().await.expect("get host").to_string();
        let port = container.get_host_port_ipv4(5432).await.expect("get port");
        info!("PostgreSQL container started at {}:{}", host, port);
        Self {
            container,
            host,
            port,
        }
    }

    /// JDBC-style connection URL (compatible with most Rust crates).
    /// JDBC 风格的连接 URL（兼容大多数 Rust crate）。
    ///
    /// Format: `postgresql://postgres:postgres@host:port/postgres`
    pub fn connection_url(&self) -> String {
        format!("postgresql://postgres:postgres@{}:{}/postgres", self.host, self.port)
    }

    /// Host the container is reachable on.
    /// 可访问容器的主机。
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Mapped host port for PostgreSQL (default container port: 5432).
    /// PostgreSQL 的映射主机端口（默认容器端口：5432）。
    pub fn port(&self) -> u16 {
        self.port
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Redis container
// ─────────────────────────────────────────────────────────────────────────────

/// A running Redis test container.
/// 正在运行的 Redis 测试容器。
///
/// Equivalent to Spring's `RedisContainer`.
/// 等价于 Spring 的 RedisContainer。
pub struct RedisContainer {
    container: ContainerAsync<Redis>,
    host: String,
    port: u16,
}

impl RedisContainer {
    /// Start a Redis container.
    /// 启动 Redis 容器。
    pub async fn start() -> Self {
        let container = testcontainers::runners::AsyncRunner::start(Redis::default())
            .await
            .expect("Redis container failed to start");
        let host = container.get_host().await.expect("get host").to_string();
        let port = container.get_host_port_ipv4(6379).await.expect("get port");
        info!("Redis container started at {}:{}", host, port);
        Self {
            container,
            host,
            port,
        }
    }

    /// Redis connection URL.
    /// Redis 连接 URL。
    ///
    /// Format: `redis://host:port`
    pub fn connection_url(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }

    /// Host the container is reachable on.
    /// 可访问容器的主机。
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Mapped host port for Redis (default container port: 6379).
    /// Redis 的映射主机端口（默认容器端口：6379）。
    pub fn port(&self) -> u16 {
        self.port
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Kafka container
// ─────────────────────────────────────────────────────────────────────────────

/// A running Kafka test container.
/// 正在运行的 Kafka 测试容器。
///
/// Equivalent to Spring's `KafkaContainer`.
/// 等价于 Spring 的 KafkaContainer。
pub struct KafkaContainer {
    container: ContainerAsync<Kafka>,
    host: String,
    port: u16,
}

impl KafkaContainer {
    /// Start a Kafka container.
    /// 启动 Kafka 容器。
    pub async fn start() -> Self {
        let container = testcontainers::runners::AsyncRunner::start(Kafka::default())
            .await
            .expect("Kafka container failed to start");
        let host = container.get_host().await.expect("get host").to_string();
        let port = container.get_host_port_ipv4(9093).await.expect("get port");
        info!("Kafka container started at {}:{}", host, port);
        Self {
            container,
            host,
            port,
        }
    }

    /// Kafka bootstrap server address.
    /// Kafka bootstrap 服务器地址。
    pub fn bootstrap_servers(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Host the container is reachable on.
    /// 可访问容器的主机。
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Mapped host port for Kafka.
    /// Kafka 的映射主机端口。
    pub fn port(&self) -> u16 {
        self.port
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ContainerSet — start multiple containers together
// ─────────────────────────────────────────────────────────────────────────────

/// Helper that starts multiple containers and exposes their connection info.
/// 启动多个容器并暴露其连接信息的助手。
///
/// # Example / 示例
/// ```rust,ignore
/// let set = ContainerSet::builder()
///     .postgres()
///     .redis()
///     .build()
///     .await;
/// let pg_url = set.postgres_url().unwrap();
/// let redis_url = set.redis_url().unwrap();
/// ```
#[derive(Default)]
pub struct ContainerSetBuilder {
    need_postgres: bool,
    need_redis: bool,
    need_kafka: bool,
}

impl ContainerSetBuilder {
    /// Create a new builder.
    /// 创建新构建器。
    pub fn new() -> Self {
        Self::default()
    }

    /// Include a PostgreSQL container.
    /// 包含 PostgreSQL 容器。
    pub fn postgres(mut self) -> Self {
        self.need_postgres = true;
        self
    }

    /// Include a Redis container.
    /// 包含 Redis 容器。
    pub fn redis(mut self) -> Self {
        self.need_redis = true;
        self
    }

    /// Include a Kafka container.
    /// 包含 Kafka 容器。
    pub fn kafka(mut self) -> Self {
        self.need_kafka = true;
        self
    }

    /// Start all requested containers concurrently.
    /// 并发启动所有请求的容器。
    pub async fn build(self) -> ContainerSet {
        let (pg, redis, kafka) = tokio::join!(
            async {
                if self.need_postgres {
                    Some(PostgresContainer::start().await)
                } else {
                    None
                }
            },
            async {
                if self.need_redis {
                    Some(RedisContainer::start().await)
                } else {
                    None
                }
            },
            async {
                if self.need_kafka {
                    Some(KafkaContainer::start().await)
                } else {
                    None
                }
            },
        );
        ContainerSet {
            postgres: pg,
            redis,
            kafka,
        }
    }
}

/// A running set of test containers.
/// 一组正在运行的测试容器。
pub struct ContainerSet {
    /// Optional PostgreSQL container.
    pub postgres: Option<PostgresContainer>,
    /// Optional Redis container.
    pub redis: Option<RedisContainer>,
    /// Optional Kafka container.
    pub kafka: Option<KafkaContainer>,
}

impl ContainerSet {
    /// Create a builder.
    /// 创建构建器。
    pub fn builder() -> ContainerSetBuilder {
        ContainerSetBuilder::new()
    }

    /// PostgreSQL connection URL if a container was started.
    /// 若容器已启动，返回 PostgreSQL 连接 URL。
    pub fn postgres_url(&self) -> Option<String> {
        self.postgres
            .as_ref()
            .map(PostgresContainer::connection_url)
    }

    /// Redis connection URL if a container was started.
    /// 若容器已启动，返回 Redis 连接 URL。
    pub fn redis_url(&self) -> Option<String> {
        self.redis.as_ref().map(RedisContainer::connection_url)
    }

    /// Kafka bootstrap servers if a container was started.
    /// 若容器已启动，返回 Kafka bootstrap 服务器地址。
    pub fn kafka_bootstrap(&self) -> Option<String> {
        self.kafka.as_ref().map(KafkaContainer::bootstrap_servers)
    }
}
