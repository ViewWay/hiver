//! Session middleware
//! 会话中间件

use crate::{Session, SessionConfig, SessionId, SessionStore};
use std::sync::Arc;

/// Session middleware
/// 会话中间件
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Order(Ordered.HIGHEST_PRECEDENCE)
/// public class SessionRepositoryFilter extends OncePerRequestFilter {
///     private final SessionRepository<Session> sessionRepository;
///
///     @Override
///     protected void doFilterInternal(HttpServletRequest request,
///                                     HttpServletResponse response)
///                                     throws ServletException, IOException {
///         // Session management logic
///     }
/// }
/// ```
#[derive(Clone)]
pub struct SessionMiddleware<S> {
    /// Inner service
    /// 内部服务
    inner: S,

    /// Session store
    /// 会话存储
    store: Arc<dyn SessionStore>,

    /// Session configuration
    /// 会话配置
    config: SessionConfig,
}

impl<S> SessionMiddleware<S> {
    /// Create new session middleware
    /// 创建新的会话中间件
    pub fn new(inner: S, store: Arc<dyn SessionStore>, config: SessionConfig) -> Self {
        Self {
            inner,
            store,
            config,
        }
    }

    /// Get session store
    /// 获取会话存储
    pub fn store(&self) -> &dyn SessionStore {
        &*self.store
    }

    /// Get session config
    /// 获取会话配置
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

/// Session context
/// 会话上下文
#[derive(Clone)]
pub struct SessionContext {
    /// Session
    /// 会话
    session: Option<Session>,

    /// Session store
    /// 会话存储
    store: Arc<dyn SessionStore>,
}

impl SessionContext {
    /// Create new session context
    /// 创建新的会话上下文
    pub fn new(store: Arc<dyn SessionStore>) -> Self {
        Self {
            session: None,
            store,
        }
    }

    /// Get session
    /// 获取会话
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// Set session
    /// 设置会话
    pub fn set_session(&mut self, session: Session) {
        self.session = Some(session);
    }

    /// Create new session
    /// 创建新会话
    pub async fn create(&mut self) -> Result<Session, String> {
        let session = self.store.create().await?;
        self.session = Some(session.clone());
        Ok(session)
    }

    /// Save current session
    /// 保存当前会话
    pub async fn save(&self) -> Result<(), String> {
        if let Some(ref session) = self.session {
            self.store.save(session).await?;
        }
        Ok(())
    }

    /// Invalidate current session
    /// 使当前会话失效
    pub async fn invalidate(&mut self) -> Result<(), String> {
        if let Some(ref session) = self.session {
            self.store.delete(session.id()).await?;
            self.session = None;
        }
        Ok(())
    }

    /// Check if session exists
    /// 检查会话是否存在
    pub fn exists(&self) -> bool {
        self.session.is_some()
    }

    /// Get session ID
    /// 获取会话ID
    pub fn session_id(&self) -> Option<&SessionId> {
        self.session.as_ref().map(super::session::Session::id)
    }

    /// Get attribute from session
    /// 从会话获取属性
    pub async fn get<T: Clone + 'static>(&self, name: &str) -> Option<T> {
        self.session.as_ref()?.get(name).await
    }

    /// Set attribute in session
    /// 在会话中设置属性
    pub async fn set<T: Send + Sync + 'static>(&self, name: impl Into<String>, value: T) {
        if let Some(ref session) = self.session {
            session.set(name, value).await;
        }
    }

    /// Remove attribute from session
    /// 从会话移除属性
    pub async fn remove(&self, name: &str) {
        if let Some(ref session) = self.session {
            session.remove(name).await;
        }
    }

    /// Clear all attributes
    /// 清除所有属性
    pub async fn clear(&self) {
        if let Some(ref session) = self.session {
            session.clear().await;
        }
    }
}

/// Session manager
/// 会话管理器
///
/// Provides high-level session management operations.
/// 提供高级会话管理操作。
#[derive(Clone)]
pub struct SessionManager {
    /// Session store
    /// 会话存储
    store: Arc<dyn SessionStore>,

    /// Session configuration
    /// 会话配置
    config: SessionConfig,
}

impl SessionManager {
    /// Create new session manager
    /// 创建新的会话管理器
    pub fn new(store: Arc<dyn SessionStore>, config: SessionConfig) -> Self {
        Self { store, config }
    }

    /// Get session store
    /// 获取会话存储
    pub fn store(&self) -> &dyn SessionStore {
        &*self.store
    }

