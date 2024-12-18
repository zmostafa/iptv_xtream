mod api;
mod models;

use crate::api::{fetch_categories, fetch_streams, login, play_stream};
use crate::models::{Category, Stream};
use chrono::{TimeZone, Utc};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Config {
    #[arg(short, long)]
    server: String,
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    password: String,
}

#[tokio::main]
async fn main() {
    let config = Config::parse();

    match login(&config.server, &config.username, &config.password).await {
        Ok(login_data) => {
            println!("Login Successful!");
            println!("User: {}", login_data.user_info.username);
            println!("Status: {}", login_data.user_info.status);

            if let Some(exp_date_str) = login_data.user_info.exp_date {
                if let Ok(exp_timestamp) = exp_date_str.parse::<i64>() {
                    let exp_date = Utc.timestamp_opt(exp_timestamp, 0).unwrap();
                    println!("Expires: {}", exp_date.format("%e %B %Y"));
                }
            }
        }
        Err(e) => {
            println!("Failed to login: {}", e);
            return;
        }
    }

    loop {
        println!("\nSelect a category:\n1. Live Streams\n2. Movies (VOD)\n3. Series\nType 'exit' to quit.");
        let mut main_choice = String::new();
        std::io::stdin().read_line(&mut main_choice).expect("Failed to read input");

        match main_choice.trim() {
            "1" => handle_category(&config, "get_live_categories", "get_live_streams", "live").await,
            "2" => handle_category(&config, "get_vod_categories", "get_vod_streams", "movie").await,
            "3" => handle_category(&config, "get_series_categories", "get_series", "series").await,
            "exit" => {
                println!("Exiting the application. Goodbye!");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

async fn handle_category(config: &Config, category_action: &str, stream_action: &str, content_type: &str) {
    match fetch_categories(&config.server, &config.username, &config.password, category_action).await
    {
        Ok(categories) => {
            println!("\nAvailable Categories:");
            for category in &categories {
                println!("- {} (ID: {})", category.category_name, category.category_id);
            }

            println!("\nEnter a Category ID to view its streams:");
            let mut category_id = String::new();
            std::io::stdin()
                .read_line(&mut category_id)
                .expect("Failed to read input");

            match fetch_streams(
                &config.server,
                &config.username,
                &config.password,
                category_id.trim(),
                stream_action,
            )
            .await
            {
                Ok(streams) => {
                    println!("\nAvailable Streams:");
                    let mut stream_extension = String::new();
                    for stream in &streams {
                        println!("- {} (ID: {}), extension: {}", stream.name, stream.stream_id, stream.container_extension);
                        stream_extension = stream.container_extension.clone();
                    }

                    println!("\nEnter a Stream ID to play:");
                    let mut stream_id = String::new();
                    std::io::stdin()
                        .read_line(&mut stream_id)
                        .expect("Failed to read input");

                    // Construct the URL dynamically based on content type
                    let stream_url = format!(
                        "{}/{}/{}/{}/{}.{}",
                        config.server,
                        content_type,
                        config.username,
                        config.password,
                        stream_id.trim(),
                        // if content_type == "vod" { "mkv" } else { "ts" }
                        stream_extension
                    );

                    play_stream(&stream_url);
                }
                Err(e) => println!("Failed to fetch streams: {}", e),
            }
        }
        Err(e) => println!("Failed to fetch categories: {}", e),
    }
}

