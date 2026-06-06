//! Redis client management
//! Redis 客户端管理

use std::sync::Arc;

use redis::aio::MultiplexedConnection;

use crate::{RedisError, RedisResult};

/// Redis client wrapper / Redis 客户端包装器
#[derive(Debug, Clone)]
pub struct RedisClient
{
    /// Inner Redis client / 内部 Redis 客户端
    client: Arc<redis::Client>,
}

impl RedisClient
{
    /// Create a new Redis client / 创建新的 Redis 客户端
    ///
    /// # Arguments / 参数
    ///
    /// * `connection_string` - Redis connection string / Redis 连接字符串
    pub async fn new(connection_string: &str) -> RedisResult<Self>
    {
        let client = redis::Client::open(connection_string)?;

        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Create from existing Redis client / 从现有的 Redis 客户端创建
    pub fn from_client(client: redis::Client) -> Self
    {
        Self {
            client: Arc::new(client),
        }
    }

    /// Get the inner client / 获取内部客户端
    pub fn inner(&self) -> &redis::Client
    {
        &self.client
    }

    /// Get a multiplexed connection / 获取多路复用连接
    pub async fn get_connection(&self) -> RedisResult<MultiplexedConnection>
    {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(RedisError::from)
    }

    /// Ping the server / 检查服务器连接
    pub async fn ping(&self) -> RedisResult<String>
    {
        let mut conn = self.get_connection().await?;
        let result: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(result)
    }

    /// Get database info / 获取数据库信息
    pub async fn info(&self) -> RedisResult<String>
    {
        let mut conn = self.get_connection().await?;
        let result: String = redis::cmd("INFO").query_async(&mut conn).await?;
        Ok(result)
    }

    /// Select database / 选择数据库
    pub async fn select(&self, db_index: i64) -> RedisResult<()>
    {
        let mut conn = self.get_connection().await?;
        redis::cmd("SELECT")
            .arg(db_index)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Flush database / 清空数据库
    pub async fn flushdb(&self) -> RedisResult<()>
    {
        let mut conn = self.get_connection().await?;
        redis::cmd("FLUSHDB").query_async::<()>(&mut conn).await?;
        Ok(())
    }

    /// Flush all databases / 清空所有数据库
    pub async fn flushall(&self) -> RedisResult<()>
    {
        let mut conn = self.get_connection().await?;
        redis::cmd("FLUSHALL").query_async::<()>(&mut conn).await?;
        Ok(())
    }

    /// Get database size / 获取数据库大小
    pub async fn dbsize(&self) -> RedisResult<u64>
    {
        let mut conn = self.get_connection().await?;
        let result: u64 = redis::cmd("DBSIZE").query_async(&mut conn).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_client_from_string()
    {
        // Just test that we can create a client from connection string
        let client = redis::Client::open("redis://127.0.0.1").unwrap();
        let redis_client = RedisClient::from_client(client);
        // Verify client was created successfully
        assert!(
            format!("{:?}", redis_client.inner().get_connection_info().addr).contains("127.0.0.1")
        );
    }
}
