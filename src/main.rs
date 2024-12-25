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
async fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    // tracing_subscriber::fmt::init(); // Initialize tracing subscriber
    let app = IPTVApp::new();
    eframe::run_native(
        "IPTV App",
        NativeOptions::default(),
        Box::new(|_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            Ok(Box::new(app))}),
    )
}
