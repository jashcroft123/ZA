use crate::{default_ipc_endpoint, default_tcp_endpoint};
use eframe::egui;

pub struct LauncherApp;

impl LauncherApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ZeroMQ Pub/Sub Test Bench");
            ui.label("This workspace now has separate desktop apps for the PUB server and the SUB client.");
            ui.add_space(8.0);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.label("Run the server app:");
                    ui.monospace("cargo run --bin server");
                    ui.add_space(6.0);
                    ui.label("Run one or more client apps:");
                    ui.monospace("cargo run --bin client");
                });

            ui.add_space(10.0);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.heading("Defaults");
                    ui.label(format!("TCP example: {}", default_tcp_endpoint()));
                    ui.label(format!("IPC example: {}", default_ipc_endpoint()));
                });

            ui.add_space(10.0);

            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.heading("Included Controls");
                    ui.label("The server binds and disconnects cleanly.");
                    ui.label("The client connects, disconnects, and updates its subscription filter.");
                    ui.label("Both apps keep activity logs so you can watch ZeroMQ behavior while testing.");
                });
        });
    }
}
