use crate::app::state::{AppView, IPTVApp};
use eframe::egui;

pub fn render_categories(app: &mut IPTVApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Categories");

        if ui.button("Live Streams").clicked() {
            app.current_view = AppView::LiveCategories;
        }

        if ui.button("Movies Streams").clicked() {
            app.current_view = AppView::MoviesCategories;
        }

        if ui.button("Series Streams").clicked() {
            app.current_view = AppView::SeriesCategories;
        }

        if ui.button("Fetch Content").clicked() {
            futures::executor::block_on(app.fetch_and_save_all_live_streams());
        }
    });
}
