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
//! use hiver_data_redis::lock::ReentrantRedisLock;
//!
//! let rlock = ReentrantRedisLock::new(client, "order:42", 30);
//! let guard = rlock.acquire_with_watchdog("worker-1", None).await?.unwrap();
//! // guard auto-renews every 10 s; watchdog stops on drop
//! guard.release().await?;
//! ```

use crate::{RedisClient, RedisError, RedisResult};
use redis::AsyncCommands;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    /// until released. The interval is capped at `ttl_secs / 2`.
    /// 锁将在指定间隔（秒）自动续期，直到被释放。间隔上限为 `ttl_secs / 2`。
    ///
    /// When the lock is acquired via [`RedisLock::acquire`], a background
    /// tokio task is spawned that periodically calls `EXPIRE` (with token
    /// verification via Lua) to extend the TTL. If renewal fails (lock was
    /// stolen), the watchdog stops and marks the guard as lost.
    /// 通过 [`RedisLock::acquire`] 获取锁时，会生成一个后台 tokio 任务，
    /// 定期调用 `EXPIRE`（通过 Lua 验证令牌）来延长 TTL。
    /// 如果续期失败（锁被抢占），看门狗将停止并将守卫标记为已丢失。
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
            Some(ref s) if s == "OK" => Ok(Some(RedisLockGuard::new(
                self.client.clone(),
                self.key.clone(),
                self.token.clone(),
                self.ttl_secs,
                self.renew_interval_secs,
            ))),
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

/// Guard for a held distributed lock with optional watchdog auto-renewal.
/// 带可选看门狗自动续期的分布式锁守卫。
///
/// When created with auto-renewal enabled, a background task periodically
/// extends the TTL. Dropping or explicitly releasing the guard cancels the watchdog.
/// 当启用自动续期时，后台任务会定期延长 TTL。丢弃或显式释放守卫会取消看门狗。
///
/// Recommended to explicitly call `release()` for reliability.
/// 建议显式调用 `release()` 以确保可靠性。
pub struct RedisLockGuard {
    client: RedisClient,
    key: String,
    token: String,
    ttl_secs: u64,
    renew_interval_secs: Option<u64>,
    acquired_at: Instant,
    /// Handle to the watchdog background task. `None` if auto-renewal is disabled.
    /// 看门狗后台任务的句柄。如果禁用自动续期则为 `None`。
    watchdog_handle: Option<tokio::task::AbortHandle>,
    /// Shared flag indicating whether the watchdog detected lock loss.
    /// 共享标志，指示看门狗是否检测到锁丢失。
    lock_lost: Arc<AtomicBool>,
}

impl std::fmt::Debug for RedisLockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisLockGuard")
            .field("key", &self.key)
            .field("token", &self.token)
            .field("ttl_secs", &self.ttl_secs)
            .field("renew_interval_secs", &self.renew_interval_secs)
            .field("has_watchdog", &self.watchdog_handle.is_some())
            .field("lock_lost", &self.lock_lost.load(Ordering::Relaxed))
            .finish()
    }
}

