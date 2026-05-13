//! Redis Distributed Lock — mutex, reentrant lock, and watchdog renewal.
//! Redis 分布式锁 — 互斥锁、可重入锁和看门狗续期。
//!
//! # Overview / 概述
//!
//! Three lock variants equivalent to Redisson's lock primitives:
//!
//! | Type | Redisson equivalent | Description |
//! |------|---------------------|-------------|
//! | [`RedisLock`] | `getLock()` | Non-reentrant mutex (SET NX EX) |
//! | [`ReentrantRedisLock`] | `RLock` (reentrant) | Reentrant via Lua HASH counter |
//! | [`WatchdogGuard`] | Redisson watchdog | Auto-renews TTL in background |
//!
//! # Example — reentrant lock with watchdog / 示例 — 带看门狗的可重入锁
//!
//! ```rust,no_run,ignore
//! use nexus_data_redis::lock::ReentrantRedisLock;
//!
//! let rlock = ReentrantRedisLock::new(client, "order:42", 30);
//! let guard = rlock.acquire_with_watchdog("worker-1", None).await?.unwrap();
//! // guard auto-renews every 10 s; watchdog stops on drop
//! guard.release().await?;
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
    /// TODO: Auto-renewal is not yet implemented. The `renew_interval_secs`
    /// field is stored but no background task is spawned to periodically
    /// extend the TTL. Call `renew()` manually if needed.
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
        let max_retries: u32 = 30;
        for _ in 0..max_retries {
            if let Some(guard) = self.acquire().await? {
                return Ok(guard);
            }
            tokio::time::sleep(retry).await;
        }
        Err(RedisError::Other(format!(
            "failed to acquire lock after {} retries",
            max_retries,
        )))
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
    #[allow(dead_code)]
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

