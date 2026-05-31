//! Simple Chat Server Example
//! 简单聊天服务器示例
//!
//! A multi-client chat server using channels for message broadcasting.
//! 使用通道进行消息广播的多客户端聊天服务器。
//!
//! Run with: cargo run --example runtime-chat-server
//! 运行: cargo run --example runtime-chat-server
//!
//! Connect multiple clients with: telnet 127.0.0.1:8080
//! 使用多个客户端连接: telnet 127.0.0.1:8080

use hiver_runtime::{Runtime, spawn};
use hiver_runtime::channel::unbounded;
use hiver_runtime::io::TcpListener;
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

type ClientId = usize;
type Message = (ClientId, String);
type Sender = hiver_runtime::Sender<Message>;
type Receiver = hiver_runtime::Receiver<Message>;

struct ChatRoom {
    clients: Arc<Mutex<HashMap<ClientId, Sender>>>,
    next_id: Arc<Mutex<ClientId>>,
}

impl ChatRoom {
    fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    fn add_client(&self, tx: Sender) -> ClientId {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let mut clients = self.clients.lock().unwrap();
        clients.insert(id, tx);
        id
    }

    fn remove_client(&self, id: ClientId) {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(&id);
    }

    fn broadcast(&self, from: ClientId, message: String) {
        let clients = self.clients.lock().unwrap();
        for (id, tx) in clients.iter() {
            if *id != from {
                // Send asynchronously, don't wait
                // 异步发送，不等待
                let tx = tx.clone();
                let msg = (from, message.clone());
                spawn(async move {
                    let _ = tx.send(msg).await;
                });
            }
        }
    }
}

fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut runtime = Runtime::new()?;
    let chat_room = Arc::new(ChatRoom::new());

    runtime.block_on(async {
        let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
        tracing::info!("Chat server listening on 127.0.0.1:8080");
        tracing::info!("聊天服务器监听 127.0.0.1:8080");
        tracing::info!("Connect with: telnet 127.0.0.1:8080");
        tracing::info!("连接方式: telnet 127.0.0.1:8080");

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    tracing::info!("New client connected from {}", addr);
                    tracing::info!("新客户端从 {} 连接", addr);

                    let chat_room = chat_room.clone();
                    let (tx, mut rx) = unbounded::<Message>();

                    // Add client to chat room
                    // 将客户端添加到聊天室
                    let client_id = chat_room.add_client(tx);

                    // Spawn task to handle incoming messages from this client
                    // 生成任务处理来自此客户端的消息
                    let chat_room_read = chat_room.clone();
                    spawn(async move {
                        let mut buf = [0u8; 1024];
                        loop {
                            match stream.read(&mut buf).await {
                                Ok(0) => {
                                    tracing::info!("Client {} disconnected", client_id);
                                    tracing::info!("客户端 {} 断开连接", client_id);
                                    chat_room_read.remove_client(client_id);
                                    break;
                                }
                                Ok(n) => {
                                    let message = String::from_utf8_lossy(&buf[..n])
                                        .trim()
                                        .to_string();
                                    if !message.is_empty() {
                                        tracing::info!("Client {}: {}", client_id, message);
                                        tracing::info!("客户端 {}: {}", client_id, message);
                                        chat_room_read.broadcast(
                                            client_id,
                                            format!("Client {}: {}", client_id, message),
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Read error: {}", e);
                                    chat_room_read.remove_client(client_id);
                                    break;
                                }
                            }
                        }
                    });

                    // Spawn task to send messages to this client
                    // 生成任务向此客户端发送消息
                    let chat_room_write = chat_room.clone();
                    spawn(async move {
                        while let Ok((from_id, message)) = rx.recv().await {
                            let msg = format!("{}\r\n", message);
                            if let Err(e) = stream.write_all(msg.as_bytes()).await {
                                tracing::error!("Write error: {}", e);
                                chat_room_write.remove_client(client_id);
                                break;
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                }
            }
        }
    })
}
