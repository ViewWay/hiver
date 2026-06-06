//! Session types
//! 会话类型

use std::{any::Any, collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::DEFAULT_SESSION_TIMEOUT_SECS;

/// Session
/// 会话
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Autowired
/// private SessionRepository<Session> sessionRepository;
///
/// HttpSession session = request.getSession();
/// session.setAttribute("user", user);
/// ```
pub struct Session
{
    /// Session ID
    /// 会话ID
    id: SessionId,

    /// Session attributes
    /// 会话属性
    attributes: Arc<RwLock<HashMap<String, SessionAttribute>>>,

    /// Creation time
    /// 创建时间
    created_at: DateTime<Utc>,

    /// Last accessed time
    /// 最后访问时间
    last_accessed_at: Arc<RwLock<DateTime<Utc>>>,

    /// Max inactive interval (seconds)
    /// 最大非活动间隔（秒）
    max_inactive_interval: Arc<RwLock<u64>>,

    /// Whether the session is new
    /// 是否为新会话
    is_new: Arc<RwLock<bool>>,
}

impl Clone for Session
{
    fn clone(&self) -> Self
    {
        Self {
            id: self.id.clone(),
            attributes: self.attributes.clone(),
            created_at: self.created_at,
            last_accessed_at: self.last_accessed_at.clone(),
            max_inactive_interval: self.max_inactive_interval.clone(),
            is_new: self.is_new.clone(),
        }
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Session
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("created_at", &self.created_at)
            .field("max_inactive_interval", &"Arc<RwLock<u64>>")
            .finish()
    }
}

impl Session
{
    /// Create new session
    /// 创建新会话
    pub fn new(id: SessionId) -> Self
    {
        let now = Utc::now();
        Self {
            id,
            attributes: Arc::new(RwLock::new(HashMap::new())),
            created_at: now,
            last_accessed_at: Arc::new(RwLock::new(now)),
            max_inactive_interval: Arc::new(RwLock::new(DEFAULT_SESSION_TIMEOUT_SECS)),
            is_new: Arc::new(RwLock::new(true)),
        }
    }

    /// Get session ID
    /// 获取会话ID
    pub fn id(&self) -> &SessionId
    {
        &self.id
    }

    /// Get creation time
    /// 获取创建时间
    pub fn created_at(&self) -> DateTime<Utc>
    {
        self.created_at
    }

    /// Get last accessed time
    /// 获取最后访问时间
    pub async fn last_accessed_at(&self) -> DateTime<Utc>
    {
        *self.last_accessed_at.read().await
    }

    /// Update last accessed time
    /// 更新最后访问时间
    pub async fn update_accessed_time(&self)
    {
        let mut last = self.last_accessed_at.write().await;
        *last = Utc::now();
    }

    /// Get max inactive interval
    /// 获取最大非活动间隔
    pub async fn max_inactive_interval(&self) -> u64
    {
        *self.max_inactive_interval.read().await
    }

    /// Set max inactive interval
    /// 设置最大非活动间隔
    pub async fn set_max_inactive_interval(&self, interval: u64)
    {
        let mut max = self.max_inactive_interval.write().await;
        *max = interval;
    }

    /// Check if session is expired
    /// 检查会话是否过期
    pub async fn is_expired(&self) -> bool
    {
        let last = *self.last_accessed_at.read().await;
        let max = *self.max_inactive_interval.read().await;
        let elapsed = Utc::now().signed_duration_since(last);
        elapsed.num_seconds() as u64 > max
    }

    /// Check if session is new
    /// 检查是否为新会话
    pub async fn is_new(&self) -> bool
    {
        *self.is_new.read().await
    }

    /// Mark session as not new
    /// 标记会话为非新会话
    pub async fn mark_not_new(&self)
    {
        let mut is_new = self.is_new.write().await;
        *is_new = false;
    }

    /// Get attribute
    /// 获取属性
    pub async fn get<T: Clone + 'static>(&self, name: &str) -> Option<T>
    {
        let attributes = self.attributes.read().await;
        attributes
            .get(name)
            .and_then(|attr| attr.downcast_ref::<T>().cloned())
    }

    /// Set attribute
    /// 设置属性
    pub async fn set<T: Send + Sync + 'static>(&self, name: impl Into<String>, value: T)
    {
        let mut attributes = self.attributes.write().await;
        attributes.insert(name.into(), SessionAttribute::new(value));
    }

    /// Remove attribute
    /// 移除属性
    pub async fn remove(&self, name: &str) -> Option<SessionAttribute>
    {
        let mut attributes = self.attributes.write().await;
        attributes.remove(name)
    }

    /// Get all attribute names
    /// 获取所有属性名称
    pub async fn attribute_names(&self) -> Vec<String>
    {
        let attributes = self.attributes.read().await;
        attributes.keys().cloned().collect()
    }

    /// Clear all attributes
    /// 清除所有属性
    pub async fn clear(&self)
    {
        let mut attributes = self.attributes.write().await;
        attributes.clear();
    }

    /// Get attribute count
    /// 获取属性数量
    pub async fn attribute_count(&self) -> usize
    {
        let attributes = self.attributes.read().await;
        attributes.len()
    }
}

/// Session ID
/// 会话ID
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId
{
    /// Generate new session ID
    /// 生成新的会话ID
    pub fn new() -> Self
    {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    /// 从字符串创建
    pub fn from_string(id: String) -> Self
    {
        Self(id)
    }

    /// Get as string
    /// 获取字符串表示
    pub fn as_str(&self) -> &str
    {
        &self.0
    }

    /// Into inner string
    /// 转换为内部字符串
    pub fn into_inner(self) -> String
    {
        self.0
    }
}

impl Default for SessionId
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AsRef<str> for SessionId
{
    fn as_ref(&self) -> &str
    {
        &self.0
    }
}

impl std::fmt::Display for SessionId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

/// Session attribute
/// 会话属性
pub struct SessionAttribute(Box<dyn Any + Send + Sync>);

impl SessionAttribute
{
    /// Create new attribute
    /// 创建新属性
    pub fn new<T: Send + Sync + 'static>(value: T) -> Self
    {
        Self(Box::new(value))
    }

    /// Get as any
    /// 获取为Any
    pub fn as_any(&self) -> &dyn Any
    {
        &*self.0
    }

    /// Downcast to reference
    /// 向下转换为引用
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T>
    {
        self.0.downcast_ref::<T>()
    }

    /// Downcast clone
    /// 向下转换并克隆
    pub fn downcast_clone<T: Clone + Send + Sync + 'static>(&self) -> Option<T>
    {
        self.downcast_ref::<T>().cloned()
    }
}

impl std::fmt::Debug for SessionAttribute
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_tuple("SessionAttribute").field(&"<any>").finish()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_session_creation()
    {
        let session = Session::new(SessionId::new());
        assert!(session.is_new().await);
        assert_eq!(session.attribute_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_attributes()
    {
        let session = Session::new(SessionId::new());
        session.set("user_id", 123).await;
        session.set("username", "john".to_string()).await;

        assert_eq!(session.get::<i32>("user_id").await, Some(123));
        assert_eq!(session.get::<String>("username").await, Some("john".to_string()));
        assert_eq!(session.attribute_count().await, 2);
    }

    #[tokio::test]
    async fn test_session_remove()
    {
        let session = Session::new(SessionId::new());
        session.set("key", "value").await;
        assert_eq!(session.attribute_count().await, 1);

        session.remove("key").await;
        assert_eq!(session.attribute_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_clear()
    {
        let session = Session::new(SessionId::new());
        session.set("key1", "value1").await;
        session.set("key2", "value2").await;
        assert_eq!(session.attribute_count().await, 2);

        session.clear().await;
        assert_eq!(session.attribute_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_expiration()
    {
        let session = Session::new(SessionId::new());
        session.set_max_inactive_interval(1).await;

        // Session should not be expired immediately
        assert!(!session.is_expired().await);

        // After more than 1 second, session should be expired
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        assert!(session.is_expired().await);
    }
}
