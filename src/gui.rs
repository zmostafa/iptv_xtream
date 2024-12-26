use crate::api_client::{
    authenticate, fetch_all_live_streams, fetch_all_movies, fetch_all_series,
    fetch_live_categories, fetch_serie_info, fetch_series_categories, fetch_vod_categories,
};
use crate::database::Database;
use crate::models::live::{Category, LiveStream};
use crate::models::movies::Movie;
use crate::models::series::{Series, SeriesInfo};
use eframe::egui;
use egui::{vec2, TextBuffer};
use egui_extras;
use futures::stream;
use harfbuzz::sys::HB_GLYPH_FLAG_DEFINED;
use isahc::prelude::*;
use rustybuzz::{Face, SerializeFlags, UnicodeBuffer};
use serde_json::to_string;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use unicode_bidi::{BidiInfo, Direction, Level};

#[derive(Clone, Debug)]
enum AppView {
    Login,
    Categories,
    LiveCategories,
    LiveStreams(String), // Holds the category_id
    MoviesCategories,
    MoviesStream(String),
    SeriesCategories,
    SeriesList(String),       // Holds category_id
    SeriesDetail(i64),        // Holds series_id
    EpisodeList(i64, String), // Holds series_id and season number
    Playback(String),
    Search,
}

#[derive(Debug)]
pub struct IPTVApp {
    // client: reqwest::Client, // Shared client
    client: isahc::HttpClient,
    api_url: String,
    username: String,
    password: String,
    authenticated: bool,
    categories: Arc<Mutex<Vec<Category>>>,
    db: Database,
    current_view: AppView,
    view_stack: Vec<AppView>,
    mpv_process: Option<Child>,
    search_query: String,       // New field for search query
    search_results: Vec<Movie>, // New field for search results
}

