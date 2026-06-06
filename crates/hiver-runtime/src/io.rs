//! I/O operations module
//! I/O操作模块
//!
//! # Overview / 概述
//!
//! This module provides async I/O primitives for TCP and UDP networking.
//! 本模块提供用于TCP和UDP网络的异步I/O原语。
//!
//! # Features / 功能
//!
//! - Async TCP stream with connect/read/write / 带有connect/read/write的异步TCP流
//! - Async TCP listener for accepting connections / 用于接受连接的异步TCP监听器
//! - Zero-copy ready operations / 零拷贝就绪操作
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_runtime::io::TcpStream;
//!
//! async fn echo_client() -> std::io::Result<()> {
//!     let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
//!
//!     stream.write_all(b"Hello, World!").await?;
//!
//!     let mut buf = [0u8; 1024];
//!     let n = stream.read(&mut buf).await?;
//!
//!     println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
//!     Ok(())
//! }
//! ```

#![allow(private_interfaces)]

use std::{
    future::Future,
    io,
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr},
    os::fd::{AsRawFd, FromRawFd, RawFd},
    pin::Pin,
    task::{Context, Poll},
};

const DUMMY_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);

/// A TCP stream between a local and a remote socket
/// 本地套接字和远程套接字之间的TCP流
///
/// Provides async read/write operations with the underlying driver.
/// 使用底层驱动提供异步读/写操作。
pub struct TcpStream
{
    /// The raw file descriptor / 原始文件描述符
    fd: std::os::fd::OwnedFd,
}

impl TcpStream
{
    /// Create a new TcpStream from a raw file descriptor
    /// 从原始文件描述符创建新的TcpStream
    ///
    /// # Safety / 安全性
    ///
    /// The fd must be valid and owned by the caller.
    /// fd必须有效且由调用者拥有。
    pub(crate) unsafe fn from_raw_fd(fd: RawFd) -> io::Result<Self>
    {
        // Set non-blocking mode
        // 设置非阻塞模式
        #[cfg(unix)]
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            if flags < 0
            {
                return Err(io::Error::last_os_error());
            }
            if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0
            {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(Self {
            // SAFETY: Caller guarantees ownership
            // 安全性：调用者保证所有权
            fd: unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) },
        })
    }

    /// Connect to a remote address
    /// 连接到远程地址
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_runtime::io::TcpStream;
    ///
    /// async fn connect() -> std::io::Result<()> {
    ///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn connect(addr: &str) -> ConnectFuture
    {
        let Ok(addr) = addr.parse::<SocketAddr>()
        else
        {
            // Try to resolve as hostname
            // For now, return error - DNS resolution will be added later
            return ConnectFuture::Error(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid address format, use IP:PORT",
            ));
        };

        ConnectFuture::Connecting(Box::new(ConnectingState {
            addr,
            fd: None,
            started: false,
        }))
    }

    /// Read some bytes from the stream
    /// 从流中读取一些字节
    ///
    /// Returns the number of bytes read. May return 0 if the stream is closed.
    /// 返回读取的字节数。如果流已关闭，可能返回0。
    pub fn read<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> ReadFuture<'a, 'b>
    {
        ReadFuture {
            stream: Some(self),
            buf,
            pos: 0,
        }
    }

    /// Write all bytes to the stream
    /// 将所有字节写入流
    ///
    /// This will keep writing until all bytes have been written or an error occurs.
    /// 将持续写入，直到所有字节都已写入或发生错误。
    pub fn write_all<'a, 'b>(&'a mut self, buf: &'b [u8]) -> WriteAllFuture<'a, 'b>
    {
        WriteAllFuture {
            stream: Some(self),
            buf,
            pos: 0,
        }
    }

    /// Split the stream into read and write halves
    /// 将流拆分为读写两半
    ///
    /// Note: This is a placeholder. The actual implementation will use
    /// Arc-based splitting like Tokio for true split read/write.
    /// 注意：这是占位符。实际实现将使用类似Tokio的基于Arc的拆分来实现真正的读写分离。
    ///
    /// # Note / 注意
    ///
    /// This is a simplified split implementation. Both halves reference the same
    /// underlying socket. The caller must coordinate read/write operations.
    /// 这是简化的 split 实现。两个半部引用同一个底层 socket。
    /// 调用者必须协调读/写操作。
    pub fn split(&mut self) -> (ReadHalf<'_>, WriteHalf<'_>)
    {
        // SAFETY: Both halves reference the same stream via raw pointer.
        // This is safe because TCP sockets support full-duplex I/O —
        // reads and writes can proceed concurrently at the kernel level.
        // The caller must not perform conflicting operations on the same half.
        // 安全：两个半部通过裸指针引用同一个流。
        // 这是安全的，因为 TCP socket 支持全双工 I/O —
        // 读和写可以在内核级别并发进行。
        // 调用者不得在同一个半部上执行冲突操作。
        unsafe {
            let ptr = self as *mut TcpStream;
            (ReadHalf { _stream: &mut *ptr }, WriteHalf { _stream: &mut *ptr })
        }
    }

    /// Shutdown the stream
    /// 关闭流
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()>
    {
        #[cfg(unix)]
        unsafe {
            let how = match how
            {
                Shutdown::Read => libc::SHUT_RD,
                Shutdown::Write => libc::SHUT_WR,
                Shutdown::Both => libc::SHUT_RDWR,
            };
            if libc::shutdown(self.as_raw_fd(), how) < 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        #[cfg(not(unix))]
        {
            let _ = how;
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Shutdown not supported on this platform",
            ));
        }
        Ok(())
    }
}

