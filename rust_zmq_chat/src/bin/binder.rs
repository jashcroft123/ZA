use rust_zmq_chat::network::{ChatMessage, NetworkEvent, run_zmq_sub_binder};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_title("ZMQ IPC Binder"),
        ..Default::default()
    };

    eframe::run_native(
        "ZMQ Binder",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(BinderApp::new()))
        }),
    )
}

struct BinderApp {
    addr: String,
    messages: Arc<Mutex<Vec<ChatMessage>>>,
    is_running: bool,
    error: Option<String>,
}

impl BinderApp {
    fn new() -> Self {
        Self {
            addr: "ipc:///tmp/zmqtest.ipc".to_string(),
            messages: Arc::new(Mutex::new(Vec::new())),
            is_running: false,
            error: None,
        }
    }

    fn start(&mut self) {
        let (evt_tx, mut evt_rx) = mpsc::channel(100);

        let bind_addr = self.addr.clone();

        if let Err(e) = run_zmq_sub_binder(bind_addr.clone(), evt_tx) {
            self.error = Some(format!("Failed to bind: {}", e));
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

impl eframe::App for BinderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ZMQ IPC SUB Binder");
            ui.label("This window binds to an IPC path and listens for messages.");
            ui.separator();

            ui.add_enabled_ui(!self.is_running, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Bind Path:");
                    ui.text_edit_singleline(&mut self.addr);
                });

                if ui.button("🚀 Bind and Listen").clicked() {
                    self.start();
                }
            });

            if let Some(err) = &self.error {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            }

            if self.is_running {
                ui.label(egui::RichText::new("Status: Binding Active").color(egui::Color32::GREEN));
                ui.label(format!("Listening on: {}", self.addr));
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        let msgs = self.messages.lock().unwrap();
                        for msg in msgs.iter() {
                            ui.label(format!("{}: {}", msg.sender, msg.content));
                        }
                    });
            }
        });
        ctx.request_repaint();
    }
}