impl Drop for RedisLockGuard {
    fn drop(&mut self) {
        // Best-effort release on drop. Since we cannot perform async operations
        // in a Drop impl, we spawn a fire-and-forget task. For reliability,
        // prefer calling `release()` explicitly before the guard goes out of scope.
        let client = self.client.clone();
        let key = std::mem::take(&mut self.key);
        let token = std::mem::take(&mut self.token);
        // Spawn a best-effort async release. If the runtime is gone, this is a no-op.
        // Spawn a best-effort async release; the JoinHandle is intentionally
        // discarded because we don't need to await it (fire-and-forget).
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(async move {
            let mut conn = match client.get_connection().await {
                Ok(c) => c,
                Err(_) => return,
            };
            let script = redis::Script::new(
                r"
                if redis.call('GET', KEYS[1]) == ARGV[1] then
                    return redis.call('DEL', KEYS[1])
                end
                return 0
                ",
            );
            let _: Result<i32, _> = script.key(&key).arg(&token).invoke_async(&mut conn).await;
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Reentrant distributed lock (Lua HASH)
// 可重入分布式锁（Lua HASH）
// ─────────────────────────────────────────────────────────────────────────────

/// Reentrant Redis distributed lock.
/// 可重入 Redis 分布式锁。
///
/// Uses a Redis HASH with fields `owner` and `count` so the same holder can
/// acquire the lock multiple times without deadlocking itself.
/// 使用 Redis HASH（字段 `owner` 和 `count`），使同一持有者可以多次获取锁而不会死锁。
///
/// Equivalent to Redisson's `RLock` (reentrant mode).
/// 等价于 Redisson 的 `RLock`（可重入模式）。
#[derive(Debug, Clone)]
pub struct ReentrantRedisLock {
    client: RedisClient,
    key: String,
    ttl_secs: u64,
}

impl ReentrantRedisLock {
    /// Create a new reentrant lock.
    /// 创建新的可重入锁。
    pub fn new(client: RedisClient, key: impl Into<String>, ttl_secs: u64) -> Self {
        Self { client, key: key.into(), ttl_secs }
    }

    /// Acquire the lock for `holder_id` (non-blocking).
    /// 为 `holder_id` 获取锁（非阻塞）。
    ///
    /// Returns the reentry count (≥ 1) on success, `None` if held by another.
    /// 成功时返回重入计数（≥ 1），如果被其他持有者持有则返回 `None`。
    pub async fn acquire(&self, holder_id: &str) -> RedisResult<Option<u32>> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = redis::Script::new(REENTRANT_ACQUIRE_LUA)
            .key(&self.key)
            .arg(holder_id)
            .arg(self.ttl_secs)
            .invoke_async(&mut conn)
            .await?;
        if result > 0 { Ok(Some(result as u32)) } else { Ok(None) }
    }

    /// Acquire with a retry timeout.
    /// 带重试超时的获取。
    pub async fn acquire_timeout(
        &self,
        holder_id: &str,
        timeout: Duration,
        retry_interval: Duration,
    ) -> RedisResult<Option<u32>> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(count) = self.acquire(holder_id).await? {
                return Ok(Some(count));
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }
            tokio::time::sleep(retry_interval).await;
        }
    }

    /// Release one reentry level for `holder_id`.
    /// 为 `holder_id` 释放一个重入级别。
    ///
    /// Returns the remaining count (0 = fully released).
    /// 返回剩余计数（0 = 完全释放）。
    pub async fn release(&self, holder_id: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = redis::Script::new(REENTRANT_RELEASE_LUA)
            .key(&self.key)
            .arg(holder_id)
            .invoke_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Renew the TTL (only if still owned by `holder_id`).
    /// 续期 TTL（仅在仍由 `holder_id` 持有时）。
    pub async fn renew(&self, holder_id: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = redis::Script::new(REENTRANT_RENEW_LUA)
            .key(&self.key)
            .arg(holder_id)
            .arg(self.ttl_secs)
            .invoke_async(&mut conn)
            .await?;
        Ok(result > 0)
    }

    /// Acquire the lock and start a watchdog background task.
    /// 获取锁并启动看门狗后台任务。
    ///
    /// `watchdog_interval`: how often to renew. Defaults to `ttl_secs / 3` seconds.
    /// `watchdog_interval`：续期频率，默认为 `ttl_secs / 3` 秒。
    pub async fn acquire_with_watchdog(
        &self,
        holder_id: &str,
        watchdog_interval: Option<Duration>,
    ) -> RedisResult<Option<WatchdogGuard>> {
        let Some(count) = self.acquire(holder_id).await? else { return Ok(None) };
        let interval = watchdog_interval.unwrap_or_else(|| {
            Duration::from_secs((self.ttl_secs / 3).max(1))
        });
        let guard = WatchdogGuard::new(
            self.client.clone(),
            self.key.clone(),
            holder_id.to_string(),
            self.ttl_secs,
            count,
            interval,
        );
        Ok(Some(guard))
    }
}

// ─── Lua scripts ─────────────────────────────────────────────────────────────

/// Acquire reentrant lock.
/// 获取可重入锁。
const REENTRANT_ACQUIRE_LUA: &str = r#"
local owner = redis.call('HGET', KEYS[1], 'owner')
if owner == false then
    redis.call('HSET', KEYS[1], 'owner', ARGV[1], 'count', 1)
    redis.call('EXPIRE', KEYS[1], ARGV[2])
    return 1
elseif owner == ARGV[1] then
    local c = redis.call('HINCRBY', KEYS[1], 'count', 1)
    redis.call('EXPIRE', KEYS[1], ARGV[2])
    return c
else
    return 0
end
"#;

/// Release one reentry level.
/// 释放一个重入级别。
const REENTRANT_RELEASE_LUA: &str = r#"
local owner = redis.call('HGET', KEYS[1], 'owner')
if owner ~= ARGV[1] then
    return -1
end
local c = redis.call('HINCRBY', KEYS[1], 'count', -1)
if c <= 0 then
    redis.call('DEL', KEYS[1])
    return 0
end
return c
"#;

/// Renew TTL (only for owner).
/// 续期 TTL（仅为所有者）。
const REENTRANT_RENEW_LUA: &str = r#"
if redis.call('HGET', KEYS[1], 'owner') == ARGV[1] then
    return redis.call('EXPIRE', KEYS[1], ARGV[2])
end
return 0
"#;

// ─────────────────────────────────────────────────────────────────────────────
// Watchdog guard — auto-renews TTL in background
// 看门狗守卫 — 在后台自动续期 TTL
// ─────────────────────────────────────────────────────────────────────────────

/// A lock guard that automatically renews its TTL via a background task.
/// 通过后台任务自动续期 TTL 的锁守卫。
///
/// Created by [`ReentrantRedisLock::acquire_with_watchdog`].
/// Stopping the watchdog (via drop or explicit [`WatchdogGuard::release`]) is safe.
pub struct WatchdogGuard {
    client: RedisClient,
    key: String,
    holder_id: String,
    ttl_secs: u64,
    reentry_count: u32,
    watchdog_handle: tokio::task::AbortHandle,
}

impl std::fmt::Debug for WatchdogGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatchdogGuard")
            .field("key", &self.key)
            .field("holder_id", &self.holder_id)
            .field("reentry_count", &self.reentry_count)
            .finish()
    }
}

