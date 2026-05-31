//! Remember-Me authentication — persistent login via cookie tokens.
//! 记住我认证 —— 通过 Cookie 令牌实现持久登录。
//!
//! Equivalent to Spring Security's `RememberMeServices` + `TokenBasedRememberMeServices`.

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// A remember-me token stored in the repository.
/// 存储在仓库中的记住我令牌。
#[derive(Debug, Clone)]
pub struct RememberMeToken {
    /// Random series identifier — ties all tokens for a login session.
    pub series: String,
    /// Random token value — rotated on each successful authentication.
    pub value: String,
    /// The username this token belongs to.
    pub username: String,
    /// Last time this token was used.
    pub last_used: DateTime<Utc>,
}

impl RememberMeToken {
    /// Encode token as `series:token` for a cookie value.
    /// 将令牌编码为 series:token 格式的 Cookie 值。
    pub fn to_cookie_value(&self) -> String {
        format!("{}:{}", self.series, self.value)
    }

    /// Parse a cookie value back into (series, value).
    /// 从 Cookie 值解析出 (series, value)。
    pub fn from_cookie_value(cookie: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = cookie.splitn(2, ':').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Refresh with a new random value, updating last_used.
    /// 用新的随机值刷新令牌，更新 last_used。
    pub fn refresh(&mut self) {
        self.value = random_hex(16);
        self.last_used = Utc::now();
    }
}

/// In-memory token repository.
/// 内存令牌仓库。
pub struct InMemoryTokenRepository {
    tokens: RwLock<HashMap<String, RememberMeToken>>,
}

impl InMemoryTokenRepository {
    /// Create an empty repository.
    /// 创建空仓库。
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Save a new token.
    /// 保存新令牌。
    pub fn save(&self, token: RememberMeToken) {
        self.tokens
            .write()
            .unwrap()
            .insert(token.series.clone(), token);
    }

    /// Find a token by its series identifier.
    /// 通过 series 标识查找令牌。
    pub fn find_by_series(&self, series: &str) -> Option<RememberMeToken> {
        self.tokens.read().unwrap().get(series).cloned()
    }

    /// Update an existing token.
    /// 更新现有令牌。
    pub fn update(&self, token: &RememberMeToken) {
        self.tokens
            .write()
            .unwrap()
            .insert(token.series.clone(), token.clone());
    }

    /// Remove a token by series.
    /// 通过 series 移除令牌。
    pub fn remove(&self, series: &str) {
        self.tokens.write().unwrap().remove(series);
    }

    /// Remove all tokens for a given user.
    /// 移除指定用户的所有令牌。
    pub fn remove_user_tokens(&self, username: &str) {
        self.tokens
            .write()
            .unwrap()
            .retain(|_, t| t.username != username);
    }
}

impl Default for InMemoryTokenRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for Remember-Me services.
/// 记住我服务的配置。
pub struct RememberMeConfig {
    /// Cookie name (default: "remember-me").
    pub cookie_name: String,
    /// How long the token is valid, in seconds.
    pub token_validity_secs: u64,
    /// Whether to set the Secure flag on the cookie.
    pub secure_cookie: bool,
    /// HMAC key for token-based signatures.
    pub key: String,
}

impl Default for RememberMeConfig {
    fn default() -> Self {
        Self {
            cookie_name: "remember-me".to_string(),
            token_validity_secs: 14 * 24 * 3600, // 2 weeks
            secure_cookie: true,
            key: "hiver-remember-me-key".to_string(),
        }
    }
}

impl RememberMeConfig {
    /// Create config with a custom key.
    /// 用自定义密钥创建配置。
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }
}

/// Remember-Me authentication services.
/// 记住我认证服务。
///
/// Equivalent to Spring Security's `TokenBasedRememberMeServices` when using
/// the token-based strategy, or `PersistentTokenBasedRememberMeServices` when
/// using the persistent repository.
pub struct RememberMeServices {
    config: RememberMeConfig,
    repository: InMemoryTokenRepository,
}

impl RememberMeServices {
    /// Create new services with default config.
    /// 用默认配置创建服务。
    pub fn new(config: RememberMeConfig) -> Self {
        Self {
            config,
            repository: InMemoryTokenRepository::new(),
        }
    }

    /// Called after a successful interactive login — creates a new persistent token.
    /// 交互式登录成功后调用 — 创建新的持久令牌。
    pub fn login_success(&self, username: &str) -> RememberMeToken {
        let token = RememberMeToken {
            series: random_hex(16),
            value: random_hex(16),
            username: username.to_string(),
            last_used: Utc::now(),
        };
        self.repository.save(token.clone());
        token
    }

    /// Attempt auto-login from a cookie value.
    /// 尝试从 Cookie 值自动登录。
    ///
    /// Returns the username if valid, or `None` if the token is missing,
    /// expired, or stolen (series found but value mismatch → all tokens removed).
    pub fn auto_login(&self, cookie_value: &str) -> Option<String> {
        let (series, value) = RememberMeToken::from_cookie_value(cookie_value)?;

        let mut token = self.repository.find_by_series(&series)?;

        // Token value mismatch → potential theft; invalidate all user tokens
        // 令牌值不匹配 → 可能被盗；清除该用户所有令牌
        if token.value != value {
            let username = token.username.clone();
            self.repository.remove_user_tokens(&username);
            tracing::warn!(
                "Remember-me token mismatch for user={}, series={}. Possible token theft.",
                username,
                series
            );
            return None;
        }

        // Check expiry
        // 检查过期
        let elapsed = Utc::now()
            .signed_duration_since(token.last_used)
            .num_seconds();
        if elapsed < 0 || elapsed as u64 > self.config.token_validity_secs {
            self.repository.remove(&series);
            return None;
        }

        // Rotate token value
        // 轮换令牌值
        let username = token.username.clone();
        token.refresh();
        self.repository.update(&token);

        Some(username)
    }

