//! Redis reactive template
//! Redis 响应式模板

use redis::AsyncCommands;

use crate::{RedisResult, client::RedisClient};

/// Redis reactive template similar to Spring Data's RedisTemplate
/// Redis 响应式模板，类似于 Spring Data 的 RedisTemplate
#[derive(Debug, Clone)]
pub struct RedisTemplate {
    client: RedisClient,
}

impl RedisTemplate {
    /// Create a new RedisTemplate / 创建新的 RedisTemplate
    pub fn new(client: redis::Client) -> Self {
        Self {
            client: RedisClient::from_client(client),
        }
    }

    /// Get the underlying client / 获取底层客户端
    pub fn client(&self) -> &RedisClient {
        &self.client
    }

    /// Set a string value / 设置字符串值
    pub async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection().await?;
        conn.set::<_, _, ()>(key, value).await?;
        Ok(())
    }

    /// Set a string value with expiration / 设置带过期时间的字符串值
    pub async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> RedisResult<()> {
        let mut conn = self.client.get_connection().await?;
        conn.set_ex::<_, _, ()>(key, value, seconds).await?;
        Ok(())
    }

    /// Get a string value / 获取字符串值
    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.client.get_connection().await?;
        let result: Option<String> = conn.get(key).await?;
        Ok(result)
    }

    /// Delete a key / 删除键
    pub async fn del(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i32 = conn.del(key).await?;
        Ok(result > 0)
    }

    /// Delete multiple keys / 删除多个键
    pub async fn del_multiple(&self, keys: &[&str]) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().await?;
        let result: u64 = conn.del(keys).await?;
        Ok(result)
    }

    /// Check if key exists / 检查键是否存在
    pub async fn exists(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i32 = conn.exists(key).await?;
        Ok(result > 0)
    }

    /// Set expiration time / 设置过期时间
    pub async fn expire(&self, key: &str, seconds: i64) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i32 = conn.expire(key, seconds).await?;
        Ok(result > 0)
    }

    /// Get time to live / 获取剩余过期时间
    pub async fn ttl(&self, key: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = conn.ttl(key).await?;
        Ok(result)
    }

    /// Increment value / 增加值
    pub async fn incr(&self, key: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = conn.incr(key, 1).await?;
        Ok(result)
    }

    /// Increment by delta / 按增量增加值
    pub async fn incr_by(&self, key: &str, delta: i64) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = conn.incr(key, delta).await?;
        Ok(result)
    }

    /// Decrement value / 减少值
    pub async fn decr(&self, key: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = conn.decr(key, 1).await?;
        Ok(result)
    }

    /// Decrement by delta / 按增量减少值
    pub async fn decr_by(&self, key: &str, delta: i64) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().await?;
        let result: i64 = conn.decr(key, delta).await?;
        Ok(result)
    }

    /// Set JSON value / 设置 JSON 值
    pub async fn json_set(
        &self,
        key: &str,
        path: &str,
        value: &serde_json::Value,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().await?;
        let json_str = serde_json::to_string(value)?;
        redis::cmd("JSON.SET")
            .arg(key)
            .arg(path)
            .arg(json_str)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Get JSON value / 获取 JSON 值
    pub async fn json_get(&self, key: &str, path: &str) -> RedisResult<serde_json::Value> {
        let mut conn = self.client.get_connection().await?;
        let result: String = redis::cmd("JSON.GET")
            .arg(key)
            .arg(path)
            .query_async(&mut conn)
            .await?;
        let value = serde_json::from_str(&result)?;
        Ok(value)
    }

    /// Add to set / 添加到集合
    pub async fn sadd(&self, key: &str, member: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i32 = conn.sadd(key, member).await?;
        Ok(result > 0)
    }

    /// Get all set members / 获取所有集合成员
    pub async fn smembers(&self, key: &str) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().await?;
        let result: Vec<String> = conn.smembers(key).await?;
        Ok(result)
    }

    /// Remove from set / 从集合移除
    pub async fn srem(&self, key: &str, member: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i32 = conn.srem(key, member).await?;
        Ok(result > 0)
    }

    /// Push to list / 推送到列表
    pub async fn lpush(&self, key: &str, value: &str) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().await?;
        let result: u64 = conn.lpush(key, value).await?;
        Ok(result)
    }

    /// Pop from list / 从列表弹出
    pub async fn lpop(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.client.get_connection().await?;
        let result: Option<String> = conn.lpop(key, None).await?;
        Ok(result)
    }

    /// Get list range / 获取列表范围
    pub async fn lrange(&self, key: &str, start: i64, stop: i64) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().await?;
        let result: Vec<String> = conn.lrange(key, start as isize, stop as isize).await?;
        Ok(result)
    }

    /// Add to sorted set / 添加到有序集合
    pub async fn zadd(&self, key: &str, score: f64, member: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i32 = conn.zadd(key, score, member).await?;
        Ok(result > 0)
    }

    /// Get sorted set range by score / 按分数获取有序集合范围
    pub async fn zrangebyscore(&self, key: &str, min: f64, max: f64) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().await?;
        let result: Vec<String> = conn.zrangebyscore(key, min, max).await?;
        Ok(result)
    }

    /// Remove from sorted set / 从有序集合移除
    pub async fn zrem(&self, key: &str, member: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().await?;
        let result: i32 = conn.zrem(key, member).await?;
        Ok(result > 0)
    }

    /// Publish message / 发布消息
    pub async fn publish(&self, channel: &str, message: &str) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().await?;
        let result: u64 = conn.publish(channel, message).await?;
        Ok(result)
    }

    /// Subscribe to channels / 订阅频道
    ///
    /// Note: This creates a new dedicated connection for pub/sub.
    /// The caller is responsible for managing the connection lifecycle.
    #[allow(deprecated)]
    pub async fn subscribe(&self, channels: &[&str]) -> RedisResult<()> {
        let client = self.client.inner().clone();
        let conn = client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(channels).await?;
        // Note: pubsub is dropped here, which unsubscribes.
        // For long-term subscriptions, manage the PubSub connection externally.
        Ok(())
    }
}

