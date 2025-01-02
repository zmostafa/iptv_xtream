use crate::{
    app::state::{AppView, IPTVApp},
    utils,
};
use eframe::egui;
use egui::vec2;
use std::sync::{Arc, Mutex};

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

        // Calculate grid properties based on screen width
        let available_width = ui.available_width();
        let min_item_width = 150.0; // Minimum width for each grid item
        let num_columns = (available_width / min_item_width).floor() as usize; // Fit as many columns as possible
        let item_size = available_width / num_columns as f32; // Dynamically size each item
                                                              // Collect the necessary data into a separate vector
        let movies_search_results = app.movies_search_results.clone();

        // Add a scrollable grid to display the movies
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("movies_grid")
                .spacing([20.0, 20.0]) // Spacing between items
                .min_col_width(item_size) // Minimum column width
                .show(ui, |ui| {
                    log::info!("Displaying movies grid");
                    for (i, stream) in movies_search_results.iter().enumerate() {
                        // Display the movie poster and name
                        ui.vertical(|ui| {
                            let image_url = stream.stream_icon.clone();

                            if app.image_cache.is_cached(&image_url) {
                                // Load image from cache
                                if let Ok(image_data) = app.image_cache.load_image(&image_url) {
                                    ui.add(
                                        egui::Image::from_bytes(stream.name.clone(), image_data)
                                            .rounding(10.0)
                                            .fit_to_exact_size(vec2(150.0, 150.0)),
                                    );
                                } else {
                                    ui.label("[Error Loading Image]");
                                }
                            } else {
                                // Placeholder for loading
                                ui.spinner();

                                if !app.ongoing_requests.contains(&image_url) {
                                    // Fetch image in the background
                                    log::info!("Fetching image in the background");
                                    app.ongoing_requests.insert(image_url.clone());
                                    let image_url_clone = image_url.clone();
                                    let cache_clone = app.image_cache.clone();
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
                            ctx.request_repaint();

                            // Display the movie name
                            let display_name = utils::preprocess_arabic_text_v1(&stream.name);
                            ui.label(&display_name);
                            let stream_url = format!(
                                "{}/{}/{}/{}/{}.{}",
                                app.api_url,
                                "movie",
                                app.username,
                                app.password,
                                stream.stream_id,
                                stream.container_extension
                            );

                            ui.horizontal(|ui| {
                                // Add a play button
                                if ui.button("▶").clicked() {
                                    app.view_stack.push(app.current_view.clone());
                                    utils::play_media(app, &stream.stream_id, &stream_url);
                                }

                                if let Some(download_path) = app.db.is_downloaded(&stream.stream_id)
                                {
                                    log::debug!("Movie is available offline");
                                    ui.colored_label(egui::Color32::GREEN, "✅");
                                    // Add a remove button
                                    if ui.button("🗑").clicked() {
                                        // Remove the downloaded episode
                                        if let Err(err) = utils::remove_downloaded_episode(
                                            app,
                                            &stream.stream_id,
                                            &download_path,
                                        ) {
                                            log::error!(
                                                "Failed to remove downloaded movie: {}",
                                                err
                                            );
                                        } else {
                                            log::info!(
                                                "Downloaded episode removed: {}",
                                                stream.name
                                            );
                                        }
                                    }
                                } else {
                                    let progress = Arc::new(Mutex::new(0.0));

                                    if ui.button("⬇").clicked() {
                                        log::debug!("Downloading Movie: {}", stream.name);
                                        let save_path = format!(
                                            "iptv_cache/{}.{}",
                                            stream.name, stream.container_extension
                                        );
                                        let db_clone = app.db.clone();
                                        let url_clone = stream_url.clone();
                                        let progress_clone_for_download = Arc::clone(&progress);
                                        let stream_clone = stream.clone();

                                        // Add progress to active downloads
                                        app.active_downloads
                                            .insert(stream.stream_id.clone(), progress.clone());
                                        let mut active_downloads_clone =
                                            app.active_downloads.clone();

                                        tokio::spawn({
                                            async move {
                                                if let Err(e) = utils::download_with_wget_async(
                                                    &url_clone,
                                                    &save_path,
                                                    progress_clone_for_download,
                                                )
                                                .await
                                                {
                                                    log::error!("Download failed: {}", e);
                                                } else {
                                                    log::info!("Download completed: {}", save_path);
                                                    log::info!("Saving download to database");
                                                    // Remove from active downloads
                                                    active_downloads_clone
                                                        .remove(&stream_clone.stream_id.clone());
                                                    db_clone.save_download(
                                                        stream_clone.stream_id.clone(),
                                                        &save_path,
                                                    );
                                                }
                                            }
                                        });
                                    }
                                    // Update progress bar in UI
                                    if let Some(progress) =
                                        app.active_downloads.get(&stream.stream_id)
                                    {
                                        let progress_value = *progress.lock().unwrap();
                                        ui.add(
                                            egui::ProgressBar::new(progress_value)
                                                .text("Downloading..."),
                                        );
                                    }
                                }
                            });
                        });

                        // Add a new row every 4 items (adjust as needed)
                        if (i + 1) % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });

        // egui::ScrollArea::vertical().show(ui, |ui| {
        //     for movie in &app.movies_search_results {
        //         ui.horizontal(|ui| {
        //             ui.label(&movie.name);
        //             if ui.button("▶").clicked() {
        //                 let stream_url = format!(
        //                     "{}/{}/{}/{}/{}.{}",
        //                     app.api_url,
        //                     "movie",
        //                     app.username,
        //                     app.password,
        //                     movie.stream_id,
        //                     movie.container_extension
        //                 );
        //                 app.view_stack.push(app.current_view.clone());
        //                 app.current_view = AppView::Playback(stream_url);
        //             }
        //         });
        //     }
        // });
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
            app.current_view = AppView::SeriesCategories;
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
                    for (i, serie) in app.series_search_results.iter().enumerate() {
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
                                    let cache_clone = app.image_cache.clone();
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
                                if let Some(serie_id) = serie.series_id {
                                    log::info!("Serie selected: {}", &serie_id);
                                    app.view_stack.push(app.current_view.clone());
                                    app.current_view = AppView::SeriesDetail(
                                        serie.category_id.clone(),
                                        serie_id,
                                        serie.category_id.to_string(),
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

        // egui::ScrollArea::vertical().show(ui, |ui| {
        //     for serie in &app.series_search_results {
        //         ui.horizontal(|ui| {
        //             // Display the serie name
        //             let display_name = utils::preprocess_arabic_text_v1(&serie.name);
        //             if ui.button(&display_name).clicked() {
        //                 if let Some(serie_id) = serie.series_id {
        //                     app.current_view = AppView::SeriesDetail(
        //                         serie.category_id.clone(),
        //                         serie_id,
        //                         serie.category_id.to_string(),
        //                     );
        //                 }
        //             }
        //         });
        //     }
        // });
    });
}
