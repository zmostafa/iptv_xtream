use crate::{
    app::state::{AppView, IPTVApp},
    utils,
};
use eframe::egui;

pub fn render_movies_search(app: &mut IPTVApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Search Movies");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut app.search_query);
        });

        if ui.button("Search").clicked() {
            app.movies_search_results = utils::search_movies(app, &app.search_query);
        }

        if ui.button("Back").clicked() {
            app.current_view = AppView::MoviesCategories;
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for movie in &app.movies_search_results {
                ui.horizontal(|ui| {
                    ui.label(&movie.name);
                    if ui.button("▶").clicked() {
                        let stream_url = format!(
                            "{}/{}/{}/{}/{}.{}",
                            app.api_url,
                            "movie",
                            app.username,
                            app.password,
                            movie.stream_id,
                            movie.container_extension
                        );
                        app.view_stack.push(app.current_view.clone());
                        app.current_view = AppView::Playback(stream_url);
                    }
                });
            }
        });
    });
}

pub fn render_series_search(app: &mut IPTVApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Search Series");

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut app.search_query);
        });

        if ui.button("Search").clicked() {
            app.series_search_results = utils::search_series(app, &app.search_query);
        }

        if ui.button("Back").clicked() {
            app.current_view = AppView::MoviesCategories;
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for serie in &app.series_search_results {
                ui.horizontal(|ui| {
                    // Display the serie name
                    let display_name = utils::preprocess_arabic_text_v1(&serie.name);
                    if ui.button(&display_name).clicked() {
                        if let Some(serie_id) = serie.series_id {
                            app.current_view = AppView::SeriesDetail(
                                serie.category_id.clone(),
                                serie_id,
                                serie.category_id.to_string(),
                            );
                        }
                    }
                });
            }
        });
    });
}