impl RedisTemplate {
    // ── Typed Operations (Serialization-based) ──
    // ── 类型化操作（基于序列化）──

    /// Typed set — serialize and store any serde type as JSON.
    /// 类型化 set — 序列化并存储任意 serde 类型为 JSON。
    pub async fn set_typed<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> RedisResult<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| crate::RedisError::serialization(format!("{} at key={}", e, key)))?;
        self.set(key, &json).await
    }

    /// Typed set with expiration. / 类型化 set 带过期。
    pub async fn set_typed_ex<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        seconds: u64,
    ) -> RedisResult<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| crate::RedisError::serialization(format!("{} at key={}", e, key)))?;
        self.set_ex(key, &json, seconds).await
    }

    /// Typed get — deserialize a previously stored serde type from JSON.
    /// 类型化 get — 从 JSON 反序列化之前存储的 serde 类型。
    pub async fn get_typed<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> RedisResult<Option<T>> {
        match self.get(key).await? {
            Some(json) => {
                let value = serde_json::from_str(&json).map_err(|e| {
                    crate::RedisError::deserialization(format!(
                        "{} at key={}. JSON: {}",
                        e, key, json
                    ))
                })?;
                Ok(Some(value))
            },
            None => Ok(None),
        }
    }

    /// Get or insert — like `get_typed` but inserts a default value if key is absent.
    /// 获取或插入 — 类似 `get_typed` 但如果键不存在则插入默认值。
    pub async fn get_or_insert<T>(&self, key: &str, default: T) -> RedisResult<T>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + Clone,
    {
        match self.get_typed(key).await? {
            Some(v) => Ok(v),
            None => {
                self.set_typed(key, &default).await?;
                Ok(default)
            },
        }
    }

    /// Get or insert with async factory function.
    /// 使用异步工厂函数获取或插入。
    pub async fn get_or_insert_with<F, Fut, T>(&self, key: &str, factory: F) -> RedisResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + Clone,
    {
        match self.get_typed(key).await? {
            Some(v) => Ok(v),
            None => {
                let value = factory().await;
                self.set_typed(key, &value).await?;
                Ok(value)
            },
        }
    }

    /// Batch set — store multiple typed values atomically via pipeline.
    /// 批量 set — 通过管道原子性地存储多个类型化值。
    pub async fn set_all_typed<T: serde::Serialize + Send + Sync>(
        &self,
        items: &[(&str, &T)],
    ) -> RedisResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        use crate::pipeline::RedisPipeline;
        let mut pipe = RedisPipeline::new();
        for (key, value) in items {
            let json = serde_json::to_string(value)
                .map_err(|e| crate::RedisError::serialization(format!("{} at key={}", e, key)))?;
            pipe = pipe.set(*key, json.into_bytes());
        }

        let mut conn = self.client.get_connection().await?;
        pipe.execute(&mut conn).await?;
        Ok(())
    }

    /// Batch get — deserialize multiple typed values atomically via pipeline.
    /// 批量 get — 通过管道原子性地反序列化多个类型化值。
    pub async fn get_all_typed<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        keys: &[&str],
    ) -> RedisResult<Vec<Option<T>>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        use crate::pipeline::RedisPipeline;
        let mut pipe = RedisPipeline::new();
        for key in keys {
            pipe = pipe.get(*key);
        }

        let mut conn = self.client.get_connection().await?;
        let result = pipe.execute(&mut conn).await?;

        let mut output = Vec::with_capacity(keys.len());
        for (i, key) in keys.iter().enumerate() {
            let val = match result.get_optional_string(i) {
                Ok(Some(json)) => Some(serde_json::from_str(&json).map_err(|e| {
                    crate::RedisError::deserialization(format!(
                        "{} at key={}. JSON: {}",
                        e, key, json
                    ))
                })?),
                Ok(None) => None,
                Err(e) => return Err(e),
            };
            output.push(val);
        }
        Ok(output)
    }

    /// Execute a callback within a pipeline, collecting responses.
    /// 在管道内执行回调，收集响应。
    ///
    /// The callback receives a `RedisPipeline` builder and returns it
    /// after adding commands. Useful for batch atomic operations.
    /// 回调接收 `RedisPipeline` 构建器并在添加命令后返回它。
    /// 适用于批量原子操作。
    pub async fn execute_pipelined<F>(&self, f: F) -> RedisResult<crate::pipeline::PipelineResult>
    where
        F: FnOnce(crate::pipeline::RedisPipeline) -> crate::pipeline::RedisPipeline,
    {
        let pipe = f(crate::pipeline::RedisPipeline::new());
        let mut conn = self.client.get_connection().await?;
        pipe.execute(&mut conn).await
    }
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let client = redis::Client::open("redis://127.0.0.1").unwrap();
        let template = RedisTemplate::new(client);
        assert!(
            format!("{:?}", template.client().inner().get_connection_info().addr)
                .contains("127.0.0.1")
        );
    }
}
