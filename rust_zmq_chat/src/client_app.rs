use crate::network::{
    start_subscriber, ConnectionState, LogEntry, LogLevel, ReceivedMessage, SubscriberEvent,
    SubscriberHandle,
};
use crate::{default_ipc_endpoint, default_tcp_endpoint};
use eframe::egui;
use std::time::Duration;

const MAX_LOGS: usize = 300;
const MAX_MESSAGES: usize = 300;

pub struct ClientApp {
    endpoint: String,
    filter: String,
    status: ConnectionState,
    session: Option<SubscriberHandle>,
    logs: Vec<LogEntry>,
    received_messages: Vec<ReceivedMessage>,
}

impl ClientApp {
    pub fn new() -> Self {
        Self {
            endpoint: default_tcp_endpoint(),
            filter: String::new(),
            status: ConnectionState::Stopped,
            session: None,
            logs: vec![LogEntry::info(
                "Ready. Connect to a PUB endpoint and watch the incoming frames.",
            )],
            received_messages: Vec::new(),
        }
    }

    fn connect(&mut self) {
        self.disconnect();

        match start_subscriber(self.endpoint.clone(), self.filter.clone()) {
            Ok(handle) => {
                self.status = ConnectionState::Running;
                self.logs.push(LogEntry::info(format!(
                    "SUB client connected to {}",
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
                "SUB client disconnected from {}",
                session.endpoint()
            )));
            trim_vec(&mut self.logs, MAX_LOGS);
        }

        self.status = ConnectionState::Stopped;
    }

    fn apply_filter(&mut self) {
        let Some(session) = &self.session else {
            self.logs.push(LogEntry::warning(
                "Connect the subscriber before applying a filter.",
            ));
            trim_vec(&mut self.logs, MAX_LOGS);
            return;
        };

        if let Err(error) = session.update_filter(self.filter.clone()) {
            self.logs.push(LogEntry::error(error));
            trim_vec(&mut self.logs, MAX_LOGS);
        }
    }

    fn drain_events(&mut self) {
        let mut disconnected = false;

        if let Some(session) = &mut self.session {
            while let Some(event) = session.try_recv() {
                match event {
                    SubscriberEvent::State(state) => {
                        self.status = state;
                        if state == ConnectionState::Stopped {
                            disconnected = true;
                        }
                    }
                    SubscriberEvent::Log(entry) => {
                        self.logs.push(entry);
                    }
                    SubscriberEvent::Message(message) => {
                        self.received_messages.push(message);
                    }
                }
            }
        }

        trim_vec(&mut self.logs, MAX_LOGS);
        trim_vec(&mut self.received_messages, MAX_MESSAGES);

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
            ConnectionState::Running => ("Connected", egui::Color32::from_rgb(116, 201, 132)),
            ConnectionState::Stopped => ("Stopped", egui::Color32::from_rgb(220, 94, 94)),
        }
    }
}

impl Drop for ClientApp {
    fn drop(&mut self) {
        self.disconnect();
    }
}

impl eframe::App for ClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            let (status_label, status_color) = self.status_text();

            ui.heading("ZeroMQ SUB Client");
            ui.label("Connect a SUB socket to any TCP or IPC PUB endpoint, then change the topic filter without restarting the app.");
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
                        ui.label("Remote Endpoint");
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
                        ui.label("Subscription Filter");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.filter)
                                .desired_width(f32::INFINITY)
                                .hint_text("Blank receives everything"),
                        );
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        self.endpoint_presets(ui, self.session.is_none());

                        if ui
                            .add_enabled(self.session.is_none(), egui::Button::new("Connect"))
                            .clicked()
                        {
                            self.connect();
                        }

                        if ui
                            .add_enabled(self.session.is_some(), egui::Button::new("Disconnect"))
                            .clicked()
                        {
                            self.disconnect();
                        }

                        if ui
                            .add_enabled(self.session.is_some(), egui::Button::new("Apply Filter"))
                            .clicked()
                        {
                            self.apply_filter();
                        }
                    });
                });

            ui.add_space(10.0);

            let compact_layout = ui.available_width() < 900.0;
            if compact_layout {
                draw_received_panel(ui, &mut self.received_messages);
                ui.add_space(10.0);
                draw_log_panel(ui, &mut self.logs);
            } else {
                ui.columns(2, |columns| {
                    draw_received_panel(&mut columns[0], &mut self.received_messages);
                    draw_log_panel(&mut columns[1], &mut self.logs);
                });
            }
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn draw_received_panel(ui: &mut egui::Ui, messages: &mut Vec<ReceivedMessage>) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Received Messages");
                if ui.button("Clear").clicked() {
                    messages.clear();
                }
            });
            ui.add_space(6.0);

            egui::ScrollArea::vertical()
                .max_height(380.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if messages.is_empty() {
                        ui.label(
                            egui::RichText::new("Nothing received yet.")
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                        return;
                    }

                    for message in messages.iter().rev() {
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
                                            message.frames.len(),
                                            if message.frames.len() == 1 { "" } else { "s" }
                                        ))
                                        .small()
                                        .color(egui::Color32::GRAY),
                                    );
                                });
                                ui.label(&message.payload);
                                if message.frames.len() > 1 {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Frames: {}",
                                            message.frames.join(" | ")
                                        ))
                                        .small()
                                        .color(egui::Color32::GRAY),
                                    );
                                }
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
