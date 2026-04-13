use crate::network::{ChatMessage, NetworkCommand, run_zmq_peer};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Transport {
    IPC,
    TCP,
}

pub struct ChatApp {
    // Message history for each peer
    peer_a_messages: Arc<Mutex<Vec<ChatMessage>>>,
    peer_b_messages: Arc<Mutex<Vec<ChatMessage>>>,
    peer_c_messages: Arc<Mutex<Vec<ChatMessage>>>,
    
    // Command channels for sending
    peer_a_tx: Option<mpsc::Sender<NetworkCommand>>,
    peer_b_tx: Option<mpsc::Sender<NetworkCommand>>,
    peer_c_tx: Option<mpsc::Sender<NetworkCommand>>,
    
    // UI input fields
    peer_a_input: String,
    peer_b_input: String,
    peer_c_input: String,

    // Transport selection
    transport: Transport,
    peer_a_addr: String,
    peer_b_addr: String,
    peer_c_addr: String,

    // Diagnostics
    startup_errors: Arc<Mutex<Vec<String>>>,
}

impl ChatApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            peer_a_messages: Arc::new(Mutex::new(Vec::new())),
            peer_b_messages: Arc::new(Mutex::new(Vec::new())),
            peer_c_messages: Arc::new(Mutex::new(Vec::new())),
            peer_a_tx: None,
            peer_b_tx: None,
            peer_c_tx: None,
            peer_a_input: String::new(),
            peer_b_input: String::new(),
            peer_c_input: String::new(),
            transport: Transport::IPC,
            peer_a_addr: String::new(),
            peer_b_addr: String::new(),
            peer_c_addr: String::new(),
            startup_errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_peers(&mut self) {
        // Setup channels
        let (a_cmd_tx, a_cmd_rx) = mpsc::channel(100);
        let (a_evt_tx, mut a_evt_rx) = mpsc::channel(100);
        let (b_cmd_tx, b_cmd_rx) = mpsc::channel(100);
        let (b_evt_tx, mut b_evt_rx) = mpsc::channel(100);
        let (c_cmd_tx, c_cmd_rx) = mpsc::channel(100);
        let (c_evt_tx, mut c_evt_rx) = mpsc::channel(100);

        self.peer_a_tx = Some(a_cmd_tx);
        self.peer_b_tx = Some(b_cmd_tx);
        self.peer_c_tx = Some(c_cmd_tx);

        // Define Mesh Addresses
        let (addr_a, addr_b, addr_c) = match self.transport {
            Transport::IPC => (
                "ipc://alice.ipc".to_string(),
                "ipc://bob.ipc".to_string(),
                "ipc://charlie.ipc".to_string(),
            ),
            Transport::TCP => (
                "tcp://127.0.0.1:5555".to_string(),
                "tcp://127.0.0.1:5556".to_string(),
                "tcp://127.0.0.1:5557".to_string(),
            ),
        };

        self.peer_a_addr = addr_a.clone();
        self.peer_b_addr = addr_b.clone();
        self.peer_c_addr = addr_c.clone();

        let errors = self.startup_errors.clone();
        let mut errs = errors.lock().unwrap();
        errs.clear();

        // Start Alice
        if let Err(e) = run_zmq_peer(addr_a.clone(), vec![addr_b.clone(), addr_c.clone()], a_cmd_rx, a_evt_tx) {
            errs.push(format!("Alice ({}) failed: {}", addr_a, e));
        }
        // Start Bob
        if let Err(e) = run_zmq_peer(addr_b.clone(), vec![addr_a.clone(), addr_c.clone()], b_cmd_rx, b_evt_tx) {
            errs.push(format!("Bob ({}) failed: {}", addr_b, e));
        }
        // Start Charlie
        if let Err(e) = run_zmq_peer(addr_c.clone(), vec![addr_a.clone(), addr_b.clone()], c_cmd_rx, c_evt_tx) {
            errs.push(format!("Charlie ({}) failed: {}", addr_c, e));
        }

        // Message listeners
        let a_msgs = self.peer_a_messages.clone();
        tokio::spawn(async move {
            while let Some(evt) = a_evt_rx.recv().await {
                a_msgs.lock().unwrap().push(evt.message);
            }
        });

        let b_msgs = self.peer_b_messages.clone();
        tokio::spawn(async move {
            while let Some(evt) = b_evt_rx.recv().await {
                b_msgs.lock().unwrap().push(evt.message);
            }
        });

        let c_msgs = self.peer_c_messages.clone();
        tokio::spawn(async move {
            while let Some(evt) = c_evt_rx.recv().await {
                c_msgs.lock().unwrap().push(evt.message);
            }
        });
    }

    fn render_chat_window(ctx: &egui::Context, name: &str, messages_vec: &Arc<Mutex<Vec<ChatMessage>>>, input: &mut String, tx: &Option<mpsc::Sender<NetworkCommand>>, address: &str) {
        egui::TopBottomPanel::bottom(format!("{}_input_panel", name))
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::singleline(input)
                        .hint_text("Type a message...")
                        .desired_width(f32::INFINITY);
                    
                    let response = ui.add(text_edit);
                    
                    if (ui.button("Send").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !input.is_empty() {
                        if let Some(chan) = tx {
                            let msg = ChatMessage::new(name.to_string(), input.clone());
                            // Add locally immediately
                            messages_vec.lock().unwrap().push(msg.clone());
                            let _ = chan.try_send(crate::network::NetworkCommand { message: msg });
                            input.clear();
                            response.request_focus();
                        }
                    }
                });
                ui.add_space(8.0);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Chat: {}", name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(address).small().color(egui::Color32::GRAY));
                });
            });
            ui.separator();

            let messages = messages_vec.lock().unwrap().clone();
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in messages {
                        let is_mine = msg.sender == name;
                        ui.horizontal(|ui| {
                            if is_mine { ui.add_space(40.0); }
                            
                            let color = if is_mine {
                                egui::Color32::from_rgb(40, 120, 40)
                            } else {
                                // Dynamic color based on sender name for others
                                let hash = msg.sender.as_bytes().iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
                                match hash % 3 {
                                    0 => egui::Color32::from_rgb(40, 60, 120), // Blue
                                    1 => egui::Color32::from_rgb(120, 60, 40), // Red
                                    _ => egui::Color32::from_rgb(100, 40, 100), // Purple
                                }
                            };

                            egui::Frame::NONE
                                .fill(color)
                                .corner_radius(8.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(format!("{}: {}", msg.sender, msg.content)).color(egui::Color32::WHITE));
                                });

                            if !is_mine { ui.add_space(40.0); }
                        });
                        ui.add_space(6.0);
                    }
                });
        });
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Alice Viewport
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("alice"),
            egui::ViewportBuilder::default().with_title("Alice").with_inner_size([400.0, 500.0]),
            |ctx, _class| {
                Self::render_chat_window(ctx, "Alice", &self.peer_a_messages, &mut self.peer_a_input, &self.peer_a_tx, &self.peer_a_addr);
            },
        );

        // Bob Viewport
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("bob"),
            egui::ViewportBuilder::default().with_title("Bob").with_inner_size([400.0, 500.0]),
            |ctx, _class| {
                Self::render_chat_window(ctx, "Bob", &self.peer_b_messages, &mut self.peer_b_input, &self.peer_b_tx, &self.peer_b_addr);
            },
        );

        // Charlie Viewport
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("charlie"),
            egui::ViewportBuilder::default().with_title("Charlie").with_inner_size([400.0, 500.0]),
            |ctx, _class| {
                Self::render_chat_window(ctx, "Charlie", &self.peer_c_messages, &mut self.peer_c_input, &self.peer_c_tx, &self.peer_c_addr);
            },
        );

        // Main Controller
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ZMQ 3-Peer Chat Controller");
            ui.separator();

            ui.add_enabled_ui(self.peer_a_tx.is_none(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Transport Protocol:");
                    ui.radio_value(&mut self.transport, Transport::IPC, "IPC");
                    ui.radio_value(&mut self.transport, Transport::TCP, "TCP (Localhost)");
                });
            });

            ui.add_space(8.0);

            if ui.button("🚀 Start Chat Application").clicked() && self.peer_a_tx.is_none() {
                self.start_peers();
            }

            // Diagnostic Section
            let errs = self.startup_errors.lock().unwrap();
            if !errs.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("⚠️ Network Errors:").color(egui::Color32::RED).strong());
                for err in errs.iter() {
                    ui.label(egui::RichText::new(format!("• {}", err)).color(egui::Color32::LIGHT_RED).small());
                }
                if ui.button("Clear Errors").clicked() {
                    drop(errs);
                    self.startup_errors.lock().unwrap().clear();
                }
            }

            if self.peer_a_tx.is_some() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Status: Running").color(egui::Color32::GREEN));
                ui.label(format!("Using protocol: {:?}", self.transport));
                ui.label("Open the separate windows for Alice, Bob, and Charlie.");
                
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("📡 Send Ping to All").clicked() {
                        let msg = ChatMessage::new("System".to_string(), "PING (Manual Test)".to_string());
                        if let Some(tx) = &self.peer_a_tx { let _ = tx.try_send(crate::network::NetworkCommand { message: msg.clone() }); }
                        if let Some(tx) = &self.peer_b_tx { let _ = tx.try_send(crate::network::NetworkCommand { message: msg.clone() }); }
                        if let Some(tx) = &self.peer_c_tx { let _ = tx.try_send(crate::network::NetworkCommand { message: msg }); }
                    }
                    ui.label("(Useful for testing LabVIEW connection)");
                });
            } else {
                ui.label("Status: Idle");
            }
        });

        ctx.request_repaint();
    }
}
