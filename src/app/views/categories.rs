use crate::app::state::{AppView, IPTVApp};
use eframe::egui;
use egui::accesskit::Size;

pub fn render_categories(app: &mut IPTVApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Categories");

        app.view_stack.push(app.current_view.clone());

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

        let current_dir = std::env::current_dir().unwrap();
        log::info!("Current dir: {:?}", current_dir);

        let img = egui_extras::image::load_svg_bytes_with_size(
            std::include_bytes!("/home/zmostafa/github/xtream/assets/movies.svg"),
            Some(egui::SizeHint::Size(150, 150)),
        );
        match img {
            Ok(i) => {
                ui.label("Image loaded successfully");
                let texture = ctx.load_texture("icon", i, Default::default());
                // ui.image(&texture);
                if ui.add(egui::ImageButton::new( &texture)).clicked() {
                    app.current_view = AppView::MoviesCategories;
                }
            }
            Err(e) => {
                ui.label(format!("Error loading image: {:?}", e));
            }
        }
    });
}
