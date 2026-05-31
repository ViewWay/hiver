//! Interest types for file descriptor registration
//! 文件描述符注册的兴趣类型

use std::os::fd::RawFd;

/// Interest types for file descriptor registration
/// 文件描述符注册的兴趣类型
///
/// Specifies which events the driver should monitor for a file descriptor.
/// 指定driver应监控文件描述符的哪些事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Interest {
    /// Monitor for readability / 监控可读性
    pub readable: bool,
    /// Monitor for writability / 监控可写性
    pub writable: bool,
    /// Priority hint for edge-triggered mode / 边缘触发模式的优先级提示
    pub priority: bool,
    /// One-shot mode: auto-deregister after one event / 单次模式：事件后自动取消注册
    pub oneshot: bool,
    /// Edge-triggered mode vs level-triggered / 边缘触发模式 vs 水平触发
    pub edge: bool,
}

impl Interest {
    /// Create a new empty interest
    /// 创建一个新的空兴趣
    #[must_use]
    pub const fn new() -> Self {
        Self {
            readable: false,
            writable: false,
            priority: false,
            oneshot: false,
            edge: false,
        }
    }

    /// Create interest for readable events
    /// 创建可读事件兴趣
    #[must_use]
    pub const fn readable() -> Self {
        Self {
            readable: true,
            writable: false,
            priority: false,
            oneshot: false,
            edge: false,
        }
    }

    /// Create interest for writable events
    /// 创建可写事件兴趣
    #[must_use]
    pub const fn writable() -> Self {
        Self {
            readable: false,
            writable: true,
            priority: false,
            oneshot: false,
            edge: false,
        }
    }

    /// Create interest for both readable and writable events
    /// 创建可读和可写事件兴趣
    #[must_use]
    pub const fn both() -> Self {
        Self {
            readable: true,
            writable: true,
            priority: false,
            oneshot: false,
            edge: false,
        }
    }

    /// Add readability to the interest
    /// 添加可读性到兴趣
    #[must_use]
    pub const fn with_readable(mut self) -> Self {
        self.readable = true;
        self
    }

    /// Add writability to the interest
    /// 添加可写性到兴趣
    #[must_use]
    pub const fn with_writable(mut self) -> Self {
        self.writable = true;
        self
    }

    /// Enable priority mode
    /// 启用优先级模式
    #[must_use]
    pub const fn with_priority(mut self) -> Self {
        self.priority = true;
        self
    }

    /// Enable one-shot mode
    /// 启用单次模式
    #[must_use]
    pub const fn with_oneshot(mut self) -> Self {
        self.oneshot = true;
        self
    }

    /// Enable edge-triggered mode
    /// 启用边缘触发模式
    #[must_use]
    pub const fn with_edge(mut self) -> Self {
        self.edge = true;
        self
    }

    /// Convert to epoll event flags
    /// 转换为epoll事件标志
    #[cfg(target_os = "linux")]
    pub const fn to_epoll_flags(self) -> u32 {
        let mut flags = 0u32;

        if self.readable {
            flags |= libc::EPOLLIN;
        }
        if self.writable {
            flags |= libc::EPOLLOUT;
        }
        if self.priority {
            flags |= libc::EPOLLPRI;
        }
        if self.oneshot {
            flags |= libc::EPOLLONESHOT;
        }
        if self.edge {
            flags |= libc::EPOLLET;
        }

        flags
    }

    /// Convert from epoll event flags
    /// 从epoll事件标志转换
    #[cfg(target_os = "linux")]
    pub fn from_epoll_flags(flags: u32) -> Self {
        Self {
            readable: (flags & libc::EPOLLIN) != 0,
            writable: (flags & libc::EPOLLOUT) != 0,
            priority: (flags & libc::EPOLLPRI) != 0,
            oneshot: (flags & libc::EPOLLONESHOT) != 0,
            edge: (flags & libc::EPOLLET) != 0,
        }
    }

    /// Convert to kqueue event flags
    /// 转换为kqueue事件标志
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    #[allow(dead_code)]
    pub fn to_kqueue_filters(&self, fd: RawFd) -> (Vec<libc::kevent>, Vec<libc::kevent>) {
        use std::mem::zeroed;

        let mut add_events = Vec::with_capacity(2);
        let remove_events = Vec::new();

        if self.readable {
            let mut event = unsafe { zeroed::<libc::kevent>() };
            event.ident = fd as libc::uintptr_t;
            event.filter = libc::EVFILT_READ;
            event.flags = libc::EV_ADD | libc::EV_RECEIPT;
            if self.edge {
                event.flags |= libc::EV_CLEAR;
            }
            if self.oneshot {
                event.flags |= libc::EV_ONESHOT;
            }
            add_events.push(event);
        }

        if self.writable {
            let mut event = unsafe { zeroed::<libc::kevent>() };
            event.ident = fd as libc::uintptr_t;
            event.filter = libc::EVFILT_WRITE;
            event.flags = libc::EV_ADD | libc::EV_RECEIPT;
            if self.edge {
                event.flags |= libc::EV_CLEAR;
            }
            if self.oneshot {
                event.flags |= libc::EV_ONESHOT;
            }
            add_events.push(event);
        }

        (add_events, remove_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_builder() {
        let interest = Interest::readable().with_writable().with_edge();

        assert!(interest.readable);
        assert!(interest.writable);
        assert!(interest.edge);
        assert!(!interest.priority);
    }

    #[test]
    fn test_interest_both() {
        let interest = Interest::both();
        assert!(interest.readable);
        assert!(interest.writable);
    }
}
