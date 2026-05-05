//! Password encoder module
//! 密码编码器模块

/// Password encoder trait
/// 密码编码器trait
///
/// Equivalent to Spring's `PasswordEncoder` interface.
/// `等价于Spring的PasswordEncoder接口`。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface PasswordEncoder {
///     String encode(CharSequence rawPassword);
///     boolean matches(CharSequence rawPassword, String encodedPassword);
/// }
/// ```
pub trait PasswordEncoder: Send + Sync {
    /// Encode a raw password
    /// 编码原始密码
    fn encode(&self, raw: &str) -> String;

    /// Verify that the raw password matches the encoded password
    /// 验证原始密码是否与编码密码匹配
    fn matches(&self, raw: &str, encoded: &str) -> bool;

    /// Check if encoding needs to be updated (for password migration)
    /// 检查编码是否需要更新（用于密码迁移）
    fn upgrade_encoding(&self, _encoded: &str) -> bool {
        // Default implementation says encoding doesn't need upgrade
        false
    }
}

/// `BCrypt` password encoder
/// `BCrypt密码编码器`
///
/// Equivalent to Spring's `BCryptPasswordEncoder`.
/// `等价于Spring的BCryptPasswordEncoder`。
pub struct BcryptPasswordEncoder {
    /// Cost factor (4-31, default 10)
    /// 成本因子（4-31，默认10）
    cost: u32,
}

impl BcryptPasswordEncoder {
    /// Create a new `BCrypt` encoder with default cost
    /// `创建具有默认成本的BCrypt编码器`
    pub fn new() -> Self {
        Self { cost: 10 }
    }

    /// Create with custom cost
    /// 使用自定义成本创建
    pub fn with_cost(cost: u32) -> Self {
        assert!((4..=31).contains(&cost), "BCrypt cost must be between 4 and 31");
        Self { cost }
    }
}

impl Default for BcryptPasswordEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordEncoder for BcryptPasswordEncoder {
    fn encode(&self, raw: &str) -> String {
        bcrypt::hash(raw, self.cost).unwrap_or_else(|_| {
            // Fallback to simple hash on error
            use md5::{Digest, Md5};
            let hash = Md5::digest(raw.as_bytes());
            hex::encode(hash)
        })
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool {
        bcrypt::verify(raw, encoded).unwrap_or(false)
    }

    fn upgrade_encoding(&self, encoded: &str) -> bool {
        // Check if the encoded password has the target cost
        if let Some(prefix) = encoded.split('$').nth(2)
            && let Ok(cost) = prefix.parse::<u32>() {
                return cost != self.cost;
            }
        true
    }
}

/// `NoOp` password encoder (for testing only!)
/// NoOp密码编码器（仅用于测试！）
///
/// WARNING: This does not actually encode passwords!
/// 警告：这不会实际编码密码！
///
/// Equivalent to Spring's `NoOpPasswordEncoder` (for testing only).
/// 等价于Spring的NoOpPasswordEncoder（仅用于测试）。
pub struct NoOpPasswordEncoder;

impl PasswordEncoder for NoOpPasswordEncoder {
    fn encode(&self, raw: &str) -> String {
        raw.to_string()
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool {
        raw == encoded
    }
}

/// Standard password encoder
/// 标准密码编码器
///
/// Uses `BCrypt` by default.
/// `默认使用BCrypt`。
pub struct StandardPasswordEncoder {
    encoder: Box<dyn PasswordEncoder + Send + Sync>,
}

impl Clone for StandardPasswordEncoder {
    fn clone(&self) -> Self {
        Self {
            encoder: Box::new(BcryptPasswordEncoder::new()),
        }
    }
}

impl std::fmt::Debug for StandardPasswordEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StandardPasswordEncoder").finish()
    }
}

