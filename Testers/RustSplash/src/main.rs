#![windows_subsystem = "windows"]

use std::env;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use eframe::egui;
use serde::Deserialize;

static PROGRESS: AtomicU32 = AtomicU32::new(0);
static STATUS: OnceLock<Arc<Mutex<String>>> = OnceLock::new();
static APP_NAME: OnceLock<String> = OnceLock::new();
static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Deserialize)]
struct SplashSignal {
    progress: Option<u32>,
    status: Option<String>,
    done: Option<bool>,
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    let (target_app, app_stem) = if args.len() >= 2 {
        let target = args[1].clone();
        let target_path = Path::new(&target);

        // If it's a relative path that exists, use the absolute version to be safe
        let mut final_path = if target_path.exists() {
            target_path
                .canonicalize()
                .unwrap_or_else(|_| target_path.to_path_buf())
        } else {
            target_path.to_path_buf()
        };

        // Strip UNC prefix on Windows if present (some apps don't handle \\?\ paths well)
        #[cfg(windows)]
        {
            let path_str = final_path.to_string_lossy();
            if path_str.starts_with(r"\\?\") {
                final_path = std::path::PathBuf::from(&path_str[4..]);
            }
        }

        let stem = final_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Application")
            .to_string();

        (Some(final_path), stem)
    } else {
        (None, "Splash Demo".to_string())
    };

    let _ = APP_NAME.set(app_stem);
    let status_arc = Arc::new(Mutex::new(String::from("Initializing...")));
    let _ = STATUS.set(status_arc.clone());

    // Launch target app if provided
    if let Some(target) = target_app {
        let _ = Command::new(target).spawn();
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_inner_size([540.0, 200.0])
            .with_resizable(false)
            .with_taskbar(false),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME.get().unwrap(),
        options,
        Box::new(|cc| {
            // Start ZMQ listener thread with the egui context
            start_zmq_listener(cc.egui_ctx.clone());
            Box::new(SplashApp::default())
        }),
    )
}

struct SplashApp {
    centered: bool,
}

impl Default for SplashApp {
    fn default() -> Self {
        Self { centered: false }
    }
}

impl eframe::App for SplashApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Fully transparent window background
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.centered {
            if let Some(cmd) = egui::ViewportCommand::center_on_screen(ctx) {
                ctx.send_viewport_cmd(cmd);
                self.centered = true;
            }
        }

        if DONE.load(Ordering::Relaxed) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Use a container with rounding and stroke for the "premium" look
        let panel_frame = egui::Frame {
            fill: egui::Color32::from_rgb(18, 18, 18),
            rounding: egui::Rounding::same(16.0),
            inner_margin: egui::Margin::same(24.0),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(40)),
            ..Default::default()
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::none()) // Transparent panel
            .show(ctx, |ui| {
                panel_frame.show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        // App Title
                        let title = APP_NAME.get().cloned().unwrap_or_default();
                        ui.add(egui::Label::new(
                            egui::RichText::new(title)
                                .color(egui::Color32::WHITE)
                                .size(32.0)
                                .strong(),
                        ));

                        ui.add_space(8.0);

                        // Status Text
                        if let Some(status_mutex) = STATUS.get() {
                            if let Ok(status) = status_mutex.lock() {
                                ui.add(egui::Label::new(
                                    egui::RichText::new(&*status)
                                        .color(egui::Color32::from_gray(160))
                                        .size(16.0),
                                ));
                            }
                        }

                        ui.add_space(32.0);

                        // Progress Bar
                        let progress = PROGRESS.load(Ordering::Relaxed) as f32 / 100.0;

                        let (rect, _) = ui.allocate_at_least(
                            egui::vec2(ui.available_width() - 40.0, 8.0),
                            egui::Sense::hover(),
                        );

                        // Track
                        ui.painter().rect_filled(
                            rect,
                            egui::Rounding::same(4.0),
                            egui::Color32::from_rgb(45, 45, 45),
                        );

                        // Fill with a slight gradient or glow effect
                        let mut fill_rect = rect;
                        fill_rect.set_width(rect.width() * progress);

                        if progress > 0.0 {
                            ui.painter().rect_filled(
                                fill_rect,
                                egui::Rounding::same(4.0),
                                egui::Color32::from_rgb(0, 122, 255), // Vibrant blue
                            );

                            // Add a subtle glow to the progress bar
                            ui.painter().rect_filled(
                                fill_rect,
                                egui::Rounding::same(4.0),
                                egui::Color32::from_rgba_premultiplied(0, 122, 255, 40),
                            );
                        }
                    });
                });
            });
    }
}

fn start_zmq_listener(ctx: egui::Context) {
    thread::spawn(move || {
        let context = zmq::Context::new();
        let socket = match context.socket(zmq::PULL) {
            Ok(s) => s,
            Err(_) => return,
        };

        let mut pipe_path = std::env::temp_dir();
        pipe_path.push("fast_splash.ipc");
        let pipe_str = pipe_path.to_string_lossy().replace("\\", "/");
        let endpoint = format!("ipc://{}", pipe_str);

        if socket.bind(&endpoint).is_ok() {
            loop {
                let mut msg = zmq::Message::new();
                if socket.recv(&mut msg, 0).is_err() {
                    break;
                }

                if let Ok(text) = std::str::from_utf8(&msg) {
                    if let Ok(signal) = serde_json::from_str::<SplashSignal>(text) {
                        if let Some(p) = signal.progress {
                            PROGRESS.store(p.min(100), Ordering::Relaxed);
                        }
                        if let Some(s) = signal.status {
                            if let Some(status_mutex) = STATUS.get() {
                                if let Ok(mut status) = status_mutex.lock() {
                                    *status = s;
                                }
                            }
                        }
                        if signal.done.unwrap_or(false) {
                            DONE.store(true, Ordering::Relaxed);
                            ctx.request_repaint();
                            break;
                        }
                        ctx.request_repaint();
                    }
                }
            }
        }
    });
}
