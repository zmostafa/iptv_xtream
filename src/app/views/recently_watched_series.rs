use crate::{
    app::state::{AppView, IPTVApp},
    utils,
};
use eframe::egui;
use egui::vec2;
use std::sync::{Arc, Mutex};

pub fn render_recently_watched_series(app: &mut IPTVApp, ctx: &egui::Context) {
    if app.series_cache.is_empty() {
        log::info!("Fetching recently watched series");
        app.series_cache = app.db.get_recently_watched_series();
    }

    let series_list = app.series_cache.clone();
    let cache = app.image_cache.clone();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Recently watched series");

        if ui.button("Back").clicked() {
            let view = app.view_stack.pop().unwrap();
            app.current_view = view;
        }

        ui.separator();

        // Calculate grid properties based on screen width
        let available_width = ui.available_width();
        let min_item_width = 150.0; // Minimum width for each grid item
        let num_columns = (available_width / min_item_width).floor() as usize; // Fit as many columns as possible
        let item_size = available_width / num_columns as f32; // Dynamically size each item

        // Add a scrollable grid to display the series
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("seriess_grid")
                .spacing([20.0, 20.0]) // Spacing between items
                .min_col_width(item_size) // Minimum column width
                .show(ui, |ui| {
                    for (i, serie) in series_list.iter().enumerate() {
                        // Display the serie poster and name
                        ui.vertical(|ui| {
                            log::info!("Getting serie info : {}", &serie.name);
                            let image_url = serie.cover.clone();

                            if app.image_cache.is_cached(&image_url) {
                                // Load image from cache
                                if let Ok(image_data) = app.image_cache.load_image(&image_url) {
                                    log::info!("Image found in cache");
                                    ui.add(
                                        egui::Image::from_bytes(serie.name.clone(), image_data)
                                            .rounding(10.0)
                                            .fit_to_exact_size(vec2(150.0, 150.0)),
                                    );
                                    ctx.request_repaint();
                                } else {
                                    ui.label("[Error Loading Image]");
                                }
                            } else {
                                // Placeholder for loading
                                ui.spinner();
                                log::info!("Image not found in cache {}, downloading", image_url);
                                if !app.ongoing_requests.contains(&image_url) {
                                    app.ongoing_requests.insert(image_url.clone());
                                    // Fetch image in the background
                                    let image_url_clone = image_url.clone();
                                    let cache_clone = cache.clone();
                                    let client_clone = app.client.clone();
                                    let ctx_clone = ctx.clone();

                                    tokio::spawn(async move {
                                        if let Err(err) = utils::fetch_and_cache_image(
                                            client_clone,
                                            cache_clone,
                                            &image_url_clone,
                                        )
                                        .await
                                        {
                                            log::error!("Failed to fetch image: {}", err);
                                        }
                                        ctx_clone.request_repaint(); // Update UI
                                    });
                                }
                            }

                            // Display the serie name
                            let display_name = utils::preprocess_arabic_text_v1(&serie.name);
                            if ui.button(&display_name).clicked() {
                                if let serie_id = serie.series_id {
                                    app.view_stack.push(app.current_view.clone());
                                    app.current_view = AppView::SeriesDetail(
                                        serie.category_id.clone().expect("REASON"),
                                        serie_id,
                                        serie.category_id.as_ref().expect("REASON").to_string(),
                                    );
                                }
                            }
                        });

                        // Add a new row every 4 items (adjust as needed)
                        if (i + 1) % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    });
}
