use eframe::NativeOptions;
use crate::gui::IPTVApp;
use log::{debug, error, log_enabled, info, Level};
use tracing_subscriber;
use egui_extras;

mod api_client;
mod database;
mod gui;
mod models;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> eframe::Result {
    env_logger::init();
    // tracing_subscriber::fmt::init(); // Initialize tracing subscriber

    let options = eframe::NativeOptions {
        run_and_return: false,
        // initial_window_size: Some([800.0, 600.0].into()),
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        // vsync: true, // Enables vertical synchronization for consistent frame rate
        ..Default::default()
    };

    eframe::run_native(
        "IPTV App",
        options,
        Box::new(|_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            Ok(Box::new(IPTVApp::new(&_cc.egui_ctx)))}),
    )
}
