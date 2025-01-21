slint::include_modules!();
mod api;
mod app;
mod db;
mod models;
mod utils;

use crate::api::authenticate;
use crate::db::{Database, ImageCache};

use async_compat::Compat;
use image::{self, EncodableLayout};
use isahc::config::Configurable;
use isahc::HttpClient;
use log::info;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;

use slint::{Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};

struct App {
    main_view: MainView,
    client: Arc<HttpClient>, // Wrap the client in an Arc
    db: Arc<Database>,
    image_cache: Arc<ImageCache>,
}

impl App {
    fn new() -> Self {
        let main_view = MainView::new().unwrap();
        let client = Arc::new(
            HttpClient::builder()
                .redirect_policy(isahc::config::RedirectPolicy::Follow)
                .tcp_keepalive(Duration::from_secs(1))
                .max_connections(1000)
                .build()
                .unwrap(),
        );

        let db = Arc::new(Database::new("iptv_cache"));
        let image_cache = Arc::new(ImageCache::new("iptv_cache/image_cache"));

        let saved_url = db.get::<String>("api_url").unwrap_or_default();
        let saved_username = db.get::<String>("username").unwrap_or_default();
        let saved_password = db.get::<String>("password").unwrap_or_default();

        main_view.set_url(saved_url.into());
        main_view.set_username(saved_username.into());
        main_view.set_password(saved_password.into());

        // Disable other icons in the app until login
        main_view.set_sidebar_enabled(false);
        // Start with the login screen (page 10)
        main_view.set_active_page(10);

        Self {
            main_view,
            client,
            db,
            image_cache,
        }
    }