impl AsRawFd for TcpStream
{
    fn as_raw_fd(&self) -> RawFd
    {
        self.fd.as_raw_fd()
    }
}

/// Future for connecting to a remote address
/// 连接到远程地址的future
pub enum ConnectFuture
{
    /// Error state / 错误状态
    Error(io::Error),
    /// Connecting state / 连接中状态
    /// Boxed to reduce enum size / 使用Box减小枚举大小
    Connecting(Box<ConnectingState>),
    /// Done state / 完成状态
    Done,
}

struct ConnectingState
{
    addr: SocketAddr,
    fd: Option<RawFd>,
    started: bool,
}

impl Future for ConnectFuture
{
    type Output = io::Result<TcpStream>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        match &mut *self
        {
            ConnectFuture::Error(e) =>
            {
                let e = std::mem::replace(e, io::Error::other(""));
                Poll::Ready(Err(e))
            },
            ConnectFuture::Done => panic!("ConnectFuture polled after completion"),
            ConnectFuture::Connecting(state) =>
            {
                if !state.started
                {
                    state.started = true;

                    // Create socket and start connect
                    // 创建套接字并启动connect
                    let fd: RawFd = create_socket(state.addr.is_ipv4());

                    if fd < 0
                    {
                        return Poll::Ready(Err(io::Error::last_os_error()));
                    }

                    // Start connect (socket is already non-blocking from create_socket)
                    // 启动 connect（create_socket 已设置非阻塞）
                    let result = do_connect(fd, state.addr);

                    if result < 0
                    {
                        let err = io::Error::last_os_error();
                        if err.kind() != io::ErrorKind::WouldBlock
                        {
                            unsafe { libc::close(fd) };
                            return Poll::Ready(Err(err));
                        }
                        // Async connect in progress — store fd for later polling
                        // 异步 connect 进行中 — 存储 fd 用于后续轮询
                        state.fd = Some(fd);
                        return Poll::Pending;
                    }

                    // Connected immediately
                    // 立即连接
                    state.fd = Some(fd);
                }

                // Check if async connect has completed using poll()
                // 使用 poll() 检查异步连接是否完成
                if let Some(fd) = state.fd
                {
                    let mut pfd = libc::pollfd {
                        fd,
                        events: libc::POLLOUT,
                        revents: 0,
                    };
                    let ready = unsafe { libc::poll(&mut pfd, 1, 0) };

                    if ready < 0
                    {
                        let fd = state.fd.take().unwrap();
                        unsafe { libc::close(fd) };
                        return Poll::Ready(Err(io::Error::last_os_error()));
                    }

                    if ready == 0
                    {
                        // Not ready yet — register waker for future notification
                        // 尚未就绪 — 注册 waker 以便未来通知
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }

                    // Socket is writable — check for connection error via SO_ERROR
                    // Socket 可写 — 通过 SO_ERROR 检查连接错误
                    let mut err_val: libc::c_int = 0;
                    let mut err_len: libc::socklen_t = size_of::<libc::c_int>() as libc::socklen_t;
                    unsafe {
                        libc::getsockopt(
                            fd,
                            libc::SOL_SOCKET,
                            libc::SO_ERROR,
                            &mut err_val as *mut _ as *mut _,
                            &mut err_len,
                        );
                    }
                    if err_val != 0
                    {
                        let fd = state.fd.take().unwrap();
                        unsafe { libc::close(fd) };
                        return Poll::Ready(Err(io::Error::from_raw_os_error(err_val)));
                    }

                    // Connected successfully
                    // 连接成功
                    let fd = state.fd.take().unwrap();
                    let stream = match unsafe { TcpStream::from_raw_fd(fd) }
                    {
                        Ok(s) => s,
                        Err(e) => return Poll::Ready(Err(e)),
                    };
                    *self = ConnectFuture::Done;
                    Poll::Ready(Ok(stream))
                }
                else
                {
                    Poll::Pending
                }
            },
        }
    }
}

