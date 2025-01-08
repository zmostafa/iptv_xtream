slint::include_modules!();
mod api;
mod db;
mod models;

use crate::api::authenticate;
use crate::db::{Database, ImageCache};
use async_compat::Compat;
use isahc::config::Configurable;
use isahc::HttpClient;
use std::sync::Arc;
use std::time::Duration;

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
        // Start with the login screen (page 6)
        main_view.set_active_page(6);

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
            let image_cache = Arc::clone(&image_cache);

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

                            // Update the UI on the main thread
                            slint::invoke_from_event_loop(move || {
                                let main_view = main_view_weak_clone.unwrap();
                                main_view.set_sidebar_enabled(true);
                                main_view.set_active_page(0);
                                // TODO: disable login view after login.
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

        // Run the Slint event loop
        log::info!("Running MainView UI");
        self.main_view.run().unwrap();
        // slint::run_event_loop().unwrap();
    }
}

fn main() {
    env_logger::init();

    let app = App::new();
    app.run();
}
