use crate::app::state::IPTVApp;
use eframe::egui;
use std::process::{Command, Stdio};

pub fn render_playback(app: &mut IPTVApp, ui: &mut egui::Ui, stream_url: String) {
    ui.heading("Now Playing:");
    ui.label(format!("Stream URL: {}", stream_url));

    if app.mpv_process.is_none() {
        app.mpv_process = Some(
            Command::new("mpv")
                .arg(&stream_url)
                .arg("--no-terminal")
                .arg("--force-window=yes")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("Failed to start MPV"),
        );
    }

    if ui.button("Stop").clicked() {
        if let Some(mut process) = app.mpv_process.take() {
            let _ = process.kill();
        }
        if let Some(view) = app.view_stack.pop() {
            app.current_view = view;
        }
    }
}