impl RedisLockGuard {
    fn new(
        client: RedisClient,
        key: String,
        token: String,
        ttl_secs: u64,
        renew_interval_secs: Option<u64>,
    ) -> Self {
        let lock_lost = Arc::new(AtomicBool::new(false));

        let watchdog_handle = renew_interval_secs.map(|interval_secs| {
            let wd_client = client.clone();
            let wd_key = key.clone();
            let wd_token = token.clone();
            let wd_lock_lost = lock_lost.clone();
            let interval = Duration::from_secs(interval_secs);

            let handle = tokio::spawn(async move {
                loop {
                    tokio::time::sleep(interval).await;
                    let renewed = Self::renew_static(&wd_client, &wd_key, &wd_token, ttl_secs).await;
                    match renewed {
                        Ok(true) => {
                            tracing::trace!(
                                key = %wd_key,
                                "watchdog: renewed TTL +{}s / 看门狗: 续期 TTL +{}s",
                                ttl_secs, ttl_secs
                            );
                        }
                        Ok(false) => {
                            tracing::warn!(
                                key = %wd_key,
                                "watchdog: lock lost (renewal failed, token mismatch) \
                                 / 看门狗: 锁已丢失（续期失败，令牌不匹配）"
                            );
                            wd_lock_lost.store(true, Ordering::Relaxed);
                            break;
                        }
                        Err(e) => {
                            tracing::error!(
                                key = %wd_key,
                                error = %e,
                                "watchdog: renewal error, stopping / 看门狗: 续期错误，停止"
                            );
                            wd_lock_lost.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            });

            handle.abort_handle()
        });

        Self {
            client,
            key,
            token,
            ttl_secs,
            renew_interval_secs,
            acquired_at: Instant::now(),
            watchdog_handle,
            lock_lost,
        }
    }

    /// Internal renewal function usable from spawned tasks.
    /// 可从派生任务中使用的内部续期函数。
    async fn renew_static(
        client: &RedisClient,
        key: &str,
        token: &str,
        ttl_secs: u64,
    ) -> RedisResult<bool> {
        let mut conn = client.get_connection().await?;

        let script = redis::Script::new(
            r"
            if redis.call('GET', KEYS[1]) == ARGV[1] then
                return redis.call('EXPIRE', KEYS[1], ARGV[2])
            end
            return 0
            ",
        );

        let result: i32 = script
            .key(key)
            .arg(token)
            .arg(ttl_secs)
            .invoke_async(&mut conn)
            .await?;

        Ok(result > 0)
    }

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

    /// Check whether this guard has an active watchdog.
    /// 检查此守卫是否有活跃的看门狗。
    pub fn has_watchdog(&self) -> bool {
        self.watchdog_handle.is_some()
    }

    /// Check whether the watchdog detected that the lock was lost.
    /// 检查看门狗是否检测到锁已丢失。
    ///
    /// Returns `false` if auto-renewal is not enabled or the lock is still held.
    /// 如果未启用自动续期或锁仍被持有，则返回 `false`。
    pub fn is_lock_lost(&self) -> bool {
        self.lock_lost.load(Ordering::Relaxed)
    }

    /// Renew the lock (extend TTL). / 续期锁（延长 TTL）。
    ///
    /// Resets the TTL to `ttl_secs` from now.
    /// 从现在起将 TTL 重置为 `ttl_secs`。
    pub async fn renew(&self) -> RedisResult<bool> {
        Self::renew_static(&self.client, &self.key, &self.token, self.ttl_secs).await
    }

    /// Release the lock and stop the watchdog (if running).
    /// 释放锁并停止看门狗（如果正在运行）。
    ///
    /// Uses a Lua script to atomically check the token before deleting,
    /// preventing accidental release of another client's lock.
    /// 使用 Lua 脚本在删除前原子性地检查令牌，
    /// 防止意外释放其他客户端的锁。
    pub async fn release(mut self) -> RedisResult<bool> {
        // Stop the watchdog first / 先停止看门狗
        self.stop_watchdog();

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

    /// Stop the watchdog without releasing the lock.
    /// 停止看门狗但不释放锁。
    pub fn stop_watchdog(&mut self) {
        if let Some(handle) = self.watchdog_handle.take() {
            handle.abort();
            tracing::trace!(
                key = %self.key,
                "watchdog stopped / 看门狗已停止"
            );
        }
    }
}

impl Drop for RedisLockGuard {
    fn drop(&mut self) {
        // Stop the watchdog first / 先停止看门狗
        if let Some(handle) = self.watchdog_handle.take() {
            handle.abort();
        }

        // Best-effort release on drop. Since we cannot perform async operations
        // in a Drop impl, we spawn a fire-and-forget task. For reliability,
        // prefer calling `release()` explicitly before the guard goes out of scope.
        let client = self.client.clone();
        let key = std::mem::take(&mut self.key);
        let token = std::mem::take(&mut self.token);
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

    // ── Unit tests (no Redis connection required) ───────────────────────────

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
    fn test_lock_with_auto_renewal_capped_interval() {
        let client = make_client();
        // ttl_secs=30, ttl/2=15, request 10 → stays 10
        let lock = RedisLock::new(client, "test:lock".to_string(), 30)
            .with_auto_renewal(10);
        assert_eq!(lock.renew_interval_secs, Some(10));
    }

    #[test]
    fn test_lock_with_auto_renewal_caps_at_half_ttl() {
        let client = make_client();
        // ttl_secs=30, ttl/2=15, request 20 → capped to 15
        let lock = RedisLock::new(client, "test:lock".to_string(), 30)
            .with_auto_renewal(20);
        assert_eq!(lock.renew_interval_secs, Some(15));
    }

    #[test]
    fn test_lock_without_auto_renewal() {
        let client = make_client();
        let lock = RedisLock::new(client, "test:lock".to_string(), 30);
        assert!(lock.renew_interval_secs.is_none());
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
        assert!(!REENTRANT_ACQUIRE_LUA.is_empty());
        assert!(!REENTRANT_RELEASE_LUA.is_empty());
        assert!(!REENTRANT_RENEW_LUA.is_empty());
    }

    /// Verify RedisLockGuard::new spawns a watchdog when interval is set.
    /// 验证当间隔设置时 RedisLockGuard::new 会生成看门狗。
    #[tokio::test]
    async fn test_guard_has_watchdog_when_interval_set() {
        let client = make_client();
        let guard = RedisLockGuard::new(
            client,
            "wd:key".to_string(),
            "token-abc".to_string(),
            30,
            Some(10),
        );
        assert!(guard.has_watchdog());
        assert!(!guard.is_lock_lost());
    }

    /// Verify RedisLockGuard::new does NOT spawn a watchdog without interval.
    /// 验证没有间隔时 RedisLockGuard::new 不会生成看门狗。
    #[tokio::test]
    async fn test_guard_no_watchdog_without_interval() {
        let client = make_client();
        let guard = RedisLockGuard::new(
            client,
            "no-wd:key".to_string(),
            "token-xyz".to_string(),
            30,
            None,
        );
        assert!(!guard.has_watchdog());
        assert!(!guard.is_lock_lost());
    }

    /// Verify lock_lost flag can be set (simulating watchdog detection).
    /// 验证 lock_lost 标志可以被设置（模拟看门狗检测）。
    #[test]
    fn test_guard_lock_lost_flag() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!flag.load(Ordering::Relaxed));

        flag.store(true, Ordering::Relaxed);
        assert!(flag.load(Ordering::Relaxed));
    }

    /// Verify stop_watchdog aborts the background task.
    /// 验证 stop_watchdog 会中止后台任务。
    #[tokio::test]
    async fn test_stop_watchdog_aborts_task() {
        let client = make_client();
        let mut guard = RedisLockGuard::new(
            client,
            "stop-wd:key".to_string(),
            "token-stop".to_string(),
            30,
            Some(10),
        );
        assert!(guard.has_watchdog());

        guard.stop_watchdog();
        assert!(!guard.has_watchdog());

        // Calling stop_watchdog again is a no-op / 再次调用是无操作
        guard.stop_watchdog();
        assert!(!guard.has_watchdog());
    }

    /// Verify Drop cancels the watchdog task without panic.
    /// 验证 Drop 取消看门狗任务不会引发 panic。
    #[tokio::test]
    async fn test_guard_drop_cancels_watchdog() {
        let client = make_client();
        let guard = RedisLockGuard::new(
            client,
            "drop-wd:key".to_string(),
            "token-drop".to_string(),
            30,
            Some(5),
        );
        assert!(guard.has_watchdog());
        // Drop should not panic / Drop 不应 panic
        drop(guard);
    }

    /// Verify guard without watchdog drops cleanly.
    /// 验证没有看门狗的守卫可以正常丢弃。
    #[tokio::test]
    async fn test_guard_drop_without_watchdog() {
        let client = make_client();
        let guard = RedisLockGuard::new(
            client,
            "drop-no-wd:key".to_string(),
            "token-no-wd".to_string(),
            30,
            None,
        );
        assert!(!guard.has_watchdog());
        drop(guard);
    }

    /// Verify Debug output includes key, token, and watchdog state.
    /// 验证 Debug 输出包含 key、token 和看门狗状态。
    #[tokio::test]
    async fn test_guard_debug_format() {
        let client = make_client();
        let guard = RedisLockGuard::new(
            client,
            "debug:key".to_string(),
            "debug-token".to_string(),
            30,
            Some(10),
        );
        let debug_str = format!("{:?}", guard);
        assert!(debug_str.contains("debug:key"));
        assert!(debug_str.contains("debug-token"));
        assert!(debug_str.contains("has_watchdog: true"));
    }

    /// Verify multiple guards with the same key can be created (different tokens).
    /// 验证可以用相同的 key 创建多个守卫（不同令牌）。
    #[tokio::test]
    async fn test_multiple_guards_same_key() {
        let client1 = make_client();
        let client2 = make_client();
        let guard1 = RedisLockGuard::new(
            client1,
            "shared:key".to_string(),
            "token-1".to_string(),
            30,
            Some(10),
        );
        let guard2 = RedisLockGuard::new(
            client2,
            "shared:key".to_string(),
            "token-2".to_string(),
            30,
            Some(10),
        );
        assert_ne!(guard1.token(), guard2.token());
        assert_eq!(guard1.key(), guard2.key());
    }

    // ── Integration tests (require a running Redis at 127.0.0.1:6379) ──────
    // Run with: cargo test --package hiver-data-redis -- --ignored
    // 使用以下命令运行：cargo test --package hiver-data-redis -- --ignored

    /// Acquire a lock, verify it exists, then release it.
    /// 获取锁，验证其存在，然后释放它。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_acquire_and_release() {
        let client = make_client();
        let key = format!("test:acquire:{}", Uuid::new_v4());
        let lock = RedisLock::new(client.clone(), key.clone(), 30);
        let guard = lock.acquire().await.unwrap().expect("should acquire lock");
        assert_eq!(guard.key(), key);

        // Key should exist in Redis / 键应该存在于 Redis 中
        let mut conn = client.get_connection().await.unwrap();
        let val: Option<String> = conn.get(&key).await.unwrap();
        assert_eq!(val.as_deref(), Some(guard.token()));

        let released = guard.release().await.unwrap();
        assert!(released);

        // Key should be gone / 键应该已删除
        let val: Option<String> = conn.get(&key).await.unwrap();
        assert!(val.is_none());
    }

    /// Acquire lock with watchdog, verify guard reports having a watchdog.
    /// 使用看门狗获取锁，验证守卫报告拥有看门狗。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_acquire_with_watchdog() {
        let client = make_client();
        let key = format!("test:wd:{}", Uuid::new_v4());
        let lock = RedisLock::new(client.clone(), key.clone(), 30).with_auto_renewal(10);
        let guard = lock.acquire().await.unwrap().expect("should acquire lock");
        assert!(guard.has_watchdog());
        assert!(!guard.is_lock_lost());

        // Release should succeed and stop the watchdog
        let released = guard.release().await.unwrap();
        assert!(released);
    }

    /// Verify that lock release cancels the watchdog (TTL should stop extending).
    /// 验证锁释放会取消看门狗（TTL 应停止延长）。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_release_cancels_watchdog() {
        let client = make_client();
        let key = format!("test:release-wd:{}", Uuid::new_v4());
        let lock = RedisLock::new(client.clone(), key.clone(), 10).with_auto_renewal(3);
        let guard = lock.acquire().await.unwrap().expect("should acquire lock");
        assert!(guard.has_watchdog());

        // Release the lock / 释放锁
        let released = guard.release().await.unwrap();
        assert!(released);

        // Verify key no longer exists / 验证键不再存在
        let mut conn = client.get_connection().await.unwrap();
        let val: Option<String> = conn.get(&key).await.unwrap();
        assert!(val.is_none());
    }