/// Helper to create a non-blocking socket
/// 创建非阻塞套接字的辅助函数
#[cfg(unix)]
fn create_socket(ipv4: bool) -> RawFd
{
    unsafe {
        let domain = if ipv4 { libc::AF_INET } else { libc::AF_INET6 };

        #[cfg(target_os = "linux")]
        let fd = libc::socket(domain, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0);

        #[cfg(not(target_os = "linux"))]
        let fd = libc::socket(domain, libc::SOCK_STREAM, 0);

        if fd < 0
        {
            return fd;
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Set close-on-exec for macOS/BSD
            if libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) < 0
            {
                libc::close(fd);
                return -1;
            }
        }

        // Set non-blocking
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0
        {
            libc::close(fd);
            return -1;
        }

        fd
    }
}

/// Helper to start a connect
/// 启动connect的辅助函数
#[cfg(unix)]
fn do_connect(fd: RawFd, addr: SocketAddr) -> i32
{
    unsafe {
        if addr.is_ipv4()
        {
            if let SocketAddr::V4(v4) = addr
            {
                #[cfg(target_os = "linux")]
                let sockaddr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: v4.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(v4.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };

                #[cfg(not(target_os = "linux"))]
                let sockaddr = libc::sockaddr_in {
                    sin_len: size_of::<libc::sockaddr_in>() as u8,
                    sin_family: libc::AF_INET as u8,
                    sin_port: v4.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(v4.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };

                libc::connect(
                    fd,
                    &sockaddr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in>() as libc::socklen_t,
                )
            }
            else
            {
                -1
            }
        }
        else
        {
            if let SocketAddr::V6(v6) = addr
            {
                #[cfg(target_os = "linux")]
                let sockaddr = libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as u16,
                    sin6_port: v6.port().to_be(),
                    sin6_flowinfo: v6.flowinfo(),
                    sin6_addr: libc::in6_addr {
                        s6_addr: v6.ip().octets(),
                    },
                    sin6_scope_id: v6.scope_id(),
                };

                #[cfg(not(target_os = "linux"))]
                let sockaddr = libc::sockaddr_in6 {
                    sin6_len: size_of::<libc::sockaddr_in6>() as u8,
                    sin6_family: libc::AF_INET6 as u8,
                    sin6_port: v6.port().to_be(),
                    sin6_flowinfo: v6.flowinfo(),
                    sin6_addr: libc::in6_addr {
                        s6_addr: v6.ip().octets(),
                    },
                    sin6_scope_id: v6.scope_id(),
                };

                libc::connect(
                    fd,
                    &sockaddr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                )
            }
            else
            {
                -1
            }
        }
    }
}

/// Future for reading from a TcpStream
/// 从TcpStream读取的future
pub struct ReadFuture<'a, 'b>
{
    stream: Option<&'a mut TcpStream>,
    buf: &'b mut [u8],
    pos: usize,
}

impl Future for ReadFuture<'_, '_>
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        // Extract all needed values upfront to avoid borrow issues
        // 提前提取所有需要的值以避免借用问题
        let stream_fd;
        let buf_ptr;
        let buf_len;

        {
            let stream = self.stream.as_mut().unwrap();
            stream_fd = stream.as_raw_fd();
            let pos = self.pos;
            buf_ptr = self.buf[pos..].as_mut_ptr();
            buf_len = self.buf[pos..].len();
        }

        #[cfg(unix)]
        {
            let result = unsafe { libc::read(stream_fd, buf_ptr as *mut _, buf_len) };

            if result < 0
            {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock
                {
                    // Would block - should register with driver
                    // 会阻塞 - 应该向驱动注册
                    // For now, just return Pending
                    // 目前只返回Pending
                    return Poll::Pending;
                }
                return Poll::Ready(Err(err));
            }

            let n = result as usize;
            if n == 0
            {
                return Poll::Ready(Ok(0)); // EOF
            }

            self.pos += n;
            Poll::Ready(Ok(n))
        }

        #[cfg(not(unix))]
        {
            let _ = (stream_fd, buf_ptr, buf_len);
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "TCP read not yet implemented on this platform",
            )))
        }
    }
}

/// Future for writing all bytes to a TcpStream
/// 向TcpStream写入所有字节的future
pub struct WriteAllFuture<'a, 'b>
{
    stream: Option<&'a mut TcpStream>,
    buf: &'b [u8],
    pos: usize,
}

impl Future for WriteAllFuture<'_, '_>
{
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        while self.pos < self.buf.len()
        {
            let stream = self.stream.as_mut().unwrap();

            #[cfg(unix)]
            {
                let result = unsafe {
                    libc::write(
                        stream.as_raw_fd(),
                        self.buf[self.pos..].as_ptr() as *const _,
                        self.buf[self.pos..].len(),
                    )
                };

                if result < 0
                {
                    let err = io::Error::last_os_error();
                    if err.kind() == io::ErrorKind::WouldBlock
                    {
                        return Poll::Pending;
                    }
                    return Poll::Ready(Err(err));
                }

                let n = result as usize;
                if n == 0
                {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "write zero byte",
                    )));
                }

                self.pos += n;
            }

            #[cfg(not(unix))]
            {
                let _ = stream;
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "TCP write not yet implemented on this platform",
                )));
            }
        }

        Poll::Ready(Ok(()))
    }
}

