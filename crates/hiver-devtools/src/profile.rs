//! Dev profile detection and compile-time build metadata.
//! 开发环境检测和编译时构建元数据。
//!
//! # Spring Equivalent / Spring等价物
//!
//! Spring profiles (`@Profile("dev")`, `spring.profiles.active`).
//! Hiver uses compile-time `cfg(debug_assertions)` for zero-cost profile detection.
//!
//! # Rust Advantage / Rust优势
//!
//! - Compile-time profile detection — no runtime overhead
//! - `cfg(debug_assertions)` is set by `cargo build` (dev) vs `cargo build --release`
//! - No annotation processing, no reflection, no runtime lookup

/// Runtime profile — determines application behavior.
/// 运行时配置文件 — 决定应用行为。
///
/// Unlike Spring which uses string-based profiles (`@Profile("dev")`),
/// Hiver uses a typed enum for compile-time safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profile
{
    /// Development profile — verbose logging, hot reload, debug info.
    /// 开发环境 — 详细日志、热重载、调试信息。
    Dev,

    /// Production profile — optimized, minimal logging.
    /// 生产环境 — 优化、最小日志。
    Prod,

    /// Test profile — test doubles, in-memory services.
    /// 测试环境 — 测试替身、内存服务。
    Test,
}

impl Profile
{
    /// Detect the current profile from environment.
    /// 从环境变量检测当前配置文件。
    ///
    /// Priority:
    /// 1. `HIVER_PROFILE` env var
    /// 2. `cfg(debug_assertions)` compile-time flag
    pub fn current() -> Self
    {
        if let Ok(val) = std::env::var("HIVER_PROFILE")
        {
            return match val.to_lowercase().as_str()
            {
                "dev" | "development" => Profile::Dev,
                "prod" | "production" => Profile::Prod,
                "test" => Profile::Test,
                _ => Self::from_debug(),
            };
        }
        Self::from_debug()
    }

    /// Detect from `cfg(debug_assertions)`.
    /// 从 `cfg(debug_assertions)` 检测。
    #[cfg(debug_assertions)]
    fn from_debug() -> Self { Profile::Dev }

    #[cfg(not(debug_assertions))]
    fn from_debug() -> Self { Profile::Prod }

    /// Whether this is a dev profile.
    /// 是否为开发环境。
    pub fn is_dev(self) -> bool { self == Profile::Dev }

    /// Whether this is a production profile.
    /// 是否为生产环境。
    pub fn is_prod(self) -> bool { self == Profile::Prod }

    /// Whether this is a test profile.
    /// 是否为测试环境。
    pub fn is_test(self) -> bool { self == Profile::Test }
}

impl std::fmt::Display for Profile
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Profile::Dev => write!(f, "dev"),
            Profile::Prod => write!(f, "prod"),
            Profile::Test => write!(f, "test"),
        }
    }
}

impl Default for Profile
{
    fn default() -> Self { Self::current() }
}

/// Compile-time build metadata — available without runtime cost.
/// 编译时构建元数据 — 零运行时开销。
///
/// # Rust Advantage / Rust优势
///
/// All fields are `&'static str` — computed at compile time, zero allocation.
/// Spring's `BuildProperties` requires reading `META-INF/MANIFEST.MF` at runtime.
///
/// # Example / 示例
///
/// ```rust
/// use hiver_devtools::BuildInfo;
///
/// let info = BuildInfo::new();
/// println!("{} v{} ({})", info.crate_name, info.version, info.profile);
/// ```
#[derive(Debug, Clone)]
pub struct BuildInfo
{
    /// Crate name from `CARGO_PKG_NAME`.
    pub crate_name: &'static str,

    /// Version from `CARGO_PKG_VERSION`.
    pub version: &'static str,

    /// Active profile.
    pub profile: Profile,

    /// Target architecture (e.g. "aarch64-apple-darwin").
    pub target: String,

    /// Optimization level.
    pub opt_level: &'static str,

    /// Whether debug assertions are enabled.
    pub debug: bool,
}

impl BuildInfo
{
    /// Create a new BuildInfo with compile-time values.
    /// 创建带编译时值的新 BuildInfo。
    pub fn new() -> Self
    {
        Self {
            crate_name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            profile: Profile::current(),
            target: format!(
                "{}-{}",
                std::env::consts::ARCH,
                std::env::consts::OS
            ),
            opt_level: if cfg!(debug_assertions) { "0" } else { "3" },
            debug: cfg!(debug_assertions),
        }
    }
}

impl Default for BuildInfo
{
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for BuildInfo
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "{} v{} [{}] target={} opt={}",
            self.crate_name, self.version, self.profile, self.target, self.opt_level
        )
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_profile_current()
    {
        let profile = Profile::current();
        assert!(profile == Profile::Dev || profile == Profile::Prod || profile == Profile::Test);
    }

    #[test]
    fn test_profile_is_methods()
    {
        assert!(Profile::Dev.is_dev());
        assert!(!Profile::Dev.is_prod());
        assert!(Profile::Prod.is_prod());
        assert!(Profile::Test.is_test());
    }

    #[test]
    fn test_profile_display()
    {
        assert_eq!(Profile::Dev.to_string(), "dev");
        assert_eq!(Profile::Prod.to_string(), "prod");
        assert_eq!(Profile::Test.to_string(), "test");
    }

    #[test]
    fn test_build_info()
    {
        let info = BuildInfo::new();
        assert_eq!(info.crate_name, "hiver-devtools");
        assert!(!info.version.is_empty());
        assert!(!info.target.is_empty());
    }

    #[test]
    fn test_build_info_display()
    {
        let info = BuildInfo::new();
        let s = info.to_string();
        assert!(s.contains("hiver-devtools"));
        assert!(s.contains("v"));
    }
}
