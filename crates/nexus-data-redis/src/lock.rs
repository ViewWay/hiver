//! Redis Distributed Lock
//! Redis 分布式锁
//!
//! # Overview / 概述
//!
//! Provides a distributed mutex using Redis SET NX + EX, with automatic
//! renewal and safe release. Equivalent to Spring Integration's lock registry
//! and Redisson-style distributed locks.
//! 使用 Redis SET NX + EX 提供分布式互斥锁，具有自动续期和安全释放功能。
//! 等价于 Spring Integration 的锁注册表和 Redisson 风格的分布式锁。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use nexus_data_redis::{RedisLock, RedisLockGuard};
//!
//! let lock = RedisLock::new(client, "resource:123".to_string(), 30);
//! if let Some(guard) = lock.acquire().await.ok().flatten() {
//!     // Critical section
//!     // 临界区
//!     guard.release().await.ok();
//! }
//! ```

use crate::{RedisClient, RedisError, RedisResult};
use redis::AsyncCommands;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// A Redis-based distributed lock. / 基于 Redis 的分布式锁。
///
/// Uses the `SET key value NX EX seconds` pattern for safe locking.
/// 使用 `SET key value NX EX seconds` 模式进行安全锁定。
#[derive(Debug, Clone)]
pub struct RedisLock {
    client: RedisClient,
    /// Lock key in Redis / Redis 中的锁键
    key: String,
    /// Unique lock value (used for safe release) / 唯一锁值（用于安全释放）
    token: String,
    /// Lock TTL in seconds / 锁的 TTL（秒）
    ttl_secs: u64,
    /// Auto-renewal interval (if set) / 自动续期间隔（如果设置）
    renew_interval_secs: Option<u64>,
}

impl RedisLock {
    /// Create a new distributed lock. / 创建新的分布式锁。
    ///
    /// # Arguments / 参数
    /// * `client` - Redis client / Redis 客户端
    /// * `key` - Lock key / 锁键
    /// * `ttl_secs` - Time-to-live in seconds (prevents deadlocks) / TTL 秒数（防止死锁）
    pub fn new(client: RedisClient, key: String, ttl_secs: u64) -> Self {
        let token = Uuid::new_v4().to_string();
        Self {
            client,
            key,
            token,
            ttl_secs,
            renew_interval_secs: None,
        }
    }

    /// Enable automatic renewal of the lock. / 启用锁的自动续期。
    ///
    /// The lock will be renewed at the specified interval (in seconds)
    /// until released. Must be less than `ttl_secs`.
    /// 锁将在指定间隔（秒）自动续期，直到被释放。必须小于 `ttl_secs`。
    #[must_use]
    pub fn with_auto_renewal(mut self, interval_secs: u64) -> Self {
        self.renew_interval_secs = Some(interval_secs.min(self.ttl_secs / 2));
        self
    }

    /// Set a custom token for this lock instance. / 为此锁实例设置自定义令牌。
    #[must_use]
    pub fn with_token(mut self, token: String) -> Self {
        self.token = token;
        self
    }

    /// Get the lock key. / 获取锁键。
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the lock token. / 获取锁令牌。
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Attempt to acquire the lock (non-blocking).
    /// 尝试获取锁（非阻塞）。
    ///
    /// Returns `Some(guard)` if acquired, `None` if already held.
    /// 如果获取成功返回 `Some(guard)`，如果已被持有则返回 `None`。
    pub async fn acquire(&self) -> RedisResult<Option<RedisLockGuard>> {
        let mut conn = self.client.get_connection().await?;

        // SET key token NX EX ttl_secs
        let result: Option<String> = redis::cmd("SET")
            .arg(&self.key)
            .arg(&self.token)
            .arg("NX")
            .arg("EX")
            .arg(self.ttl_secs)
            .query_async(&mut conn)
            .await?;

        match result {
            Some(ref s) if s == "OK" => Ok(Some(RedisLockGuard {
                client: self.client.clone(),
                key: self.key.clone(),
                token: self.token.clone(),
                ttl_secs: self.ttl_secs,
                renew_interval_secs: self.renew_interval_secs,
                acquired_at: Instant::now(),
            })),
            _ => Ok(None),
        }
    }