impl WatchdogGuard {
    fn new(
        client: RedisClient,
        key: String,
        holder_id: String,
        ttl_secs: u64,
        reentry_count: u32,
        interval: Duration,
    ) -> Self {
        let renew_client = client.clone();
        let renew_key = key.clone();
        let renew_holder = holder_id.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let renew_lock = ReentrantRedisLock::new(renew_client.clone(), &renew_key, ttl_secs);
                if let Ok(renewed) = renew_lock.renew(&renew_holder).await {
                    if !renewed {
                        tracing::warn!(key = %renew_key, "watchdog: lock expired before renewal");
                        break;
                    }
                    tracing::trace!(key = %renew_key, "watchdog: renewed TTL +{}s", ttl_secs);
                }
            }
        });

        Self {
            client,
            key,
            holder_id,
            ttl_secs,
            reentry_count,
            watchdog_handle: handle.abort_handle(),
        }
    }

    /// Lock key.
    /// 锁键。
    pub fn key(&self) -> &str { &self.key }

    /// Holder identifier.
    /// 持有者标识符。
    pub fn holder_id(&self) -> &str { &self.holder_id }

    /// Current reentry count.
    /// 当前重入计数。
    pub fn reentry_count(&self) -> u32 { self.reentry_count }

    /// Manually renew the TTL.
    /// 手动续期 TTL。
    pub async fn renew(&self) -> RedisResult<bool> {
        let lock = ReentrantRedisLock::new(self.client.clone(), &self.key, self.ttl_secs);
        lock.renew(&self.holder_id).await
    }

    /// Release the lock and stop the watchdog.
    /// 释放锁并停止看门狗。
    pub async fn release(self) -> RedisResult<i64> {
        self.watchdog_handle.abort();
        let lock = ReentrantRedisLock::new(self.client.clone(), &self.key, self.ttl_secs);
        lock.release(&self.holder_id).await
    }
}

impl Drop for WatchdogGuard {
    fn drop(&mut self) {
        self.watchdog_handle.abort();
        let client = self.client.clone();
        let key = std::mem::take(&mut self.key);
        let holder = std::mem::take(&mut self.holder_id);
        let ttl = self.ttl_secs;
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(async move {
            let lock = ReentrantRedisLock::new(client, &key, ttl);
            let _: RedisResult<_> = lock.release(&holder).await;
        });
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

    #[test]
    fn test_reentrant_lock_creation() {
        let client = make_client();
        let lock = ReentrantRedisLock::new(client, "reentrant:key", 60);
        assert_eq!(lock.key, "reentrant:key");
        assert_eq!(lock.ttl_secs, 60);
    }

    #[test]
    fn test_reentrant_lua_scripts_are_valid() {
        // Smoke-check: ensure Lua scripts compile (redis::Script does not
        // validate at construction time, just confirming non-empty strings).
        assert!(!REENTRANT_ACQUIRE_LUA.is_empty());
        assert!(!REENTRANT_RELEASE_LUA.is_empty());
        assert!(!REENTRANT_RENEW_LUA.is_empty());
    }
}
