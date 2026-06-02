//! Redis cache implementation
//! Redis 缓存实现

use crate::client::RedisClient;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Redis cache implementation / Redis 缓存实现
///
/// Uses Redis as a cache backend, similar to Spring's RedisCache.
/// 使用 Redis 作为缓存后端，类似于 Spring 的 RedisCache。
#[derive(Debug, Clone)]
pub struct RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    client: RedisClient,
    key_prefix: String,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new Redis cache / 创建新的 Redis 缓存
    pub fn new(client: RedisClient) -> Self {
        Self {
            client,
            key_prefix: String::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create with key prefix / 使用键前缀创建
    pub fn with_prefix(client: RedisClient, prefix: &str) -> Self {
        Self {
            client,
            key_prefix: prefix.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the full key with prefix / 获取带前缀的完整键
    fn full_key(&self, key: &K) -> String
    where
        K: Serialize,
    {
        if self.key_prefix.is_empty() {
            serde_json::to_string(key).unwrap_or_default()
        } else {
            format!("{}{}", self.key_prefix, serde_json::to_string(key).unwrap_or_default())
        }
    }

    /// Serialize value / 序列化值
    fn serialize_value(&self, value: &V) -> Option<Vec<u8>>
    where
        V: Serialize,
    {
        serde_json::to_vec(value).ok()
    }

    /// Deserialize value / 反序列化值
    fn deserialize_value(&self, data: Vec<u8>) -> Option<V>
    where
        V: for<'de> Deserialize<'de>,
    {
        serde_json::from_slice(&data).ok()
    }
}

impl<K, V> RedisCache<K, V>
where
    K: Hash + Eq + Send + Sync + Serialize + 'static,
    V: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Get a value from the cache / 从缓存获取值
    pub async fn get(&self, key: &K) -> Option<V> {
        let full_key = self.full_key(key);
        let mut conn = self.client.get_connection().await.ok()?;

        let result: Option<String> = redis::cmd("GET")
            .arg(full_key)
            .query_async(&mut conn)
            .await
            .ok()?;

        match result {
            Some(s) => self.deserialize_value(s.into_bytes()),
            None => None,
        }
    }

    /// Put a value in the cache / 向缓存放入值
    pub async fn put(&self, key: K, value: V) {
        let full_key = self.full_key(&key);
        let Ok(mut conn) = self.client.get_connection().await else {
            return;
        };

        if let Some(data) = self.serialize_value(&value) {
            let _ = redis::cmd("SET")
                .arg(full_key)
                .arg(data)
                .query_async::<()>(&mut conn)
                .await;
        }
    }

    /// Put a value in the cache with TTL / 向缓存放入带TTL的值
    pub async fn put_with_ttl(&self, key: K, value: V, ttl_secs: u64) {
        let full_key = self.full_key(&key);
        let Ok(mut conn) = self.client.get_connection().await else {
            return;
        };

        if let Some(data) = self.serialize_value(&value) {
            let _ = redis::cmd("SETEX")
                .arg(full_key)
                .arg(ttl_secs)
                .arg(data)
                .query_async::<()>(&mut conn)
                .await;
        }
    }

    /// Invalidate a specific key / 使特定key失效
    pub async fn invalidate(&self, key: &K) {
        let full_key = self.full_key(key);
        let Ok(mut conn) = self.client.get_connection().await else {
            return;
        };

        let _ = redis::cmd("DEL")
            .arg(full_key)
            .query_async::<()>(&mut conn)
            .await;
    }

    /// Invalidate all entries / 使所有条目失效
    pub async fn invalidate_all(&self) {
        if self.key_prefix.is_empty() {
            return;
        }

        let pattern = format!("{}*", self.key_prefix);
        let Ok(mut conn) = self.client.get_connection().await else {
            return;
        };

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .ok()
            .unwrap_or_default();

        if !keys.is_empty() {
            let _ = redis::cmd("DEL")
                .arg(keys)
                .query_async::<()>(&mut conn)
                .await;
        }
    }

    /// Check if cache contains key / 检查缓存是否包含key
    pub async fn contains_key(&self, key: &K) -> bool {
        let full_key = self.full_key(key);
        if let Ok(mut conn) = self.client.get_connection().await {
            redis::cmd("EXISTS")
                .arg(full_key)
                .query_async::<i32>(&mut conn)
                .await
                .map(|v| v > 0)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Get cache size / 获取缓存大小
    pub async fn size(&self) -> usize {
        if self.key_prefix.is_empty() {
            return 0;
        }

        let pattern = format!("{}*", self.key_prefix);
        if let Ok(mut conn) = self.client.get_connection().await {
            redis::cmd("KEYS")
                .arg(&pattern)
                .query_async::<Vec<String>>(&mut conn)
                .await
                .map(|keys| keys.len())
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Clear the cache / 清除缓存
    pub async fn clear(&self) {
        self.invalidate_all().await;
    }
}

/// Redis cache manager / Redis 缓存管理器
///
/// Manages multiple Redis cache instances, similar to Spring's RedisCacheManager.
/// 管理多个 Redis 缓存实例，类似于 Spring 的 RedisCacheManager。
#[derive(Debug, Clone)]
pub struct RedisCacheManager {
    client: RedisClient,
}

impl RedisCacheManager {
    /// Create a new Redis cache manager / 创建新的 Redis 缓存管理器
    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }

    /// Get the underlying client / 获取底层客户端
    pub fn client(&self) -> &RedisClient {
        &self.client
    }

    /// Get a cache by name / 按名称获取缓存
    pub fn get_cache<K, V>(&self, name: &str) -> RedisCache<K, V>
    where
        K: Hash + Eq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let prefix = format!("cache:{}:", name);
        RedisCache::with_prefix(self.client.clone(), &prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let client = redis::Client::open("redis://127.0.0.1").unwrap();
        let redis_client = RedisClient::from_client(client);
        let cache: RedisCache<String, String> = RedisCache::new(redis_client);
        assert_eq!(cache.key_prefix, "");
    }

    #[test]
    fn test_cache_with_prefix() {
        let client = redis::Client::open("redis://127.0.0.1").unwrap();
        let redis_client = RedisClient::from_client(client);
        let cache: RedisCache<String, String> = RedisCache::with_prefix(redis_client, "myapp:");
        assert_eq!(cache.key_prefix, "myapp:");
    }

    #[test]
    fn test_cache_manager() {
        let client = redis::Client::open("redis://127.0.0.1").unwrap();
        let redis_client = RedisClient::from_client(client);
        let manager = RedisCacheManager::new(redis_client);
        let cache: RedisCache<String, String> = manager.get_cache("test");
        assert_eq!(cache.key_prefix, "cache:test:");
    }
}