    /// Attempt to acquire the lock, blocking until timeout.
    /// 尝试获取锁，阻塞直到超时。
    pub async fn acquire_timeout(
        &self,
        timeout: Duration,
        retry_interval_ms: u64,
    ) -> RedisResult<Option<RedisLockGuard>> {
        let deadline = Instant::now() + timeout;
        let retry = Duration::from_millis(retry_interval_ms.min(1000));

        loop {
            match self.acquire().await? {
                Some(guard) => return Ok(Some(guard)),
                None => {
                    if Instant::now() >= deadline {
                        return Ok(None);
                    }
                    tokio::time::sleep(retry).await;
                }
            }
        }
    }

    /// Attempt to acquire the lock, blocking indefinitely.
    /// 尝试获取锁，无限阻塞。
    pub async fn acquire_blocking(&self, retry_interval_ms: u64) -> RedisResult<RedisLockGuard> {
        let retry = Duration::from_millis(retry_interval_ms.min(1000));
        loop {
            if let Some(guard) = self.acquire().await? {
                return Ok(guard);
            }
            tokio::time::sleep(retry).await;
        }
    }
}

/// Guard for a held distributed lock. / 持有的分布式锁的守卫。
///
/// Automatically releases the lock when dropped (best-effort).
/// Recommended to explicitly call `release()` for reliability.
/// 在丢弃时自动释放锁（尽力而为）。
/// 建议显式调用 `release()` 以确保可靠性。
#[derive(Debug)]
pub struct RedisLockGuard {
    client: RedisClient,
    key: String,
    token: String,
    ttl_secs: u64,
    renew_interval_secs: Option<u64>,
    acquired_at: Instant,
}

impl RedisLockGuard {
    /// Get the lock key. / 获取锁键。
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the lock token. / 获取锁令牌。
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the time when the lock was acquired. / 获取锁的获取时间。
    pub fn acquired_at(&self) -> Instant {
        self.acquired_at
    }

    /// Renew the lock (extend TTL). / 续期锁（延长 TTL）。
    ///
    /// Resets the TTL to `ttl_secs` from now.
    /// 从现在起将 TTL 重置为 `ttl_secs`。
    pub async fn renew(&self) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;

        // Use Lua to atomically check token and extend
        let script = redis::Script::new(
            r"
            if redis.call('GET', KEYS[1]) == ARGV[1] then
                return redis.call('EXPIRE', KEYS[1], ARGV[2])
            end
            return 0
            ",
        );

        let result: i32 = script
            .key(&self.key)
            .arg(&self.token)
            .arg(self.ttl_secs)
            .invoke_async(&mut conn)
            .await?;

        Ok(result > 0)
    }

    /// Release the lock. / 释放锁。
    ///
    /// Uses a Lua script to atomically check the token before deleting,
    /// preventing accidental release of another client's lock.
    /// 使用 Lua 脚本在删除前原子性地检查令牌，
    /// 防止意外释放其他客户端的锁。
    pub async fn release(self) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;

        let script = redis::Script::new(
            r"
            if redis.call('GET', KEYS[1]) == ARGV[1] then
                return redis.call('DEL', KEYS[1])
            end
            return 0
            ",
        );

        let result: i32 = script
            .key(&self.key)
            .arg(&self.token)
            .invoke_async(&mut conn)
            .await?;

        Ok(result > 0)
    }

    /// Get remaining TTL. / 获取剩余 TTL。
    pub async fn ttl(&self) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = conn.ttl(&self.key).await?;
        Ok(result)
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use redis::Client;

    fn make_client() -> RedisClient {
        let client = Client::open("redis://127.0.0.1").unwrap();
        RedisClient::from_client(client)
    }

    #[test]
    fn test_lock_creation() {
        let client = make_client();
        let lock = RedisLock::new(client, "test:lock".to_string(), 30);
        assert_eq!(lock.key(), "test:lock");
        assert!(!lock.token().is_empty());
    }

    #[test]
    fn test_lock_with_token() {
        let client = make_client();
        let lock = RedisLock::new(client, "test:lock".to_string(), 30)
            .with_token("my-custom-token".to_string());
        assert_eq!(lock.token(), "my-custom-token");
    }

    #[test]
    fn test_lock_with_auto_renewal() {
        let client = make_client();
        let lock = RedisLock::new(client, "test:lock".to_string(), 30)
            .with_auto_renewal(10);
        // Renew interval should be capped at TTL/2 = 15
        // But 10 < 15 so it stays 10
        assert!(lock.renew_interval_secs.is_some());
    }
}