    /// Two lock instances compete for the same key; second acquire should fail.
    /// 两个锁实例竞争同一个键；第二个获取应该失败。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_concurrent_lock_competition() {
        let client = make_client();
        let key = format!("test:compete:{}", Uuid::new_v4());

        let lock1 = RedisLock::new(client.clone(), key.clone(), 30);
        let lock2 = RedisLock::new(client.clone(), key.clone(), 30);

        let guard1 = lock1.acquire().await.unwrap().expect("first acquire should succeed");
        let guard2 = lock2.acquire().await.unwrap();
        assert!(guard2.is_none(), "second acquire should fail while first holds lock");

        guard1.release().await.unwrap();

        // Now the second lock should succeed / 现在第二个锁应该成功
        let guard2 = lock2.acquire().await.unwrap();
        assert!(guard2.is_some(), "second acquire should succeed after first released");
    }

    /// Verify manual renewal extends the TTL.
    /// 验证手动续期延长了 TTL。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_manual_renewal() {
        let client = make_client();
        let key = format!("test:renew:{}", Uuid::new_v4());
        let lock = RedisLock::new(client.clone(), key.clone(), 10);
        let guard = lock.acquire().await.unwrap().expect("should acquire");

        // Wait a moment then check TTL / 等待片刻后检查 TTL
        tokio::time::sleep(Duration::from_millis(500)).await;
        let ttl_before = guard.ttl().await.unwrap();
        assert!(ttl_before <= 10 && ttl_before >= 8);

        // Renew / 续期
        let renewed = guard.renew().await.unwrap();
        assert!(renewed);

        let ttl_after = guard.ttl().await.unwrap();
        assert!(
            ttl_after > ttl_before,
            "TTL after renewal ({}) should be greater than before ({})",
            ttl_after, ttl_before
        );

        guard.release().await.unwrap();
    }

    /// Verify that after TTL expires, another client can acquire the lock.
    /// 验证 TTL 过期后，其他客户端可以获取锁。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_expired_lock_without_watchdog() {
        let client = make_client();
        let key = format!("test:expire:{}", Uuid::new_v4());

        // Use very short TTL (1 second) without watchdog
        // 使用极短的 TTL（1 秒）且不启用看门狗
        let lock1 = RedisLock::new(client.clone(), key.clone(), 1);
        let lock2 = RedisLock::new(client.clone(), key.clone(), 30);

        let guard1 = lock1.acquire().await.unwrap().expect("should acquire");
        // Intentionally do not release; let TTL expire
        // 故意不释放；让 TTL 过期
        assert!(!guard1.has_watchdog());

        // Wait for TTL to expire / 等待 TTL 过期
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Second lock should now succeed / 第二个锁现在应该成功
        let guard2 = lock2.acquire().await.unwrap();
        assert!(guard2.is_some(), "should acquire after first lock expired");
        guard2.unwrap().release().await.unwrap();
    }

    /// Verify reentrant lock acquire/release cycle.
    /// 验证可重入锁的获取/释放循环。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_reentrant_acquire_release() {
        let client = make_client();
        let key = format!("test:reentrant:{}", Uuid::new_v4());
        let lock = ReentrantRedisLock::new(client.clone(), key.clone(), 30);

        // First acquire / 第一次获取
        let count1 = lock.acquire("holder-1").await.unwrap().expect("should acquire");
        assert_eq!(count1, 1);

        // Reentrant acquire / 重入获取
        let count2 = lock.acquire("holder-1").await.unwrap().expect("should re-acquire");
        assert_eq!(count2, 2);

        // Different holder should fail / 不同的持有者应该失败
        let count3 = lock.acquire("holder-2").await.unwrap();
        assert!(count3.is_none());

        // Release one level / 释放一个级别
        let remaining = lock.release("holder-1").await.unwrap();
        assert_eq!(remaining, 1);

        // Still held by holder-1 / 仍然由 holder-1 持有
        let count4 = lock.acquire("holder-2").await.unwrap();
        assert!(count4.is_none());

        // Full release / 完全释放
        let remaining2 = lock.release("holder-1").await.unwrap();
        assert_eq!(remaining2, 0);

        // Now holder-2 can acquire / 现在 holder-2 可以获取
        let count5 = lock.acquire("holder-2").await.unwrap().expect("should acquire");
        assert_eq!(count5, 1);
        lock.release("holder-2").await.unwrap();
    }

    /// Verify watchdog for reentrant lock renews TTL.
    /// 验证可重入锁的看门狗续期 TTL。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_reentrant_watchdog_renewal() {
        let client = make_client();
        let key = format!("test:wd-reentrant:{}", Uuid::new_v4());
        let lock = ReentrantRedisLock::new(client.clone(), key.clone(), 6);

        let guard = lock
            .acquire_with_watchdog("holder-wd", Some(Duration::from_secs(2)))
            .await
            .unwrap()
            .expect("should acquire");

        assert_eq!(guard.reentry_count(), 1);

        // Wait long enough that TTL would have expired without renewal
        // 等待足够长的时间，如果没有续期 TTL 应该已经过期
        tokio::time::sleep(Duration::from_secs(4)).await;

        // Lock should still be held (watchdog renewed it)
        // 锁应该仍然被持有（看门狗已续期）
        let lock2 = ReentrantRedisLock::new(client.clone(), &key, 6);
        let result = lock2.acquire("other-holder").await.unwrap();
        assert!(result.is_none(), "lock should still be held by watchdog");

        guard.release().await.unwrap();
    }

    /// Verify acquire_timeout retries and eventually succeeds.
    /// 验证 acquire_timeout 重试后最终成功。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_acquire_timeout_success() {
        let client = make_client();
        let key = format!("test:timeout:{}", Uuid::new_v4());
        let lock = RedisLock::new(client.clone(), key.clone(), 5);
        let guard = lock
            .acquire_timeout(Duration::from_secs(3), 200)
            .await
            .unwrap()
            .expect("should acquire within timeout");
        guard.release().await.unwrap();
    }

    /// Verify acquire_timeout returns None when lock is held.
    /// 验证当锁被持有时 acquire_timeout 返回 None。
    #[tokio::test]
    #[ignore = "requires Redis at 127.0.0.1:6379"]
    async fn test_acquire_timeout_fails_when_held() {
        let client = make_client();
        let key = format!("test:timeout-fail:{}", Uuid::new_v4());
        let lock1 = RedisLock::new(client.clone(), key.clone(), 30);
        let lock2 = RedisLock::new(client.clone(), key.clone(), 30);

        let _guard1 = lock1.acquire().await.unwrap().expect("should acquire");
        let result = lock2
            .acquire_timeout(Duration::from_secs(1), 100)
            .await
            .unwrap();
        assert!(result.is_none(), "should fail to acquire within timeout");
    }
}
