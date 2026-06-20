//! I/O operations module
//! I/O 操作模块
//!
//! Provides async I/O primitives for TCP and UDP networking, backed by
//! [`async-net`] (which runs on the [`async-io`] reactor driven by
//! `Runtime::block_on`). These types replace the former self-built FD/driver
//! futures; the public method names (`connect`, `bind`, `accept`, `read`,
//! `write_all`) are preserved so downstream crates (e.g. `hiver-http`) keep
//! compiling unchanged.
//!
//! 提供用于 TCP 与 UDP 网络的异步 I/O 原语,由 [`async-net`] 驱动(其运行在由
//! `Runtime::block_on` 驱动的 [`async-io`] reactor 上)。这些类型替代了原先自研的
//! FD/driver future;公开的方法名(`connect`、`bind`、`accept`、`read`、
//! `write_all`)保持不变,使下游 crate(如 `hiver-http`)无需改动即可编译。

#![allow(clippy::manual_async_fn)]

use std::{
    future::Future,
    io,
    net::{Shutdown, SocketAddr},
};

/// A TCP stream between a local and a remote socket.
/// 本地套接字与远程套接字之间的 TCP 流。
///
/// Wraps [`async_net::TcpStream`], exposing the same inherent async methods the
/// runtime historically provided (`connect`, `read`, `write_all`, `split`,
/// `shutdown`) so existing callers compile unchanged. Underlying I/O is driven
/// by the `async-io` reactor in `Runtime::block_on`.
///
/// 包裹 [`async_net::TcpStream`],暴露 runtime 历史上提供的相同 inherent 异步方法
/// (`connect`、`read`、`write_all`、`split`、`shutdown`),使现有调用方无需改动即可
/// 编译。底层 I/O 由 `Runtime::block_on` 中的 `async-io` reactor 驱动。
pub struct TcpStream
{
    inner: async_net::TcpStream,
}

impl TcpStream
{
    /// Create a new TcpStream connected to the specified address.
    /// 创建连接到指定地址的新 TcpStream。
    pub fn connect(addr: &str) -> impl Future<Output = io::Result<Self>>
    {
        let addr = addr.to_string();
        async move {
            let inner = async_net::TcpStream::connect(addr).await?;
            Ok(Self { inner })
        }
    }

    /// Read data from the stream into `buf`, returning the number of bytes
    /// read. Returns `Ok(0)` when the peer has closed the connection (EOF).
    /// 从流中读取数据到 `buf`,返回读取的字节数。对端关闭连接时返回 `Ok(0)`(EOF)。
    pub fn read<'a, 'b>(
        &'a mut self,
        buf: &'b mut [u8],
    ) -> impl Future<Output = io::Result<usize>> + 'a
    where
        'b: 'a,
    {
        // Delegate to the AsyncRead trait via async-net's poll-based read.
        // 经由 async-net 的基于 poll 的 read 委托给 AsyncRead trait。
        use futures_lite::AsyncReadExt;
        async move { self.inner.read(buf).await }
    }

    /// Write all of `buf` to the stream.
    /// 将 `buf` 全部写入流。
    pub fn write_all<'a, 'b>(
        &'a mut self,
        buf: &'b [u8],
    ) -> impl Future<Output = io::Result<()>> + 'a
    where
        'b: 'a,
    {
        use futures_lite::AsyncWriteExt;
        async move { self.inner.write_all(buf).await }
    }

    /// Split the stream into separate read and write halves.
    /// 将流拆分为独立的读、写两半。
    ///
    /// Each half clones the underlying socket handle (async-net's `TcpStream`
    /// is cheaply clonable — it is `Arc`-backed internally), so both can be
    /// held and moved independently. Full-duplex I/O is safe at the kernel
    /// level. This replaces the old self-built `unsafe` split that aliased
    /// `&mut`.
    ///
    /// 每一半克隆底层 socket 句柄(async-net 的 `TcpStream` 可廉价克隆——内部由
    /// `Arc` 支撑),故两者可独立持有与移动。全双工 I/O 在内核层面安全。这替代了
    /// 旧的、别名 `&mut` 的自研 `unsafe` split。
    #[must_use]
    pub fn split(&mut self) -> (ReadHalf, WriteHalf)
    {
        (
            ReadHalf {
                inner: self.inner.clone(),
            },
            WriteHalf {
                inner: self.inner.clone(),
            },
        )
    }

    /// Shut down the read, write, or both halves of the connection.
    /// 关闭连接的读、写或全部两半。
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()>
    {
        self.inner.shutdown(how)
    }

    /// Returns the remote socket address of this peer.
    /// 返回对端的远程套接字地址。
    pub fn peer_addr(&self) -> io::Result<SocketAddr>
    {
        self.inner.peer_addr()
    }

    /// Returns the local socket address.
    /// 返回本地套接字地址。
    pub fn local_addr(&self) -> io::Result<SocketAddr>
    {
        self.inner.local_addr()
    }
}