    /// Logout — remove the token identified by the cookie.
    /// 登出 — 移除 Cookie 对应的令牌。
    pub fn logout(&self, cookie_value: &str) {
        if let Some((series, _)) = RememberMeToken::from_cookie_value(cookie_value) {
            self.repository.remove(&series);
        }
    }

    /// Generate a token-based signature (HMAC-SHA256).
    /// 生成基于令牌的签名 (HMAC-SHA256)。
    pub fn hash_token(&self, token: &RememberMeToken) -> String {
        let data = format!(
            "{}:{}:{}:{}",
            token.username, token.series, token.value, self.config.key
        );
        let mut mac = HmacSha256::new_from_slice(self.config.key.as_bytes())
            .expect("HMAC key length is valid");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}

/// Generate `n` random bytes as a hex string.
/// 生成 n 个随机字节的十六进制字符串。
fn random_hex(n: usize) -> String {
    let mut buf = vec![0u8; n];
    rand::rng().fill_bytes(&mut buf);
    hex::encode(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_roundtrip() {
        let token = RememberMeToken {
            series: "abc123".to_string(),
            value: "def456".to_string(),
            username: "alice".to_string(),
            last_used: Utc::now(),
        };
        let cookie = token.to_cookie_value();
        let (series, value) = RememberMeToken::from_cookie_value(&cookie).unwrap();
        assert_eq!(series, "abc123");
        assert_eq!(value, "def456");
    }

    #[test]
    fn test_invalid_cookie_values() {
        assert!(RememberMeToken::from_cookie_value("").is_none());
        assert!(RememberMeToken::from_cookie_value(":").is_none());
        assert!(RememberMeToken::from_cookie_value("onlyseries:").is_none());
        assert!(RememberMeToken::from_cookie_value(":onlyvalue").is_none());
    }

    #[test]
    fn test_token_refresh() {
        let mut token = RememberMeToken {
            series: "s".to_string(),
            value: "v".to_string(),
            username: "bob".to_string(),
            last_used: Utc::now(),
        };
        let old_value = token.value.clone();
        token.refresh();
        assert_ne!(token.value, old_value);
        assert_eq!(token.series, "s");
    }

    #[test]
    fn test_repository_crud() {
        let repo = InMemoryTokenRepository::new();
        let token = RememberMeToken {
            series: "s1".to_string(),
            value: "v1".to_string(),
            username: "alice".to_string(),
            last_used: Utc::now(),
        };
        repo.save(token.clone());
        assert!(repo.find_by_series("s1").is_some());
        assert!(repo.find_by_series("s2").is_none());

        let mut found = repo.find_by_series("s1").unwrap();
        found.value = "v2".to_string();
        repo.update(&found);
        assert_eq!(repo.find_by_series("s1").unwrap().value, "v2");

        repo.remove("s1");
        assert!(repo.find_by_series("s1").is_none());
    }

    #[test]
    fn test_remove_user_tokens() {
        let repo = InMemoryTokenRepository::new();
        repo.save(RememberMeToken {
            series: "s1".to_string(),
            value: "v1".to_string(),
            username: "alice".to_string(),
            last_used: Utc::now(),
        });
        repo.save(RememberMeToken {
            series: "s2".to_string(),
            value: "v2".to_string(),
            username: "alice".to_string(),
            last_used: Utc::now(),
        });
        repo.save(RememberMeToken {
            series: "s3".to_string(),
            value: "v3".to_string(),
            username: "bob".to_string(),
            last_used: Utc::now(),
        });
        repo.remove_user_tokens("alice");
        assert!(repo.find_by_series("s1").is_none());
        assert!(repo.find_by_series("s2").is_none());
        assert!(repo.find_by_series("s3").is_some());
    }

    #[test]
    fn test_login_auto_logout_cycle() {
        let services = RememberMeServices::new(RememberMeConfig::default());

        // Login
        let token = services.login_success("alice");
        let cookie = token.to_cookie_value();

        // Auto-login succeeds
        let username = services.auto_login(&cookie).unwrap();
        assert_eq!(username, "alice");

        // After auto-login, token was rotated; old cookie no longer works
        assert!(services.auto_login(&cookie).is_none());
    }

    #[test]
    fn test_token_theft_detection() {
        let services = RememberMeServices::new(RememberMeConfig::default());
        let token = services.login_success("alice");

        // Simulate auto-login (rotates value)
        let cookie1 = token.to_cookie_value();
        let _ = services.auto_login(&cookie1);

        // Get the new token value after rotation
        let new_token = services.repository.find_by_series(&token.series).unwrap();
        let cookie2 = new_token.to_cookie_value();

        // Old cookie value (pre-rotation) should fail and trigger theft detection
        let result = services.auto_login(&cookie1);
        assert!(result.is_none());

        // Even the valid new cookie should fail now — all user tokens removed
        let result2 = services.auto_login(&cookie2);
        assert!(result2.is_none());
    }

    #[test]
    fn test_logout() {
        let services = RememberMeServices::new(RememberMeConfig::default());
        let token = services.login_success("alice");
        let cookie = token.to_cookie_value();

        services.logout(&cookie);
        assert!(services.auto_login(&cookie).is_none());
    }

    #[test]
    fn test_hash_token() {
        let config = RememberMeConfig::default().with_key("test-key");
        let services = RememberMeServices::new(config);
        let token = RememberMeToken {
            series: "s".to_string(),
            value: "v".to_string(),
            username: "u".to_string(),
            last_used: Utc::now(),
        };
        let hash1 = services.hash_token(&token);
        let hash2 = services.hash_token(&token);
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}
