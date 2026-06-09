//! Session events and listeners
//! 会话事件和监听器
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Session |
//! |-------|---------------|
//! | `SessionEvent` | `SessionEvent` |
//! | `SessionEventListener` | `ApplicationListener<SessionEvent>` |
//! | `SessionEventPublisher` | `ApplicationEventPublisher` |
//! | `ConcurrentSessionControl` | `ConcurrentSessionControl` |
//!
//! Provides event-driven session management with listeners for session
//! lifecycle events, and concurrent session control to limit sessions per user.
//!
//! 提供事件驱动的会话管理，带有会话生命周期事件的监听器，
//! 以及并发会话控制以限制每个用户的会话数。
//!
//! # Example / 示例
//!
//! ```rust,ignore
//! use hiver_session::events::{SessionEventPublisher, SessionEventListener, LoggingSessionListener};
//!
//! let publisher = SessionEventPublisher::new();
//! publisher.add_listener(Arc::new(LoggingSessionListener));
//!
//! // Events are emitted automatically by the session store
//! publisher.emit_created(session_id.clone(), "user123").await;
//! ```

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::SessionId;

// ──────────────────────────────────────────────────────────────────────────────
// Session Event
// ──────────────────────────────────────────────────────────────────────────────

/// Session lifecycle events.
/// 会话生命周期事件。
///
/// Equivalent to Spring Session's `SessionEvent`.
/// 等价于 Spring Session 的 `SessionEvent`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// // Spring Session events
/// SessionCreatedEvent
/// SessionDestroyedEvent
/// SessionExpiredEvent
/// SessionDeletedEvent
/// ```
#[derive(Debug, Clone)]
pub enum SessionEvent
{
    /// A new session was created.
    /// 新会话已创建。
    Created
    {
        /// The session ID.
        /// 会话 ID。
        session_id: SessionId,
        /// The user principal associated with the session, if known.
        /// 与会话关联的用户主体（如果已知）。
        principal: Option<String>,
    },

    /// A session has expired due to inactivity.
    /// 会话因不活动而过期。
    Expired
    {
        /// The session ID.
        /// 会话 ID。
        session_id: SessionId,
        /// The user principal.
        /// 用户主体。
        principal: Option<String>,
    },

    /// A session was explicitly destroyed (invalidated).
    /// 会话被显式销毁（失效）。
    Destroyed
    {
        /// The session ID.
        /// 会话 ID。
        session_id: SessionId,
        /// The user principal.
        /// 用户主体。
        principal: Option<String>,
    },

    /// A session attribute was changed.
    /// 会话属性已更改。
    AttributeChanged
    {
        /// The session ID.
        /// 会话 ID。
        session_id: SessionId,
        /// The name of the changed attribute.
        /// 已更改的属性名称。
        attribute_name: String,
    },

    /// The maximum number of concurrent sessions for a user was exceeded.
    /// 超出了用户的最大并发会话数。
    MaxSessionsExceeded
    {
        /// The user principal.
        /// 用户主体。
        principal: String,
        /// The current number of sessions for this user.
        /// 此用户的当前会话数。
        current_count: usize,
        /// The maximum allowed sessions.
        /// 允许的最大会话数。
        max_allowed: usize,
    },
}

impl SessionEvent
{
    /// Get the session ID associated with this event, if any.
    /// 获取与此事件关联的会话 ID（如果有）。
    pub fn session_id(&self) -> Option<&SessionId>
    {
        match self
        {
            SessionEvent::Created { session_id, .. }
            | SessionEvent::Expired { session_id, .. }
            | SessionEvent::Destroyed { session_id, .. }
            | SessionEvent::AttributeChanged { session_id, .. } => Some(session_id),
            SessionEvent::MaxSessionsExceeded { .. } => None,
        }
    }

    /// Get the principal associated with this event, if any.
    /// 获取与此事件关联的主体（如果有）。
    pub fn principal(&self) -> Option<&str>
    {
        match self
        {
            SessionEvent::Created { principal, .. }
            | SessionEvent::Expired { principal, .. }
            | SessionEvent::Destroyed { principal, .. } => principal.as_deref(),
            SessionEvent::AttributeChanged { .. } => None,
            SessionEvent::MaxSessionsExceeded { principal, .. } => Some(principal),
        }
    }

