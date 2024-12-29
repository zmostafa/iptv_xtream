use crate::api_client::{
    authenticate, fetch_all_live_streams, fetch_all_movies, fetch_all_series,
    fetch_live_categories, fetch_serie_info, fetch_series_categories, fetch_vod_categories,
};
use crate::cache::ImageCache;
use crate::database::Database;
use crate::models::movies::Movie;
use eframe::egui;
use egui::{cache, vec2, TextBuffer};
use isahc::config::Configurable;
use isahc::{AsyncReadResponseExt, HttpClient, ReadResponseExt};
use rustybuzz::{Face, UnicodeBuffer};
use std::error::Error;
use std::io::BufRead;
use std::io::{ErrorKind, Write};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Semaphore;
use unicode_bidi::BidiInfo;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
enum AppView {
    Login,
    Categories,
    LiveCategories,
    LiveStreams(String, String), // Holds the category_id, category_name
    MoviesCategories,
    MoviesStream(String, String),
    SeriesCategories,
    SeriesList(String, String), // Holds category_id ,and category_name
    SeriesDetail(String, i64, String), // Holds category_id, series_id, category_name
    EpisodeList(String, i64, String, String), // Holds category_id, series_id, season number and category_name
    Playback(String),
    Search,
}

#[derive(Debug)]
pub struct IPTVApp {
    image_cache: Arc<Mutex<HashMap<String, Vec<u8>>>>, // Cached images
    ongoing_requests: HashSet<String>,
    client: isahc::HttpClient,
    api_url: String,
    username: String,
    password: String,
    authenticated: bool,
    db: Database,
    cache: ImageCache,
    current_view: AppView,
    view_stack: Vec<AppView>,
    mpv_process: Option<Child>,
    search_query: String,       // New field for search query
    search_results: Vec<Movie>, // New field for search results
}

