//! Session store implementations
//! 会话存储实现

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
#[cfg(any(feature = "redis", feature = "mongodb"))]
use chrono::{DateTime, Utc};
#[cfg(any(feature = "redis", feature = "mongodb"))]
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{Session, SessionId};

/// Serializable session data for storage (used by Redis and MongoDB stores).
/// 可序列化的会话数据用于存储（由 Redis 和 MongoDB 存储使用）。
#[cfg(any(feature = "redis", feature = "mongodb"))]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SessionData
{
    /// Session ID
    /// 会话ID
    pub id: String,

    /// Creation time
    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// Last accessed time
    /// 最后访问时间
    pub last_accessed_at: DateTime<Utc>,

    /// Max inactive interval (seconds)
    /// 最大非活动间隔（秒）
    pub max_inactive_interval: u64,

    /// Session attributes as JSON
    /// 会话属性为JSON
    pub attributes: HashMap<String, serde_json::Value>,
}

#[cfg(any(feature = "redis", feature = "mongodb"))]
impl SessionData
{
    /// Create from Session
    /// 从Session创建
    async fn from_session(session: &Session) -> Self
    {
        Self {
            id: session.id().as_str().to_string(),
            created_at: session.created_at(),
            last_accessed_at: session.last_accessed_at().await,
            max_inactive_interval: session.max_inactive_interval().await,
            attributes: HashMap::new(), // Attributes not serialized for distributed stores
        }
    }

    /// Create new Session from `SessionData`
    /// `从SessionData创建新的Session`
    fn to_session(&self) -> Session
    {
        let session = Session::new(SessionId::from_string(self.id.clone()));
        // Note: attributes are not preserved in distributed stores
        // This is a known limitation - for full attribute support,
        // use the in-memory store or implement custom serialization
        session
    }
}

/// Session store trait
/// 会话存储trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface SessionRepository<S extends Session> {
///     S createSession();
///     void save(Session session);
///     Session findById(String id);
///     void deleteById(String id);
/// }
/// ```
#[async_trait]
pub trait SessionStore: Send + Sync
{
    /// Create new session
    /// 创建新会话
    async fn create(&self) -> Result<Session, String>;

    /// Save session
    /// 保存会话
    async fn save(&self, session: &Session) -> Result<(), String>;

    /// Get session by ID
    /// 根据ID获取会话
    async fn get(&self, id: &SessionId) -> Result<Option<Session>, String>;

    /// Delete session by ID
    /// 根据ID删除会话
    async fn delete(&self, id: &SessionId) -> Result<(), String>;

    /// Check if session exists
    /// 检查会话是否存在
    async fn exists(&self, id: &SessionId) -> Result<bool, String>
    {
        Ok(self.get(id).await?.is_some())
    }

    /// Get all session IDs
    /// 获取所有会话ID
    async fn ids(&self) -> Result<Vec<SessionId>, String>;

    /// Clean up expired sessions
    /// 清理过期会话
    async fn cleanup_expired(&self) -> Result<usize, String>;
}

