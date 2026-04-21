use eframe::egui;
use rust_zmq_chat::client_app::ClientApp;

fn main() -> Result<(), eframe::Error> {
    let _ = tracing_subscriber::fmt::try_init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 760.0])
            .with_title("ZeroMQ SUB Client"),
        ..Default::default()
    };

    eframe::run_native(
        "ZeroMQ SUB Client",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ClientApp::new()))
        }),
    )
}

/*
use rust_zmq_chat::default_chat_server_addr;
use rust_zmq_chat::network::{ChatMessage, run_zmq_sub_client};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_title("ZMQ Chat Client (SUB)"),
        ..Default::default()
    };

    eframe::run_native(
        "ZMQ Client",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ClientApp::new()))
        }),
    )
}

struct ClientApp {
    addr: String,
    messages: Arc<Mutex<Vec<ChatMessage>>>,
    is_running: bool,
    error: Option<String>,
}

impl ClientApp {
    fn new() -> Self {
        let addr = default_chat_server_addr();
        Self {
            addr,
            messages: Arc::new(Mutex::new(Vec::new())),
            is_running: false,
            error: None,
        }
    }

    fn start(&mut self) {
        let (evt_tx, mut evt_rx) = mpsc::channel(100);

        if let Err(e) = run_zmq_sub_client(self.addr.clone(), evt_tx) {
            self.error = Some(format!("Failed to connect: {}", e));
            return;
        }

        let msgs = self.messages.clone();
        tokio::spawn(async move {
            while let Some(evt) = evt_rx.recv().await {
                msgs.lock().unwrap().push(evt.message);
            }
        });

        self.is_running = true;
        self.error = None;
    }
}

impl eframe::App for ClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📡 ZMQ IPC Client (Receiver)");
            ui.label("This client CONNECTS to the server and listens for broadcasts.");
            ui.separator();

            ui.add_enabled_ui(!self.is_running, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Server Addr:");
                    ui.text_edit_singleline(&mut self.addr);
                });

                if ui.button("🔌 Connect to Server").clicked() {
                    self.start();
                }
            });

            if let Some(err) = &self.error {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            }

            if self.is_running {
                ui.label(egui::RichText::new("Status: Connected").color(egui::Color32::GREEN));
                ui.label(format!("Listening on: {}", self.addr));
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let msgs = self.messages.lock().unwrap();
                        for msg in msgs.iter() {
                            ui.label(egui::RichText::new(format!("{}: {}", msg.sender, msg.content)).monospace());
                        }
                    });
            }
        });
        ctx.request_repaint();
    }
}
*/