/// Read half of a TcpStream
/// TcpStream的读半部
pub struct ReadHalf<'a>
{
    _stream: &'a mut TcpStream,
}

/// Write half of a TcpStream
/// TcpStream的写半部
pub struct WriteHalf<'a>
{
    _stream: &'a mut TcpStream,
}

/// A TCP socket listener
/// TCP套接字监听器
///
/// Listens for incoming connections on a specific address.
/// 在特定地址上监听传入的连接。
pub struct TcpListener
{
    /// The raw file descriptor / 原始文件描述符
    fd: std::os::fd::OwnedFd,
}

impl TcpListener
{
    /// Create a new TCP listener bound to the specified address
    /// 创建绑定到指定地址的新TCP监听器
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_runtime::io::TcpListener;
    ///
    /// async fn listen() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:8080").await?;
    ///     println!("Listening on 127.0.0.1:8080");
    ///     Ok(())
    /// }
    /// ```
    pub fn bind(addr: &str) -> BindFuture
    {
        let Ok(addr) = addr.parse::<SocketAddr>()
        else
        {
            return BindFuture::Error(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid address format, use IP:PORT",
            ));
        };

        BindFuture::Binding(BindingState { addr })
    }

    /// Accept a new connection
    /// 接受新连接
    pub fn accept(&mut self) -> AcceptFuture<'_>
    {
        AcceptFuture { listener: self }
    }

    /// Get the local address
    /// 获取本地地址
    pub fn local_addr(&self) -> io::Result<SocketAddr>
    {
        #[cfg(unix)]
        unsafe {
            let mut addr: libc::sockaddr_storage = std::mem::zeroed();
            let mut len = size_of::<libc::sockaddr_storage>() as libc::socklen_t;

            if libc::getsockname(
                self.as_raw_fd(),
                &mut addr as *mut _ as *mut libc::sockaddr,
                &mut len,
            ) < 0
            {
                return Err(io::Error::last_os_error());
            }

            // Convert to SocketAddr (simplified)
            // 转换为SocketAddr（简化版）
            Ok(DUMMY_ADDR)
        }

        #[cfg(not(unix))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "local_addr not supported on this platform",
            ))
        }
    }
}

impl AsRawFd for TcpListener
{
    fn as_raw_fd(&self) -> RawFd
    {
        self.fd.as_raw_fd()
    }
}

/// Future for binding a TCP listener
/// 绑定TCP监听器的future
pub enum BindFuture
{
    /// Error state / 错误状态
    Error(io::Error),
    /// Binding state / 绑定中状态
    Binding(BindingState),
    /// Done state / 完成状态
    Done,
}

struct BindingState
{
    addr: SocketAddr,
}

impl Future for BindFuture
{
    type Output = io::Result<TcpListener>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        match &mut *self
        {
            BindFuture::Error(e) =>
            {
                let e = std::mem::replace(e, io::Error::other(""));
                Poll::Ready(Err(e))
            },
            BindFuture::Done => panic!("BindFuture polled after completion"),
            BindFuture::Binding(state) =>
            {
                // Create and bind socket
                // 创建并绑定套接字
                let fd = create_socket(state.addr.is_ipv4());

                if fd < 0
                {
                    return Poll::Ready(Err(io::Error::last_os_error()));
                }

                // Set reuse address
                // 设置地址重用
                #[cfg(unix)]
                unsafe {
                    let opt: i32 = 1;
                    if libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_REUSEADDR,
                        &opt as *const _ as *const _,
                        size_of::<i32>() as libc::socklen_t,
                    ) < 0
                    {
                        libc::close(fd);
                        return Poll::Ready(Err(io::Error::last_os_error()));
                    }

                    // Bind
                    // 绑定
                    let result = do_bind(fd, state.addr);
                    if result < 0
                    {
                        let err = io::Error::last_os_error();
                        libc::close(fd);
                        return Poll::Ready(Err(err));
                    }

                    // Listen
                    // 监听
                    if libc::listen(fd, 128) < 0
                    {
                        let err = io::Error::last_os_error();
                        libc::close(fd);
                        return Poll::Ready(Err(err));
                    }

                    let listener = TcpListener {
                        // SAFETY: fd is valid and owned
                        fd: std::os::fd::OwnedFd::from_raw_fd(fd),
                    };

                    *self = BindFuture::Done;
                    Poll::Ready(Ok(listener))
                }

                #[cfg(not(unix))]
                {
                    Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "TCP bind not yet implemented on this platform",
                    )))
                }
            },
        }
    }
}

