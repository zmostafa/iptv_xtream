use reqwest::Client;
use std::error::Error;
use std::time::Duration;
use serde::ser::StdError;
use crate::models::live::{Category, LiveStream};
use crate::models::movies::Movie;
use crate::models::series::{Series, SeriesInfo};

pub async fn authenticate(client:&Client, api_url: &str, username: &str, password: &str) -> Result<bool, Box<dyn Error>> {
    let url = format!("{}/player_api.php?username={}&password={}", api_url, username, password);
    log::debug!("Url: {}\n", url);
    let response = client.get(&url).send().await?;
    log::info!("[AUTH] Authentication Request\n");
    Ok(response.status().is_success())
}

pub async fn fetch_live_categories(client:&Client, api_url: &str, username: &str, password: &str) -> Result<Vec<Category>, Box<dyn Error>> {
    let url = format!("{}/player_api.php?username={}&password={}&action=get_live_categories", api_url, username, password);
    let response = client.get(&url).send().await?;
    let categories: Vec<Category> = response.json().await?;
    log::info!("[FETCH] Live Stream Categories\n");
    Ok(categories)
}

pub async fn fetch_vod_categories(client:&Client, api_url: &str, username: &str, password: &str) -> Result<Vec<Category>, Box<dyn Error>> {
    let url = format!("{}/player_api.php?username={}&password={}&action=get_vod_categories", api_url, username, password);
    let response = client.get(&url).send().await?;
    let categories: Vec<Category> = response.json().await?;
    log::info!("[FETCH] Movies Categories\n");
    Ok(categories)
}

pub async fn fetch_series_categories(
    client: &reqwest::Client,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<Category>, Box<dyn std::error::Error + Send + Sync>> {
    // Construct the URL
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series_categories",
        api_url, username, password
    );

    log::debug!("Fetching series categories from URL: {}", url);

    // Send the request
    let response = client.get(&url).send().await?;
    log::debug!("Received response: {:?}", response);

    // Deserialize the response
    let categories: Vec<Category> = response.json().await?;
    log::debug!("Fetched categories: {:?}", categories);

    log::info!("[FETCH] Series Categories");

    // Return the categories
    Ok(categories)
}

pub async fn fetch_live_streams(client:&Client, api_url: &str, username: &str, password: &str, category_id: Option<&str>) -> Result<Vec<LiveStream>, Box<dyn Error>> {
    let url = match category_id {
        Some(id) => format!("{}/player_api.php?username={}&password={}&action=get_live_streams&category_id={}", api_url, username, password, id),
        None => format!("{}/player_api.php?username={}&password={}&action=get_live_streams", api_url, username, password),
    };
    log::debug!("[LIVE STREAM] Url: {}\n", url);
    let response = client.get(&url).timeout(Duration::from_secs(5)).send().await?;
    log::debug!("[LIVE STREAM] Response: {:?}", response);
    log::debug!("[LIVE STREAM] Responsed!");
    let streams: Vec<LiveStream> = response.json().await?;
    log::info!("[FETCH LIVE STREAM] Live Stream Channels: {}\n", url);
    Ok(streams)
}

pub async fn fetch_vod_streams(client:&Client, api_url: &str, username: &str, password: &str, category_id: Option<&str>) -> Result<Vec<Movie>, Box<dyn Error>> {
    let url = match category_id {
        Some(id) => format!("{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}", api_url, username, password, id),
        None => format!("{}/player_api.php?username={}&password={}&action=get_vod_streams", api_url, username, password),
    };
    log::debug!("Url: {}\n", url);
    let response = client.get(&url).send().await?;
    let streams: Vec<Movie> = response.json().await?;
    log::info!("[FETCH] Movies: {}\n", url);
    Ok(streams)
}

pub async fn fetch_series(client:&Client, api_url: &str, username: &str, password: &str, category_id: Option<&str>) -> Result<Vec<SeriesInfo>, Box<dyn StdError + Send + Sync>> {
    let url = match category_id {
        Some(id) => format!("{}/player_api.php?username={}&password={}&action=get_series&category_id={}", api_url, username, password, id),
        None => format!("{}/player_api.php?username={}&password={}&action=get_series", api_url, username, password),
    };
    log::debug!("Url: {}\n", url);
    let response = client.get(&url).send().await;
    // let series: Vec<Series> = response.json().await?;
    match response {
        Ok(resp) => {
            let status = resp.status();
            log::debug!(
                "[FETCH SERIES STREAM] Response Status: {}, Headers: {:?}",
                resp.status(),
                resp.headers()
            );
            let body = resp.text().await?;
            log::debug!("[FETCH SERIES STREAM] Response Body: {}", body);

            if status.is_success() {
                let series: Vec<SeriesInfo> = serde_json::from_str(&body)?;
                Ok(series)
            } else {
                log::error!("[FETCH SERIES STREAM] Error response: {}", body);
                Err("request failed".to_string().into())
            }
        }
        Err(err) => {
            log::error!("[FETCH SERIES STREAM] Request Error: {}", err);
            Err(Box::new(err))
        }
    }
    // log::info!("[FETCH] Series\n");
    // Ok(series)
}

pub async fn fetch_serie_info(
    client: &reqwest::Client,
    api_url: &str,
    username: &str,
    password: &str,
    series_id: &i64,
) -> Result<Series, Box<dyn std::error::Error + Send + Sync>> {
    // Construct the URL
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
        api_url, username, password, series_id
    );

    log::debug!("[FETCH SERIES INFO] Fetching series info from URL: {}", url);

    // Send the request
    let response = client.get(&url).send().await?;
    log::debug!("[FETCH SERIES INFO] Received response: {:?}", response);

    // Deserialize the response
    let series_info: Series = response.json().await?;
    log::debug!("[FETCH SERIES INFO] Fetched series info: {:?}", series_info);

    log::info!("[FETCH SERIES INFO] Series Episodes for series ID: {}", series_id);

    // Return the series info
    Ok(series_info)
}


// I can download using wget.