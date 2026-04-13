use rust_zmq_chat::app::ChatApp;
use eframe::egui;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Set up logging for debugging
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 200.0])
            .with_title("ZMQ Chat Controller"),
        ..Default::default()
    };

    eframe::run_native(
        "ZMQ Chat",
        options,
        Box::new(|cc| {
            // High visibility / Dark mode
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ChatApp::new(cc)))
        }),
    )
}