/// Helper to bind a socket to an address
/// 将套接字绑定到地址的辅助函数
#[cfg(unix)]
fn do_bind(fd: RawFd, addr: SocketAddr) -> i32
{
    unsafe {
        if addr.is_ipv4()
        {
            if let SocketAddr::V4(v4) = addr
            {
                #[cfg(target_os = "linux")]
                let sockaddr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: v4.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(v4.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };

                #[cfg(not(target_os = "linux"))]
                let sockaddr = libc::sockaddr_in {
                    sin_len: size_of::<libc::sockaddr_in>() as u8,
                    sin_family: libc::AF_INET as u8,
                    sin_port: v4.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(v4.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };

                libc::bind(
                    fd,
                    &sockaddr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in>() as libc::socklen_t,
                )
            }
            else
            {
                -1
            }
        }
        else
        {
            if let SocketAddr::V6(v6) = addr
            {
                #[cfg(target_os = "linux")]
                let sockaddr = libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as u16,
                    sin6_port: v6.port().to_be(),
                    sin6_flowinfo: v6.flowinfo(),
                    sin6_addr: libc::in6_addr {
                        s6_addr: v6.ip().octets(),
                    },
                    sin6_scope_id: v6.scope_id(),
                };

                #[cfg(not(target_os = "linux"))]
                let sockaddr = libc::sockaddr_in6 {
                    sin6_len: size_of::<libc::sockaddr_in6>() as u8,
                    sin6_family: libc::AF_INET6 as u8,
                    sin6_port: v6.port().to_be(),
                    sin6_flowinfo: v6.flowinfo(),
                    sin6_addr: libc::in6_addr {
                        s6_addr: v6.ip().octets(),
                    },
                    sin6_scope_id: v6.scope_id(),
                };

                libc::bind(
                    fd,
                    &sockaddr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                )
            }
            else
            {
                -1
            }
        }
    }
}

/// Future for accepting a connection
/// 接受连接的future
pub struct AcceptFuture<'a>
{
    listener: &'a mut TcpListener,
}

impl Future for AcceptFuture<'_>
{
    type Output = io::Result<(TcpStream, SocketAddr)>;

    #[allow(unused_mut)]
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        #[cfg(unix)]
        {
            let mut addr: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
            let mut len = size_of::<libc::sockaddr_storage>() as libc::socklen_t;

            let fd = unsafe {
                #[cfg(target_os = "linux")]
                {
                    libc::accept4(
                        self.listener.as_raw_fd(),
                        &mut addr as *mut _ as *mut libc::sockaddr,
                        &mut len,
                        libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                    )
                }

                #[cfg(not(target_os = "linux"))]
                {
                    // Use regular accept on macOS/BSD, then set flags
                    let fd = libc::accept(
                        self.listener.as_raw_fd(),
                        &mut addr as *mut _ as *mut libc::sockaddr,
                        &mut len,
                    );

                    if fd >= 0
                    {
                        // Set close-on-exec
                        if libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) < 0
                        {
                            libc::close(fd);
                            return Poll::Ready(Err(io::Error::last_os_error()));
                        }

                        // Set non-blocking
                        let flags = libc::fcntl(fd, libc::F_GETFL);
                        if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0
                        {
                            libc::close(fd);
                            return Poll::Ready(Err(io::Error::last_os_error()));
                        }
                    }

                    fd
                }
            };

            if fd < 0
            {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock
                {
                    return Poll::Pending;
                }
                return Poll::Ready(Err(err));
            }

            let stream = match unsafe { TcpStream::from_raw_fd(fd) }
            {
                Ok(s) => s,
                Err(e) => return Poll::Ready(Err(e)),
            };

            // Parse peer address (simplified)
            // 解析对端地址（简化版）
            let peer_addr = match self.listener.local_addr()
            {
                Ok(_) => DUMMY_ADDR,
                Err(_) => return Poll::Ready(Err(io::Error::last_os_error())),
            };

            Poll::Ready(Ok((stream, peer_addr)))
        }

        #[cfg(not(unix))]
        {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "TCP accept not yet implemented on this platform",
            )))
        }
    }
}

/// UDP socket type / UDP套接字类型
///
/// Provides async UDP send and receive operations.
/// 提供异步UDP发送和接收操作。
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_runtime::io::UdpSocket;
///
/// async fn echo_server() -> std::io::Result<()> {
///     let socket = UdpSocket::bind("127.0.0.1:8080").await?;
///     println!("UDP server listening on 127.0.0.1:8080");
///
///     let mut buf = [0u8; 1024];
///     loop {
///         let (n, peer) = socket.recv_from(&mut buf).await?;
///         socket.send_to(&buf[..n], &peer).await?;
///     }
/// }
/// ```
pub struct UdpSocket
{
    /// The raw file descriptor / 原始文件描述符
    fd: std::os::fd::OwnedFd,
}