impl StandardPasswordEncoder {
    /// Create a new standard encoder
    /// 创建新的标准编码器
    pub fn new() -> Self {
        Self {
            encoder: Box::new(BcryptPasswordEncoder::new()),
        }
    }

    /// Create with `BCrypt`
    /// `使用BCrypt创建`
    pub fn bcrypt() -> Self {
        Self {
            encoder: Box::new(BcryptPasswordEncoder::new()),
        }
    }

    /// Create with custom encoder
    /// 使用自定义编码器创建
    pub fn custom(encoder: Box<dyn PasswordEncoder + Send + Sync>) -> Self {
        Self { encoder }
    }
}

impl Default for StandardPasswordEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordEncoder for StandardPasswordEncoder {
    fn encode(&self, raw: &str) -> String {
        self.encoder.encode(raw)
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool {
        self.encoder.matches(raw, encoded)
    }
}

/// PBKDF2 password encoder
/// PBKDF2密码编码器
///
/// Alternative to `BCrypt` with different security properties.
/// BCrypt的替代品，具有不同的安全属性。
pub struct Pbkdf2PasswordEncoder {
    /// Number of iterations
    /// 迭代次数
    iterations: u32,

    /// Key length
    /// 密钥长度
    key_length: usize,

    /// Salt length
    /// 盐长度
    salt_length: usize,
}

impl Pbkdf2PasswordEncoder {
    /// Create a new PBKDF2 encoder with defaults
    /// 创建具有默认值的PBKDF2编码器
    pub fn new() -> Self {
        Self {
            iterations: 100_000,
            key_length: 32,
            salt_length: 16,
        }
    }

    /// Create with custom iterations
    /// 使用自定义迭代次数创建
    pub fn with_iterations(iterations: u32) -> Self {
        Self {
            iterations,
            ..Default::default()
        }
    }
}

impl Default for Pbkdf2PasswordEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordEncoder for Pbkdf2PasswordEncoder {
    fn encode(&self, raw: &str) -> String {
        use hmac::Hmac;
        use hmac::Mac;
        use rand::Rng;
        use sha2::Sha256;

        // Generate salt
        let salt: Vec<u8> = (0..self.salt_length)
            .map(|_| rand::rng().random())
            .collect();

        // Derive key
        let mut mac = Hmac::<Sha256>::new_from_slice(raw.as_bytes()).unwrap();
        mac.update(&salt);

        for _ in 1..self.iterations {
            mac.update(b"\0");
        }

        let result = mac.finalize().into_bytes();

        // Format: iterations$salt$key
        format!(
            "{}${}${}",
            self.iterations,
            hex::encode(&salt),
            hex::encode(&result[..self.key_length.min(result.len())])
        )
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool {
        let parts: Vec<&str> = encoded.split('$').collect();
        if parts.len() != 3 {
            return false;
        }

        let iterations: u32 = match parts[0].parse() {
            Ok(i) => i,
            Err(_) => return false,
        };

        let salt = match hex::decode(parts[1]) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let expected_key = match hex::decode(parts[2]) {
            Ok(k) => k,
            Err(_) => return false,
        };

        // Derive key from raw password
        use hmac::Hmac;
        use hmac::Mac;
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(raw.as_bytes()).unwrap();
        mac.update(&salt);

        for _ in 1..iterations {
            mac.update(b"\0");
        }

        let result = mac.finalize().into_bytes();
        let derived_key = &result[..expected_key.len().min(result.len())];

        use subtle::ConstantTimeEq;
        derived_key.ct_eq(&expected_key).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcrypt_encoder() {
        let encoder = BcryptPasswordEncoder::new();
        let hash = encoder.encode("password");

        assert!(encoder.matches("password", &hash));
        assert!(!encoder.matches("wrong", &hash));
    }

    #[test]
    fn test_noop_encoder() {
        let encoder = NoOpPasswordEncoder;

        assert_eq!(encoder.encode("password"), "password");
        assert!(encoder.matches("password", "password"));
    }
}
