//! 环境管理 / Environment Management
//!
//! 管理应用的环境（dev, test, prod 等）。
//! Manages application environments (dev, test, prod, etc.).

use std::fmt;

// ============================================================================
// Profile / 配置文件
// ============================================================================

/// 配置文件（Profile）
/// Profile (dev, test, prod, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profile {
    /// 开发环境
    Dev,

    /// 测试环境
    Test,

    /// 生产环境
    Prod,

    /// 自定义环境
    Custom(&'static str),
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Profile {
    /// 获取 Profile 名称
    pub fn name(&self) -> &str {
        match self {
            Profile::Dev => "dev",
            Profile::Test => "test",
            Profile::Prod => "prod",
            Profile::Custom(name) => name,
        }
    }

    /// 从字符串创建 Profile
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Profile::Dev,
            "test" | "testing" => Profile::Test,
            "prod" | "production" => Profile::Prod,
            name => Profile::Custom(Box::leak(name.to_string().into_boxed_str())),
        }
    }

    /// 是否为开发环境
    pub fn is_dev(&self) -> bool {
        matches!(self, Profile::Dev)
    }

    /// 是否为生产环境
    pub fn is_prod(&self) -> bool {
        matches!(self, Profile::Prod)
    }
}

// ============================================================================
// Environment / 环境
// ============================================================================

/// 应用环境
/// Application environment
///
/// 包含当前活动的 Profile 和环境信息。
/// Contains the currently active profile and environment information.
#[derive(Debug, Clone)]
pub struct Environment {
    /// 当前活动的 Profile
    active_profile: Profile,

    /// 所有可用的 Profile
    profiles: Vec<Profile>,
}

impl Environment {
    /// 创建新的环境
    pub fn new() -> Self {
        let active_profile = Self::detect_profile();
        let profiles = vec![active_profile, Profile::Dev];

        Self {
            active_profile,
            profiles,
        }
    }

    /// 从环境变量检测 Profile
    fn detect_profile() -> Profile {
        std::env::var("HIVER_PROFILE")
            .or_else(|_| std::env::var("APP_PROFILE"))
            .or_else(|_| std::env::var("SPRING_PROFILES_ACTIVE"))
            .map_or(Profile::Dev, |s| Profile::from_str(s.as_str()))
    }

    /// 获取当前活动的 Profile
    pub fn active_profile(&self) -> Profile {
        self.active_profile
    }

    /// 设置活动 Profile
    pub fn set_active_profile(&mut self, profile: Profile) {
        self.active_profile = profile;
        if !self.profiles.contains(&profile) {
            self.profiles.push(profile);
        }
    }

    /// 检查是否包含某个 Profile
    pub fn contains(&self, profile: Profile) -> bool {
        self.profiles.contains(&profile)
    }

    /// 添加 Profile
    pub fn add_profile(&mut self, profile: Profile) {
        if !self.profiles.contains(&profile) {
            self.profiles.push(profile);
        }
    }

    /// 获取所有 Profile
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试 / Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_from_str() {
        assert_eq!(Profile::from_str("dev"), Profile::Dev);
        assert_eq!(Profile::from_str("development"), Profile::Dev);
        assert_eq!(Profile::from_str("test"), Profile::Test);
        assert_eq!(Profile::from_str("prod"), Profile::Prod);
        assert_eq!(Profile::from_str("production"), Profile::Prod);
    }

    #[test]
    fn test_profile_name() {
        assert_eq!(Profile::Dev.name(), "dev");
        assert_eq!(Profile::Test.name(), "test");
        assert_eq!(Profile::Prod.name(), "prod");
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::new();
        assert_eq!(env.active_profile(), Profile::Dev);
    }

    #[test]
    fn test_environment_set_profile() {
        let mut env = Environment::new();
        env.set_active_profile(Profile::Prod);
        assert_eq!(env.active_profile(), Profile::Prod);
    }
}
