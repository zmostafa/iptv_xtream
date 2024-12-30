use crate::app::state::{AppView, IPTVApp};
use eframe::egui;

pub fn render_login(app: &mut IPTVApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Login to IPTV");

        egui::Grid::new("login_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.label("Server URL: ");
                ui.text_edit_singleline(&mut app.api_url);
                ui.end_row();

                ui.label("Username: ");
                ui.text_edit_singleline(&mut app.username);
                ui.end_row();

                ui.label("Password: ");
                ui.text_edit_singleline(&mut app.password);
                ui.end_row();
            });

        if ui.button("Login").clicked() {
            if futures::executor::block_on(app.authenticate()) {
                app.authenticated = true;
                app.current_view = AppView::Categories;
            } else {
                log::error!("Authentication failed.");
            }
        }
    });
}