impl IPTVApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let db = Database::new("iptv_cache");
        let cache = ImageCache::new("iptv_cache/image_cache");
        let api_url = db.get::<String>("api_url").unwrap_or_default();
        let username = db.get::<String>("username").unwrap_or_default();
        let password = db.get::<String>("password").unwrap_or_default();

        // Set fonts
        configure_fonts(ctx);

        IPTVApp {
            image_cache: Arc::new(Mutex::new(HashMap::new())),
            ongoing_requests: HashSet::new(),
            // client: isahc::HttpClient::new().expect("Failed to create a Client"),
            client: isahc::HttpClient::builder()
                .redirect_policy(isahc::config::RedirectPolicy::Follow)
                .tcp_keepalive(Duration::from_secs(1))
                .max_connections(1000) // Adjust based on expected concurrency
                .build()
                .unwrap(),
            api_url,
            username,
            password,
            authenticated: false,
            db,
            cache,
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

        log::info!("[INFO] Fetching content is done");
    }

    pub async fn fetch_and_save_series_data(&mut self) {
        // Fetch categories
        match fetch_series_categories(&self.client, &self.api_url, &self.username, &self.password)
            .await
        {
            Ok(categories) => {
                self.db.save_series_categories(categories.clone());
                log::info!("Series categories fetched and saved.");
            }
            Err(err) => {
                log::error!("Failed to fetch series categories: {}", err);
            }
        }

        match fetch_all_series(&self.client, &self.api_url, &self.username, &self.password).await {
            Ok(series_info) => {
                for serie_info in series_info {
                    self.db
                        .save_serieInfo_for_all_series(&serie_info.category_id, &serie_info);
                }
            }
            Err(err) => {
                log::error!("Failed to fetch all series: {}", err);
            }
        }
    }

    fn render_login(&mut self, ctx: &egui::Context) {
        log::info!("Rendering login view...");

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Login to IPTV");

            egui::Grid::new("login_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Server URL: ");
                    ui.text_edit_singleline(&mut self.api_url);
                    ui.end_row();

                    ui.label("Username: ");
                    ui.text_edit_singleline(&mut self.username);
                    ui.end_row();

                    ui.label("Password: ");
                    ui.text_edit_singleline(&mut self.password);
                    ui.end_row();
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
                futures::executor::block_on(self.fetch_and_save_all_live_streams());
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
                        self.current_view = AppView::LiveStreams(
                            category.category_id.clone(),
                            display_name.clone(),
                        );
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
                        self.current_view = AppView::MoviesStream(
                            category.category_id.clone(),
                            display_name.clone(),
                        );
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
                        self.current_view =
                            AppView::SeriesList(category.category_id.clone(), display_name.clone());
                    }
                }
            });
        });
    }

    fn render_live_streams(&mut self, ctx: &egui::Context, category_id: &str, category_name: &str) {
        let streams = self.db.get_live_streams(category_id);
        let cache = self.cache.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(category_name);

            if ui.button("Back").clicked() {
                self.current_view = AppView::LiveCategories;
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

                                if self.cache.is_cached(&image_url) {
                                    // Load image from cache
                                    if let Ok(image_data) = self.cache.load_image(&image_url) {
                                        ui.add(
                                            egui::Image::from_bytes(
                                                stream.name.clone(),
                                                image_data,
                                            )
                                            .rounding(10.0)
                                            .fit_to_exact_size(vec2(150.0, 150.0)),
                                        );
                                    } else {
                                        ui.label("[Error Loading Image]");
                                    }
                                } else {
                                    // Placeholder for loading
                                    ui.label("[Loading...]");

                                    if !self.ongoing_requests.contains(&image_url) {
                                        self.ongoing_requests.insert(image_url.clone());
                                        // Fetch image in the background
                                        let image_url_clone = image_url.clone();
                                        let cache_clone = cache.clone();
                                        let client_clone = self.client.clone();
                                        let ctx_clone = ctx.clone();

                                        tokio::spawn(async move {
                                            if let Err(err) = Self::fetch_and_cache_image(
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

                                // Display the live name
                                let display_name = Self::preprocess_arabic_text_v1(&stream.name);
                                ui.label(&display_name);

                                // Play button
                                if ui.button("Play").clicked() {
                                    let stream_url = format!(
                                        "{}/{}/{}/{}/{}.{}",
                                        self.api_url,
                                        "live",
                                        self.username,
                                        self.password,
                                        stream.stream_id,
                                        "ts"
                                    );
                                    self.view_stack.push(self.current_view.clone());
                                    self.current_view =
                                        AppView::LiveStreams(stream_url, category_name.to_string());
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

    fn render_movies_streams(
        &mut self,
        ctx: &egui::Context,
        category_id: &str,
        category_name: &str,
    ) {
        let mut streams = self.db.get_movies_streams(category_id);
        streams.sort_by(|a, b| b.added.cmp(&a.added));
        log::debug!("Sroted Movies by adding date");
        let cache = self.cache.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(category_name);

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
                        log::info!("Displaying movies grid");
                        for (i, stream) in streams.iter().enumerate() {
                            // Display the movie poster and name
                            ui.vertical(|ui| {
                                let image_url = stream.stream_icon.clone();

                                if self.cache.is_cached(&image_url) {
                                    // Load image from cache
                                    if let Ok(image_data) = self.cache.load_image(&image_url) {
                                        ui.add(
                                            egui::Image::from_bytes(
                                                stream.name.clone(),
                                                image_data,
                                            )
                                            .rounding(10.0)
                                            .fit_to_exact_size(vec2(150.0, 150.0)),
                                        );
                                    } else {
                                        ui.label("[Error Loading Image]");
                                    }
                                } else {
                                    // Placeholder for loading
                                    ui.label("[Loading...]");

                                    if !self.ongoing_requests.contains(&image_url) {
                                        // Fetch image in the background
                                        log::info!("Fetching image in the background");
                                        self.ongoing_requests.insert(image_url.clone());
                                        let image_url_clone = image_url.clone();
                                        let cache_clone = cache.clone();
                                        let client_clone = self.client.clone();
                                        let ctx_clone = ctx.clone();

                                        tokio::spawn(async move {
                                            if let Err(err) = Self::fetch_and_cache_image(
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

                                // Display the movie name
                                let display_name = Self::preprocess_arabic_text_v1(&stream.name);
                                ui.label(&display_name);
                                let stream_url = format!(
                                    "{}/{}/{}/{}/{}.{}",
                                    self.api_url,
                                    "movie",
                                    self.username,
                                    self.password,
                                    stream.stream_id,
                                    stream.container_extension
                                );

                                // Add a play button
                                if ui.button("Play").clicked() {
                                    self.view_stack.push(self.current_view.clone());
                                    self.play_media(&stream.stream_id, &stream_url);
                                }
                                let progress = Arc::new(Mutex::new(0.0));

                                if ui.button("Download").clicked() {
                                    log::debug!("Downloading Movie: {}", stream.name);
                                    let save_path = format!(
                                        "iptv_cache/{}.{}",
                                        stream.stream_id, stream.container_extension
                                    );
                                    let db_clone = self.db.clone();
                                    let url_clone = stream_url.clone();
                                    let progress_clone_for_download = Arc::clone(&progress);
                                    let stream_clone = stream.clone();

                                    tokio::spawn({
                                        async move {
                                            if let Err(e) = Self::download_with_wget_async(
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
                                                db_clone.save_download(
                                                    stream_clone.stream_id.clone(),
                                                    &save_path,
                                                );
                                            }
                                        }
                                    });
                                }
                                // Update progress bar in UI
                                let progress_clone_for_ui = Arc::clone(&progress);
                                ui.add(
                                    egui::ProgressBar::new(*progress_clone_for_ui.lock().unwrap())
                                        .text("Downloading..."),
                                );
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

    fn render_series_list(&mut self, ctx: &egui::Context, category_id: &str, category_name: &str) {
        let mut series_list = self.db.get_serieInfo_for_all_series(category_id);
        series_list.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        let cache = self.cache.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(category_name);

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
                                log::info!("Getting serie info : {}", &serie.name);
                                let image_url = serie.cover.clone();

                                if self.cache.is_cached(&image_url) {
                                    // Load image from cache
                                    if let Ok(image_data) = self.cache.load_image(&image_url) {
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
                                    log::info!(
                                        "Image not found in cache {}, downloading",
                                        image_url
                                    );
                                    ui.label("[Loading...]");

                                    if !self.ongoing_requests.contains(&image_url) {
                                        self.ongoing_requests.insert(image_url.clone());
                                        // Fetch image in the background
                                        let image_url_clone = image_url.clone();
                                        let cache_clone = cache.clone();
                                        let client_clone = self.client.clone();
                                        let ctx_clone = ctx.clone();

                                        tokio::spawn(async move {
                                            if let Err(err) = Self::fetch_and_cache_image(
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
                                let display_name = Self::preprocess_arabic_text_v1(&serie.name);
                                ui.label(&display_name);
                                if ui.button(&display_name).clicked() {
                                    if let Some(serie_id) = serie.series_id {
                                        self.current_view = AppView::SeriesDetail(
                                            serie.category_id.clone(),
                                            serie_id,
                                            category_name.to_string(),
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

    fn render_series_detail(
        &mut self,
        ctx: &egui::Context,
        category_id: &str,
        series_id: &i64,
        category_name: &str,
    ) {
        let series_detail = self
            .db
            .get_series_info(category_id, series_id)
            .unwrap_or_else(|| {
                let serie = futures::executor::block_on(fetch_serie_info(
                    &self.client,
                    &self.api_url,
                    &self.username,
                    &self.password,
                    series_id,
                ))
                .unwrap();
                self.db.save_series_info(&category_id, &series_id, &serie);
                serie
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
                self.current_view = AppView::SeriesList(
                    series_detail.info.category_id.clone(),
                    category_name.to_string(),
                );
            }

            // Convert episodes keys to a sorted vector
            // HashMap does not maintain order,so looping through them will change how they are displayed
            let mut sorted_seasons: Vec<_> = series_detail.episodes.keys().cloned().collect();
            sorted_seasons.sort();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for season in sorted_seasons {
                    if ui.button(format!("Season {}", season)).clicked() {
                        self.current_view = AppView::EpisodeList(
                            category_id.to_string(),
                            series_id.to_owned(),
                            season,
                            category_name.to_string(),
                        );
                    }
                }
            });
        });
    }

    fn render_episode_list(
        &mut self,
        ctx: &egui::Context,
        category_id: &str,
        series_id: &i64,
        season: String,
        category_name: &str,
    ) {
        let series_detail = self
            .db
            .get_series_info(&category_id, series_id)
            .unwrap_or_else(|| {
                let serie = futures::executor::block_on(fetch_serie_info(
                    &self.client,
                    &self.api_url,
                    &self.username,
                    &self.password,
                    series_id,
                ))
                .unwrap();
                self.db.save_series_info(&category_id, &series_id, &serie);
                serie
            });
        let binding = vec![];
        let episodes = series_detail
            .episodes
            .get(&season.to_string())
            .unwrap_or(&binding);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Season {} Episodes", season));

            if ui.button("Back").clicked() {
                self.current_view = AppView::SeriesDetail(
                    category_id.to_string(),
                    series_id.to_owned(),
                    category_name.to_string(),
                );
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

    fn play_media(&mut self, stream_id: &u32, stream_url: &str) {
        if let Some(local_path) = self.db.get_download_path(stream_id) {
            if std::path::Path::new(&local_path).exists() {
                self.current_view = AppView::Playback(local_path);
                return;
            }
        }
        self.current_view = AppView::Playback(stream_url.to_string());
    }

    pub async fn download_with_wget_async(
        url: &str,
        save_path: &str,
        progress: Arc<Mutex<f32>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Spawn the wget process
        let mut child = Command::new("wget")
            .arg("-O")
            .arg(save_path)
            .arg(url)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Capture stderr for progress updates (wget outputs progress on stderr)
        if let Some(stderr) = child.stderr.take() {
            let reader = std::io::BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Some(line) = lines.next() {
                // Parse progress from wget's stderr output
                if let Some(progress_value) = Self::parse_wget_progress(&line.unwrap()) {
                    let mut progress_guard = progress.lock().unwrap();
                    *progress_guard = progress_value;
                }

                // log::info!("wget: {:?}", line);
            }
        }

        // Wait for the process to finish
        let status = child.wait()?;
        if status.success() {
            log::info!("Download completed successfully: {}", save_path);
            Ok(())
        } else {
            Err(format!("wget failed with status: {}", status).into())
        }
    }

    // Helper function to parse wget progress from a line of output
    fn parse_wget_progress(line: &str) -> Option<f32> {
        // Example wget progress line:
        // 23% [======>                           ] 1,229,312   12.0KB/s    ETA 10s
        if let Some(percentage) = line.split_whitespace().next() {
            if percentage.ends_with('%') {
                if let Ok(value) = percentage.trim_end_matches('%').parse::<f32>() {
                    return Some(value / 100.0); // Convert to 0.0-1.0 range
                }
            }
        }
        None
    }

    async fn fetch_and_cache_image(
        client: HttpClient,
        cache: ImageCache,
        image_url: &str,
    ) -> Result<(), String> {
        if cache.is_cached(image_url) {
            log::info!("Image already cached: {}", image_url);
            return Ok(());
        }

        log::info!("Downloading image: {}", image_url);

        match client.get_async(image_url).await {
            Ok(mut response) => {
                if response.status().is_success() {
                    let image_data = response
                        .bytes()
                        .await
                        .map_err(|e| format!("Failed to read image data: {}", e))?;
                    cache
                        .save_image(image_url, &image_data)
                        .map_err(|e| format!("Failed to save image to cache: {}", e))?;
                    Ok(())
                } else {
                    Err(format!("Unexpected status code: {}", response.status()))
                }
            }
            Err(err) => Err(format!("Failed to fetch image: {}", err)),
        }
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
            AppView::LiveStreams(category_id, category_name) => {
                self.render_live_streams(ctx, &category_id, &category_name)
            }
            AppView::MoviesStream(category_id, category_name) => {
                self.render_movies_streams(ctx, &category_id, &category_name)
            }
            AppView::SeriesList(category_id, category_name) => {
                self.render_series_list(ctx, &category_id, &category_name)
            }
            AppView::SeriesDetail(category_id, series_id, category_name) => {
                self.render_series_detail(ctx, &category_id, &series_id, &category_name)
            }
            AppView::EpisodeList(category_id, series_id, season, category_name) => {
                self.render_episode_list(ctx, &category_id, &series_id, season, &category_name)
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
        egui::FontData::from_static(include_bytes!("../assets/Amiri-Regular.ttf")).into(),
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