/// In-memory session store
/// 内存会话存储
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public MapSessionRepository sessionRepository() {
///     return new MapSessionRepository();
/// }
/// ```
#[derive(Clone)]
pub struct MemorySessionStore
{
    /// Sessions
    /// 会话
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl MemorySessionStore
{
    /// Create new in-memory session store
    /// 创建新的内存会话存储
    pub fn new() -> Self
    {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get session count
    /// 获取会话数量
    pub async fn count(&self) -> usize
    {
        self.sessions.read().await.len()
    }

    /// Clear all sessions
    /// 清除所有会话
    pub async fn clear(&self)
    {
        self.sessions.write().await.clear();
    }
}

impl Default for MemorySessionStore
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for MemorySessionStore
{
    async fn create(&self) -> Result<Session, String>
    {
        let session = Session::new(SessionId::new());
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id().clone(), session.clone());
        Ok(session)
    }

    async fn save(&self, session: &Session) -> Result<(), String>
    {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id().clone(), session.clone());
        Ok(())
    }

    async fn get(&self, id: &SessionId) -> Result<Option<Session>, String>
    {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn delete(&self, id: &SessionId) -> Result<(), String>
    {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        Ok(())
    }

    async fn ids(&self) -> Result<Vec<SessionId>, String>
    {
        let sessions = self.sessions.read().await;
        Ok(sessions.keys().cloned().collect())
    }

    async fn cleanup_expired(&self) -> Result<usize, String>
    {
        let mut sessions = self.sessions.write().await;
        let mut expired = Vec::new();

        for (id, session) in sessions.iter()
        {
            if session.is_expired().await
            {
                expired.push(id.clone());
            }
        }

        for id in &expired
        {
            sessions.remove(id);
        }

        Ok(expired.len())
    }
}

/// Redis session store
/// Redis会话存储
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @EnableRedisHttpSession
/// @Bean
/// public LettuceConnectionFactory redisConnectionFactory() {
///     return new LettuceConnectionFactory();
/// }
/// ```
#[cfg(feature = "redis")]
#[derive(Clone)]
pub struct RedisSessionStore
{
    /// Redis client
    /// Redis客户端
    client: Arc<hiver_data_redis::RedisClient>,

    /// Key prefix
    /// 键前缀
    key_prefix: String,

    /// Session timeout (seconds)
    /// 会话超时时间（秒）
    timeout: u64,
}

#[cfg(feature = "redis")]
impl RedisSessionStore
{
    /// Create new Redis session store
    /// 创建新的Redis会话存储
    pub fn new(client: hiver_data_redis::RedisClient) -> Self
    {
        Self {
            client: Arc::new(client),
            key_prefix: "session:".to_string(),
            timeout: crate::DEFAULT_SESSION_TIMEOUT_SECS,
        }
    }

    /// Set key prefix
    /// 设置键前缀
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self
    {
        self.key_prefix = prefix.into();
        self
    }

    /// Set session timeout
    /// 设置会话超时时间
    pub fn with_timeout(mut self, timeout: u64) -> Self
    {
        self.timeout = timeout;
        self
    }

    /// Get session key
    /// 获取会话键
    fn session_key(&self, id: &SessionId) -> String
    {
        format!("{}{}", self.key_prefix, id.as_str())
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl SessionStore for RedisSessionStore
{
    async fn create(&self) -> Result<Session, String>
    {
        let session = Session::new(SessionId::new());
        self.save(&session).await?;
        Ok(session)
    }

    async fn save(&self, session: &Session) -> Result<(), String>
    {
        let key = self.session_key(session.id());
        let data = SessionData::from_session(session).await;
        let json = serde_json::to_string(&data)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        // Set with expiration
        // 设置过期时间
        self.client
            .setex(&key, self.timeout, json.as_bytes())
            .await
            .map_err(|e| format!("Failed to save session: {}", e))
    }

    async fn get(&self, id: &SessionId) -> Result<Option<Session>, String>
    {
        let key = self.session_key(id);
        let value = self
            .client
            .get(&key)
            .await
            .map_err(|e| format!("Failed to get session: {}", e))?;

        match value
        {
            Some(v) =>
            {
                let json =
                    String::from_utf8(v).map_err(|e| format!("Failed to parse session: {}", e))?;
                let data: SessionData = serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to deserialize session: {}", e))?;
                Ok(Some(data.to_session()))
            },
            None => Ok(None),
        }
    }

    async fn delete(&self, id: &SessionId) -> Result<(), String>
    {
        let key = self.session_key(id);
        self.client
            .del(&key)
            .await
            .map_err(|e| format!("Failed to delete session: {}", e))?;
        Ok(())
    }

    async fn ids(&self) -> Result<Vec<SessionId>, String>
    {
        // Use SCAN to get all session keys
        // 使用SCAN获取所有会话键
        let pattern = format!("{}*", self.key_prefix);
        let keys = self
            .client
            .keys(&pattern)
            .await
            .map_err(|e| format!("Failed to get session IDs: {}", e))?;

        let ids = keys
            .into_iter()
            .filter_map(|k| {
                let key_str = String::from_utf8(k).ok()?;
                let id_str = key_str.strip_prefix(&self.key_prefix)?;
                Some(SessionId::from_string(id_str.to_string()))
            })
            .collect();

        Ok(ids)
    }

    async fn cleanup_expired(&self) -> Result<usize, String>
    {
        // Redis handles expiration automatically
        // Redis自动处理过期
        Ok(0)
    }
}

/// MongoDB session store
/// MongoDB会话存储
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @EnableMongoHttpSession
/// @Bean
/// public MongoTemplate mongoTemplate() {
///     return new MongoTemplate(mongoClient(), "sessions");
/// }
/// ```
#[cfg(feature = "mongodb")]
#[derive(Clone)]
pub struct MongoSessionStore
{
    /// MongoDB client
    /// MongoDB客户端
    client: Arc<hiver_data_mongodb::MongoClient>,

    /// Database name
    /// 数据库名称
    database: String,

    /// Collection name
    /// 集合名称
    collection: String,

    /// Index expiration (seconds)
    /// 索引过期时间（秒）
    ttl: u64,
}

#[cfg(feature = "mongodb")]
impl MongoSessionStore
{
    /// Create new MongoDB session store
    /// 创建新的MongoDB会话存储
    pub fn new(client: hiver_data_mongodb::MongoClient) -> Self
    {
        Self {
            client: Arc::new(client),
            database: "sessions".to_string(),
            collection: "session_data".to_string(),
            ttl: crate::DEFAULT_SESSION_TIMEOUT_SECS,
        }
    }

    /// Set database name
    /// 设置数据库名称
    pub fn with_database(mut self, database: impl Into<String>) -> Self
    {
        self.database = database.into();
        self
    }

    /// Set collection name
    /// 设置集合名称
    pub fn with_collection(mut self, collection: impl Into<String>) -> Self
    {
        self.collection = collection.into();
        self
    }

    /// Set TTL
    /// 设置TTL
    pub fn with_ttl(mut self, ttl: u64) -> Self
    {
        self.ttl = ttl;
        self
    }
}

#[cfg(feature = "mongodb")]
#[async_trait]
impl SessionStore for MongoSessionStore
{
    async fn create(&self) -> Result<Session, String>
    {
        let session = Session::new(SessionId::new());
        self.save(&session).await?;
        Ok(session)
    }

    async fn save(&self, session: &Session) -> Result<(), String>
    {
        let data = SessionData::from_session(session).await;
        let json = serde_json::to_string(&data)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;
        let doc: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("Failed to convert session: {}", e))?;

        self.client
            .insert(&self.database, &self.collection, doc)
            .await
            .map_err(|e| format!("Failed to save session: {}", e))
    }

    async fn get(&self, id: &SessionId) -> Result<Option<Session>, String>
    {
        let filter = serde_json::json!({ "id": id.as_str() });
        let result = self
            .client
            .find_one(&self.database, &self.collection, filter)
            .await
            .map_err(|e| format!("Failed to get session: {}", e))?;

        match result
        {
            Some(doc) =>
            {
                let json = serde_json::to_string(&doc)
                    .map_err(|e| format!("Failed to serialize document: {}", e))?;
                let data: SessionData = serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to deserialize session: {}", e))?;
                Ok(Some(data.to_session()))
            },
            None => Ok(None),
        }
    }

    async fn delete(&self, id: &SessionId) -> Result<(), String>
    {
        let filter = serde_json::json!({ "id": id.as_str() });
        self.client
            .delete(&self.database, &self.collection, filter)
            .await
            .map_err(|e| format!("Failed to delete session: {}", e))?;
        Ok(())
    }

    async fn ids(&self) -> Result<Vec<SessionId>, String>
    {
        let results = self
            .client
            .find(&self.database, &self.collection, serde_json::json!({}))
            .await
            .map_err(|e| format!("Failed to get session IDs: {}", e))?;

        let ids = results
            .into_iter()
            .filter_map(|doc| {
                let id = doc.get("_id")?.as_str()?;
                Some(SessionId::from_string(id.to_string()))
            })
            .collect();

        Ok(ids)
    }

    async fn cleanup_expired(&self) -> Result<usize, String>
    {
        // MongoDB handles TTL indexes automatically
        // MongoDB自动处理TTL索引
        Ok(0)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_memory_session_store()
    {
        let store = MemorySessionStore::new();

        // Create session
        let session = store.create().await.unwrap();
        assert!(session.is_new().await);

        // Get session
        let found = store.get(session.id()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), session.id());
    }

    #[tokio::test]
    async fn test_memory_session_store_delete()
    {
        let store = MemorySessionStore::new();
        let session = store.create().await.unwrap();

        store.delete(session.id()).await.unwrap();

        let found = store.get(session.id()).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_memory_session_store_ids()
    {
        let store = MemorySessionStore::new();
        let s1 = store.create().await.unwrap();
        let s2 = store.create().await.unwrap();

        let ids = store.ids().await.unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(s1.id()));
        assert!(ids.contains(s2.id()));
    }
}
