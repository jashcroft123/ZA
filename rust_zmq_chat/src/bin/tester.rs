use eframe::egui;
use rust_zmq_chat::client_app::ClientApp;

fn main() -> Result<(), eframe::Error> {
    let _ = tracing_subscriber::fmt::try_init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 760.0])
            .with_title("ZeroMQ External Subscriber Tester"),
        ..Default::default()
    };

    eframe::run_native(
        "ZeroMQ External Subscriber Tester",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ClientApp::new()))
        }),
    )
}

/*
use rust_zmq_chat::network::{ChatMessage, NetworkCommand, NetworkEvent, run_zmq_peer};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_title("ZMQ LabVIEW Tester"),
        ..Default::default()
    };

    eframe::run_native(
        "ZMQ Tester",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(TesterApp::new()))
        }),
    )
}

struct TesterApp {
    name: String,
    local_addr: String,
    remote_addr: String,
    messages: Arc<Mutex<Vec<ChatMessage>>>,
    tx: Option<mpsc::Sender<NetworkCommand>>,
    input: String,
    is_running: bool,
    error: Option<String>,
}

impl TesterApp {
    fn new() -> Self {
        Self {
            name: "Tester".to_string(),
            local_addr: "ipc://tester.ipc".to_string(),
            remote_addr: "ipc://alice.ipc".to_string(),
            messages: Arc::new(Mutex::new(Vec::new())),
            tx: None,
            input: String::new(),
            is_running: false,
            error: None,
        }
    }

    fn start(&mut self) {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (evt_tx, mut evt_rx) = mpsc::channel(100);

        self.tx = Some(cmd_tx);

        let local = self.local_addr.clone();
        let remote = self.remote_addr.clone();

        if let Err(e) = run_zmq_peer(local.clone(), vec![remote.clone()], cmd_rx, evt_tx) {
            self.error = Some(format!("Failed to start: {}", e));
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

impl eframe::App for TesterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ZMQ LabVIEW / External Tester");
            ui.separator();

            ui.add_enabled_ui(!self.is_running, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Your Name:");
                    ui.text_edit_singleline(&mut self.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Local Addr:");
                    ui.text_edit_singleline(&mut self.local_addr);
                });
                ui.horizontal(|ui| {
                    ui.label("Remote Addr:");
                    ui.text_edit_singleline(&mut self.remote_addr);
                });

                if ui.button("🚀 Start Peer").clicked() {
                    self.start();
                }
            });

            if let Some(err) = &self.error {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            }

            if self.is_running {
                ui.label(egui::RichText::new("Status: Connected/Listening").color(egui::Color32::GREEN));
                ui.separator();

                // Chat area
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let msgs = self.messages.lock().unwrap();
                        for msg in msgs.iter() {
                            ui.label(format!("{}: {}", msg.sender, msg.content));
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    let res = ui.text_edit_singleline(&mut self.input);
                    if (ui.button("Send").clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !self.input.is_empty() {
                        if let Some(tx) = &self.tx {
                            let msg = ChatMessage::new(self.name.clone(), self.input.clone());
                            self.messages.lock().unwrap().push(msg.clone());
                            let _ = tx.try_send(NetworkCommand { message: msg });
                            self.input.clear();
                            res.request_focus();
                        }
                    }
                });
            }
        });
        ctx.request_repaint();
    }
}
*/
