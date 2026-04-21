use eframe::egui;
use rust_zmq_chat::launcher::LauncherApp;

fn main() -> Result<(), eframe::Error> {
    let _ = tracing_subscriber::fmt::try_init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 360.0])
            .with_title("ZeroMQ Test Bench"),
        ..Default::default()
    };

    eframe::run_native(
        "ZeroMQ Test Bench",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(LauncherApp::new(cc)))
        }),
    )
}