    fn run(&self) {
        let main_view_weak = self.main_view.as_weak();
        let client = Arc::clone(&self.client); // Clone the Arc
        let db = Arc::clone(&self.db);
        let image_cache = Arc::clone(&self.image_cache);
        log::info!("MainView UI created");

        // Handle login
        self.main_view.on_login(move |url, username, password| {
            // let main_view = main_view_weak.unwrap();
            let client = Arc::clone(&client); // Clone the Arc for the async task
            let db = Arc::clone(&db);
            // let image_cache = Arc::clone(&image_cache);

            if !url.is_empty() && !username.is_empty() && !password.is_empty() {
                // Clone the data to ensure it has a 'static lifetime
                let url = url.to_string();
                let username = username.to_string();
                let password = password.to_string();

                // Spawn an async task for the login API call
                let main_view_weak_clone = main_view_weak.clone();
                slint::spawn_local(Compat::new(async move {
                    match authenticate(&client, &url, &username, &password).await {
                        Ok(_) => {
                            log::info!("Login successful!");
                            db.save("api_url", &url);
                            db.save("username", &username);
                            db.save("password", &password);

                            let tv = db.get_live_categories();
                            let movies = db.get_movies_categories();
                            let series = db.get_series_categories();

                            // let the_model: Rc<VecModel<SharedString>> =
                            //     Rc::new(VecModel::from(vec!["Hello".into(), "World".into()]));
                            // // Convert it to a ModelRc.
                            // let the_model_rc = ModelRc::from(the_model.clone());

                            // Convert Vec<api::Category> to Vec<slint::Category>
                            let live_tv_categories: Vec<slint_generatedMainView::Category> = tv
                                .into_iter()
                                .map(|cat| slint_generatedMainView::Category {
                                    category_id: cat.category_id.into(),
                                    category_name: cat.category_name.into(),
                                    parent_id: cat.parent_id as i32,
                                })
                                .collect();

                            let movies_categories: Vec<slint_generatedMainView::Category> = movies
                                .into_iter()
                                .map(|cat| slint_generatedMainView::Category {
                                    category_id: cat.category_id.into(),
                                    category_name: cat.category_name.into(),
                                    parent_id: cat.parent_id as i32,
                                })
                                .collect();

                            let series_categories: Vec<slint_generatedMainView::Category> = series
                                .into_iter()
                                .map(|cat| slint_generatedMainView::Category {
                                    category_id: cat.category_id.into(),
                                    category_name: cat.category_name.into(),
                                    parent_id: cat.parent_id as i32,
                                })
                                .collect();
                            // Update the UI on the main thread
                            slint::invoke_from_event_loop(move || {
                                // To come over the issue of safely sending Rc between threads, we create the ModelRc here.
                                let live_tv_categories =
                                    ModelRc::new(VecModel::from(live_tv_categories));
                                let movies_categories =
                                    ModelRc::new(VecModel::from(movies_categories));
                                let series_categories =
                                    ModelRc::new(VecModel::from(series_categories));

                                let main_view = main_view_weak_clone.unwrap();
                                main_view.set_sidebar_enabled(true);
                                main_view.set_active_page(0);
                                // TODO: disable login view after login.
                                main_view.set_live_tv_categories(live_tv_categories);
                                main_view.set_movies_categories(movies_categories);
                                main_view.set_series_categories(series_categories);
                            })
                            .unwrap();
                        }
                        Err(_) => {
                            log::error!("Login failed: Authentication error.");
                        }
                    }
                }))
                .unwrap();
            } else {
                log::error!("Login failed: Username and password cannot be empty.");
            }
        });

        // Handle category selection
        let db = Arc::clone(&self.db);
        let image_cache = Arc::clone(&image_cache);
        let client = Arc::clone(&self.client);
        let main_view_weak = self.main_view.as_weak();

        self.main_view
            .on_handle_category_selected(move |page_number, category| {
                let main_view = main_view_weak.unwrap();
                log::info!(
                    "Category selected: {} (ID: {}) on page {}",
                    category.category_name,
                    category.category_id,
                    page_number
                );

                // Use the category_id or other fields to fetch more data
                let category_id = category.category_id.clone();
                let category_name = category.category_name.clone();
                let parent_id = category.parent_id;

                log::info!("Category ID: {}, Parent ID: {}", category_id, parent_id);

                match page_number {
                    0 => {
                        // Handle category selection (e.g., fetch detailed content)
                        let live_streams = db.get_live_streams(&category_id);

                        // Convert streams to Slint-compatible format
                        let slint_streams: Vec<slint_generatedMainView::LiveStream> = live_streams
                            .into_iter()
                            .map(|stream| {
                                // Try to load the image from the cache
                                let img = image_cache
                                    .load_image(&stream.stream_icon)
                                    .unwrap_or_default();

                                // If the image is not in the cache, download it asynchronously
                                if !image_cache.is_cached(&stream.stream_icon) {
                                    let client_clone = Arc::clone(&client);
                                    let image_cache_clone: Arc<ImageCache> =
                                        Arc::clone(&image_cache);
                                    let stream_icon = stream.stream_icon.clone();
                                    let main_view_weak = main_view_weak.clone();
                                    let stream_id = stream.stream_id;

                                    // Spawn a background task to download the image
                                    slint::spawn_local(Compat::new(async move {
                                        if let Err(err) = utils::fetch_and_cache_image(
                                            (*client_clone).clone(),
                                            (*image_cache_clone).clone(),
                                            &stream_icon,
                                        )
                                        .await
                                        {
                                            log::error!("Failed to download image: {}", err);
                                        } else {
                                            // Image downloaded successfully, update the UI
                                            slint::invoke_from_event_loop(move || {
                                                if let Some(main_view) = main_view_weak.upgrade() {
                                                    // Find the stream in the current list and update its image
                                                    let mut streams = main_view
                                                        .get_livestreams()
                                                        .iter()
                                                        .collect::<Vec<_>>();
                                                    if let Some(stream) = streams
                                                        .iter_mut()
                                                        .find(|s| s.stream_id == stream_id as i32)
                                                    {
                                                        if let Ok(img) = image_cache_clone
                                                            .load_image(&stream_icon)
                                                        {
                                                            if let Ok(image) =
                                                                image::load_from_memory(&img)
                                                            {
                                                                let image = image.into_rgba8();
                                                                stream.stream_icon =
                                                                    Image::from_rgba8(
                                                                        SharedPixelBuffer::<
                                                                            Rgba8Pixel,
                                                                        >::clone_from_slice(
                                                                            &image.as_bytes(),
                                                                            image.width(),
                                                                            image.height(),
                                                                        ),
                                                                    );
                                                            }
                                                        }
                                                    }
                                                    // Update the UI with the new stream list
                                                    main_view.set_livestreams(
                                                        ModelRc::new(VecModel::from(streams))
                                                            .into(),
                                                    );
                                                }
                                            })
                                            .unwrap();
                                        }
                                    }))
                                    .unwrap();
                                }

                                // Create the LiveStream object with a placeholder image
                                slint_generatedMainView::LiveStream {
                                    num: stream.num as i32,
                                    name: stream.name.into(),
                                    stream_type: stream.stream_type.into(),
                                    stream_id: stream.stream_id as i32,
                                    stream_icon: if img.is_empty() {
                                        // Use a placeholder image if the image is not yet downloaded
                                        Image::default()
                                    } else {
                                        // Use the cached image
                                        if let Ok(image) = image::load_from_memory(&img) {
                                            let image = image.into_rgba8();
                                            Image::from_rgba8(
                                                SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                                                    &image.as_bytes(),
                                                    image.width(),
                                                    image.height(),
                                                ),
                                            )
                                        } else {
                                            Image::default()
                                        }
                                    },
                                    epg_channel_id: stream
                                        .epg_channel_id
                                        .unwrap_or_default()
                                        .into(),
                                    added: stream.added.unwrap_or_default().into(),
                                    is_adult: stream.is_adult.unwrap_or_default().into(),
                                    category_id: stream.category_id.unwrap_or_default().into(),
                                    custom_sid: stream.custom_sid.unwrap_or_default().into(),
                                    tv_archive: stream.tv_archive.unwrap_or_default().into(),
                                    direct_source: stream.direct_source.unwrap_or_default().into(),
                                    tv_archive_duration: stream
                                        .tv_archive_duration
                                        .unwrap_or_default()
                                        as i32,
                                }
                            })
                            .collect();

                        // Update the UI with the initial list of streams (some images may be placeholders)
                        main_view
                            .set_livestreams(ModelRc::new(VecModel::from(slint_streams)).into());
                    }
                    1 => {
                        // Handle category selection (e.g., fetch detailed content)
                        let movies_streams = db.get_movies_streams(&category_id);

                        // Convert streams to Slint-compatible format
                        let mut slint_streams: Vec<slint_generatedMainView::Movie> = movies_streams
                            .into_iter()
                            .map(|stream| {
                                // If the image is not in the cache, download it asynchronously
                                let client_clone = Arc::clone(&client);
                                let image_cache_clone = Arc::clone(&image_cache);
                                let stream_icon = stream.stream_icon.clone();
                                let main_view_weak = main_view_weak.clone();
                                let stream_id = stream.stream_id;

                                // Spawn a background task to download the image
                                slint::spawn_local(Compat::new(async move {
                                    if !image_cache_clone.is_cached(&stream_icon) {
                                        if let Err(err) = utils::fetch_and_cache_image(
                                            (*client_clone).clone(),
                                            (*image_cache_clone).clone(),
                                            &stream_icon,
                                        )
                                        .await
                                        {
                                            log::error!("Failed to download image: {}", err);
                                        }
                                    }
                                    // Image downloaded successfully, update the UI
                                    slint::invoke_from_event_loop(move || {
                                        if let Some(main_view) = main_view_weak.upgrade() {
                                            // Find the stream in the current list and update its image
                                            log::debug!("Updating ui main thread");
                                            let mut streams = main_view
                                                .get_movies_streams()
                                                .iter()
                                                .collect::<Vec<_>>();
                                            if let Some(stream) = streams
                                                .iter_mut()
                                                .find(|s| s.stream_id == stream_id as i32)
                                            {
                                                if let Ok(img) =
                                                    image_cache_clone.load_image(&stream_icon)
                                                {
                                                    if let Ok(image) = image::load_from_memory(&img)
                                                    {
                                                        let image = image.into_rgba8();
                                                        stream.stream_icon =
                                                            Image::from_rgba8(SharedPixelBuffer::<
                                                                Rgba8Pixel,
                                                            >::clone_from_slice(
                                                                &image.as_bytes(),
                                                                image.width(),
                                                                image.height(),
                                                            ));
                                                    }
                                                }
                                            }
                                            // Update the UI with the new stream list
                                            main_view.set_movies_streams(
                                                ModelRc::new(VecModel::from(streams)).into(),
                                            );
                                        }
                                    })
                                    .unwrap();
                                    // }
                                }))
                                .unwrap();
                                // }

                                // Create the Movie object with a placeholder image
                                slint_generatedMainView::Movie {
                                    num: stream.num as i32,
                                    name: stream.name.into(),
                                    stream_type: stream.stream_type.into(),
                                    stream_id: stream.stream_id as i32,
                                    stream_icon: Image::default(),
                                    added: stream.added.unwrap_or_default().into(),
                                    is_adult: stream.is_adult.into(),
                                    category_id: stream.category_id.into(),
                                    custom_sid: stream.custom_sid.into(),
                                    rating_5based: stream.rating_5based.into(),
                                    container_extension: stream.container_extension.into(),
                                    direct_source: stream.direct_source.into(),
                                    download_path: db
                                        .is_downloaded(&stream_id)
                                        .unwrap_or_default()
                                        .into(),
                                }
                            })
                            .collect();

                        slint_streams.sort_by(|a, b| b.added.cmp(&a.added)); // Sort by `added` date in descending order

                        // Update the UI with the initial list of streams (some images may be placeholders)
                        main_view
                            .set_movies_streams(ModelRc::new(VecModel::from(slint_streams)).into());
                    }
                    2 => {}
                    _ => {
                        log::error!("Unkmown page");
                    }
                }
                main_view.set_selected_category(category);
            });

