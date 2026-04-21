use crate::network::{
    start_publisher, ConnectionState, LogEntry, LogLevel, PublishedMessage, PublisherEvent,
    PublisherHandle,
};
use crate::{default_ipc_endpoint, default_tcp_endpoint};
use eframe::egui;
use std::time::Duration;

const MAX_LOGS: usize = 300;
const MAX_MESSAGES: usize = 200;

pub struct ServerApp {
    endpoint: String,
    topic: String,
    payload: String,
    status: ConnectionState,
    session: Option<PublisherHandle>,
    logs: Vec<LogEntry>,
    sent_messages: Vec<PublishedMessage>,
}

impl ServerApp {
    pub fn new() -> Self {
        Self {
            endpoint: default_tcp_endpoint(),
            topic: "demo".to_string(),
            payload: "hello from the ZeroMQ PUB server".to_string(),
            status: ConnectionState::Stopped,
            session: None,
            logs: vec![LogEntry::info(
                "Ready. Bind the PUB socket, then start one or more client windows.",
            )],
            sent_messages: Vec::new(),
        }
    }

    fn bind(&mut self) {
        self.disconnect();

        match start_publisher(self.endpoint.clone()) {
            Ok(handle) => {
                self.status = ConnectionState::Running;
                self.logs.push(LogEntry::info(format!(
                    "PUB server is live on {}",
                    handle.endpoint()
                )));
                trim_vec(&mut self.logs, MAX_LOGS);
                self.session = Some(handle);
            }
            Err(error) => {
                self.status = ConnectionState::Stopped;
                self.logs.push(LogEntry::error(error));
                trim_vec(&mut self.logs, MAX_LOGS);
            }
        }
    }

    fn disconnect(&mut self) {
        if let Some(mut session) = self.session.take() {
            session.shutdown();
            self.logs.push(LogEntry::warning(format!(
                "PUB server disconnected from {}",
                session.endpoint()
            )));
            trim_vec(&mut self.logs, MAX_LOGS);
        }

        self.status = ConnectionState::Stopped;
    }

    fn publish(&mut self) {
        let payload = self.payload.trim().to_string();
        if payload.is_empty() {
            self.logs
                .push(LogEntry::warning("Enter a payload before publishing."));
            trim_vec(&mut self.logs, MAX_LOGS);
            return;
        }

        let Some(session) = &self.session else {
            self.logs
                .push(LogEntry::warning("Bind the server before publishing."));
            trim_vec(&mut self.logs, MAX_LOGS);
            return;
        };

        if let Err(error) = session.publish(self.topic.clone(), payload) {
            self.logs.push(LogEntry::error(error));
            trim_vec(&mut self.logs, MAX_LOGS);
        }
    }

    fn drain_events(&mut self) {
        let mut disconnected = false;

        if let Some(session) = &mut self.session {
            while let Some(event) = session.try_recv() {
                match event {
                    PublisherEvent::State(state) => {
                        self.status = state;
                        if state == ConnectionState::Stopped {
                            disconnected = true;
                        }
                    }
                    PublisherEvent::Log(entry) => {
                        self.logs.push(entry);
                    }
                    PublisherEvent::Published(message) => {
                        self.sent_messages.push(message);
                    }
                }
            }
        }

        trim_vec(&mut self.logs, MAX_LOGS);
        trim_vec(&mut self.sent_messages, MAX_MESSAGES);

        if disconnected {
            self.session = None;
        }
    }

    fn endpoint_presets(&mut self, ui: &mut egui::Ui, enabled: bool) {
        ui.add_enabled_ui(enabled, |ui| {
            if ui.button("TCP Example").clicked() {
                self.endpoint = default_tcp_endpoint();
            }

            if ui.button("IPC Example").clicked() {
                self.endpoint = default_ipc_endpoint();
            }
        });
    }

    fn status_text(&self) -> (&'static str, egui::Color32) {
        match self.status {
            ConnectionState::Running => ("Bound", egui::Color32::from_rgb(116, 201, 132)),
            ConnectionState::Stopped => ("Stopped", egui::Color32::from_rgb(220, 94, 94)),
        }
    }
}

impl Drop for ServerApp {
    fn drop(&mut self) {
        self.disconnect();
    }
}

