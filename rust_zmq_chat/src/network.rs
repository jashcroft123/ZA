use chrono::Local;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const POLL_INTERVAL_MS: i64 = 100;
const STARTUP_TIMEOUT_SECS: u64 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Running,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub at: String,
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            at: timestamp_now(),
            level: LogLevel::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            at: timestamp_now(),
            level: LogLevel::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            at: timestamp_now(),
            level: LogLevel::Error,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PublishedMessage {
    pub at: String,
    pub endpoint: String,
    pub topic: String,
    pub payload: String,
    pub frame_count: usize,
}

#[derive(Clone, Debug)]
pub struct ReceivedMessage {
    pub at: String,
    pub endpoint: String,
    pub topic: String,
    pub payload: String,
    pub frames: Vec<String>,
}

pub enum PublisherCommand {
    Publish { topic: String, payload: String },
    Shutdown,
}

pub enum PublisherEvent {
    State(ConnectionState),
    Log(LogEntry),
    Published(PublishedMessage),
}

pub enum SubscriberCommand {
    UpdateFilter(String),
    Shutdown,
}

pub enum SubscriberEvent {
    State(ConnectionState),
    Log(LogEntry),
    Message(ReceivedMessage),
}

pub struct PublisherHandle {
    endpoint: String,
    command_tx: Option<Sender<PublisherCommand>>,
    event_rx: Receiver<PublisherEvent>,
    join_handle: Option<JoinHandle<()>>,
}

impl PublisherHandle {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn publish(
        &self,
        topic: impl Into<String>,
        payload: impl Into<String>,
    ) -> Result<(), String> {
        let Some(command_tx) = &self.command_tx else {
            return Err("The publisher is not running.".to_string());
        };

        command_tx
            .send(PublisherCommand::Publish {
                topic: topic.into(),
                payload: payload.into(),
            })
            .map_err(|_| "The publisher worker is no longer available.".to_string())
    }

