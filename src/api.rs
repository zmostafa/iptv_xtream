use crate::models::{Category, LoginResponse, Stream, StreamVariant};
use reqwest::Client;
use serde_json::to_vec;
use sled::Db;
use std::error::Error;

pub async fn login(
    server: &str,
    username: &str,
    password: &str,
) -> Result<LoginResponse, reqwest::Error> {
    let login_url = format!(
        "{}/player_api.php?username={}&password={}",
        server, username, password
    );
    let response = reqwest::get(&login_url).await?;
    let login_data = response.json::<LoginResponse>().await?;
    Ok(login_data)
}

use tokio::task;

pub async fn fetch_and_store_categories(
    server: &str,
    username: &str,
    password: &str,
    action: &str,
    db: &Db,
) -> Result<Vec<Category>, reqwest::Error> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action={}",
        server, username, password, action
    );

    let response = reqwest::get(&url).await?;
    let categories: Vec<Category> = response.json().await?;

    let db_key = if action == "get_live_categories" {
        "live_categories"
    } else {
        "vod_categories"
    };

    db.insert(db_key, serde_json::to_vec(&categories).unwrap())
        .expect("Failed to store categories in the database");

    Ok(categories)
}

pub async fn fetch_and_store_streams(
    server: &str,
    username: &str,
    password: &str,
    category_id: &str,
    action: &str,
    db: &Db,
) -> Result<Vec<StreamVariant>, reqwest::Error> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action={}&category_id={}",
        server, username, password, action, category_id
    );

    let response = reqwest::get(&url).await?;
    let streams: Vec<StreamVariant> = response.json().await?;

    let db_key = if action == "get_live_streams" {
        format!("live_streams_{}", category_id)
    } else {
        format!("vod_streams_{}", category_id)
    };

    db.insert(db_key, serde_json::to_vec(&streams).unwrap())
        .expect("Failed to store streams in the database");

    Ok(streams)
}

pub fn play_stream(stream_url: &str) {
    println!("Attempting to play stream: {}", stream_url);

    if let Err(e) = std::process::Command::new("vlc")
        .arg(stream_url)
        .spawn()
        .map(|mut child| child.wait())
    {
        eprintln!("Failed to play stream: {}", e);
    }
}
