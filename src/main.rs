// Updated main.rs
mod api;
mod models;

use crate::api::{fetch_categories, fetch_series_info, fetch_streams, login, play_stream};
use chrono::{TimeZone, Utc};
use clap::Parser;
use std::io;

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
        io::stdin()
            .read_line(&mut main_choice)
            .expect("Failed to read input");

        match main_choice.trim() {
            "1" => {
                handle_category(&config, "get_live_categories", "get_live_streams", "live").await
            }
            "2" => handle_category(&config, "get_vod_categories", "get_vod_streams", "movie").await,
            "3" => handle_series(&config).await,
            "exit" => {
                println!("Exiting the application. Goodbye!");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

async fn handle_category(
    config: &Config,
    category_action: &str,
    stream_action: &str,
    content_type: &str,
) {
    match fetch_categories(
        &config.server,
        &config.username,
        &config.password,
        category_action,
    )
    .await
    {
        Ok(categories) => {
            println!("\nAvailable Categories:");
            for category in &categories {
                println!(
                    "- {} (ID: {})",
                    category.category_name, category.category_id
                );
            }

            println!("\nEnter a Category ID to view its streams:");
            let mut category_id = String::new();
            io::stdin()
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
                    for stream in &streams {
                        println!(
                            "- {}: {} (ID: {})",
                            stream.get_type(),
                            stream.get_name(),
                            stream.get_id()
                        );
                    }

                    println!("\nEnter a Stream ID to play:");
                    let mut stream_id = String::new();
                    io::stdin()
                        .read_line(&mut stream_id)
                        .expect("Failed to read input");

                    let stream_url = format!(
                        "{}/{}/{}/{}/{}.{}",
                        config.server,
                        content_type,
                        config.username,
                        config.password,
                        stream_id.trim(),
                        streams
                            .iter()
                            .find(|s| s.get_id() == stream_id.trim())
                            .map(|s| s.get_extension())
                            .unwrap_or_else(|| "ts".to_string())
                    );

                    play_stream(&stream_url);
                }
                Err(e) => println!("Failed to fetch streams: {}", e),
            }
        }
        Err(e) => println!("Failed to fetch categories: {}", e),
    }
}

async fn handle_series(config: &Config) {
    match fetch_categories(
        &config.server,
        &config.username,
        &config.password,
        "get_series_categories",
    )
    .await
    {
        Ok(categories) => {
            println!("\nAvailable Categories:");
            for category in &categories {
                println!(
                    "- {} (ID: {})",
                    category.category_name, category.category_id
                );
            }
        }
        Err(e) => println!("Failed to fetch series categories: {}", e),
    }
    println!("\nEnter a Category ID to view its streams:");
    let mut category_id = String::new();
    io::stdin()
        .read_line(&mut category_id)
        .expect("Failed to read input");

    match fetch_streams(
        &config.server,
        &config.username,
        &config.password,
        category_id.trim(),
        "get_series",
    )
    .await
    {
        Ok(streams) => {
            println!("\nAvailable Streams:");
            for stream in &streams {
                println!(
                    "- {}: {} (ID: {})",
                    stream.get_type(),
                    stream.get_name(),
                    stream.get_id()
                );
            }
            println!("\nEnter the Series ID to fetch details:");
            let mut series_id = String::new();
            io::stdin()
                .read_line(&mut series_id)
                .expect("Failed to read input");

            match fetch_series_info(
                &config.server,
                &config.username,
                &config.password,
                series_id.trim(),
            )
            .await
            {
                Ok(series_info) => {
                    println!("\nSeries Info:");
                    println!("Name: {}", series_info.info.name);
                    println!("Plot: {}", series_info.info.plot);
                    println!("Cast: {}", series_info.info.cast);
                    println!("Director: {:?}", series_info.info.director);
                    println!("Genre: {}", series_info.info.genre);

                    println!("\nSeasons:");
                    for season in &series_info.seasons {
                        println!(
                            "- Season {} (Episodes: {})",
                            season.season_number, season.episode_count
                        );
                    }

                    println!("\nEnter a Season Number to view episodes:");
                    let mut season_choice = String::new();
                    io::stdin()
                        .read_line(&mut season_choice)
                        .expect("Failed to read input");
                    let season_choice: u32 = season_choice.trim().parse().unwrap_or(0);

                    if let Some(episodes) = series_info.episodes.get(&season_choice.to_string()) {
                        println!("\nEpisodes:");
                        for episode in episodes {
                            println!(
                                "- {}: {} (ID: {})",
                                episode.episode_num, episode.title, episode.id
                            );
                        }

                        println!("\nEnter an Episode ID to play:");
                        let mut episode_id = String::new();
                        io::stdin()
                            .read_line(&mut episode_id)
                            .expect("Failed to read input");

                        if let Some(episode) = episodes.iter().find(|ep| ep.id == episode_id.trim())
                        {
                            let stream_url = format!(
                                "{}/series/{}/{}/{}.{}",
                                config.server,
                                config.username,
                                config.password,
                                episode.id,
                                episode.container_extension
                            );

                            play_stream(&stream_url);
                        } else {
                            println!("Invalid Episode ID.");
                        }
                    } else {
                        println!("Invalid Season Number.");
                    }
                }
                Err(e) => println!("Failed to fetch series info: {}", e),
            }
        }
        Err(e) => println!("Failed to fetch series: {}", e),
    }
}
