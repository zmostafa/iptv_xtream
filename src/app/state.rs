use crate::api::{
    authenticate, fetch_all_live_streams, fetch_all_movies, fetch_all_series,
    fetch_live_categories, fetch_series_categories, fetch_vod_categories,
};
use crate::app::views::{
    render_categories, render_episodes_list, render_live_categories, render_live_streams,
    render_login, render_movies_categories, render_movies_search, render_movies_streams,
    render_playback, render_playlist, render_recently_watched_movies,
    render_recently_watched_series, render_series_categories, render_series_details,
    render_series_list, render_series_search,
};
use crate::db::{Database, ImageCache};
use crate::models::{Movie, SeriesInfo};
use crate::utils;
use isahc::config::Configurable;
use isahc::HttpClient;
use std::collections::{HashMap, HashSet};
use std::env::current_dir;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use egui_extras::RetainedImage;

#[derive(Clone, Debug)]
pub enum AppView {
    Login,
    Categories,
    LiveCategories,
    LiveStreams(String, String),
    MoviesCategories,
    MoviesStream(String, String),
    SeriesCategories,
    SeriesList(String, String),
    SeriesDetail(String, i64, String),
    EpisodeList(String, i64, String, String),
    Playback(String),
    PlaylsitPlayback(Vec<String>),
    MoviesSearch,
    SeriesSearch,
    RecentlyWatchedMovies,
    RecentlyWatchedSeries,
}

pub struct IPTVApp {
    pub active_downloads: HashMap<u32, Arc<Mutex<f32>>>,
    pub ongoing_requests: HashSet<String>,
    pub client: HttpClient,
    pub api_url: String,
    pub username: String,
    pub password: String,
    pub authenticated: bool,
    pub db: Database,
    pub image_cache: ImageCache,
    pub current_view: AppView,
    pub view_stack: Vec<AppView>,
    pub mpv_process: Option<Child>,
    pub search_query: String,
    pub movies_search_results: Vec<Movie>,
    pub series_search_results: Vec<SeriesInfo>,
    pub movies_cache: Vec<Movie>,
    pub series_cache: Vec<SeriesInfo>,
    pub background: RetainedImage,
}

impl IPTVApp {
    pub fn new(_ctx: &egui::Context) -> Self {
        let db = Database::new("iptv_cache");
        let image_cache = ImageCache::new("iptv_cache/image_cache");
        let api_url = db.get::<String>("api_url").unwrap_or_default();
        let username = db.get::<String>("username").unwrap_or_default();
        let password = db.get::<String>("password").unwrap_or_default();
        let background_image = RetainedImage::from_color_image(
            "background",
            utils::load_image_from_file(current_dir().unwrap().join("assets/background.jpg")).unwrap(),
        );
    
        // Set fonts
        utils::configure_fonts(_ctx);

        IPTVApp {
            active_downloads: HashMap::new(),
            ongoing_requests: HashSet::new(),
            client: HttpClient::builder()
                .redirect_policy(isahc::config::RedirectPolicy::Follow)
                .tcp_keepalive(Duration::from_secs(1))
                .max_connections(1000)
                .build()
                .unwrap(),
            api_url,
            username,
            password,
            authenticated: false,
            db,
            image_cache,
            current_view: AppView::Login,
            view_stack: vec![],
            mpv_process: None,
            search_query: String::new(),
            movies_search_results: vec![],
            series_search_results: vec![],
            movies_cache: vec![],
            series_cache: vec![],
            background: background_image,
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
                        .save_serie_info_for_all_series(&serie_info.category_id, &serie_info);
                }
            }
            Err(err) => {
                log::error!("Failed to fetch all series: {}", err);
            }
        }
    }
}

impl eframe::App for IPTVApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        log::info!("IPTVApp Update for {:?}", self.current_view);
        // configure_fonts(ctx);
        ctx.request_repaint();
        let current_view = self.current_view.clone();

        match current_view {
            AppView::Login => render_login(self, ctx),
            AppView::Categories => render_categories(self, ctx),
            AppView::LiveCategories => render_live_categories(self, ctx),
            AppView::MoviesCategories => render_movies_categories(self, ctx),
            AppView::SeriesCategories => render_series_categories(self, ctx),
            AppView::LiveStreams(category_id, category_name) => {
                render_live_streams(self, ctx, &category_id, &category_name)
            }
            AppView::MoviesStream(category_id, category_name) => {
                render_movies_streams(self, ctx, &category_id, &category_name)
            }
            AppView::SeriesList(category_id, category_name) => {
                render_series_list(self, ctx, &category_id, &category_name)
            }
            AppView::SeriesDetail(category_id, series_id, category_name) => {
                render_series_details(self, ctx, &category_id, &series_id, &category_name)
            }
            AppView::EpisodeList(category_id, series_id, season, category_name) => {
                render_episodes_list(self, ctx, &category_id, &series_id, season, &category_name)
            }
            AppView::MoviesSearch => render_movies_search(self, ctx), // Render the search view
            AppView::SeriesSearch => render_series_search(self, ctx), // Render the search view
            AppView::Playback(stream_url) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_playback(self, ui, stream_url.clone());
                });
            }
            AppView::PlaylsitPlayback(playlist) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    futures::executor::block_on(render_playlist(self, ui, &playlist));
                });
            }
            AppView::RecentlyWatchedMovies => {
                render_recently_watched_movies(self, ctx);
            }
            AppView::RecentlyWatchedSeries => {
                render_recently_watched_series(self, ctx);
            }
        }
    }
}

impl Clone for IPTVApp {
    fn clone(&self) -> Self {
        IPTVApp {
            active_downloads: self.active_downloads.clone(),
            ongoing_requests: self.ongoing_requests.clone(),
            client: self.client.clone(),
            api_url: self.api_url.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            authenticated: self.authenticated,
            db: self.db.clone(),
            image_cache: self.image_cache.clone(),
            current_view: self.current_view.clone(),
            view_stack: self.view_stack.clone(),
            mpv_process: None,
            search_query: self.search_query.clone(),
            movies_search_results: self.movies_search_results.clone(),
            series_search_results: self.series_search_results.clone(),
            movies_cache: self.movies_cache.clone(),
            series_cache: self.series_cache.clone(),
            background: RetainedImage::from_color_image(
                "background",
                utils::load_image_from_file("../../assets/background.jpg").unwrap(),
            ),
        }
    }
}
