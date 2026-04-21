pub mod client_app;
pub mod launcher;
pub mod network;
pub mod server_app;

pub fn default_tcp_endpoint() -> String {
    "tcp://127.0.0.1:5555".to_string()
}

pub fn default_ipc_endpoint() -> String {
    if cfg!(windows) {
        "ipc://C:/git/ZA/rust_zmq_chat/zmq_pubsub.ipc".to_string()
    } else {
        "ipc:///tmp/zmq_pubsub.ipc".to_string()
    }
}