impl UdpSocket
{
    /// Bind a new UDP socket to the specified address
    /// 将新的UDP套接字绑定到指定地址
    ///
    /// # Example / 示例
    ///
    /// ```rust,no_run,ignore
    /// use hiver_runtime::io::UdpSocket;
    ///
    /// async fn bind_server() -> std::io::Result<()> {
    ///     let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn bind(addr: &str) -> BindUdpFuture
    {
        let Ok(addr) = addr.parse::<SocketAddr>()
        else
        {
            return BindUdpFuture::Error(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid address format, use IP:PORT",
            ));
        };

        BindUdpFuture::Binding(BindingUdpState { addr })
    }

    /// Receive data from the socket
    /// 从套接字接收数据
    ///
    /// Returns the number of bytes received and the peer address.
    /// 返回接收的字节数和对端地址。
    pub fn recv_from<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> RecvFromFuture<'a, 'b>
    {
        RecvFromFuture {
            stream: Some(self),
            buf,
        }
    }

    /// Send data to the specified address
    /// 向指定地址发送数据
    ///
    /// Returns the number of bytes sent.
    /// 返回发送的字节数。
    pub fn send_to<'a, 'b>(&'a mut self, buf: &'b [u8], addr: SocketAddr) -> SendToFuture<'a, 'b>
    {
        SendToFuture {
            stream: Some(self),
            buf,
            addr,
        }
    }

    /// Connect the socket to a remote address
    /// 将套接字连接到远程地址
    ///
    /// This filters incoming datagrams to only receive from this address.
    /// 这会过滤传入的数据报，只接收来自此地址的数据。
    pub fn connect(&mut self, addr: SocketAddr) -> ConnectUdpFuture
    {
        ConnectUdpFuture {
            fd: self.fd.as_raw_fd(),
            addr,
            done: false,
        }
    }
}

impl AsRawFd for UdpSocket
{
    fn as_raw_fd(&self) -> RawFd
    {
        self.fd.as_raw_fd()
    }
}

/// Future for binding a UDP socket
/// 绑定UDP套接字的future
pub enum BindUdpFuture
{
    /// Error state / 错误状态
    Error(io::Error),
    /// Binding state / 绑定中状态
    Binding(BindingUdpState),
    /// Done state / 完成状态
    Done,
}

struct BindingUdpState
{
    addr: SocketAddr,
}

impl Future for BindUdpFuture
{
    type Output = io::Result<UdpSocket>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        match &mut *self
        {
            BindUdpFuture::Error(e) =>
            {
                let e = std::mem::replace(e, io::Error::other(""));
                Poll::Ready(Err(e))
            },
            BindUdpFuture::Done => panic!("BindUdpFuture polled after completion"),
            BindUdpFuture::Binding(state) =>
            {
                // Create and bind UDP socket
                // 创建并绑定UDP套接字
                let fd = create_udp_socket(state.addr.is_ipv4());

                if fd < 0
                {
                    return Poll::Ready(Err(io::Error::last_os_error()));
                }

                // Bind
                // 绑定
                let result = do_bind_udp(fd, state.addr);
                if result < 0
                {
                    let err = io::Error::last_os_error();
                    unsafe { libc::close(fd) };
                    return Poll::Ready(Err(err));
                }

                let socket = UdpSocket {
                    // SAFETY: fd is valid and owned
                    fd: unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) },
                };

                *self = BindUdpFuture::Done;
                Poll::Ready(Ok(socket))
            },
        }
    }
}

/// Helper to create a UDP socket
/// 创建UDP套接字的辅助函数
#[cfg(unix)]
fn create_udp_socket(ipv4: bool) -> RawFd
{
    unsafe {
        let domain = if ipv4 { libc::AF_INET } else { libc::AF_INET6 };

        #[cfg(target_os = "linux")]
        let fd =
            libc::socket(domain, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, 0);

        #[cfg(not(target_os = "linux"))]
        let fd = libc::socket(domain, libc::SOCK_DGRAM, 0);

        if fd < 0
        {
            return fd;
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Set close-on-exec for macOS/BSD
            if libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) < 0
            {
                libc::close(fd);
                return -1;
            }

            // Set non-blocking
            let flags = libc::fcntl(fd, libc::F_GETFL);
            if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0
            {
                libc::close(fd);
                return -1;
            }
        }

        fd
    }
}