    pub fn try_recv(&mut self) -> Option<PublisherEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn shutdown(&mut self) {
        if let Some(command_tx) = self.command_tx.take() {
            let _ = command_tx.send(PublisherCommand::Shutdown);
        }

        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl Drop for PublisherHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub struct SubscriberHandle {
    endpoint: String,
    command_tx: Option<Sender<SubscriberCommand>>,
    event_rx: Receiver<SubscriberEvent>,
    join_handle: Option<JoinHandle<()>>,
}

impl SubscriberHandle {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn update_filter(&self, filter: impl Into<String>) -> Result<(), String> {
        let Some(command_tx) = &self.command_tx else {
            return Err("The subscriber is not running.".to_string());
        };

        command_tx
            .send(SubscriberCommand::UpdateFilter(filter.into()))
            .map_err(|_| "The subscriber worker is no longer available.".to_string())
    }

    pub fn try_recv(&mut self) -> Option<SubscriberEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn shutdown(&mut self) {
        if let Some(command_tx) = self.command_tx.take() {
            let _ = command_tx.send(SubscriberCommand::Shutdown);
        }

        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl Drop for SubscriberHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn start_publisher(endpoint: impl Into<String>) -> Result<PublisherHandle, String> {
    let endpoint = endpoint.into();
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();
    let worker_endpoint = endpoint.clone();

    let join_handle = thread::Builder::new()
        .name("zmq-publisher".to_string())
        .spawn(move || run_publisher_worker(worker_endpoint, command_rx, event_tx, ready_tx))
        .map_err(|error| error.to_string())?;

    match ready_rx.recv_timeout(Duration::from_secs(STARTUP_TIMEOUT_SECS)) {
        Ok(Ok(())) => Ok(PublisherHandle {
            endpoint,
            command_tx: Some(command_tx),
            event_rx,
            join_handle: Some(join_handle),
        }),
        Ok(Err(error)) => {
            let _ = join_handle.join();
            Err(error)
        }
        Err(_) => {
            let _ = join_handle.join();
            Err("Timed out while binding the PUB socket.".to_string())
        }
    }
}

pub fn start_subscriber(
    endpoint: impl Into<String>,
    filter: impl Into<String>,
) -> Result<SubscriberHandle, String> {
    let endpoint = endpoint.into();
    let filter = filter.into();
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();
    let worker_endpoint = endpoint.clone();
    let worker_filter = filter.clone();

    let join_handle = thread::Builder::new()
        .name("zmq-subscriber".to_string())
        .spawn(move || {
            run_subscriber_worker(
                worker_endpoint,
                worker_filter,
                command_rx,
                event_tx,
                ready_tx,
            )
        })
        .map_err(|error| error.to_string())?;

    match ready_rx.recv_timeout(Duration::from_secs(STARTUP_TIMEOUT_SECS)) {
        Ok(Ok(())) => Ok(SubscriberHandle {
            endpoint,
            command_tx: Some(command_tx),
            event_rx,
            join_handle: Some(join_handle),
        }),
        Ok(Err(error)) => {
            let _ = join_handle.join();
            Err(error)
        }
        Err(_) => {
            let _ = join_handle.join();
            Err("Timed out while connecting the SUB socket.".to_string())
        }
    }
}

fn run_publisher_worker(
    endpoint: String,
    command_rx: Receiver<PublisherCommand>,
    event_tx: Sender<PublisherEvent>,
    ready_tx: Sender<Result<(), String>>,
) {
    let context = zmq::Context::new();
    let socket = match context.socket(zmq::PUB) {
        Ok(socket) => socket,
        Err(error) => {
            let message = format!("Failed to create PUB socket: {error}");
            let _ = ready_tx.send(Err(message.clone()));
            let _ = event_tx.send(PublisherEvent::Log(LogEntry::error(message)));
            return;
        }
    };

    if let Err(error) = socket.set_linger(0) {
        let message = format!("Failed to configure PUB socket: {error}");
        let _ = ready_tx.send(Err(message.clone()));
        let _ = event_tx.send(PublisherEvent::Log(LogEntry::error(message)));
        return;
    }

    if let Err(error) = socket.bind(&endpoint) {
        let message = format!("Failed to bind PUB socket to {endpoint}: {error}");
        let _ = ready_tx.send(Err(message.clone()));
        let _ = event_tx.send(PublisherEvent::Log(LogEntry::error(message)));
        return;
    }

    let _ = ready_tx.send(Ok(()));
    let _ = event_tx.send(PublisherEvent::State(ConnectionState::Running));
    let _ = event_tx.send(PublisherEvent::Log(LogEntry::info(format!(
        "PUB socket bound to {endpoint}"
    ))));

    loop {
        match command_rx.recv_timeout(Duration::from_millis(POLL_INTERVAL_MS as u64)) {
            Ok(PublisherCommand::Publish { topic, payload }) => {
                let send_result = if topic.is_empty() {
                    socket.send(payload.as_bytes(), 0)
                } else {
                    socket.send_multipart([topic.as_bytes(), payload.as_bytes()], 0)
                };

                match send_result {
                    Ok(()) => {
                        let frame_count = if topic.is_empty() { 1 } else { 2 };
                        let _ = event_tx.send(PublisherEvent::Published(PublishedMessage {
                            at: timestamp_now(),
                            endpoint: endpoint.clone(),
                            topic,
                            payload,
                            frame_count,
                        }));
                    }
                    Err(error) => {
                        let _ = event_tx.send(PublisherEvent::Log(LogEntry::error(format!(
                            "Publish failed on {endpoint}: {error}"
                        ))));
                    }
                }
            }
            Ok(PublisherCommand::Shutdown) => break,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = event_tx.send(PublisherEvent::Log(LogEntry::info(format!(
        "PUB socket for {endpoint} stopped"
    ))));
    let _ = event_tx.send(PublisherEvent::State(ConnectionState::Stopped));
}

fn run_subscriber_worker(
    endpoint: String,
    initial_filter: String,
    command_rx: Receiver<SubscriberCommand>,
    event_tx: Sender<SubscriberEvent>,
    ready_tx: Sender<Result<(), String>>,
) {
    let context = zmq::Context::new();
    let socket = match context.socket(zmq::SUB) {
        Ok(socket) => socket,
        Err(error) => {
            let message = format!("Failed to create SUB socket: {error}");
            let _ = ready_tx.send(Err(message.clone()));
            let _ = event_tx.send(SubscriberEvent::Log(LogEntry::error(message)));
            return;
        }
    };

    if let Err(error) = socket.set_linger(0) {
        let message = format!("Failed to configure SUB socket: {error}");
        let _ = ready_tx.send(Err(message.clone()));
        let _ = event_tx.send(SubscriberEvent::Log(LogEntry::error(message)));
        return;
    }

    if let Err(error) = socket.connect(&endpoint) {
        let message = format!("Failed to connect SUB socket to {endpoint}: {error}");
        let _ = ready_tx.send(Err(message.clone()));
        let _ = event_tx.send(SubscriberEvent::Log(LogEntry::error(message)));
        return;
    }

    if let Err(error) = socket.set_subscribe(initial_filter.as_bytes()) {
        let message = format!(
            "Failed to subscribe with filter {:?}: {error}",
            initial_filter
        );
        let _ = ready_tx.send(Err(message.clone()));
        let _ = event_tx.send(SubscriberEvent::Log(LogEntry::error(message)));
        return;
    }

    let _ = ready_tx.send(Ok(()));
    let _ = event_tx.send(SubscriberEvent::State(ConnectionState::Running));
    let _ = event_tx.send(SubscriberEvent::Log(LogEntry::info(format!(
        "SUB socket connected to {endpoint} with filter {}",
        display_filter(&initial_filter)
    ))));

    let mut current_filter = initial_filter;
    let mut items = [socket.as_poll_item(zmq::POLLIN)];
    let mut should_stop = false;

    while !should_stop {
        loop {
            match command_rx.try_recv() {
                Ok(SubscriberCommand::UpdateFilter(next_filter)) => {
                    if let Err(error) = replace_subscription(&socket, &current_filter, &next_filter)
                    {
                        let _ = event_tx.send(SubscriberEvent::Log(LogEntry::error(format!(
                            "Failed to update filter to {:?}: {error}",
                            next_filter
                        ))));
                    } else {
                        current_filter = next_filter.clone();
                        let _ = event_tx.send(SubscriberEvent::Log(LogEntry::info(format!(
                            "Subscription filter updated to {}",
                            display_filter(&next_filter)
                        ))));
                    }
                }
                Ok(SubscriberCommand::Shutdown) => {
                    should_stop = true;
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    should_stop = true;
                    break;
                }
            }
        }

        if should_stop {
            break;
        }

        match zmq::poll(&mut items, POLL_INTERVAL_MS) {
            Ok(count) if count > 0 && items[0].is_readable() => match socket.recv_multipart(0) {
                Ok(frames) => {
                    let frames = frames
                        .into_iter()
                        .map(|frame| render_frame(&frame))
                        .collect::<Vec<_>>();

                    let (topic, payload) = split_frames(&frames);
                    let _ = event_tx.send(SubscriberEvent::Message(ReceivedMessage {
                        at: timestamp_now(),
                        endpoint: endpoint.clone(),
                        topic,
                        payload,
                        frames,
                    }));
                }
                Err(error) => {
                    let _ = event_tx.send(SubscriberEvent::Log(LogEntry::error(format!(
                        "Receive failed on {endpoint}: {error}"
                    ))));
                }
            },
            Ok(_) => {}
            Err(error) => {
                let _ = event_tx.send(SubscriberEvent::Log(LogEntry::error(format!(
                    "Polling failed on {endpoint}: {error}"
                ))));
                break;
            }
        }
    }

    let _ = event_tx.send(SubscriberEvent::Log(LogEntry::info(format!(
        "SUB socket for {endpoint} stopped"
    ))));
    let _ = event_tx.send(SubscriberEvent::State(ConnectionState::Stopped));
}

fn replace_subscription(
    socket: &zmq::Socket,
    current_filter: &str,
    next_filter: &str,
) -> Result<(), zmq::Error> {
    socket.set_unsubscribe(current_filter.as_bytes())?;
    socket.set_subscribe(next_filter.as_bytes())?;
    Ok(())
}

fn split_frames(frames: &[String]) -> (String, String) {
    match frames {
        [] => (String::new(), String::new()),
        [payload] => (String::new(), payload.clone()),
        [topic, payload] => (topic.clone(), payload.clone()),
        [topic, rest @ ..] => (topic.clone(), rest.join(" | ")),
    }
}

fn render_frame(frame: &[u8]) -> String {
    match String::from_utf8(frame.to_vec()) {
        Ok(text) => text,
        Err(_) => {
            let preview = frame
                .iter()
                .take(24)
                .map(|byte| format!("{byte:02X}"))
                .collect::<Vec<_>>()
                .join(" ");
            let suffix = if frame.len() > 24 { " ..." } else { "" };
            format!("<{} bytes: {}{}>", frame.len(), preview, suffix)
        }
    }
}

fn display_filter(filter: &str) -> String {
    if filter.is_empty() {
        "<all messages>".to_string()
    } else {
        format!("{filter:?}")
    }
}

fn timestamp_now() -> String {
    Local::now().format("%H:%M:%S").to_string()
}
