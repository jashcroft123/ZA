use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use chrono::Utc;
use uuid::Uuid;
use std::thread;
use std::time::Duration;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: i64,
}

impl ChatMessage {
    pub fn new(sender: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            content,
            timestamp: Utc::now().timestamp(),
        }
    }
}

pub struct NetworkCommand {
    pub message: ChatMessage,
}

pub struct NetworkEvent {
    pub message: ChatMessage,
}

pub fn run_zmq_peer(
    pub_addr: String,
    sub_addrs: Vec<String>,
    mut cmd_rx: mpsc::Receiver<NetworkCommand>,
    event_tx: mpsc::Sender<NetworkEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let context = zmq::Context::new();
    let pub_socket = context.socket(zmq::PUB)?;
    let sub_socket = context.socket(zmq::SUB)?;

    // Bind publisher to local address
    println!("Publisher binding to {}", pub_addr);
    pub_socket.bind(&pub_addr)?;

    // Set permissions if it's an IPC socket
    if pub_addr.starts_with("ipc://") {
        let path = pub_addr.trim_start_matches("ipc://");
        if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o666)) {
            eprintln!("Warning: Failed to set permissions on {}: {}", path, e);
        }
    }

    // Connect subscriber to all other peers
    for addr in sub_addrs {
        println!("Subscriber connecting to {}", addr);
        sub_socket.connect(&addr)?;
    }
    // Subscribe to all topics (empty filter)
    sub_socket.set_subscribe(b"")?;

    thread::spawn(move || {
        let mut items = [sub_socket.as_poll_item(zmq::POLLIN)];
        
        loop {
            // 1. Handle outgoing messages (PUBLISH)
            while let Ok(cmd) = cmd_rx.try_recv() {
                if let Ok(json) = serde_json::to_string(&cmd.message) {
                    // Pub/Sub usually takes a topic; we use empty or just the json
                    if let Err(e) = pub_socket.send(&json, 0) {
                        eprintln!("[{}] PUB send error: {}", pub_addr, e);
                    }
                }
            }

            // 2. Handle incoming messages (SUBSCRIBE)
            // Poll with 50ms timeout
            match zmq::poll(&mut items, 50) {
                Ok(n) if n > 0 => {
                    // Check SUB socket
                    if items[0].is_readable() {
                        match sub_socket.recv_string(0) {
                            Ok(Ok(json)) => {
                                if let Ok(msg) = serde_json::from_str::<ChatMessage>(&json) {
                                    let _ = event_tx.try_send(NetworkEvent { message: msg });
                                }
                            }
                            Ok(Err(bytes)) => {
                                eprintln!("[{}] Received non-UTF8 data: {:?}", pub_addr, bytes);
                            }
                            Err(e) => {
                                eprintln!("[{}] SUB recv error: {}", pub_addr, e);
                            }
                        }
                    }
                }
                Ok(_) => {} // Timeout
                Err(e) => {
                    eprintln!("[{}] ZMQ Poll error: {}", pub_addr, e);
                    break;
                }
            }
            
            thread::sleep(Duration::from_millis(5));
        }
    });

    Ok(())
}

pub fn run_zmq_sub_binder(
    addr: String,
    event_tx: mpsc::Sender<NetworkEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let context = zmq::Context::new();
    let sub_socket = context.socket(zmq::SUB)?;

    // Connect subscriber to address
    println!("Subscriber connecting to {}", addr);
    sub_socket.connect(&addr)?;

    // Subscribe to all topics (empty filter)
    sub_socket.set_subscribe(b"")?;

    thread::spawn(move || {
        let mut items = [sub_socket.as_poll_item(zmq::POLLIN)];
        println!("[{}] Subscriber polling thread started.", addr);
        
        loop {
            match zmq::poll(&mut items, 100) {
                Ok(n) if n > 0 => {
                    if items[0].is_readable() {
                        // Receive as raw bytes for deep debugging
                        match sub_socket.recv_bytes(0) {
                            Ok(bytes) => {
                                println!("[{}] Received {} bytes.", addr, bytes.len());
                                
                                // Convert to string regardless of UTF-8 errors (lossy)
                                let content = String::from_utf8_lossy(&bytes).to_string();
                                println!("[{}] Content: {}", addr, content);

                                // Try parsing as ChatMessage JSON
                                if let Ok(msg) = serde_json::from_str::<ChatMessage>(&content) {
                                    let _ = event_tx.try_send(NetworkEvent { message: msg });
                                } else {
                                    // Raw text fallback
                                    let raw_msg = ChatMessage::new("External".to_string(), content);
                                    let _ = event_tx.try_send(NetworkEvent { message: raw_msg });
                                }
                            }
                            Err(e) => {
                                eprintln!("[{}] SUB recv error: {}", addr, e);
                            }
                        }
                    }
                }
                Ok(_) => {} // Timeout
                Err(e) => {
                    eprintln!("[{}] ZMQ Poll error: {}", addr, e);
                    break;
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
    });

    Ok(())
}
