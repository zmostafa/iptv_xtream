use crate::models::{Category, LoginResponse, Stream, VOD, Series};
use reqwest::Client;

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

pub async fn fetch_categories(
    server: &str,
    username: &str,
    password: &str,
    action: &str,
) -> Result<Vec<Category>, reqwest::Error> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action={}",
        server, username, password, action
    );
    let response = reqwest::get(&url).await?;
    let categories = response.json::<Vec<Category>>().await?;
    Ok(categories)
}

pub async fn fetch_streams(
    server: &str,
    username: &str,
    password: &str,
    category_id: &str,
    action: &str,
) -> Result<Vec<Stream>, reqwest::Error> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action={}&category_id={}",
        server, username, password, action, category_id
    );
    let response = reqwest::get(&url).await?;
    let streams = response.json::<Vec<Stream>>().await?;
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