    /// Get session config
    /// 获取会话配置
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Load session from cookie
    /// 从Cookie加载会话
    pub async fn load_from_cookie(&self, cookie_value: &str) -> Result<Option<Session>, String> {
        let session_id = SessionId::from_string(cookie_value.to_string());
        self.store.get(&session_id).await
    }

    /// Create new session
    /// 创建新会话
    pub async fn create(&self) -> Result<Session, String> {
        self.store.create().await
    }

    /// Save session
    /// 保存会话
    pub async fn save(&self, session: &Session) -> Result<(), String> {
        self.store.save(session).await
    }

    /// Delete session
    /// 删除会话
    pub async fn delete(&self, session: &Session) -> Result<(), String> {
        self.store.delete(session.id()).await
    }

    /// Refresh session (create new ID, keep attributes)
    /// 刷新会话（创建新ID，保留属性）
    pub async fn refresh(&self, session: &Session) -> Result<Session, String> {
        // Create new session
        let new_session = self.store.create().await?;

        // Copy attributes from old session
        let names = session.attribute_names().await;
        for name in names {
            if let Some(value) = session.get::<String>(&name).await {
                new_session.set(&name, value).await;
            }
        }

        // Delete old session
        self.store.delete(session.id()).await?;

        Ok(new_session)
    }

    /// Clean up expired sessions
    /// 清理过期会话
    pub async fn cleanup_expired(&self) -> Result<usize, String> {
        self.store.cleanup_expired().await
    }
}

/// Session extractor for HTTP handlers
/// HTTP处理器的会话提取器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @GetMapping
/// public String home(@SessionAttribute String user) {
///     return "Welcome, " + user;
/// }
/// ```
#[derive(Clone)]
pub struct SessionExtractor {
    /// Session context
    /// 会话上下文
    context: SessionContext,
}

impl SessionExtractor {
    /// Create new session extractor
    /// 创建新的会话提取器
    pub fn new(context: SessionContext) -> Self {
        Self { context }
    }

    /// Get session
    /// 获取会话
    pub fn session(&self) -> Option<&Session> {
        self.context.session()
    }

    /// Require session (returns error if no session)
    /// 需要会话（如果没有会话则返回错误）
    pub fn require_session(&self) -> Result<&Session, String> {
        self.context.session().ok_or_else(|| "No session found".to_string())
    }

    /// Get session attribute
    /// 获取会话属性
    pub async fn get<T: Clone + 'static>(&self, name: &str) -> Result<T, String> {
        self.context
            .get(name)
            .await
            .ok_or_else(|| format!("Session attribute '{}' not found", name))
    }

    /// Get session attribute or default
    /// 获取会话属性或默认值
    pub async fn get_or_default<T: Clone + Default + 'static>(&self, name: &str) -> T {
        self.context.get(name).await.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemorySessionStore;

    #[tokio::test]
    async fn test_session_context() {
        let store = Arc::new(MemorySessionStore::new()) as Arc<dyn SessionStore>;
        let mut context = SessionContext::new(store.clone());

        assert!(!context.exists());

        // Create session
        let session = context.create().await.unwrap();
        assert!(context.exists());
        assert_eq!(context.session_id(), Some(session.id()));

        // Set and get attribute
        context.set("user_id", 123).await;
        assert_eq!(context.get::<i32>("user_id").await, Some(123));

        // Save session
        context.save().await.unwrap();

        // Invalidate session
        context.invalidate().await.unwrap();
        assert!(!context.exists());
    }

    #[tokio::test]
    async fn test_session_manager() {
        let store = Arc::new(MemorySessionStore::new()) as Arc<dyn SessionStore>;
        let config = SessionConfig::default();
        let manager = SessionManager::new(store.clone(), config);

        // Create session
        let session = manager.create().await.unwrap();
        session.set("test_key", "test_value".to_string()).await;

        // Save session
        manager.save(&session).await.unwrap();

        // Load session
        let loaded = manager
            .load_from_cookie(session.id().as_str())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(loaded.id(), session.id());
        assert_eq!(
            loaded.get::<String>("test_key").await,
            Some("test_value".to_string())
        );

        // Refresh session
        let refreshed = manager.refresh(&session).await.unwrap();
        assert_ne!(refreshed.id(), session.id());
        assert_eq!(
            refreshed.get::<String>("test_key").await,
            Some("test_value".to_string())
        );
    }
}
