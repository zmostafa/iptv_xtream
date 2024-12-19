mod api;
mod models;
mod storage;

use crate::api::{fetch_and_store_categories, fetch_and_store_streams, login};
use crate::models::{Category, StreamVariant};
use crate::storage::{load_credentials, save_credentials, Credentials};
use eframe::egui;
use std::process::{Child, Command, Stdio};

struct IPTVApp {
    db: sled::Db,
    server: String,
    username: String,
    password: String,
    login_status: String,
    live_categories: Vec<Category>,
    vod_categories: Vec<Category>,
    streams: Vec<StreamVariant>,
    mpv_process: Option<Child>,
    current_view: View,
}

enum View {
    Login,
    MainMenu,
    Categories(String), // Live or VOD
    Streams(String),    // Category ID
    Playback(String),   // Stream URL
}

impl Default for IPTVApp {
    fn default() -> Self {
        let creds = load_credentials().unwrap_or_default();
        Self {
            db: sled::open("iptv.db").expect("Failed to open database"),
            server: creds.server,
            username: creds.username,
            password: creds.password,
            login_status: "Not Logged In".to_string(),
            live_categories: vec![],
            vod_categories: vec![],
            streams: vec![],
            mpv_process: None,
            current_view: View::Login,
        }
    }
}

impl eframe::App for IPTVApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        configure_fonts(ctx);

        egui::CentralPanel::default().show(ctx, |ui| match &self.current_view {
            View::Login => self.display_login(ui),
            View::MainMenu => self.display_main_menu(ui),
            View::Categories(category_type) => self.display_categories(ui, category_type.clone()),
            View::Streams(category_id) => self.display_streams(ui, category_id.clone()),
            View::Playback(stream_url) => self.display_playback(ui, stream_url.clone()),
        });
    }
}

impl IPTVApp {
    fn display_login(&mut self, ui: &mut egui::Ui) {
        ui.heading("Login to IPTV");

        ui.horizontal(|ui| {
            ui.label("Server:");
            ui.text_edit_singleline(&mut self.server);
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);
            ui.label("Password:");
            let mut masked_password = "*".repeat(self.password.len());
            if ui.text_edit_singleline(&mut masked_password).changed() {
                self.password = masked_password.clone();
            }
        });

        if ui.button("Login").clicked() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(login(&self.server, &self.username, &self.password));
            if result.is_ok() {
                self.login_status = "Login Successful!".to_string();
                save_credentials(&Credentials {
                    server: self.server.clone(),
                    username: self.username.clone(),
                    password: self.password.clone(),
                })
                .expect("Failed to save credentials");
                self.current_view = View::MainMenu;
            } else {
                self.login_status = "Login Failed.".to_string();
            }
        }

        ui.label(&self.login_status);
    }

    fn display_main_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button("Live Streams").clicked() {
            self.current_view = View::Categories("live".to_string());
        }

        if ui.button("VOD").clicked() {
            self.current_view = View::Categories("vod".to_string());
        }
    }

    fn display_categories(&mut self, ui: &mut egui::Ui, category_type: String) {
        let categories = if category_type == "live" {
            &mut self.live_categories
        } else {
            &mut self.vod_categories
        };

        if categories.is_empty() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            *categories = rt
                .block_on(fetch_and_store_categories(
                    &self.server,
                    &self.username,
                    &self.password,
                    if category_type == "live" {
                        "get_live_categories"
                    } else {
                        "get_vod_categories"
                    },
                    &self.db,
                ))
                .unwrap_or_default();
        }

        for category in categories {
            if ui.button(&category.category_name).clicked() {
                self.current_view = View::Streams(category.category_id.clone());
            }
        }

        if ui.button("Back").clicked() {
            self.current_view = View::MainMenu;
        }
    }

    fn display_streams(&mut self, ui: &mut egui::Ui, category_id: String) {
        if self.streams.is_empty() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            self.streams = rt
                .block_on(fetch_and_store_streams(
                    &self.server,
                    &self.username,
                    &self.password,
                    &category_id,
                    if category_id.starts_with("live") {
                        "get_live_streams"
                    } else {
                        "get_vod_streams"
                    },
                    &self.db,
                ))
                .unwrap_or_default();
        }

        for stream in &self.streams {
            if ui.button(&stream.get_name()).clicked() {
                let stream_url = format!(
                    "{}/{}/{}/{}/{}.{}",
                    self.server,
                    if category_id.starts_with("live") {
                        "live"
                    } else {
                        "movie"
                    },
                    self.username,
                    self.password,
                    stream.get_id(),
                    stream.get_extension()
                );
                self.current_view = View::Playback(stream_url);
            }
        }

        if ui.button("Back").clicked() {
            self.streams.clear();
            self.current_view = View::Categories(if category_id.starts_with("live") {
                "live".to_string()
            } else {
                "vod".to_string()
            });
        }
    }

    fn display_playback(&mut self, ui: &mut egui::Ui, stream_url: String) {
        ui.heading("Now Playing:");
        ui.label(format!("Stream URL: {}", stream_url));

        if self.mpv_process.is_none() {
            self.mpv_process = Some(
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
            if let Some(mut process) = self.mpv_process.take() {
                let _ = process.kill();
            }
            self.current_view = View::MainMenu;
        }
    }
}

fn configure_fonts(ctx: &egui::Context) {
    use egui::FontFamily::{Monospace, Proportional};

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "custom_arabic".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/Lateef-Regular.ttf")),
    );

    fonts
        .families
        .entry(Proportional)
        .or_default()
        .insert(0, "custom_arabic".to_owned());

    fonts
        .families
        .entry(Monospace)
        .or_default()
        .insert(0, "custom_arabic".to_owned());

    ctx.set_fonts(fonts);
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "IPTV Application",
        options,
        Box::new(|_| Box::new(IPTVApp::default())),
    )
}