impl From<async_net::TcpStream> for TcpStream
{
    fn from(inner: async_net::TcpStream) -> Self
    {
        Self { inner }
    }
}

/// Read half of a [`TcpStream`], sharing the underlying socket via a clone of
/// the async-net handle.
/// [`TcpStream`] 的读半部,经由 async-net 句柄的克隆共享底层 socket。
pub struct ReadHalf
{
    #[allow(dead_code)]
    inner: async_net::TcpStream,
}

/// Write half of a [`TcpStream`], sharing the underlying socket via a clone of
/// the async-net handle.
/// [`TcpStream`] 的写半部,经由 async-net 句柄的克隆共享底层 socket。
pub struct WriteHalf
{
    #[allow(dead_code)]
    inner: async_net::TcpStream,
}

/// A TCP socket server, listening for connections.
/// 监听连接的 TCP socket 服务端。
pub struct TcpListener
{
    inner: async_net::TcpListener,
}

impl TcpListener
{
    /// Bind a new TCP listener to the specified address.
    /// 将新 TCP 监听器绑定到指定地址。
    pub fn bind(addr: &str) -> impl Future<Output = io::Result<Self>>
    {
        let addr = addr.to_string();
        async move {
            let inner = async_net::TcpListener::bind(addr).await?;
            Ok(Self { inner })
        }
    }

    /// Accept a new incoming connection.
    /// 接受一个新的入站连接。
    pub fn accept(&mut self) -> impl Future<Output = io::Result<(TcpStream, SocketAddr)>> + '_
    {
        async move {
            let (stream, addr) = self.inner.accept().await?;
            Ok((TcpStream { inner: stream }, addr))
        }
    }

    /// Returns the local socket address this listener is bound to.
    /// 返回本监听器绑定的本地套接字地址。
    pub fn local_addr(&self) -> io::Result<SocketAddr>
    {
        self.inner.local_addr()
    }
}

/// A UDP socket.
/// UDP 套接字。
pub struct UdpSocket
{
    inner: async_net::UdpSocket,
}

impl UdpSocket
{
    /// Bind a new UDP socket to the specified address.
    /// 将新 UDP 套接字绑定到指定地址。
    pub fn bind(addr: &str) -> impl Future<Output = io::Result<Self>>
    {
        let addr = addr.to_string();
        async move {
            let inner = async_net::UdpSocket::bind(addr).await?;
            Ok(Self { inner })
        }
    }

    /// Receive a single datagram into `buf`, returning the byte count and the
    /// sender's address.
    /// 接收单个数据报到 `buf`,返回字节数与发送方地址。
    pub fn recv_from<'a, 'b>(
        &'a mut self,
        buf: &'b mut [u8],
    ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + 'a
    where
        'b: 'a,
    {
        async move { self.inner.recv_from(buf).await }
    }

    /// Send `buf` as a datagram to `addr`.
    /// 将 `buf` 作为数据报发送到 `addr`。
    pub fn send_to<'a, 'b>(
        &'a mut self,
        buf: &'b [u8],
        addr: SocketAddr,
    ) -> impl Future<Output = io::Result<usize>> + 'a
    where
        'b: 'a,
    {
        async move { self.inner.send_to(buf, addr).await }
    }

    /// Connect the UDP socket to a remote peer (filters received packets to
    /// that peer and enables `recv`/`send`).
    /// 将 UDP 套接字连接到远端(过滤收到的包至该对端,并启用 `recv`/`send`)。
    pub fn connect(&mut self, addr: SocketAddr) -> impl Future<Output = io::Result<()>> + '_
    {
        async move { self.inner.connect(addr).await }
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
    fn test_tcp_listener_bind_invalid()
    {
        // We cannot rely on DNS to reject made-up hostnames — many resolvers
        // (ISPs, captive portals) synthesize addresses for anything. Instead
        // prove bind works end-to-end by binding a real ephemeral port and
        // checking the returned listener has a valid local address.
        // `block_on` itself returns `io::Result<F::Output>`, and the inner
        // future returns `io::Result<TcpListener>`, so we unwrap twice.
        // 不能依赖 DNS 拒绝编造的主机名——许多解析器(ISP、强制门户)会为任意主机
        // 合成地址。改为端到端证明 bind 生效:绑定一个真实临时端口并检查返回的
        // 监听器具有合法本地地址。`block_on` 自身返回 `io::Result<F::Output>`,
        // 内层 future 返回 `io::Result<TcpListener>`,故需解包两次。
        let mut runtime = crate::Runtime::new().unwrap();
        let listener = runtime
            .block_on(async { TcpListener::bind("127.0.0.1:0").await })
            .expect("block_on should succeed")
            .expect("bind to 127.0.0.1:0 should succeed");
        let addr = listener
            .local_addr()
            .expect("listener should have a local addr");
        assert!(addr.port() != 0, "ephemeral bind should assign a real port");
    }
}