        let db = Arc::clone(&self.db);
        self.main_view
            .on_handle_livestream_selected(move |livestream| {
                log::warn!("Stream: {:?}", livestream);

                let stream_url = format!(
                    "{}/{}/{}/{}/{}.{}",
                    db.get::<String>("api_url").unwrap_or_default(),
                    "live",
                    db.get::<String>("username").unwrap_or_default(),
                    db.get::<String>("password").unwrap_or_default(),
                    livestream,
                    "ts"
                );

                log::info!("Playing Live Stream {}", &stream_url);

                // Socket path
                let temp_dir = std::env::current_dir().unwrap().join("iptv_cache");
                let socket_path = temp_dir.join("mpvsocket");

                // TODO: only ONE player should be allowed.
                Command::new("mpv")
                    .arg(&stream_url)
                    .arg("--no-terminal")
                    .arg("--force-window=yes")
                    .arg(format!(
                        "--input-ipc-server={}",
                        socket_path.to_string_lossy()
                    ))
                    .arg("--resume-playback")
                    .arg("--save-position-on-quit")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to start MPV");
            });

        let db = Arc::clone(&self.db);
        self.main_view
            .on_handle_movie_selected(move |movie, extension| {
                log::warn!("Stream: {:?}", movie);

                let stream_url = if let Some(offline) = db.is_downloaded(&(movie as u32)) {
                    log::info!("Stream found offline");
                    offline
                } else {
                    format!(
                        "{}/{}/{}/{}/{}.{}",
                        db.get::<String>("api_url").unwrap_or_default(),
                        "movie",
                        db.get::<String>("username").unwrap_or_default(),
                        db.get::<String>("password").unwrap_or_default(),
                        movie,
                        extension
                    )
                };

                log::info!("Playing Live Stream {:?}", &stream_url);

                // Socket path
                let temp_dir = std::env::current_dir().unwrap().join("iptv_cache");
                let socket_path = temp_dir.join("mpvsocket");

                // TODO: only ONE player should be allowed.
                Command::new("mpv")
                    .arg(&stream_url)
                    .arg("--no-terminal")
                    .arg("--force-window=yes")
                    .arg(format!(
                        "--input-ipc-server={}",
                        socket_path.to_string_lossy()
                    ))
                    .arg("--resume-playback")
                    .arg("--save-position-on-quit")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to start MPV");
            });

