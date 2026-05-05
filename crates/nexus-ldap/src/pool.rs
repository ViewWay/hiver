//! LDAP connection pooling / LDAP连接池
//!
//! Equivalent to Spring LDAP Pooling Support
//! 等价于 Spring LDAP 池化支持

use crate::context::LdapContextSource;
use crate::error::LdapResult;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Configuration for LDAP connection pool / LDAP连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_size: usize,
    pub max_idle: usize,
    pub min_idle: usize,
    pub max_wait_ms: u64,
    pub test_on_borrow: bool,
    pub test_on_return: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 8,
            max_idle: 4,
            min_idle: 1,
            max_wait_ms: 3000,
            test_on_borrow: false,
            test_on_return: false,
        }
    }
}

/// A connection in the pool / 池中的连接
#[derive(Debug)]
struct PooledConnection {
    active: bool,
}

/// LDAP connection pool / LDAP连接池
///
/// Manages a pool of LDAP context sources for efficient connection reuse.
/// 管理一个LDAP上下文源池以实现高效的连接重用。
pub struct LdapPool {
    config: PoolConfig,
    context_source: LdapContextSource,
    idle: Arc<Mutex<VecDeque<PooledConnection>>>,
    active_count: Arc<Mutex<usize>>,
}

impl LdapPool {
    /// Create a new LDAP pool / 创建新的LDAP池
    pub fn new(context_source: LdapContextSource, config: PoolConfig) -> Self {
        // Pre-create idle connections
        let mut idle = VecDeque::new();
        for _ in 0..config.min_idle {
            idle.push_back(PooledConnection { active: false });
        }

        Self {
            config,
            context_source,
            idle: Arc::new(Mutex::new(idle)),
            active_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Borrow a connection from the pool / 从池中借用连接
    pub fn borrow(&self) -> LdapResult<LdapContextSource> {
        let mut idle = self.idle.lock().unwrap();
        if let Some(conn) = idle.pop_front() {
            let _ = conn;
            *self.active_count.lock().unwrap() += 1;
            Ok(self.context_source.clone())
        } else {
            // Create new if under max
            let active = *self.active_count.lock().unwrap();
            if active < self.config.max_size {
                *self.active_count.lock().unwrap() += 1;
                Ok(self.context_source.clone())
            } else {
                Err(crate::error::LdapError::Connection("Pool exhausted".into()))
            }
        }
    }

    /// Return a connection to the pool / 将连接归还到池
    pub fn return_connection(&self, _conn: LdapContextSource) {
        let mut idle = self.idle.lock().unwrap();
        *self.active_count.lock().unwrap() -= 1;

        if idle.len() < self.config.max_idle {
            idle.push_back(PooledConnection { active: false });
        }
    }

    /// Get pool statistics / 获取池统计信息
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            active: *self.active_count.lock().unwrap(),
            idle: self.idle.lock().unwrap().len(),
            max_size: self.config.max_size,
        }
    }
}

/// Pool statistics / 池统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active: usize,
    pub idle: usize,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let pool = LdapPool::new(ctx, PoolConfig::default());
        let stats = pool.stats();
        assert_eq!(stats.idle, 1); // min_idle
        assert_eq!(stats.max_size, 8);
    }

    #[test]
    fn test_pool_borrow_return() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let pool = LdapPool::new(ctx, PoolConfig::default());
        let conn = pool.borrow().unwrap();
        assert!(pool.stats().active == 1);
        pool.return_connection(conn);
        assert!(pool.stats().active == 0);
    }
}
