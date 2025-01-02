use crate::{
    app::state::{AppView, IPTVApp},
    utils,
};
use eframe::egui;
use egui::vec2;

pub fn render_live_streams(
    app: &mut IPTVApp,
    ctx: &egui::Context,
    category_id: &str,
    category_name: &str,
) {
    let streams = app.db.get_live_streams(category_id);
    let cache = app.image_cache.clone();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(category_name);

        if ui.button("Back").clicked() {
            let view = app.view_stack.pop().unwrap();
            app.current_view = view;
        }

        ui.separator();

        // Calculate grid properties
        let available_width = ui.available_width();
        let min_item_width = 150.0;
        let num_columns = (available_width / min_item_width).floor() as usize;
        let item_size = available_width / num_columns as f32;

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("live_grid")
                .spacing([20.0, 20.0])
                .min_col_width(item_size)
                .show(ui, |ui| {
                    for (i, stream) in streams.iter().enumerate() {
                        ui.vertical(|ui| {
                            let image_url = stream.stream_icon.clone();

                            if app.image_cache.is_cached(&image_url) {
                                // Load image from cache
                                if let Ok(image_data) = app.image_cache.load_image(&image_url) {
                                    ui.add(
                                        egui::Image::from_bytes(stream.name.clone(), image_data)
                                            .rounding(10.0)
                                            .fit_to_exact_size(egui::vec2(150.0, 150.0)),
                                    );
                                } else {
                                    ui.label("[Error Loading Image]");
                                }
                            } else {
                                // Placeholder for loading
                                ui.spinner();
                                if !app.ongoing_requests.contains(&image_url) {
                                    app.ongoing_requests.insert(image_url.clone());
                                    // Fetch image in the background
                                    let image_url_clone = image_url.clone();
                                    let cache_clone = cache.clone();
                                    let client_clone = app.client.clone();
                                    // let ctx_clone = ctx.clone();

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
                                        // ctx_clone.request_repaint(); // Update UI
                                    });
                                }
                            }

                            // Display the live name
                            let display_name = utils::preprocess_arabic_text_v1(&stream.name);
                            ui.label(&display_name);

                            // Play button
                            if ui.button("▶").clicked() {
                                let stream_url = format!(
                                    "{}/{}/{}/{}/{}.{}",
                                    app.api_url,
                                    "live",
                                    app.username,
                                    app.password,
                                    stream.stream_id,
                                    "ts"
                                );
                                app.view_stack.push(app.current_view.clone());
                                app.current_view = AppView::Playback(stream_url.to_string());
                            }
                        });

                        // End row every `num_columns` items
                        if (i + 1) % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    });
}