        let db = Arc::clone(&self.db);
        let client = Arc::clone(&self.client);
        let main_view_weak = self.main_view.as_weak();

        self.main_view
            .on_handle_movie_download(move |movie, extension| {
                log::info!("Start donwloading movie {}", movie);
                let stream_url = format!(
                    "{}/{}/{}/{}/{}.{}",
                    db.get::<String>("api_url").unwrap_or_default(),
                    "movie",
                    db.get::<String>("username").unwrap_or_default(),
                    db.get::<String>("password").unwrap_or_default(),
                    movie,
                    extension
                );

                let save_path = format!("iptv_cache/{}.{}", movie, extension);
                let progress = Arc::new(Mutex::new(0.0));
                let progress_clone_for_download = Arc::clone(&progress);

                let db = db.clone();
                tokio::spawn({
                    async move {
                        if let Err(e) = utils::download_with_wget_async(
                            &stream_url,
                            &save_path,
                            progress_clone_for_download,
                        )
                        .await
                        {
                            log::error!("Download failed: {}", e);
                        } else {
                            log::info!("Download completed: {}", save_path);
                            log::info!("Saving download to database");
                            db.save_download(movie.clone() as u32, &save_path);
                        }
                    }
                });

                // Listen for progress updates and update the UI
                let main_view_weak = main_view_weak.clone();
                // Spawn a task to listen for progress updates and update the UI
                tokio::spawn(async move {
                    loop {
                        // Sleep for a short duration to avoid busy-waiting
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                        // Lock the progress mutex to read the current progress
                        let progress_value = *progress.lock().unwrap();
                        log::info!("Download progress ..... {}", progress_value);

                        // Update the UI with the current progress
                        let main_view_weak_progress = main_view_weak.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(main_view) = main_view_weak_progress.upgrade() {
                                if progress_value < 1.0 {
                                    main_view.set_is_downloading(true);
                                }
                                main_view.set_download_progress(progress_value);
                            }
                        })
                        .unwrap();

                        // Stop updating progress when download is 100% and update UI
                        if progress_value == 1.0 {
                            let main_view_weak_downloaded = main_view_weak.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(main_view) = main_view_weak_downloaded.upgrade() {
                                    main_view.set_is_downloaded(true);
                                    main_view.set_is_downloading(false); // Ensure downloading state is reset
                                }
                            })
                            .unwrap();
                            break; // Exit the loop when download is complete
                        }
                    }
                });
            });

        let db = Arc::clone(&self.db);
        let image_cache = Arc::clone(&self.image_cache);
        let main_view_weak = self.main_view.as_weak();

        self.main_view.on_handle_search(move |query| {
            let main_view = main_view_weak.unwrap();
            let movies = db
                .get_movies_categories()
                .iter()
                .flat_map(|category| db.get_movies_streams(&category.category_id))
                .collect::<Vec<_>>();

            let search_result = movies
                .into_iter()
                .filter(|movie| movie.name.to_lowercase().contains(&query.to_lowercase()))
                .collect::<Vec<_>>();

            let mut slint_search_streams: Vec<slint_generatedMainView::Movie> = search_result
                .into_iter()
                .map(|stream| {
                    let client_clone = Arc::clone(&client);
                    let image_cache_clone = Arc::clone(&image_cache);
                    let stream_icon = stream.stream_icon.clone();
                    let main_view_weak = main_view_weak.clone();
                    let stream_id = stream.stream_id;

                    // Spawn a background task to download the image
                    slint::spawn_local(Compat::new(async move {
                        if !image_cache_clone.is_cached(&stream_icon) {
                            if let Err(err) = utils::fetch_and_cache_image(
                                (*client_clone).clone(),
                                (*image_cache_clone).clone(),
                                &stream_icon,
                            )
                            .await
                            {
                                log::error!("Failed to download image: {}", err);
                            }
                        }
                        // Image downloaded successfully, update the UI
                        slint::invoke_from_event_loop(move || {
                            if let Some(main_view) = main_view_weak.upgrade() {
                                // Find the stream in the current list and update its image
                                log::debug!("Updating ui main thread");
                                let mut streams =
                                    main_view.get_movies_streams().iter().collect::<Vec<_>>();
                                if let Some(stream) =
                                    streams.iter_mut().find(|s| s.stream_id == stream_id as i32)
                                {
                                    if let Ok(img) = image_cache_clone.load_image(&stream_icon) {
                                        if let Ok(image) = image::load_from_memory(&img) {
                                            let image = image.into_rgba8();
                                            stream.stream_icon = Image::from_rgba8(
                                                SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                                                    &image.as_bytes(),
                                                    image.width(),
                                                    image.height(),
                                                ),
                                            );
                                        }
                                    }
                                }
                                // Update the UI with the new stream list
                                main_view.set_movies_streams(
                                    ModelRc::new(VecModel::from(streams)).into(),
                                );
                            }
                        })
                        .unwrap();
                        // }
                    }))
                    .unwrap();
                    // }

                    // Create the Movie object with a placeholder image
                    slint_generatedMainView::Movie {
                        num: stream.num as i32,
                        name: stream.name.into(),
                        stream_type: stream.stream_type.into(),
                        stream_id: stream.stream_id as i32,
                        stream_icon: Image::default(),
                        added: stream.added.unwrap_or_default().into(),
                        is_adult: stream.is_adult.into(),
                        category_id: stream.category_id.into(),
                        custom_sid: stream.custom_sid.into(),
                        rating_5based: stream.rating_5based.into(),
                        container_extension: stream.container_extension.into(),
                        direct_source: stream.direct_source.into(),
                        download_path: db.is_downloaded(&stream_id).unwrap_or_default().into(),
                    }
                })
                .collect();

            slint_search_streams.sort_by(|a, b| b.added.cmp(&a.added)); // Sort by `added` date in descending order

            // Update the UI with the initial list of streams (some images may be placeholders)
            main_view.set_movies_streams(ModelRc::new(VecModel::from(slint_search_streams)).into());
        });

        let db = Arc::clone(&self.db);
        let main_view_weak = self.main_view.as_weak();

        self.main_view.on_handle_movie_delete(move |movie| {
            log::warn!("Deleting downloaded media .....");

            let db = db.clone();
            tokio::spawn(async move {
                let save_path = format!("iptv_cache/{}.*", movie);
                let _ = std::fs::remove_file(save_path);
                db.remove_download_path(&(movie as u32));
            });

            let main_view_weak = main_view_weak.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(main_view) = main_view_weak.upgrade() {
                    log::debug!("Update ui main thread from delete");
                    main_view.set_is_downloaded(false);
                    main_view.set_is_downloading(false);
                }
            })
            .unwrap();
            log::info!("Deleting downloaded media is done");
        });

        // Run the Slint event loop
        log::info!("Running MainView UI");
        self.main_view.run().unwrap();
        // slint::run_event_loop().unwrap();
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = App::new();
    app.run();
}