impl eframe::App for ServerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            let (status_label, status_color) = self.status_text();

            ui.heading("ZeroMQ PUB Server");
            ui.label("Bind a PUB socket to any TCP or IPC endpoint, then publish topic and payload frames.");
            ui.label(
                egui::RichText::new(
                    "Tip: PUB/SUB has a slow-joiner effect, so connect the client before you send the first test message.",
                )
                .small()
                .color(egui::Color32::GRAY),
            );
            ui.add_space(8.0);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Status");
                        ui.label(egui::RichText::new(status_label).strong().color(status_color));
                        if let Some(session) = &self.session {
                            ui.label(
                                egui::RichText::new(session.endpoint())
                                    .small()
                                    .color(egui::Color32::GRAY),
                            );
                        }
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Bind Endpoint");
                        ui.add_enabled_ui(self.session.is_none(), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.endpoint)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("tcp://127.0.0.1:5555 or ipc://..."),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        self.endpoint_presets(ui, self.session.is_none());

                        if ui
                            .add_enabled(self.session.is_none(), egui::Button::new("Bind"))
                            .clicked()
                        {
                            self.bind();
                        }

                        if ui
                            .add_enabled(self.session.is_some(), egui::Button::new("Disconnect"))
                            .clicked()
                        {
                            self.disconnect();
                        }
                    });
                });

            ui.add_space(10.0);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.heading("Publisher");
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Topic");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.topic)
                                .desired_width(f32::INFINITY)
                                .hint_text("Blank sends a single-frame payload"),
                        );
                    });

                    ui.add_space(6.0);
                    ui.label("Payload");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.payload)
                            .desired_rows(6)
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(self.session.is_some(), egui::Button::new("Publish"))
                            .clicked()
                        {
                            self.publish();
                        }

                        if ui.button("Fill Ping").clicked() {
                            self.topic = "demo".to_string();
                            self.payload = "{\"kind\":\"ping\",\"source\":\"server\"}".to_string();
                        }

                        if ui.button("Clear Payload").clicked() {
                            self.payload.clear();
                        }
                    });
                });

            ui.add_space(10.0);

            let compact_layout = ui.available_width() < 900.0;
            if compact_layout {
                draw_sent_panel(ui, &self.sent_messages);
                ui.add_space(10.0);
                draw_log_panel(ui, &mut self.logs);
            } else {
                ui.columns(2, |columns| {
                    draw_sent_panel(&mut columns[0], &self.sent_messages);
                    draw_log_panel(&mut columns[1], &mut self.logs);
                });
            }
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn draw_sent_panel(ui: &mut egui::Ui, sent_messages: &[PublishedMessage]) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Sent Messages");
                ui.label(
                    egui::RichText::new(format!("{}", sent_messages.len()))
                        .small()
                        .color(egui::Color32::GRAY),
                );
            });
            ui.add_space(6.0);

            egui::ScrollArea::vertical()
                .max_height(320.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if sent_messages.is_empty() {
                        ui.label(
                            egui::RichText::new("Nothing published yet.")
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                        return;
                    }

                    for message in sent_messages.iter().rev() {
                        egui::Frame::group(ui.style())
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&message.at)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                    );
                                    ui.label(
                                        egui::RichText::new(if message.topic.is_empty() {
                                            "<no topic>"
                                        } else {
                                            &message.topic
                                        })
                                        .strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} frame{}",
                                            message.frame_count,
                                            if message.frame_count == 1 { "" } else { "s" }
                                        ))
                                        .small()
                                        .color(egui::Color32::GRAY),
                                    );
                                });
                                ui.label(&message.payload);
                                ui.label(
                                    egui::RichText::new(&message.endpoint)
                                        .small()
                                        .color(egui::Color32::GRAY),
                                );
                            });
                        ui.add_space(6.0);
                    }
                });
        });
}

fn draw_log_panel(ui: &mut egui::Ui, logs: &mut Vec<LogEntry>) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Activity Log");
                if ui.button("Clear").clicked() {
                    logs.clear();
                }
            });
            ui.add_space(6.0);

            if logs.is_empty() {
                ui.label(
                    egui::RichText::new("No log entries.")
                        .small()
                        .color(egui::Color32::GRAY),
                );
                return;
            }

            let hidden_count = logs.len().saturating_sub(12);
            if hidden_count > 0 {
                ui.label(
                    egui::RichText::new(format!(
                        "Showing the latest 12 entries out of {}.",
                        logs.len()
                    ))
                    .small()
                    .color(egui::Color32::GRAY),
                );
                ui.add_space(4.0);
            }

            for entry in logs.iter().rev().take(12) {
                let color = match entry.level {
                    LogLevel::Info => egui::Color32::from_rgb(179, 200, 228),
                    LogLevel::Warning => egui::Color32::from_rgb(235, 189, 82),
                    LogLevel::Error => egui::Color32::from_rgb(235, 112, 112),
                };

                ui.label(
                    egui::RichText::new(format!("[{}] {}", entry.at, entry.message)).color(color),
                );
            }
        });
}

fn trim_vec<T>(items: &mut Vec<T>, max_len: usize) {
    if items.len() > max_len {
        let extra = items.len() - max_len;
        items.drain(0..extra);
    }
}
