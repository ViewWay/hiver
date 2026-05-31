//! Driver configuration and factory
//! Driver配置和工厂
//!
//! This module provides configuration types and factory methods for creating
//! different driver implementations.
//!
//! 本模块提供配置类型和用于创建不同driver实现的工厂方法。

use std::sync::Arc;

use crate::driver::Driver;

/// Driver type selector using strategy pattern
/// 使用策略模式的Driver类型选择器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverType {
    /// Use epoll driver (Linux) / 使用epoll driver (Linux)
    Epoll,
    /// Use io-uring driver (Linux 5.1+) / 使用io-uring driver (Linux 5.1+)
    IOUring,
    /// Use kqueue driver (macOS/BSD) / 使用kqueue driver (macOS/BSD)
    Kqueue,
    /// Automatically detect and use the best available driver
    /// 自动检测并使用最佳可用driver
    Auto,
}

/// Driver configuration using builder pattern
/// 使用Builder模式的Driver配置
#[derive(Debug, Clone, Copy)]
pub struct DriverConfig {
    /// Queue depth (must be power of 2 for ring buffer efficiency)
    /// 队列深度（必须是2的幂以优化环形缓冲区效率）
    pub entries: u32,
    /// Wait for completion on submit (blocking mode)
    /// 提交时等待完成（阻塞模式）
    pub submit_wait: bool,
    /// CPU core affinity (None = no affinity)
    /// CPU核心亲和性（None = 无亲和性）
    pub cpu_affinity: Option<usize>,
    /// Enable deferred task wake-up
    /// 启用延迟任务唤醒
    pub defer_wakeup: bool,
    /// Maximum number of concurrent operations per FD
    /// 每个文件描述符的最大并发操作数
    pub max_ops_per_fd: u32,
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            entries: 256,
            submit_wait: false,
            cpu_affinity: None,
            defer_wakeup: true,
            max_ops_per_fd: 32,
        }
    }
}

/// Driver configuration builder
/// Driver配置构建器
///
/// Provides a fluent API for constructing driver configurations.
/// 提供用于构建driver配置的流畅API。
#[derive(Debug, Clone)]
pub struct DriverConfigBuilder {
    config: DriverConfig,
}

impl DriverConfigBuilder {
    /// Create a new builder with default configuration
    /// 创建具有默认配置的新构建器
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DriverConfig::default(),
        }
    }

    /// Set the queue depth (will be rounded up to next power of 2)
    /// 设置队列深度（将向上舍入到下一个2的幂）
    #[must_use]
    pub fn entries(mut self, entries: u32) -> Self {
        self.config.entries = entries.next_power_of_two();
        self
    }

    /// Enable or disable submit-wait mode
    /// 启用或禁用提交等待模式
    #[must_use]
    pub const fn submit_wait(mut self, wait: bool) -> Self {
        self.config.submit_wait = wait;
        self
    }

    /// Set CPU affinity for the driver thread
    /// 为driver线程设置CPU亲和性
    #[must_use]
    pub const fn cpu_affinity(mut self, core: usize) -> Self {
        self.config.cpu_affinity = Some(core);
        self
    }

    /// Clear CPU affinity (no affinity)
    /// 清除CPU亲和性（无亲和性）
    #[must_use]
    pub const fn no_affinity(mut self) -> Self {
        self.config.cpu_affinity = None;
        self
    }

    /// Enable or disable deferred task wake-up
    /// 启用或禁用延迟任务唤醒
    #[must_use]
    pub const fn defer_wakeup(mut self, defer: bool) -> Self {
        self.config.defer_wakeup = defer;
        self
    }

    /// Set maximum operations per file descriptor
    /// 设置每个文件描述符的最大操作数
    #[must_use]
    pub const fn max_ops_per_fd(mut self, max: u32) -> Self {
        self.config.max_ops_per_fd = max;
        self
    }

    /// Build the configuration
    /// 构建配置
    #[must_use]
    pub const fn build(self) -> DriverConfig {
        self.config
    }
}

impl Default for DriverConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver factory using factory pattern
/// 使用工厂模式的Driver工厂
///
/// Provides a unified interface for creating different driver implementations.
/// 提供用于创建不同driver实现的统一接口。
pub struct DriverFactory;

impl DriverFactory {
    /// Create a driver with the specified type and default configuration
    /// 使用指定类型和默认配置创建driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if:
    /// 返回错误如果：
    /// - The specified driver type is not available on this platform
    /// - 指定的driver类型在此平台上不可用
    /// - Driver initialization fails
    /// - Driver初始化失败
    ///
    /// # Examples / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_runtime::driver::{DriverFactory, DriverType};
    ///
    /// let driver = DriverFactory::create(DriverType::Auto).unwrap();
    /// ```
    pub fn create(driver_type: DriverType) -> std::io::Result<Arc<dyn Driver>> {
        Self::create_with_config(driver_type, DriverConfig::default())
    }