impl IPTVApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let db = Database::new("iptv_cache");
        let api_url = db.get::<String>("api_url").unwrap_or_default();
        let username = db.get::<String>("username").unwrap_or_default();
        let password = db.get::<String>("password").unwrap_or_default();

        // Set fonts
        configure_fonts(ctx);

        IPTVApp {
            // client: reqwest::Client::builder()
            //     .timeout(Duration::from_secs(5))
            //     .build()
            //     .unwrap(),
            client: isahc::HttpClient::new().expect("Failed to create a Client"),
            api_url: "http://lionztv.com:8080".to_string(),
            username: "zyad67855".to_string(),
            password: "00985678".to_string(),
            authenticated: false,
            categories: Arc::new(Mutex::new(vec![])),
            db,
            current_view: AppView::Login,
            view_stack: vec![],
            mpv_process: None,
            search_query: String::new(), // Initialize search query
            search_results: vec![],      // Initialize search results
        }
    }

    pub async fn authenticate(&mut self) -> bool {
        match authenticate(&self.client, &self.api_url, &self.username, &self.password).await {
            Ok(auth_success) => {
                self.authenticated = auth_success;
                if auth_success {
                    self.db.save("api_url", &self.api_url);
                    self.db.save("username", &self.username);
                    self.db.save("password", &self.password);
                }
                auth_success
            }
            Err(err) => {
                log::error!("Failed to authenticate: {}", err);
                false
            }
        }
    }

    pub async fn fetch_and_save_all_live_streams(&mut self) {
        // Fetch all live streams
        match fetch_live_categories(&self.client, &self.api_url, &self.username, &self.password)
            .await
        {
            Ok(categories) => {
                self.db.save_live_categories(categories.clone());
                log::info!("Live categories fetched and saved.");
                log::info!(
                    "[CATIGORIES] Number of Live stream categories: {}",
                    categories.len()
                );
            }
            Err(err) => {
                log::error!("Failed to fetch live categories: {}", err);
            }
        }

        match fetch_vod_categories(&self.client, &self.api_url, &self.username, &self.password)
            .await
        {
            Ok(categories) => {
                self.db.save_movies_categories(categories.clone());
                log::info!("Movies categories fetched and saved.");
            }
            Err(err) => {
                log::error!("Failed to fetch Movies categories: {}", err);
            }
        }

        match fetch_all_live_streams(&self.client, &self.api_url, &self.username, &self.password)
            .await
        {
            Ok(live_streams) => {
                log::info!("Fetched {} live streams.", live_streams.len());
                for stream in live_streams {
                    if let Some(category_id) = &stream.category_id {
                        self.db.save_individual_live_stream(category_id, &stream);
                    } else {
                        log::warn!("Stream {} has no category ID.", stream.name);
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to fetch all live streams: {}", err);
            }
        }

        // Fetch all movies
        match fetch_all_movies(&self.client, &self.api_url, &self.username, &self.password).await {
            Ok(movies) => {
                log::info!("Fetched {} movies.", movies.len());
                for movie in movies {
                    self.db.save_individual_movie(&movie.category_id, &movie);
                }
            }
            Err(err) => {
                log::error!("Failed to fetch all movies: {}", err);
            }
        }

        (self.fetch_and_save_series_data().await);
        // log::info!("[INFO] Fetching is done");
    }

    pub async fn fetch_and_save_series_data(&self) {
        // Fetch categories
        match fetch_series_categories(&self.client, &self.api_url, &self.username, &self.password)
            .await
        {
            Ok(categories) => {
                self.db.save_series_categories(categories.clone());
                log::info!("Series categories fetched and saved.");

                // Process each category sequentially
                // for category in categories {
                //     log::info!("Fetching series for category: {}", category.category_name);

                //     // Fetch series for the category
                //     match fetch_series(
                //         &self.client,
                //         &self.api_url,
                //         &self.username,
                //         &self.password,
                //         Some(&category.category_id),
                //     )
                //     .await
                //     {
                //         Ok(series_list) => {
                //             self.db
                //                 .save_series_streams(&category.category_id, series_list.clone());
                //             log::info!("Saved Series for category: {}", category.category_name);

                //             // Process each series sequentially
                //             for series in series_list {
                //                 if let Some(series_id) = series.series_id {
                //                     log::info!("Fetching details for series ID: {}", series_id);

                //                     match fetch_serie_info(
                //                         &self.client,
                //                         &self.api_url,
                //                         &self.username,
                //                         &self.password,
                //                         &series_id,
                //                     )
                //                     .await
                //                     {
                //                         Ok(series_info) => {
                //                             self.db.save_series_info(&series_id, &series_info);
                //                             log::info!(
                //                                 "Saved series info for series ID: {}",
                //                                 series_id
                //                             );
                //                         }
                //                         Err(err) => {
                //                             log::error!(
                //                                 "Failed to fetch series info for series ID {}: {}",
                //                                 series_id,
                //                                 err
                //                             );
                //                         }
                //                     }
                //                 }
                //             }
                //         }
                //         Err(err) => {
                //             log::error!(
                //                 "Failed to fetch series for category {}: {}",
                //                 category.category_name,
                //                 err
                //             );
                //         }
                //     }
                // }
            }
            Err(err) => {
                log::error!("Failed to fetch series categories: {}", err);
            }
        }

        match fetch_all_series(&self.client, &self.api_url, &self.username, &self.password).await {
            Ok(series) => {
                for serie in series {
                    self.db
                        .save_individual_serieInfo(&serie.category_id, &serie);
                }
            }
            Err(err) => {
                log::error!("Failed to fetch all series: {}", err);
            }
        }

        // Process each series sequentially
        // let series_categories = self.db.get_series_categories();
        // for category in series_categories {
        
        let series_list = self.db.get_all_seriesInfo();
        // for series in series_list {
        //     if let Some(series_id) = series.series_id {
        //         log::info!("Fetching details for series ID: {}", series_id);

        //         match fetch_serie_info(
        //             &self.client,
        //             &self.api_url,
        //             &self.username,
        //             &self.password,
        //             &series_id,
        //         )
        //         .await
        //         {
        //             Ok(series_info) => {
        //                 self.db.save_series_info(&series_id, &series_info);
        //                 log::info!("Saved series info for series ID: {}", series_id);
        //             }
        //             Err(err) => {
        //                 log::error!(
        //                     "Failed to fetch series info for series ID {}: {}",
        //                     series_id,
        //                     err
        //                 );
        //             }
        //         }
        //     }
        //     // }
        // }
    }

    fn render_login(&mut self, ctx: &egui::Context) {
        log::info!("Rendering login view...");

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Login to IPTV");

            ui.horizontal(|ui| {
                ui.label("Server URL: ");
                log::info!("Before: {}", self.api_url); // Log state before input
                if ui.text_edit_singleline(&mut self.api_url).changed() {
                    log::info!("Updated Server URL: {}", self.api_url);
                }
            });
            log::info!("After: {}", self.api_url); // Log state after input

            ui.horizontal(|ui| {
                ui.label("Username: ");
                ui.text_edit_singleline(&mut self.username);
            });
            ui.horizontal(|ui| {
                ui.label("Password: ");
                ui.text_edit_singleline(&mut self.password);
            });

            if ui.button("Login").clicked() {
                if futures::executor::block_on(self.authenticate()) {
                    self.authenticated = true;
                    self.current_view = AppView::Categories;
                } else {
                    log::error!("Authentication failed.");
                }
            }
        });
    }

    fn render_categories(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Categories");

            if ui.button("Live Streams").clicked() {
                self.current_view = AppView::LiveCategories;
            }

            if ui.button("Movies Streams").clicked() {
                self.current_view = AppView::MoviesCategories;
            }

            if ui.button("Series Streams").clicked() {
                self.current_view = AppView::SeriesCategories;
            }

            if ui.button("Fetch Content").clicked() {
                // let mut app_clone = self.clone();
                // let rt = self.runtime.handle().clone();
                // rt.spawn(async move {
                futures::executor::block_on(self.fetch_and_save_all_live_streams());
                // self.fetch_and_save_all_live_streams().await;
                // });
            }
        });
    }

    fn render_live_categories(&mut self, ctx: &egui::Context) {
        let categories = self.db.get_live_categories();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live Categories");

            if ui.button("Back").clicked() {
                self.current_view = AppView::Categories;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for category in categories {
                    let display_name = Self::preprocess_arabic_text_v1(&category.category_name);
                    if ui.button(&display_name).clicked() {
                        self.current_view = AppView::LiveStreams(category.category_id.clone());
                    }
                }
            });
        });
    }

    fn render_movies_categories(&mut self, ctx: &egui::Context) {
        let categories = self.db.get_movies_categories();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Movies Categories");

            if ui.button("Search Movies").clicked() {
                self.current_view = AppView::Search;
            }

            if ui.button("Back").clicked() {
                self.current_view = AppView::Categories;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for category in categories {
                    let display_name = Self::preprocess_arabic_text_v1(&category.category_name);
                    if ui.button(&display_name).clicked() {
                        self.current_view = AppView::MoviesStream(category.category_id.clone());
                    }
                }
            });
        });
    }

    fn render_series_categories(&mut self, ctx: &egui::Context) {
        let categories = self.db.get_series_categories();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Series Categories");
            if ui.button("Back").clicked() {
                self.current_view = AppView::Categories;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for category in categories {
                    let display_name = Self::preprocess_arabic_text_v1(&category.category_name);
                    if ui.button(&display_name).clicked() {
                        self.current_view = AppView::SeriesList(category.category_id.clone());
                    }
                }
            });
        });
    }

    fn render_live_streams(&mut self, ctx: &egui::Context, category_id: &str) {
        let streams = self.db.get_live_streams(category_id);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Live Streams");

            if ui.button("Back").clicked() {
                self.current_view = AppView::LiveCategories;
            }

            ui.separator();

            // Calculate grid properties based on screen width
            let available_width = ui.available_width();
            let min_item_width = 150.0; // Minimum width for each grid item
            let num_columns = (available_width / min_item_width).floor() as usize; // Fit as many columns as possible
            let item_size = available_width / num_columns as f32; // Dynamically size each item

            // Add a scrollable grid to display the movies
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("live_grid")
                    .spacing([20.0, 20.0]) // Spacing between items
                    .min_col_width(item_size) // Minimum column width
                    .show(ui, |ui| {
                        for (i, stream) in streams.iter().enumerate() {
                            // Display the live icon and name
                            ui.vertical(|ui| {
                                if !stream.stream_icon.is_empty() {
                                    // Attempt to display the image
                                    ui.add(
                                        egui::Image::from_uri(&stream.stream_icon)
                                            .rounding(10.0)
                                            .fit_to_exact_size(vec2(150.0, 150.0)),
                                    );
                                } else {
                                    // Placeholder for missing image
                                    ui.label("[No Image]");
                                }

                                // Display the live name
                                let display_name = Self::preprocess_arabic_text_v1(&stream.name);
                                ui.label(&display_name);

                                // Add a play button
                                if ui.button("Play").clicked() {
                                    println!("Play stream: {}", stream.stream_id);
                                    let stream_url = format!(
                                        "{}/{}/{}/{}/{}.{}",
                                        self.api_url,
                                        "live",
                                        self.username,
                                        self.password,
                                        stream.stream_id,
                                        "ts".to_string()
                                    );

                                    self.view_stack.push(self.current_view.clone());
                                    self.current_view = AppView::LiveStreams(stream_url);
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

    fn render_movies_streams(&mut self, ctx: &egui::Context, category_id: &str) {
        let streams = self.db.get_movies_streams(category_id);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Movies Streams");

            if ui.button("Back").clicked() {
                self.current_view = AppView::MoviesCategories;
            }

            ui.separator();

            // Calculate grid properties based on screen width
            let available_width = ui.available_width();
            let min_item_width = 150.0; // Minimum width for each grid item
            let num_columns = (available_width / min_item_width).floor() as usize; // Fit as many columns as possible
            let item_size = available_width / num_columns as f32; // Dynamically size each item

            // Add a scrollable grid to display the movies
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("movies_grid")
                    .spacing([20.0, 20.0]) // Spacing between items
                    .min_col_width(item_size) // Minimum column width
                    .show(ui, |ui| {
                        for (i, stream) in streams.iter().enumerate() {
                            // Display the movie poster and name
                            ui.vertical(|ui| {
                                if !stream.stream_icon.is_empty() {
                                    // Attempt to display the image
                                    ui.add(
                                        egui::Image::from_uri(&stream.stream_icon)
                                            .rounding(10.0)
                                            .fit_to_exact_size(vec2(150.0, 150.0)),
                                    );
                                } else {
                                    // Placeholder for missing image
                                    ui.label("[No Image]");
                                }

                                // Display the movie name
                                let display_name = Self::preprocess_arabic_text_v1(&stream.name);
                                ui.label(&display_name);

                                // Add a play button
                                if ui.button("Play").clicked() {
                                    println!("Play stream: {}", stream.stream_id);
                                    let stream_url = format!(
                                        "{}/{}/{}/{}/{}.{}",
                                        self.api_url,
                                        "movie",
                                        self.username,
                                        self.password,
                                        stream.stream_id,
                                        stream.container_extension
                                    );

                                    self.view_stack.push(self.current_view.clone());
                                    self.current_view = AppView::Playback(stream_url);
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

    fn render_series_list(&mut self, ctx: &egui::Context, category_id: &str) {
        let series_list = self.db.get_individual_seriesInfo(category_id);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Series List");

            if ui.button("Back").clicked() {
                self.current_view = AppView::SeriesCategories;
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
                                if !serie.cover.is_empty() {
                                    // Attempt to display the image
                                    ui.add(
                                        egui::Image::from_uri(&serie.cover)
                                            .rounding(10.0)
                                            .fit_to_exact_size(vec2(150.0, 150.0)),
                                    );
                                } else {
                                    // Placeholder for missing image
                                    ui.label("[No Image]");
                                }

                                // Display the serie name
                                let display_name = Self::preprocess_arabic_text_v1(&serie.name);
                                ui.label(&display_name);
                                if ui.button(&display_name).clicked() {
                                    if let Some(serie_id) = serie.series_id {
                                        self.current_view = AppView::SeriesDetail(serie_id);
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

    fn render_series_detail(&mut self, ctx: &egui::Context, series_id: &i64) {
        // TODO: fetch series info when a series is selected and it's no in DB
        let series_detail = self
            .db
            .get_series_info(series_id)
            .unwrap_or_else(|| Series {
                seasons: vec![],
                info: SeriesInfo {
                    num: Some(0),
                    name: "".to_string(),
                    series_id: Some(0),
                    cover: "".to_string(),
                    plot: "".to_string(),
                    cast: "".to_string(),
                    director: "".to_string(),
                    genre: "".to_string(),
                    releaseDate: "".to_string(),
                    last_modified: "".to_string(),
                    // rating: "".to_string(),
                    // rating_5based: 0.0,
                    // backdrop_path: None,
                    youtube_trailer: "".to_string(),
                    episode_run_time: "".to_string(),
                    category_id: "".to_string(),
                },
                episodes: HashMap::new(),
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!(
                "Series: {}",
                Self::preprocess_arabic_text_v1(series_detail.info.name.as_str())
            ));
            ui.label(format!(
                "Plot: {}",
                Self::preprocess_arabic_text_v1(series_detail.info.plot.as_str())
            ));

            if ui.button("Back").clicked() {
                self.current_view = AppView::SeriesList(series_detail.info.category_id.clone());
            }
            // ui.label(format!("Rating: {}", series_detail.info.rating));
            // Convert episodes keys to a sorted vector
            // HashMap does not maintain order,so looping through them will change how they are displayed
            let mut sorted_seasons: Vec<_> = series_detail.episodes.keys().cloned().collect();
            sorted_seasons.sort();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for season in sorted_seasons {
                    if ui.button(format!("Season {}", season)).clicked() {
                        self.current_view = AppView::EpisodeList(series_id.to_owned(), season);
                    }
                }
            });
        });
    }

    fn render_episode_list(&mut self, ctx: &egui::Context, series_id: &i64, season: String) {
        let series_detail = self
            .db
            .get_series_info(series_id)
            .unwrap_or_else(|| Series {
                seasons: vec![],
                info: SeriesInfo {
                    num: Some(0),
                    name: "".to_string(),
                    series_id: Some(0),
                    cover: "".to_string(),
                    plot: "".to_string(),
                    cast: "".to_string(),
                    director: "".to_string(),
                    genre: "".to_string(),
                    releaseDate: "".to_string(),
                    last_modified: "".to_string(),
                    // rating: "".to_string(),
                    // rating_5based: 0.0,
                    // backdrop_path: None,
                    youtube_trailer: "".to_string(),
                    episode_run_time: "".to_string(),
                    category_id: "".to_string(),
                },
                episodes: HashMap::new(),
            });
        let binding = vec![];
        let episodes = series_detail
            .episodes
            .get(&season.to_string())
            .unwrap_or(&binding);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Season {} Episodes", season));

            if ui.button("Back").clicked() {
                self.current_view = AppView::SeriesDetail(series_id.to_owned());
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for episode in episodes {
                    ui.horizontal(|ui| {
                        ui.label(&episode.title);
                        if ui.button("Play").clicked() {
                            let stream_url = format!(
                                "{}/{}/{}/{}/{}.{}",
                                self.api_url,
                                "series",
                                self.username,
                                self.password,
                                episode.id,
                                episode.container_extension
                            );
                            self.view_stack.push(self.current_view.clone());
                            self.current_view = AppView::Playback(stream_url);
                        }
                    });
                }
            });
        });
    }

    fn render_search(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Search Movies");

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
            });

            if ui.button("Search").clicked() {
                self.search_results = self.search_movies(&self.search_query);
            }

            if ui.button("Back").clicked() {
                self.current_view = AppView::MoviesCategories;
            }

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for movie in &self.search_results {
                    ui.horizontal(|ui| {
                        ui.label(&movie.name);
                        if ui.button("Play").clicked() {
                            let stream_url = format!(
                                "{}/{}/{}/{}/{}.{}",
                                self.api_url,
                                "movie",
                                self.username,
                                self.password,
                                movie.stream_id,
                                movie.container_extension
                            );
                            self.view_stack.push(self.current_view.clone());
                            self.current_view = AppView::Playback(stream_url);
                        }
                    });
                }
            });
        });
    }

    fn search_movies(&self, query: &str) -> Vec<Movie> {
        let all_movies = self
            .db
            .get_movies_categories()
            .iter()
            .flat_map(|category| self.db.get_movies_streams(&category.category_id))
            .collect::<Vec<_>>();

        all_movies
            .into_iter()
            .filter(|movie| movie.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
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
            if let Some(view) = self.view_stack.pop() {
                self.current_view = view;
            }
        }
    }

    fn preprocess_arabic_text_v1(input: &str) -> String {
        // 1. Perform bidirectional text processing
        let bidi_info = BidiInfo::new(input, None);
        let para = &bidi_info.paragraphs[0];
        let line = para.range.clone();
        let reordered = bidi_info.reorder_line(para, line.clone());
        use arabic_reshaper::arabic_reshape;

        arabic_reshape(reordered.as_str())
    }

    fn preprocess_arabic_text_v2(input: &str) -> String {
        // 1. Split the text using '-' as a delimiter
        let segments: Vec<&str> = input.split('-').collect();

        // 2. Load the font for shaping Arabic text
        let font_data = include_bytes!("/home/zmostafa/github/xtream/assets/Amiri-Regular.ttf");
        let face = Face::from_slice(font_data, 0).expect("Failed to create Rustybuzz Face");

        // 3. Process each segment
        let mut processed_segments = vec![];
        for segment in segments {
            // Check if the segment contains Arabic characters
            if segment.chars().any(|c| c >= '\u{0600}' && c <= '\u{06FF}') {
                // Process Arabic text: Shape and reorder
                let bidi_info = BidiInfo::new(segment, None);
                let para = &bidi_info.paragraphs[0];
                let reordered = bidi_info.reorder_line(para, para.range.clone());

                let mut unicode_buffer = UnicodeBuffer::new();
                unicode_buffer.push_str(&reordered);
                unicode_buffer.set_direction(rustybuzz::Direction::RightToLeft);

                let glyph_buffer = rustybuzz::shape(&face, &[], unicode_buffer);

                let mut shaped_text = String::new();
                for glyph in glyph_buffer.glyph_infos() {
                    if let Some(ch) = char::from_u32(glyph.cluster) {
                        shaped_text.push(ch);
                    }
                }
                // processed_segments.push(
                //     glyph_buffer
                //         .serialize(&face, SerializeFlags::default())
                //         .to_string(),
                // );
                processed_segments.push(shaped_text);
            } else {
                // Append English or non-Arabic segments as-is
                processed_segments.push(segment.to_string());
            }
        }

        // 4. Rejoin the segments with the separator
        processed_segments.join(" - ")
    }
}

impl eframe::App for IPTVApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        log::info!("IPTVApp Update for {:?}", self.current_view);
        // configure_fonts(ctx);
        let current_view = self.current_view.clone();

        match current_view {
            AppView::Login => self.render_login(ctx),
            AppView::Categories => self.render_categories(ctx),
            AppView::LiveCategories => self.render_live_categories(ctx),
            AppView::MoviesCategories => self.render_movies_categories(ctx),
            AppView::SeriesCategories => self.render_series_categories(ctx),
            AppView::LiveStreams(category_id) => self.render_live_streams(ctx, &category_id),
            AppView::MoviesStream(category_id) => self.render_movies_streams(ctx, &category_id),
            AppView::SeriesList(category_id) => self.render_series_list(ctx, &category_id),
            AppView::SeriesDetail(series_id) => self.render_series_detail(ctx, &series_id),
            AppView::EpisodeList(series_id, season) => {
                self.render_episode_list(ctx, &series_id, season)
            }
            AppView::Search => self.render_search(ctx), // Render the search view
            AppView::Playback(stream_url) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.display_playback(ui, stream_url.clone());
                });
            }
        }
    }
}

fn configure_fonts(ctx: &egui::Context) {
    log::info!("Configuring fonts.");
    use egui::FontFamily::{Monospace, Proportional};

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "custom_arabic".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "/home/zmostafa/github/xtream/assets/Amiri-Regular.ttf"
        ))
        .into(),
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
