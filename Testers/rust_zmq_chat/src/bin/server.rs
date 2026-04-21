use eframe::egui;
use rust_zmq_chat::server_app::ServerApp;

fn main() -> Result<(), eframe::Error> {
    let _ = tracing_subscriber::fmt::try_init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 760.0])
            .with_title("ZeroMQ PUB Server"),
        ..Default::default()
    };

    eframe::run_native(
        "ZeroMQ PUB Server",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ServerApp::new()))
        }),
    )
}

/*
use rust_zmq_chat::default_chat_server_addr;
use rust_zmq_chat::network::{ChatMessage, NetworkCommand, run_zmq_pub_server};
use eframe::egui;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("ZMQ Chat Server (PUB)"),
        ..Default::default()
    };

    eframe::run_native(
        "ZMQ Server",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ServerApp::new()))
        }),
    )
}

struct ServerApp {
    addr: String,
    input: String,
    tx: Option<mpsc::Sender<NetworkCommand>>,
    is_running: bool,
    error: Option<String>,
    sent_count: usize,
}

impl ServerApp {
    fn new() -> Self {
        let addr = default_chat_server_addr();
        Self {
            addr,
            input: String::new(),
            tx: None,
            is_running: false,
            error: None,
            sent_count: 0,
        }
    }

    fn start(&mut self) {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        self.tx = Some(cmd_tx);

        if let Err(e) = run_zmq_pub_server(self.addr.clone(), cmd_rx) {
            self.error = Some(format!("Failed to bind server: {}", e));
            return;
        }

        self.is_running = true;
        self.error = None;
    }

    fn send_message(&mut self) {
        if let Some(tx) = &self.tx {
            let msg = ChatMessage::new("SERVER".to_string(), self.input.clone());
            if let Ok(_) = tx.try_send(NetworkCommand { message: msg }) {
                self.sent_count += 1;
                self.input.clear();
            }
        }
    }
}

impl eframe::App for ServerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🚀 ZMQ IPC Server (Broadcast)");
            ui.label("This server BINDS to the IPC path and broadcasts to all clients.");
            ui.separator();

            ui.add_enabled_ui(!self.is_running, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Bind Path:");
                    ui.text_edit_singleline(&mut self.addr);
                });

                if ui.button("⚡ Start Server").clicked() {
                    self.start();
                }
            });

            if let Some(err) = &self.error {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            }

            if self.is_running {
                ui.label(egui::RichText::new("Status: Broadcasting").color(egui::Color32::GREEN));
                ui.label(format!("Endpoint: {}", self.addr));
                ui.label(format!("Messages Sent: {}", self.sent_count));
                ui.separator();

                ui.horizontal(|ui| {
                    let res = ui.text_edit_singleline(&mut self.input);
                    if (ui.button("Broadcast").clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !self.input.is_empty() {
                        self.send_message();
                        res.request_focus();
                    }
                });
            }
        });
        ctx.request_repaint();
    }
}
*/
