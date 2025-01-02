use crate::api::fetch_serie_info;
use crate::app::state::{AppView, IPTVApp};
use crate::utils;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn render_episodes_list(
    app: &mut IPTVApp,
    ctx: &egui::Context,
    category_id: &str,
    series_id: &i64,
    season: String,
    category_name: &str,
) {
    let series_detail = app
        .db
        .get_series_info(&category_id, series_id)
        .unwrap_or_else(|| {
            let serie = futures::executor::block_on(fetch_serie_info(
                &app.client,
                &app.api_url,
                &app.username,
                &app.password,
                series_id,
            ))
            .unwrap();
            app.db.save_series_info(&category_id, &series_id, &serie);
            serie
        });
    // let binding = vec![];
    let episodes = series_detail.episodes.get(&season.to_string()).unwrap();
    // .unwrap_or(&binding);

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!("Season {} Episodes", season));

        if ui.button("Back").clicked() {
            app.current_view = AppView::SeriesDetail(
                category_id.to_string(),
                series_id.to_owned(),
                category_name.to_string(),
            );
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, episode) in episodes.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(&episode.title);

                    let stream_url = format!(
                        "{}/{}/{}/{}/{}.{}",
                        app.api_url,
                        "series",
                        app.username,
                        app.password,
                        episode.id,
                        episode.container_extension
                    );

                    if ui.button("▶").clicked() {
                        app.view_stack.push(app.current_view.clone());
                        app.db.save_watched(&episode.id.parse::<u32>().unwrap());
                        // Create a playlist starting from the clicked episode
                        let playlist: Vec<String> = episodes[index..]
                            .iter()
                            .map(|ep| {
                                format!(
                                    "{}/{}/{}/{}/{}.{}",
                                    app.api_url,
                                    "series",
                                    app.username,
                                    app.password,
                                    ep.id,
                                    ep.container_extension
                                )
                            })
                            .collect();

                        // Play the playlist
                        app.current_view = AppView::PlaylsitPlayback(playlist);
                        // utils::play_playlist(app, &playlist);
                        // utils::play_media(app, &episode.id.parse::<u32>().unwrap(), &stream_url);
                    }

                    if let Some(download_path) =
                        app.db.is_downloaded(&episode.id.parse::<u32>().unwrap())
                    {
                        log::debug!("Episode is available offline");
                        // ui.colored_label(egui::Color32::GREEN, "✅");

                        // Add a remove button
                        if ui.button("🗑").clicked() {
                            // Remove the downloaded episode
                            if let Err(err) = utils::remove_downloaded_episode(
                                app,
                                &episode.id.parse::<u32>().unwrap(),
                                &download_path,
                            ) {
                                log::error!("Failed to remove downloaded episode: {}", err);
                            } else {
                                log::info!("Downloaded episode removed: {}", episode.title);
                            }
                        }
                    } else {
                        let progress = Arc::new(Mutex::new(0.0));

                        if ui.button("⬇").clicked() {
                            log::debug!("Downloading Episode: {}", episode.title);
                            let save_path = format!(
                                "iptv_cache/{}.{}",
                                episode.title, episode.container_extension
                            );
                            let db_clone = app.db.clone();
                            let url_clone = stream_url.clone();
                            let progress_clone_for_download = Arc::clone(&progress);
                            let episode_clone = episode.clone();

                            // Add progress to active downloads
                            app.active_downloads.insert(
                                episode.id.clone().parse::<u32>().unwrap(),
                                progress.clone(),
                            );
                            let mut active_downloads_clone = app.active_downloads.clone();

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
                                        active_downloads_clone.remove(
                                            &episode_clone.id.clone().parse::<u32>().unwrap(),
                                        );
                                        db_clone.save_download(
                                            episode_clone.id.clone().parse::<u32>().unwrap(),
                                            &save_path,
                                        );
                                    }
                                }
                            });
                        }
                        // Update progress bar in UI
                        if let Some(progress) = app
                            .active_downloads
                            .get(&episode.id.parse::<u32>().unwrap())
                        {
                            let progress_value = *progress.lock().unwrap();
                            ui.add(egui::ProgressBar::new(progress_value).text("Downloading..."));
                        }
                    }
                    if app
                        .db
                        .is_watched(&episode.id.parse::<u32>().unwrap())
                        .is_some()
                    {
                        log::debug!("Episode is watched");
                        ui.colored_label(egui::Color32::GREEN, "✅");
                        // ui.label("👀");
                    }
                });
            }
        });
    });
}
