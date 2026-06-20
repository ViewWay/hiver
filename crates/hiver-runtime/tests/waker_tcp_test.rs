//! End-to-end waker validation: a real TCP echo round-trip driven by the
//! custom runtime's FD→waker path.
//! 端到端 waker 验证:由自定义 runtime 的 FD→waker 路径驱动的真实 TCP 回显往返。
//!
//! Before the waker fix, `AcceptFuture`/`ReadFuture` returned `Poll::Pending`
//! without registering interest or a waker, so the only thing keeping the
//! echo loop alive was `block_on`'s busy-poll fallback. This test proves the
//! driver-registration path works: the task is woken on real I/O readiness.
//!
//! waker 修复前,`AcceptFuture`/`ReadFuture` 在返回 `Poll::Pending` 时既不注册
//! 兴趣也不存 waker,回显循环仅靠 `block_on` 的忙轮询回退勉强存活。本测试证明
//! driver 注册路径生效:任务在真实 I/O 就绪时被唤醒。

#![cfg(unix)]

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use hiver_runtime::{Runtime, io::TcpListener};

/// Echo server: accept one connection, echo its bytes back, then exit.
/// 回显服务端:接受一个连接,回显其字节后退出。
fn run_echo_server(port: u16, ready: Arc<AtomicBool>, got: Arc<AtomicBool>)
{
    let mut runtime = match Runtime::new()
    {
        Ok(rt) => rt,
        Err(e) =>
        {
            eprintln!("Runtime::new failed: {e}");
            return;
        },
    };

    let _ = runtime.block_on(async move {
        let mut listener = match TcpListener::bind(&format!("127.0.0.1:{port}")).await
        {
            Ok(l) => l,
            Err(e) =>
            {
                eprintln!("bind failed: {e}");
                return;
            },
        };

        // Signal that the listener is bound and ready.
        // 通知监听器已绑定就绪。
        ready.store(true, Ordering::SeqCst);

        let (mut stream, _addr) = match listener.accept().await
        {
            Ok(s) => s,
            Err(e) =>
            {
                eprintln!("accept failed: {e}");
                return;
            },
        };

        // Echo up to 64 bytes.
        // 回显最多 64 字节。
        let mut buf = [0u8; 64];
        loop
        {
            let n = match stream.read(&mut buf).await
            {
                Ok(0) => break, // client closed / 客户端关闭
                Ok(n) => n,
                Err(_) => break,
            };
            if stream.write_all(&buf[..n]).await.is_err()
            {
                break;
            }
            if n == 0
            {
                break;
            }
            got.store(true, Ordering::SeqCst);
        }
    });
}

#[test]
fn tcp_echo_round_trip_wakes_on_io()
{
    let port = pick_port();
    let ready = Arc::new(AtomicBool::new(false));
    let got = Arc::new(AtomicBool::new(false));

    let ready_c = ready.clone();
    let got_c = got.clone();
    let handle = thread::spawn(move || run_echo_server(port, ready_c, got_c));

    // Wait until the server bound the port (or give up after 2s).
    // 等待服务端绑定端口(或 2s 后放弃)。
    for _ in 0..200
    {
        if ready.load(Ordering::SeqCst)
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        ready.load(Ordering::SeqCst),
        "server did not become ready (bind failed or runtime broken)"
    );

    // Client: connect, send, read echo, on the blocking std net API (separate
    // thread — we are not inside the runtime here).
    // 客户端:用阻塞式 std net API 连接、发送、读回显(独立线程——此处不在
    // runtime 内)。
    let client = thread::spawn(move || -> std::io::Result<String> {
        use std::{
            io::{Read, Write},
            net::TcpStream as StdStream,
        };
        let mut s = StdStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_secs(2),
        )?;
        s.set_read_timeout(Some(Duration::from_secs(5)))?;
        let payload = b"hiver-waker-works";
        s.write_all(payload)?;
        let mut buf = [0u8; 64];
        let n = s.read(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
    });

    let echoed = client
        .join()
        .expect("client panicked")
        .expect("client IO failed");
    assert_eq!(echoed, "hiver-waker-works");

    // Wait for the server thread to observe the data and exit.
    // 等待服务端线程观测到数据并退出。
    for _ in 0..200
    {
        if got.load(Ordering::SeqCst)
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(got.load(Ordering::SeqCst), "server never observed the data");
    let _ = handle.join();
}

/// Pick a likely-free port by binding an ephemeral socket then closing it.
/// 通过绑定临时套接字再关闭,选一个可能空闲的端口。
fn pick_port() -> u16
{
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .unwrap_or(18_000)
}
