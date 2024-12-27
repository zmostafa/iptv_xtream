use crate::models::live::{Category, LiveStream};
use crate::models::movies::Movie;
use crate::models::series::{Series, SeriesInfo};
use isahc::prelude::*;
use isahc::HttpClient;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub enum ApiError {
    NetworkError(String),
    InvalidResponse(String),
    AuthenticationFailed,
    RateLimited(String),
    Unknown(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            ApiError::InvalidResponse(msg) => write!(f, "Invalid Response: {}", msg),
            ApiError::AuthenticationFailed => write!(f, "Authentication Failed."),
            ApiError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            ApiError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

pub async fn authenticate(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<bool, ApiError> {
    let url = format!(
        "{}/player_api.php?username={}&password={}",
        api_url, username, password
    );
    log::debug!("Url: {}\n", url);
    let response = client.get(&url).map_err(|e| {
        ApiError::NetworkError(format!("Failed to send Authentication request: {}", e))
    })?;
    log::info!("[AUTH] Authentication Request\n");
    if response.status().is_success() {
        Ok(true)
    } else if response.status() == 401 {
        Err(ApiError::AuthenticationFailed)
    } else {
        Err(ApiError::InvalidResponse(format!(
            "Unexpected status code: {}",
            response.status()
        )))
    }
}

pub async fn fetch_live_categories(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<Category>, Box<dyn Error>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_live_categories",
        api_url, username, password
    );

    log::info!("[FETCH] Sending request to: {}", url);

    let mut attempts = 0;
    let max_attempts = 3;

    while attempts < max_attempts {
        attempts += 1;

        match client.get(&url) {
            Ok(mut response) => {
                if response.status().is_success() {
                    let body = response.text()?;
                    let categories: Vec<Category> =
                        serde_json::from_str::<Vec<Category>>(&body).unwrap();
                    log::info!("[FETCH SUCCESS] Categories: {:?}", categories);
                    return Ok(categories);
                } else {
                    log::warn!("[FETCH FAILURE] Status: {}", response.status());
                }
            }
            Err(err) => {
                log::error!("[FETCH ERROR] Attempt {}: {}", attempts, err);
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await; // Retry delay
    }

    Err("Failed to fetch live categories after multiple attempts".into())
}

pub async fn fetch_vod_categories(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<Category>, Box<dyn Error>> {
    let max_attempts = 3;
    let mut attempt = 0;
    let base_delay = Duration::from_secs(1);

    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_vod_categories",
        api_url, username, password
    );

    log::info!("[FETCH] Requesting movie categories");

    while attempt < max_attempts {
        attempt += 1;

        match client.get(&url) {
            Ok(mut response) => {
                if response.status().is_success() {
                    let body = response.text()?;
                    match serde_json::from_str::<Vec<Category>>(&body) {
                        Ok(categories) => {
                            log::info!(
                                "[FETCH SUCCESS] Retrieved {} movie categories",
                                categories.len()
                            );
                            return Ok(categories);
                        }
                        Err(err) => {
                            log::error!("[FETCH ERROR] Failed to parse JSON response: {}", err);
                            return Err(Box::new(err));
                        }
                    }
                } else {
                    log::warn!("[FETCH WARNING] Non-success status: {}", response.status());
                }
            }
            Err(err) => {
                log::error!("[FETCH ERROR] Attempt {}: {}", attempt, err);
            }
        }

        let delay = base_delay * attempt;
        log::info!(
            "[FETCH RETRY] Retrying in {:?} (attempt {}/{})",
            delay,
            attempt,
            max_attempts
        );
        sleep(delay).await;
    }

    log::error!("[FETCH FAILURE] Failed to fetch movie categories after multiple attempts");
    Err("Failed to fetch movie categories".into())
}

pub async fn fetch_series_categories(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<Category>, Box<dyn Error>> {
    let max_attempts = 3;
    let mut attempt = 0;
    let base_delay = Duration::from_secs(1);

    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series_categories",
        api_url, username, password
    );

    log::info!("[FETCH] Requesting series categories");

    while attempt < max_attempts {
        attempt += 1;

        match client.get(&url) {
            Ok(mut response) => {
                if response.status().is_success() {
                    let body = response.text()?;
                    match serde_json::from_str::<Vec<Category>>(&body) {
                        Ok(categories) => {
                            log::info!(
                                "[FETCH SUCCESS] Retrieved {} series categories",
                                categories.len()
                            );
                            return Ok(categories);
                        }
                        Err(err) => {
                            log::error!("[FETCH ERROR] Failed to parse JSON response: {}", err);
                            return Err(Box::new(err));
                        }
                    }
                } else {
                    log::warn!("[FETCH WARNING] Non-success status: {}", response.status());
                }
            }
            Err(err) => {
                log::error!("[FETCH ERROR] Attempt {}: {}", attempt, err);
            }
        }

        let delay = base_delay * attempt;
        log::info!(
            "[FETCH RETRY] Retrying in {:?} (attempt {}/{})",
            delay,
            attempt,
            max_attempts
        );
        sleep(delay).await;
    }

    log::error!("[FETCH FAILURE] Failed to fetch series categories after multiple attempts");
    Err("Failed to fetch series categories".into())
}

pub async fn fetch_serie_info(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
    series_id: &i64,
) -> Result<Series, Box<dyn Error + Send + Sync>> {
    let max_attempts = 3;
    let mut attempt = 0;
    let base_delay = Duration::from_secs(1);

    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
        api_url, username, password, series_id
    );

    log::info!(
        "[FETCH] Requesting series info for series_id: {}",
        series_id
    );

    while attempt < max_attempts {
        attempt += 1;

        match client.get(&url) {
            Ok(mut response) => {
                let status = response.status();
                log::debug!("[FETCH SERIES INFO] Response Status: {}", status);

                if status.is_success() {
                    let body = response.text()?;
                    log::debug!("[FETCH SERIES INFO] Response Body: {}", body);

                    match serde_json::from_str::<Series>(&body) {
                        Ok(series_info) => {
                            log::info!(
                                "[FETCH SUCCESS] Retrieved series info for series_id: {}",
                                series_id
                            );
                            return Ok(series_info);
                        }
                        Err(err) => {
                            log::error!(
                                "[FETCH ERROR] Failed to parse JSON response for series_id {}: {}",
                                series_id,
                                err
                            );
                            return Err(Box::new(err));
                        }
                    }
                } else {
                    log::warn!(
                        "[FETCH WARNING] Non-success status for series_id {}: {}",
                        series_id,
                        status
                    );
                }
            }
            Err(err) => {
                log::error!(
                    "[FETCH ERROR] Attempt {}: Failed to fetch series info for series_id {}: {}",
                    attempt,
                    series_id,
                    err
                );
            }
        }

        let delay = base_delay * attempt;
        log::info!(
            "[FETCH RETRY] Retrying in {:?} (attempt {}/{})",
            delay,
            attempt,
            max_attempts
        );
        sleep(delay).await;
    }

    log::error!(
        "[FETCH FAILURE] Failed to fetch series info for series_id {} after multiple attempts",
        series_id
    );
    Err("Failed to fetch series info".into())
}

pub async fn fetch_all_live_streams(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<LiveStream>, ApiError> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_live_streams",
        api_url, username, password
    );

    let mut response = client
        .get(&url)
        .map_err(|e| ApiError::NetworkError(format!("Failed to fetch all live streams: {}", e)))?;

    if response.status().is_success() {
        let raw_body = response.text().map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to read response body: {}", e))
        })?;

        serde_json::from_str::<Vec<LiveStream>>(&raw_body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Live Streams: {}", e)))
    } else {
        Err(ApiError::Unknown(format!(
            "Unexpected status code: {}",
            response.status()
        )))
    }
}

pub async fn fetch_all_movies(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<Movie>, ApiError> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_vod_streams",
        api_url, username, password
    );

    let mut response = client
        .get(&url)
        .map_err(|e| ApiError::NetworkError(format!("Failed to fetch all movies: {}", e)))?;

    if response.status().is_success() {
        let raw_body = response.text().map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to read response body: {}", e))
        })?;

        serde_json::from_str::<Vec<Movie>>(&raw_body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Movies: {}", e)))
    } else {
        Err(ApiError::Unknown(format!(
            "Unexpected status code: {}",
            response.status()
        )))
    }
}

pub async fn fetch_all_series(
    client: &HttpClient,
    api_url: &str,
    username: &str,
    password: &str,
) -> Result<Vec<SeriesInfo>, ApiError> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series",
        api_url, username, password
    );

    let mut response = client
        .get(&url)
        .map_err(|e| ApiError::NetworkError(format!("Failed to fetch all Series: {}", e)))?;

    if response.status().is_success() {
        let raw_body = response.text().map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to read response body: {}", e))
        })?;

        serde_json::from_str::<Vec<SeriesInfo>>(&raw_body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Series: {}", e)))
    } else {
        Err(ApiError::Unknown(format!(
            "Unexpected status code: {}",
            response.status()
        )))
    }
}