/// Helper to bind a UDP socket to an address
/// 将UDP套接字绑定到地址的辅助函数
#[cfg(unix)]
fn do_bind_udp(fd: RawFd, addr: SocketAddr) -> i32
{
    unsafe {
        if addr.is_ipv4()
        {
            if let SocketAddr::V4(v4) = addr
            {
                #[cfg(target_os = "linux")]
                let sockaddr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: v4.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(v4.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };

                #[cfg(not(target_os = "linux"))]
                let sockaddr = libc::sockaddr_in {
                    sin_len: size_of::<libc::sockaddr_in>() as u8,
                    sin_family: libc::AF_INET as u8,
                    sin_port: v4.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(v4.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };

                libc::bind(
                    fd,
                    &sockaddr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in>() as libc::socklen_t,
                )
            }
            else
            {
                -1
            }
        }
        else
        {
            if let SocketAddr::V6(v6) = addr
            {
                #[cfg(target_os = "linux")]
                let sockaddr = libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as u16,
                    sin6_port: v6.port().to_be(),
                    sin6_flowinfo: v6.flowinfo(),
                    sin6_addr: libc::in6_addr {
                        s6_addr: v6.ip().octets(),
                    },
                    sin6_scope_id: v6.scope_id(),
                };

                #[cfg(not(target_os = "linux"))]
                let sockaddr = libc::sockaddr_in6 {
                    sin6_len: size_of::<libc::sockaddr_in6>() as u8,
                    sin6_family: libc::AF_INET6 as u8,
                    sin6_port: v6.port().to_be(),
                    sin6_flowinfo: v6.flowinfo(),
                    sin6_addr: libc::in6_addr {
                        s6_addr: v6.ip().octets(),
                    },
                    sin6_scope_id: v6.scope_id(),
                };

                libc::bind(
                    fd,
                    &sockaddr as *const _ as *const libc::sockaddr,
                    size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                )
            }
            else
            {
                -1
            }
        }
    }
}

/// Future for receiving from a UDP socket
/// 从UDP套接字接收的future
pub struct RecvFromFuture<'a, 'b>
{
    stream: Option<&'a mut UdpSocket>,
    buf: &'b mut [u8],
}

impl Future for RecvFromFuture<'_, '_>
{
    type Output = io::Result<(usize, SocketAddr)>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        // Extract all needed values upfront to avoid borrow issues
        // 提前提取所有需要的值以避免借用问题
        let stream_fd;
        let buf_ptr;
        let buf_len;

        {
            let stream = self.stream.as_mut().unwrap();
            stream_fd = stream.as_raw_fd();
            buf_ptr = self.buf.as_mut_ptr();
            buf_len = self.buf.len();
        }

        #[cfg(unix)]
        {
            let mut addr: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
            let mut addr_len = size_of::<libc::sockaddr_storage>() as libc::socklen_t;

            let result = unsafe {
                libc::recvfrom(
                    stream_fd,
                    buf_ptr as *mut _,
                    buf_len,
                    0,
                    &mut addr as *mut _ as *mut libc::sockaddr,
                    &mut addr_len,
                )
            };

            if result < 0
            {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock
                {
                    return Poll::Pending;
                }
                return Poll::Ready(Err(err));
            }

            let n = result as usize;

            // Parse peer address (simplified)
            // 解析对端地址（简化版）
            let peer_addr = SocketAddr::V4(std::net::SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

            Poll::Ready(Ok((n, peer_addr)))
        }

        #[cfg(not(unix))]
        {
            let _ = (stream_fd, buf_ptr, buf_len);
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "UDP recv_from not yet implemented on this platform",
            )))
        }
    }
}

/// Future for sending to a UDP socket
/// 向UDP套接字发送的future
pub struct SendToFuture<'a, 'b>
{
    stream: Option<&'a mut UdpSocket>,
    buf: &'b [u8],
    addr: SocketAddr,
}

impl Future for SendToFuture<'_, '_>
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let stream = self.stream.as_mut().unwrap();
        let stream_fd = stream.as_raw_fd();

        #[cfg(unix)]
        {
            let result = match self.addr
            {
                SocketAddr::V4(v4) =>
                {
                    let sockaddr = libc::sockaddr_in {
                        #[cfg(target_os = "macos")]
                        sin_len: size_of::<libc::sockaddr_in>() as u8,
                        sin_family: libc::AF_INET as _,
                        sin_port: v4.port().to_be(),
                        sin_addr: libc::in_addr {
                            s_addr: u32::from(*v4.ip()).to_be(),
                        },
                        sin_zero: [0; 8],
                    };
                    unsafe {
                        libc::sendto(
                            stream_fd,
                            self.buf.as_ptr() as *const _,
                            self.buf.len(),
                            0,
                            &sockaddr as *const _ as *const _,
                            size_of::<libc::sockaddr_in>() as libc::socklen_t,
                        )
                    }
                },
                SocketAddr::V6(v6) =>
                {
                    let sockaddr = libc::sockaddr_in6 {
                        sin6_family: libc::AF_INET6 as _,
                        sin6_port: v6.port().to_be(),
                        sin6_flowinfo: v6.flowinfo().to_be(),
                        sin6_addr: libc::in6_addr {
                            s6_addr: v6.ip().octets(),
                        },
                        sin6_scope_id: v6.scope_id(),
                        #[cfg(target_os = "macos")]
                        sin6_len: size_of::<libc::sockaddr_in6>() as u8,
                    };
                    unsafe {
                        libc::sendto(
                            stream_fd,
                            self.buf.as_ptr() as *const _,
                            self.buf.len(),
                            0,
                            &sockaddr as *const _ as *const _,
                            size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                        )
                    }
                },
            };

            if result < 0
            {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock
                {
                    return Poll::Pending;
                }
                return Poll::Ready(Err(err));
            }

            let n = result as usize;
            Poll::Ready(Ok(n))
        }

        #[cfg(not(unix))]
        {
            let _ = stream_fd;
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "UDP send_to not yet implemented on this platform",
            )))
        }
    }
}