    /// Get a human-readable event name.
    /// 获取人类可读的事件名称。
    pub fn event_name(&self) -> &'static str
    {
        match self
        {
            SessionEvent::Created { .. } => "SessionCreated",
            SessionEvent::Expired { .. } => "SessionExpired",
            SessionEvent::Destroyed { .. } => "SessionDestroyed",
            SessionEvent::AttributeChanged { .. } => "SessionAttributeChanged",
            SessionEvent::MaxSessionsExceeded { .. } => "MaxSessionsExceeded",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Session Event Listener
// ──────────────────────────────────────────────────────────────────────────────

/// Trait for listening to session events.
/// 监听会话事件的 trait。
///
/// Equivalent to Spring's `ApplicationListener<SessionEvent>`.
/// 等价于 Spring 的 `ApplicationListener<SessionEvent>`。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_session::events::{SessionEventListener, SessionEvent};
///
/// struct AuditListener;
///
/// #[async_trait]
/// impl SessionEventListener for AuditListener {
///     async fn on_event(&self, event: &SessionEvent) {
///         println!("Session event: {}", event.event_name());
///     }
/// }
/// ```
#[async_trait]
pub trait SessionEventListener: Send + Sync
{
    /// Called when a session event occurs.
    /// 当会话事件发生时调用。
    async fn on_event(&self, event: &SessionEvent);
}

// ──────────────────────────────────────────────────────────────────────────────
// Session Event Publisher
// ──────────────────────────────────────────────────────────────────────────────

/// Publisher for session events. Manages a list of listeners and dispatches events.
/// 会话事件发布器。管理监听器列表并分发事件。
///
/// Equivalent to Spring's `ApplicationEventPublisher`.
/// 等价于 Spring 的 `ApplicationEventPublisher`。
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_session::events::{SessionEventPublisher, LoggingSessionListener};
///
/// let publisher = SessionEventPublisher::new();
/// publisher.add_listener(Arc::new(LoggingSessionListener));
///
/// publisher.emit_created(session_id, Some("user123".to_string())).await;
/// ```
#[derive(Clone, Default)]
pub struct SessionEventPublisher
{
    listeners: Arc<RwLock<Vec<Arc<dyn SessionEventListener>>>>,
}

impl SessionEventPublisher
{
    /// Create a new event publisher with no listeners.
    /// 创建没有监听器的新事件发布器。
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Add a listener to the publisher.
    /// 向发布器添加监听器。
    pub fn add_listener(&self, listener: Arc<dyn SessionEventListener>)
    {
        // We need to block on this since it's a sync method adding to an async structure.
        // In practice this is called during setup, not in hot paths.
        let listeners = self.listeners.clone();
        // Use try_write to avoid blocking; if we can't write, we log and skip.
        if let Ok(mut guard) = listeners.try_write()
        {
            guard.push(listener);
        }
    }

    /// Remove all listeners.
    /// 移除所有监听器。
    pub fn clear_listeners(&self)
    {
        if let Ok(mut guard) = self.listeners.try_write()
        {
            guard.clear();
        }
    }

    /// Get the number of registered listeners.
    /// 获取已注册的监听器数量。
    pub fn listener_count(&self) -> usize
    {
        self.listeners.try_read().map_or(0, |g| g.len())
    }

    /// Emit a session event to all registered listeners.
    /// 向所有已注册的监听器发出会话事件。
    pub async fn emit(&self, event: SessionEvent)
    {
        let listeners = self.listeners.read().await;
        for listener in listeners.iter()
        {
            listener.on_event(&event).await;
        }
    }

    /// Emit a session created event.
    /// 发出会话创建事件。
    pub async fn emit_created(&self, session_id: SessionId, principal: Option<String>)
    {
        self.emit(SessionEvent::Created {
            session_id,
            principal,
        })
        .await;
    }

    /// Emit a session expired event.
    /// 发出会话过期事件。
    pub async fn emit_expired(&self, session_id: SessionId, principal: Option<String>)
    {
        self.emit(SessionEvent::Expired {
            session_id,
            principal,
        })
        .await;
    }

    /// Emit a session destroyed event.
    /// 发出会话销毁事件。
    pub async fn emit_destroyed(&self, session_id: SessionId, principal: Option<String>)
    {
        self.emit(SessionEvent::Destroyed {
            session_id,
            principal,
        })
        .await;
    }

    /// Emit an attribute changed event.
    /// 发出属性更改事件。
    pub async fn emit_attribute_changed(&self, session_id: SessionId, attribute_name: String)
    {
        self.emit(SessionEvent::AttributeChanged {
            session_id,
            attribute_name,
        })
        .await;
    }

    /// Emit a max sessions exceeded event.
    /// 发出最大会话超出事件。
    pub async fn emit_max_sessions_exceeded(
        &self,
        principal: String,
        current_count: usize,
        max_allowed: usize,
    )
    {
        self.emit(SessionEvent::MaxSessionsExceeded {
            principal,
            current_count,
            max_allowed,
        })
        .await;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Built-in Listeners
// ──────────────────────────────────────────────────────────────────────────────

/// Logging session event listener — logs all session events.
/// 日志会话事件监听器 — 记录所有会话事件。
///
/// Equivalent to Spring's default session event logging.
/// 等价于 Spring 的默认会话事件日志记录。
pub struct LoggingSessionListener;

#[async_trait]
impl SessionEventListener for LoggingSessionListener
{
    async fn on_event(&self, event: &SessionEvent)
    {
        match event
        {
            SessionEvent::Created {
                session_id,
                principal,
            } =>
            {
                println!(
                    "[Session] Created: {} (principal: {})",
                    session_id,
                    principal.as_deref().unwrap_or("anonymous")
                );
            },
            SessionEvent::Expired {
                session_id,
                principal,
            } =>
            {
                println!(
                    "[Session] Expired: {} (principal: {})",
                    session_id,
                    principal.as_deref().unwrap_or("anonymous")
                );
            },
            SessionEvent::Destroyed {
                session_id,
                principal,
            } =>
            {
                println!(
                    "[Session] Destroyed: {} (principal: {})",
                    session_id,
                    principal.as_deref().unwrap_or("anonymous")
                );
            },
            SessionEvent::AttributeChanged {
                session_id,
                attribute_name,
            } =>
            {
                println!("[Session] Attribute changed: {} [{}]", session_id, attribute_name);
            },
            SessionEvent::MaxSessionsExceeded {
                principal,
                current_count,
                max_allowed,
            } =>
            {
                println!(
                    "[Session] Max sessions exceeded for {}: {}/{}",
                    principal, current_count, max_allowed
                );
            },
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Concurrent Session Control
// ──────────────────────────────────────────────────────────────────────────────

/// Strategy for handling concurrent session limits.
/// 处理并发会话限制的策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrentSessionStrategy
{
    /// Prevent new session creation when limit is reached.
    /// 达到限制时阻止创建新会话。
    PreventNew,

    /// Expire the oldest session when a new one is created.
    /// 创建新会话时使最旧的会话过期。
    ExpireOldest,
}

/// Concurrent session control — limits the number of sessions per user.
/// 并发会话控制 — 限制每个用户的会话数。
///
/// Equivalent to Spring Security's concurrent session control.
/// 等价于 Spring Security 的并发会话控制。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public HttpSessionEventPublisher httpSessionEventPublisher() {
///     return new HttpSessionEventPublisher();
/// }
///
/// http.sessionManagement()
///     .maximumSessions(1)
///     .maxSessionsPreventsLogin(true)
///     .expiredSessionStrategy(new SimpleRedirectSessionInformationExpiredStrategy());
/// ```
///
/// # Example / 示例
///
/// ```rust,ignore
/// use hiver_session::events::{ConcurrentSessionControl, ConcurrentSessionStrategy};
///
/// let control = ConcurrentSessionControl::new(3)
///     .with_strategy(ConcurrentSessionStrategy::ExpireOldest);
///
/// // Check before creating session
/// let allowed = control.check_and_register("user123", session_id).await;
/// if !allowed {
///     // Handle max sessions reached
/// }
/// ```
#[derive(Clone)]
pub struct ConcurrentSessionControl
{
    /// Maximum sessions per user.
    /// 每个用户的最大会话数。
    max_sessions: usize,

    /// Strategy when limit is reached.
    /// 达到限制时的策略。
    strategy: ConcurrentSessionStrategy,

    /// Map of principal -> list of session IDs.
    /// 主体 -> 会话 ID 列表的映射。
    user_sessions: Arc<RwLock<HashMap<String, Vec<SessionId>>>>,

    /// Optional event publisher.
    /// 可选的事件发布器。
    publisher: Option<SessionEventPublisher>,
}

impl ConcurrentSessionControl
{
    /// Create a new concurrent session control with the given max sessions.
    /// 创建具有给定最大会话数的并发会话控制。
    pub fn new(max_sessions: usize) -> Self
    {
        Self {
            max_sessions,
            strategy: ConcurrentSessionStrategy::PreventNew,
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            publisher: None,
        }
    }

    /// Set the concurrent session strategy.
    /// 设置并发会话策略。
    pub fn with_strategy(mut self, strategy: ConcurrentSessionStrategy) -> Self
    {
        self.strategy = strategy;
        self
    }

    /// Set the event publisher.
    /// 设置事件发布器。
    pub fn with_publisher(mut self, publisher: SessionEventPublisher) -> Self
    {
        self.publisher = Some(publisher);
        self
    }

    /// Get the max sessions limit.
    /// 获取最大会话限制。
    pub fn max_sessions(&self) -> usize
    {
        self.max_sessions
    }

    /// Get the current session count for a user.
    /// 获取用户的当前会话数。
    pub async fn session_count(&self, principal: &str) -> usize
    {
        let sessions = self.user_sessions.read().await;
        sessions.get(principal).map_or(0, Vec::len)
    }

    /// Get all session IDs for a user.
    /// 获取用户的所有会话 ID。
    pub async fn sessions_for(&self, principal: &str) -> Vec<SessionId>
    {
        let sessions = self.user_sessions.read().await;
        sessions.get(principal).cloned().unwrap_or_default()
    }

    /// Check if a user can create a new session, and register it if allowed.
    /// 检查用户是否可以创建新会话，如果允许则注册。
    ///
    /// Returns `true` if the session was registered, `false` if the limit
    /// was reached and the strategy is `PreventNew`.
    ///
    /// 如果会话已注册则返回 `true`，如果达到限制且策略为 `PreventNew` 则返回 `false`。
    pub async fn check_and_register(&self, principal: &str, new_session_id: SessionId) -> bool
    {
        // Phase 1: Determine action under lock
        let action = {
            let mut sessions = self.user_sessions.write().await;
            let user_sessions = sessions.entry(principal.to_string()).or_default();

            if user_sessions.len() >= self.max_sessions
            {
                match self.strategy
                {
                    ConcurrentSessionStrategy::PreventNew =>
                    {
                        let current_count = user_sessions.len();
                        drop(sessions);
                        if let Some(ref publisher) = self.publisher
                        {
                            publisher
                                .emit_max_sessions_exceeded(
                                    principal.to_string(),
                                    current_count,
                                    self.max_sessions,
                                )
                                .await;
                        }
                        return false;
                    },
                    ConcurrentSessionStrategy::ExpireOldest =>
                    {
                        // Remove the oldest session
                        let expired = if user_sessions.is_empty()
                        {
                            None
                        }
                        else
                        {
                            let oldest = user_sessions.remove(0);
                            Some(oldest)
                        };
                        user_sessions.push(new_session_id);
                        expired
                    },
                }
            }
            else
            {
                user_sessions.push(new_session_id);
                None
            }
        };

        // Phase 2: Emit events outside the lock
        if let Some(expired_id) = action
            && let Some(ref publisher) = self.publisher
        {
            publisher
                .emit_expired(expired_id, Some(principal.to_string()))
                .await;
        }

        true
    }

    /// Remove a session for a user (e.g., on logout or expiration).
    /// 移除用户的会话（例如注销或过期时）。
    pub async fn remove_session(&self, principal: &str, session_id: &SessionId)
    {
        let mut sessions = self.user_sessions.write().await;
        if let Some(user_sessions) = sessions.get_mut(principal)
        {
            user_sessions.retain(|id| id != session_id);
            if user_sessions.is_empty()
            {
                sessions.remove(principal);
            }
        }
    }

    /// Clean up all sessions for a user.
    /// 清理用户的所有会话。
    pub async fn clear_user(&self, principal: &str)
    {
        let mut sessions = self.user_sessions.write().await;
        sessions.remove(principal);
    }

    /// Get the total number of tracked users.
    /// 获取跟踪的用户总数。
    pub async fn tracked_user_count(&self) -> usize
    {
        let sessions = self.user_sessions.read().await;
        sessions.len()
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
mod tests
{
    use super::*;

    #[test]
    fn test_session_event_name()
    {
        let id = SessionId::new();
        assert_eq!(
            SessionEvent::Created {
                session_id: id.clone(),
                principal: None
            }
            .event_name(),
            "SessionCreated"
        );
        assert_eq!(
            SessionEvent::Expired {
                session_id: id.clone(),
                principal: None
            }
            .event_name(),
            "SessionExpired"
        );
        assert_eq!(
            SessionEvent::Destroyed {
                session_id: id.clone(),
                principal: None
            }
            .event_name(),
            "SessionDestroyed"
        );
        assert_eq!(
            SessionEvent::AttributeChanged {
                session_id: id,
                attribute_name: "key".to_string()
            }
            .event_name(),
            "SessionAttributeChanged"
        );
        assert_eq!(
            SessionEvent::MaxSessionsExceeded {
                principal: "user".to_string(),
                current_count: 3,
                max_allowed: 2
            }
            .event_name(),
            "MaxSessionsExceeded"
        );
    }

    #[test]
    fn test_session_event_principal()
    {
        let id = SessionId::new();
        let event = SessionEvent::Created {
            session_id: id,
            principal: Some("user123".to_string()),
        };
        assert_eq!(event.principal(), Some("user123"));
    }

    #[tokio::test]
    async fn test_event_publisher()
    {
        let publisher = SessionEventPublisher::new();
        assert_eq!(publisher.listener_count(), 0);

        publisher.add_listener(Arc::new(LoggingSessionListener));
        assert_eq!(publisher.listener_count(), 1);

        // Emit should not panic
        publisher
            .emit_created(SessionId::new(), Some("user".to_string()))
            .await;
    }

    #[tokio::test]
    async fn test_concurrent_session_control_prevent_new()
    {
        let control =
            ConcurrentSessionControl::new(2).with_strategy(ConcurrentSessionStrategy::PreventNew);

        let s1 = SessionId::new();
        let s2 = SessionId::new();
        let s3 = SessionId::new();

        assert!(control.check_and_register("user1", s1).await);
        assert!(control.check_and_register("user1", s2).await);
        assert!(!control.check_and_register("user1", s3).await); // Exceeded

        assert_eq!(control.session_count("user1").await, 2);
    }

    #[tokio::test]
    async fn test_concurrent_session_control_expire_oldest()
    {
        let control =
            ConcurrentSessionControl::new(2).with_strategy(ConcurrentSessionStrategy::ExpireOldest);

        let s1 = SessionId::new();
        let s2 = SessionId::new();
        let s3 = SessionId::new();
        let s1_clone = s1.clone();

        assert!(control.check_and_register("user1", s1).await);
        assert!(control.check_and_register("user1", s2).await);
        assert!(control.check_and_register("user1", s3).await); // Oldest expired

        assert_eq!(control.session_count("user1").await, 2);

        let sessions = control.sessions_for("user1").await;
        assert!(!sessions.contains(&s1_clone)); // s1 was expired
    }

    #[tokio::test]
    async fn test_concurrent_session_remove()
    {
        let control = ConcurrentSessionControl::new(5);

        let s1 = SessionId::new();
        let s2 = SessionId::new();

        control.check_and_register("user1", s1.clone()).await;
        control.check_and_register("user1", s2.clone()).await;
        assert_eq!(control.session_count("user1").await, 2);

        control.remove_session("user1", &s1).await;
        assert_eq!(control.session_count("user1").await, 1);

        control.remove_session("user1", &s2).await;
        assert_eq!(control.session_count("user1").await, 0);
        assert_eq!(control.tracked_user_count().await, 0);
    }

    #[tokio::test]
    async fn test_concurrent_session_multiple_users()
    {
        let control = ConcurrentSessionControl::new(1);

        let s1 = SessionId::new();
        let s2 = SessionId::new();

        assert!(control.check_and_register("user1", s1).await);
        assert!(control.check_and_register("user2", s2).await);

        assert_eq!(control.session_count("user1").await, 1);
        assert_eq!(control.session_count("user2").await, 1);
        assert_eq!(control.tracked_user_count().await, 2);
    }

    #[tokio::test]
    async fn test_concurrent_session_with_publisher()
    {
        let publisher = SessionEventPublisher::new();
        publisher.add_listener(Arc::new(LoggingSessionListener));

        let control = ConcurrentSessionControl::new(1)
            .with_strategy(ConcurrentSessionStrategy::PreventNew)
            .with_publisher(publisher);

        let s1 = SessionId::new();
        let s2 = SessionId::new();

        assert!(control.check_and_register("user1", s1).await);
        assert!(!control.check_and_register("user1", s2).await);
    }
}