    /// Create a driver with the specified type and configuration
    /// 使用指定类型和配置创建driver
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if driver initialization fails.
    /// 如果driver初始化失败则返回错误。
    pub fn create_with_config(
        driver_type: DriverType,
        config: DriverConfig,
    ) -> std::io::Result<Arc<dyn Driver>> {
        let ty = if matches!(driver_type, DriverType::Auto) {
            Self::detect_best_driver()?
        } else {
            driver_type
        };

        match ty {
            #[cfg(target_os = "linux")]
            DriverType::Epoll => {
                Ok(Arc::new(crate::driver::epoll::EpollDriver::with_config(config)?))
            },
            #[cfg(target_os = "linux")]
            DriverType::IOUring => {
                Ok(Arc::new(crate::driver::iouring::IoUringDriver::with_config(config)?))
            },
            #[cfg(any(
                target_os = "macos",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "dragonfly"
            ))]
            DriverType::Kqueue => {
                Ok(Arc::new(crate::driver::kqueue::KqueueDriver::with_config(config)?))
            },
            #[cfg(not(target_os = "linux"))]
            DriverType::Epoll => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "epoll driver is only available on Linux",
            )),
            #[cfg(not(target_os = "linux"))]
            DriverType::IOUring => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "io-uring driver is only available on Linux",
            )),
            #[cfg(not(any(
                target_os = "macos",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "dragonfly"
            )))]
            DriverType::Kqueue => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "kqueue driver is only available on macOS/BSD",
            )),
            DriverType::Auto => {
                // Auto should have been resolved by detect_best_driver() above.
                // If we reach here, detection failed — return a clear error.
                // Auto 应该已被上面的 detect_best_driver() 解析。
                // 如果到达这里，说明检测失败 — 返回明确的错误。
                Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Failed to detect a suitable driver for this platform",
                ))
            }
        }
    }

    /// Detect the best available driver for the current platform
    /// 检测当前平台的最佳可用driver
    fn detect_best_driver() -> std::io::Result<DriverType> {
        #[cfg(target_os = "linux")]
        {
            // Check kernel version for io-uring support
            // 检查内核版本以支持io-uring
            if Self::has_io_uring_support() {
                Ok(DriverType::IOUring)
            } else {
                Ok(DriverType::Epoll)
            }
        }

        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        ))]
        {
            Ok(DriverType::Kqueue)
        }

        #[cfg(not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        )))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "No suitable driver found for this platform",
            ))
        }
    }

    /// Check if the system supports io-uring (Linux only)
    /// 检查系统是否支持io-uring（仅Linux）
    #[cfg(target_os = "linux")]
    fn has_io_uring_support() -> bool {
        // Check for io_uring_setup system call availability
        // 检查io_uring_setup系统调用的可用性
        // io-uring requires Linux 5.1+
        // io-uring需要Linux 5.1+
        let mut uname = libc::utsname {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        };

        unsafe {
            if libc::uname(&mut uname) != 0 {
                return false;
            }

            // Parse kernel version
            // 解析内核版本
            let release = std::ffi::CStr::from_ptr(uname.release.as_ptr()).to_string_lossy();

            if let Some((major, rest)) = release.split_once('.') {
                if let Some((minor, _)) = rest.split_once('.') {
                    if let (Ok(maj), Ok(min)) = (major.parse::<u32>(), minor.parse::<u32>()) {
                        return maj > 5 || (maj == 5 && min >= 1);
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = DriverConfigBuilder::new()
            .entries(512)
            .submit_wait(true)
            .cpu_affinity(0)
            .defer_wakeup(false)
            .build();

        // Should be rounded up to next power of 2
        // 应向上舍入到下一个2的幂
        assert_eq!(config.entries, 512);
        assert!(config.submit_wait);
        assert_eq!(config.cpu_affinity, Some(0));
        assert!(!config.defer_wakeup);
    }

    #[test]
    fn test_config_rounding() {
        let config = DriverConfigBuilder::new().entries(100).build();

        // 100 rounds up to 128 (next power of 2)
        // 100向上舍入到128（下一个2的幂）
        assert_eq!(config.entries, 128);
    }

    #[test]
    fn test_config_default() {
        let config = DriverConfig::default();
        assert_eq!(config.entries, 256);
        assert!(!config.submit_wait);
        assert_eq!(config.cpu_affinity, None);
        assert!(config.defer_wakeup);
    }
}