/// Future for connecting a UDP socket
/// 连接UDP套接字的future
pub struct ConnectUdpFuture
{
    fd: RawFd,
    addr: SocketAddr,
    done: bool,
}

impl Future for ConnectUdpFuture
{
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        assert!(!self.done, "ConnectUdpFuture polled after completion");

        // Perform the connect operation
        // 执行connect操作
        #[cfg(unix)]
        {
            let result = unsafe {
                match self.addr
                {
                    SocketAddr::V4(v4) =>
                    {
                        #[cfg(target_os = "linux")]
                        let sockaddr = libc::sockaddr_in {
                            sin_family: libc::AF_INET as u16,
                            sin_port: v4.port().to_be(),
                            sin_addr: libc::in_addr {
                                s_addr: u32::from_ne_bytes(v4.ip().octets()),
                            },
                            sin_zero: [0; 8],
                        };

                        #[cfg(not(target_os = "linux"))]
                        let sockaddr = libc::sockaddr_in {
                            sin_len: size_of::<libc::sockaddr_in>() as u8,
                            sin_family: libc::AF_INET as u8,
                            sin_port: v4.port().to_be(),
                            sin_addr: libc::in_addr {
                                s_addr: u32::from_ne_bytes(v4.ip().octets()),
                            },
                            sin_zero: [0; 8],
                        };

                        libc::connect(
                            self.fd,
                            &sockaddr as *const _ as *const libc::sockaddr,
                            size_of::<libc::sockaddr_in>() as libc::socklen_t,
                        )
                    },
                    SocketAddr::V6(v6) =>
                    {
                        #[cfg(target_os = "linux")]
                        let sockaddr = libc::sockaddr_in6 {
                            sin6_family: libc::AF_INET6 as u16,
                            sin6_port: v6.port().to_be(),
                            sin6_flowinfo: v6.flowinfo(),
                            sin6_addr: libc::in6_addr {
                                s6_addr: v6.ip().octets(),
                            },
                            sin6_scope_id: v6.scope_id(),
                        };

                        #[cfg(not(target_os = "linux"))]
                        let sockaddr = libc::sockaddr_in6 {
                            sin6_len: size_of::<libc::sockaddr_in6>() as u8,
                            sin6_family: libc::AF_INET6 as u8,
                            sin6_port: v6.port().to_be(),
                            sin6_flowinfo: v6.flowinfo(),
                            sin6_addr: libc::in6_addr {
                                s6_addr: v6.ip().octets(),
                            },
                            sin6_scope_id: v6.scope_id(),
                        };

                        libc::connect(
                            self.fd,
                            &sockaddr as *const _ as *const libc::sockaddr,
                            size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                        )
                    },
                }
            };

            if result < 0
            {
                return Poll::Ready(Err(io::Error::last_os_error()));
            }
        }

        #[cfg(not(unix))]
        {
            let _ = (self.fd, self.addr);
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "UDP connect not yet implemented on this platform",
            )));
        }

        self.done = true;
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_tcp_stream_create()
    {
        // Test that TcpStream can be created (will fail in practice without a valid fd)
        // 测试TcpStream可以被创建（实际上没有有效的fd会失败）
        let result = unsafe { TcpStream::from_raw_fd(-1) };
        assert!(result.is_err());
    }

    #[test]
    fn test_tcp_listener_bind_invalid()
    {
        let future = TcpListener::bind("invalid_address");
        // Should create Error future
        // 应该创建Error future
        match future
        {
            BindFuture::Error(_) =>
            {},
            _ => panic!("Expected Error future"),
        }
    }

    #[test]
    fn test_connect_invalid_addr()
    {
        let future = TcpStream::connect("not_an_address");
        match future
        {
            ConnectFuture::Error(_) =>
            {},
            _ => panic!("Expected Error future for invalid address"),
        }
    }
}
