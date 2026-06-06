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
pub trait PasswordEncoder: Send + Sync
{
    /// Encode a raw password
    /// 编码原始密码
    fn encode(&self, raw: &str) -> String;

    /// Verify that the raw password matches the encoded password
    /// 验证原始密码是否与编码密码匹配
    fn matches(&self, raw: &str, encoded: &str) -> bool;

    /// Check if encoding needs to be updated (for password migration)
    /// 检查编码是否需要更新（用于密码迁移）
    fn upgrade_encoding(&self, _encoded: &str) -> bool
    {
        // Default implementation says encoding doesn't need upgrade
        false
    }
}

/// `BCrypt` password encoder
/// `BCrypt密码编码器`
///
/// Equivalent to Spring's `BCryptPasswordEncoder`.
/// `等价于Spring的BCryptPasswordEncoder`。
pub struct BcryptPasswordEncoder
{
    /// Cost factor (4-31, default 10)
    /// 成本因子（4-31，默认10）
    cost: u32,
}

impl BcryptPasswordEncoder
{
    /// Create a new `BCrypt` encoder with default cost
    /// `创建具有默认成本的BCrypt编码器`
    pub fn new() -> Self
    {
        Self { cost: 10 }
    }

    /// Create with custom cost
    /// 使用自定义成本创建
    pub fn with_cost(cost: u32) -> Self
    {
        assert!((4..=31).contains(&cost), "BCrypt cost must be between 4 and 31");
        Self { cost }
    }
}

