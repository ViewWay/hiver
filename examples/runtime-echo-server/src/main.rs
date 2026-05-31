//! TCP Echo Server Example
//! TCP回显服务器示例
//!
//! A simple TCP echo server that echoes back any data it receives.
//! 一个简单的TCP回显服务器，回显接收到的任何数据。
//!
//! Run with: cargo run --example runtime-echo-server
//! 运行: cargo run --example runtime-echo-server
//!
//! Then connect with: telnet 127.0.0.1:8080
//! 然后连接: telnet 127.0.0.1:8080

use hiver_runtime::{Runtime, spawn};
use hiver_runtime::io::TcpListener;
use std::io;

fn main() -> io::Result<()> {
    // Initialize tracing
    // 初始化追踪
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut runtime = Runtime::new()?;

    runtime.block_on(async {
        // Bind to address
        // 绑定地址
        let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
        tracing::info!("Echo server listening on 127.0.0.1:8080");
        tracing::info!("回显服务器监听 127.0.0.1:8080");
        tracing::info!("Connect with: telnet 127.0.0.1:8080");
        tracing::info!("连接方式: telnet 127.0.0.1:8080");

        // Accept connections
        // 接受连接
        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    tracing::info!("Accepted connection from {}", addr);
                    tracing::info!("接受来自 {} 的连接", addr);

                    // Spawn a task to handle each connection
                    // 生成任务处理每个连接
                    spawn(async move {
                        let mut buf = [0u8; 1024];
                        loop {
                            match stream.read(&mut buf).await {
                                Ok(0) => {
                                    // Connection closed
                                    // 连接关闭
                                    tracing::info!("Connection from {} closed", addr);
                                    tracing::info!("来自 {} 的连接已关闭", addr);
                                    break;
                                }
                                Ok(n) => {
                                    // Echo back the data
                                    // 回显数据
                                    if let Err(e) = stream.write_all(&buf[..n]).await {
                                        tracing::error!("Write error: {}", e);
                                        tracing::error!("写入错误: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Read error: {}", e);
                                    tracing::error!("读取错误: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                    tracing::error!("接受错误: {}", e);
                }
            }
        }
    })
}