impl Default for BcryptPasswordEncoder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl PasswordEncoder for BcryptPasswordEncoder
{
    fn encode(&self, raw: &str) -> String
    {
        // SECURITY: Never silently degrade to a weaker hash algorithm.
        // If bcrypt fails, that's a fatal error requiring investigation.
        // 安全：绝不静默降级到更弱的哈希算法。
        // 如果 bcrypt 失败，那是需要调查的致命错误。
        bcrypt::hash(raw, self.cost)
            .expect("BCrypt encoding failed — this is a fatal error, not a condition for fallback")
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool
    {
        bcrypt::verify(raw, encoded).unwrap_or(false)
    }

    fn upgrade_encoding(&self, encoded: &str) -> bool
    {
        // Check if the encoded password has the target cost
        if let Some(prefix) = encoded.split('$').nth(2)
            && let Ok(cost) = prefix.parse::<u32>()
        {
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

impl PasswordEncoder for NoOpPasswordEncoder
{
    fn encode(&self, raw: &str) -> String
    {
        raw.to_string()
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool
    {
        raw == encoded
    }
}

/// Standard password encoder
/// 标准密码编码器
///
/// Uses `BCrypt` by default.
/// `默认使用BCrypt`。
pub struct StandardPasswordEncoder
{
    encoder: Box<dyn PasswordEncoder + Send + Sync>,
}

impl Clone for StandardPasswordEncoder
{
    fn clone(&self) -> Self
    {
        Self {
            encoder: Box::new(BcryptPasswordEncoder::new()),
        }
    }
}

impl std::fmt::Debug for StandardPasswordEncoder
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_struct("StandardPasswordEncoder").finish()
    }
}

impl StandardPasswordEncoder
{
    /// Create a new standard encoder
    /// 创建新的标准编码器
    pub fn new() -> Self
    {
        Self {
            encoder: Box::new(BcryptPasswordEncoder::new()),
        }
    }

    /// Create with `BCrypt`
    /// `使用BCrypt创建`
    pub fn bcrypt() -> Self
    {
        Self {
            encoder: Box::new(BcryptPasswordEncoder::new()),
        }
    }

    /// Create with custom encoder
    /// 使用自定义编码器创建
    pub fn custom(encoder: Box<dyn PasswordEncoder + Send + Sync>) -> Self
    {
        Self { encoder }
    }
}

impl Default for StandardPasswordEncoder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl PasswordEncoder for StandardPasswordEncoder
{
    fn encode(&self, raw: &str) -> String
    {
        self.encoder.encode(raw)
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool
    {
        self.encoder.matches(raw, encoded)
    }
}

/// PBKDF2 password encoder
/// PBKDF2密码编码器
///
/// Alternative to `BCrypt` with different security properties.
/// BCrypt的替代品，具有不同的安全属性。
pub struct Pbkdf2PasswordEncoder
{
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

impl Pbkdf2PasswordEncoder
{
    /// Create a new PBKDF2 encoder with defaults
    /// 创建具有默认值的PBKDF2编码器
    pub fn new() -> Self
    {
        Self {
            iterations: 100_000,
            key_length: 32,
            salt_length: 16,
        }
    }

    /// Create with custom iterations
    /// 使用自定义迭代次数创建
    pub fn with_iterations(iterations: u32) -> Self
    {
        Self {
            iterations,
            ..Default::default()
        }
    }
}

impl Default for Pbkdf2PasswordEncoder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl PasswordEncoder for Pbkdf2PasswordEncoder
{
    fn encode(&self, raw: &str) -> String
    {
        use hmac::{Hmac, Mac};
        use rand::Rng;
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // Generate random salt
        // 生成随机盐
        let salt: Vec<u8> = (0..self.salt_length)
            .map(|_| rand::rng().random())
            .collect();

        // PBKDF2 key derivation per RFC 2898:
        // DK = T1 || T2 || ... || Tdklen/hlen
        // Ti = F(Password, Salt, c, i)
        // F(Password, Salt, c, i) = U1 ^ U2 ^ ... ^ Uc
        // U1 = PRF(Password, Salt || INT(i))
        // U2 = PRF(Password, U1)
        // ...
        // PBKDF2 密钥推导（RFC 2898）
        let hash_len = 32; // SHA-256 output size / SHA-256 输出长度
        let blocks_needed = self.key_length.div_ceil(hash_len);
        let mut dk = Vec::with_capacity(blocks_needed * hash_len);

        for block_idx in 1..=blocks_needed
        {
            // U1 = PRF(Password, Salt || INT(block_idx))
            let mut mac =
                HmacSha256::new_from_slice(raw.as_bytes()).expect("HMAC accepts any key length");
            mac.update(&salt);
            mac.update(&(block_idx as u32).to_be_bytes());
            let mut u = mac.finalize().into_bytes();
            let mut result = u.clone();

            // U2..Uc: each iteration applies PRF(Password, U_prev)
            // 每次迭代应用 PRF(Password, U_prev)
            for _ in 1..self.iterations
            {
                let mut mac = HmacSha256::new_from_slice(raw.as_bytes())
                    .expect("HMAC accepts any key length");
                mac.update(&u);
                u = mac.finalize().into_bytes();
                // XOR: result ^= u
                for (r, u_byte) in result.iter_mut().zip(u.iter())
                {
                    *r ^= u_byte;
                }
            }

            dk.extend_from_slice(&result);
        }

        // Truncate to desired key length
        // 截断到所需密钥长度
        dk.truncate(self.key_length);

        // Format: iterations$salt$key
        format!("{}${}${}", self.iterations, hex::encode(&salt), hex::encode(&dk))
    }

    fn matches(&self, raw: &str, encoded: &str) -> bool
    {
        let parts: Vec<&str> = encoded.split('$').collect();
        if parts.len() != 3
        {
            return false;
        }

        let iterations: u32 = match parts[0].parse()
        {
            Ok(i) => i,
            Err(_) => return false,
        };

        let salt = match hex::decode(parts[1])
        {
            Ok(s) => s,
            Err(_) => return false,
        };

        let expected_key = match hex::decode(parts[2])
        {
            Ok(k) => k,
            Err(_) => return false,
        };

        // Derive key from raw password using the same PBKDF2 (RFC 2898)
        // 使用相同的 PBKDF2（RFC 2898）从原始密码派生密钥
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;
        let hash_len = 32;

        let blocks_needed = expected_key.len().div_ceil(hash_len);
        let mut dk = Vec::with_capacity(blocks_needed * hash_len);

        for block_idx in 1..=blocks_needed
        {
            let mut mac =
                HmacSha256::new_from_slice(raw.as_bytes()).expect("HMAC accepts any key length");
            mac.update(&salt);
            mac.update(&(block_idx as u32).to_be_bytes());
            let mut u = mac.finalize().into_bytes();
            let mut result = u.clone();

            for _ in 1..iterations
            {
                let mut mac = HmacSha256::new_from_slice(raw.as_bytes())
                    .expect("HMAC accepts any key length");
                mac.update(&u);
                u = mac.finalize().into_bytes();
                for (r, u_byte) in result.iter_mut().zip(u.iter())
                {
                    *r ^= u_byte;
                }
            }

            dk.extend_from_slice(&result);
        }

        dk.truncate(expected_key.len());

        // Constant-time comparison to prevent timing attacks
        // 常量时间比较以防止时序攻击
        use subtle::ConstantTimeEq;
        dk.ct_eq(&expected_key).into()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_bcrypt_encoder()
    {
        let encoder = BcryptPasswordEncoder::new();
        let hash = encoder.encode("password");

        assert!(encoder.matches("password", &hash));
        assert!(!encoder.matches("wrong", &hash));
    }

    #[test]
    fn test_noop_encoder()
    {
        let encoder = NoOpPasswordEncoder;

        assert_eq!(encoder.encode("password"), "password");
        assert!(encoder.matches("password", "password"));
    }
}
