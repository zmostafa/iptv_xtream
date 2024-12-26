use crate::models::live::{Category, LiveStream};
use crate::models::movies::Movie;
use crate::models::series::{Series, SeriesInfo};
use iced::widget::shader::wgpu::naga::proc::ResolveError;
use isahc::http::StatusCode;
use isahc::prelude::*;
use isahc::HttpClient;
use reqwest::Client;
use serde::ser::StdError;
use serde_json::from_str;
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

// pub async fn fetch_live_categories_v1(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
// ) -> Result<Vec<Category>, Box<dyn Error>> {
//     let url = format!(
//         "{}/player_api.php?username={}&password={}&action=get_live_categories",
//         api_url, username, password
//     );
//     let response = client.get(&url).send().await?;
//     let categories: Vec<Category> = response.json().await?;
//     log::info!("[FETCH] Live Stream Categories\n");
//     Ok(categories)
// }

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
                    let categories: Vec<Category> = serde_json::from_str::<Vec<Category>>(&body).unwrap();
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

// pub async fn fetch_vod_categories_v1(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
// ) -> Result<Vec<Category>, Box<dyn Error>> {
//     let url = format!(
//         "{}/player_api.php?username={}&password={}&action=get_vod_categories",
//         api_url, username, password
//     );
//     let response = client.get(&url).send().await?;
//     let categories: Vec<Category> = response.json().await?;
//     log::info!("[FETCH] Movies Categories\n");
//     Ok(categories)
// }

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

// pub async fn fetch_series_categories_v1(
//     client: &reqwest::Client,
//     api_url: &str,
//     username: &str,
//     password: &str,
// ) -> Result<Vec<Category>, Box<dyn std::error::Error + Send + Sync>> {
//     // Construct the URL
//     let url = format!(
//         "{}/player_api.php?username={}&password={}&action=get_series_categories",
//         api_url, username, password
//     );

//     log::debug!("Fetching series categories from URL: {}", url);

//     // Send the request
//     let response = client.get(&url).send().await?;
//     log::debug!("Received response: {:?}", response);

//     // Deserialize the response
//     let categories: Vec<Category> = response.json().await?;
//     log::debug!("Fetched categories: {:?}", categories);

//     log::info!("[FETCH] Series Categories");

//     // Return the categories
//     Ok(categories)
// }

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

// pub async fn fetch_live_streams_v1(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
//     category_id: Option<&str>,
// ) -> Result<Vec<LiveStream>, Box<dyn Error>> {
//     let url = match category_id {
//         Some(id) => format!(
//             "{}/player_api.php?username={}&password={}&action=get_live_streams&category_id={}",
//             api_url, username, password, id
//         ),
//         None => format!(
//             "{}/player_api.php?username={}&password={}&action=get_live_streams",
//             api_url, username, password
//         ),
//     };
//     log::debug!("[LIVE STREAM] Url: {}\n", url);
//     let response = client
//         .get(&url)
//         .timeout(Duration::from_secs(5))
//         .send()
//         .await?;
//     log::debug!("[LIVE STREAM] Response: {:?}", response);
//     log::debug!("[LIVE STREAM] Responsed!");
//     let streams: Vec<LiveStream> = response.json().await?;
//     log::info!("[FETCH LIVE STREAM] Live Stream Channels: {}\n", url);
//     Ok(streams)
// }

// pub async fn fetch_live_streams(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
//     category_id: Option<&str>,
// ) -> Result<Vec<LiveStream>, Box<dyn Error>> {
//     let max_attempts = 3; // Number of retry attempts
//     let mut attempt = 0;
//     let base_delay = Duration::from_secs(1); // Base delay between retries

//     let url = format!(
//         "{}/player_api.php?username={}&password={}&action=get_live_streams&category_id={}",
//         api_url,
//         username,
//         password,
//         category_id.unwrap_or("")
//     );

//     log::info!(
//         "[FETCH] Requesting live streams for category_id: {:?}",
//         category_id
//     );

//     while attempt < max_attempts {
//         attempt += 1;

//         match client.get(&url).send().await {
//             Ok(response) => {
//                 if response.status().is_success() {
//                     match response.json::<Vec<LiveStream>>().await {
//                         Ok(streams) => {
//                             log::info!(
//                                 "[FETCH SUCCESS] Retrieved {} streams for category_id: {:?}",
//                                 streams.len(),
//                                 category_id
//                             );
//                             return Ok(streams);
//                         }
//                         Err(err) => {
//                             log::error!(
//                                 "[FETCH ERROR] Failed to parse JSON response for category_id {:?}: {}",
//                                 category_id,
//                                 err
//                             );
//                             return Err(Box::new(err));
//                         }
//                     }
//                 } else {
//                     log::warn!(
//                         "[FETCH WARNING] Received non-success status {} for category_id {:?}",
//                         response.status(),
//                         category_id
//                     );
//                 }
//             }
//             Err(err) => {
//                 log::error!(
//                     "[FETCH ERROR] Attempt {}: Failed to fetch streams for category_id {:?}: {}",
//                     attempt,
//                     category_id,
//                     err
//                 );
//             }
//         }

//         // Apply exponential backoff
//         let delay = base_delay * attempt;
//         log::info!(
//             "[FETCH RETRY] Retrying in {:?} (attempt {}/{})",
//             delay,
//             attempt,
//             max_attempts
//         );
//         sleep(delay).await;
//     }

//     log::error!(
//         "[FETCH FAILURE] All attempts failed to fetch streams for category_id {:?}",
//         category_id
//     );
//     Err("Failed to fetch live streams after multiple attempts".into())
// }

// pub async fn fetch_vod_streams_v1(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
//     category_id: Option<&str>,
// ) -> Result<Vec<Movie>, Box<dyn Error>> {
//     let url = match category_id {
//         Some(id) => format!(
//             "{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}",
//             api_url, username, password, id
//         ),
//         None => format!(
//             "{}/player_api.php?username={}&password={}&action=get_vod_streams",
//             api_url, username, password
//         ),
//     };
//     log::debug!("Url: {}\n", url);
//     let response = client.get(&url).send().await?;
//     let streams: Vec<Movie> = response.json().await?;
//     log::info!("[FETCH] Movies: {}\n", url);
//     Ok(streams)
// }

// pub async fn fetch_vod_streams(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
//     category_id: Option<&str>,
// ) -> Result<Vec<Movie>, Box<dyn Error>> {
//     let max_attempts = 3;
//     let mut attempt = 0;
//     let base_delay = Duration::from_secs(1);

//     let url = format!(
//         "{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}",
//         api_url,
//         username,
//         password,
//         category_id.unwrap_or("")
//     );

//     log::info!(
//         "[FETCH] Requesting movie streams for category_id: {:?}",
//         category_id
//     );

//     while attempt < max_attempts {
//         attempt += 1;

//         match client.get(&url).send().await {
//             Ok(response) => {
//                 if response.status().is_success() {
//                     match response.json::<Vec<Movie>>().await {
//                         Ok(streams) => {
//                             log::info!(
//                                 "[FETCH SUCCESS] Retrieved {} movie streams for category_id: {:?}",
//                                 streams.len(),
//                                 category_id
//                             );
//                             return Ok(streams);
//                         }
//                         Err(err) => {
//                             log::error!("[FETCH ERROR] Failed to parse JSON response: {}", err);
//                             return Err(Box::new(err));
//                         }
//                     }
//                 } else {
//                     log::warn!("[FETCH WARNING] Non-success status: {}", response.status());
//                 }
//             }
//             Err(err) => {
//                 log::error!("[FETCH ERROR] Attempt {}: {}", attempt, err);
//             }
//         }

//         let delay = base_delay * attempt;
//         log::info!(
//             "[FETCH RETRY] Retrying in {:?} (attempt {}/{})",
//             delay,
//             attempt,
//             max_attempts
//         );
//         sleep(delay).await;
//     }

//     log::error!(
//         "[FETCH FAILURE] Failed to fetch movie streams for category_id {:?}",
//         category_id
//     );
//     Err("Failed to fetch movie streams".into())
// }

// pub async fn fetch_series_v1(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
//     category_id: Option<&str>,
// ) -> Result<Vec<SeriesInfo>, Box<dyn StdError + Send + Sync>> {
//     let url = match category_id {
//         Some(id) => format!(
//             "{}/player_api.php?username={}&password={}&action=get_series&category_id={}",
//             api_url, username, password, id
//         ),
//         None => format!(
//             "{}/player_api.php?username={}&password={}&action=get_series",
//             api_url, username, password
//         ),
//     };
//     log::debug!("Url: {}\n", url);
//     let response = client.get(&url).send().await;
//     // let series: Vec<Series> = response.json().await?;
//     match response {
//         Ok(resp) => {
//             let status = resp.status();
//             log::debug!(
//                 "[FETCH SERIES STREAM] Response Status: {}, Headers: {:?}",
//                 resp.status(),
//                 resp.headers()
//             );
//             let body = resp.text().await?;
//             log::debug!("[FETCH SERIES STREAM] Response Body: {}", body);

//             if status.is_success() {
//                 let series: Vec<SeriesInfo> = serde_json::from_str(&body)?;
//                 Ok(series)
//             } else {
//                 log::error!("[FETCH SERIES STREAM] Error response: {}", body);
//                 Err("request failed".to_string().into())
//             }
//         }
//         Err(err) => {
//             log::error!("[FETCH SERIES STREAM] Request Error: {}", err);
//             Err(Box::new(err))
//         }
//     }
//     // log::info!("[FETCH] Series\n");
//     // Ok(series)
// }

// pub async fn fetch_series(
//     client: &HttpClient,
//     api_url: &str,
//     username: &str,
//     password: &str,
//     category_id: Option<&str>,
// ) -> Result<Vec<SeriesInfo>, Box<dyn Error + Send + Sync>> {
//     let max_attempts = 3;
//     let mut attempt = 0;
//     let base_delay = Duration::from_secs(2);

//     let url = match category_id {
//         Some(id) => format!(
//             "{}/player_api.php?username={}&password={}&action=get_series&category_id={}",
//             api_url, username, password, id
//         ),
//         None => format!(
//             "{}/player_api.php?username={}&password={}&action=get_series",
//             api_url, username, password
//         ),
//     };

//     log::info!(
//         "[FETCH] Requesting series for category_id: {:?}",
//         category_id
//     );

//     while attempt < max_attempts {
//         attempt += 1;

//         match client.get(&url).send().await {
//             Ok(response) => {
//                 let status = response.status();
//                 log::debug!("[FETCH SERIES] Response Status: {}", status);

//                 if status.is_success() {
//                     let body = response.text().await?;
//                     log::debug!("[FETCH SERIES] Response Body: {}", body);

//                     match serde_json::from_str::<Vec<SeriesInfo>>(&body) {
//                         Ok(series) => {
//                             log::info!(
//                                 "[FETCH SUCCESS] Retrieved {} series for category_id: {:?}",
//                                 series.len(),
//                                 category_id
//                             );
//                             return Ok(series);
//                         }
//                         Err(err) => {
//                             log::error!("[FETCH ERROR] Failed to parse JSON response: {}", err);
//                             return Err(Box::new(err));
//                         }
//                     }
//                 } else {
//                     log::warn!("[FETCH WARNING] Non-success status: {}", status);
//                 }
//             }
//             Err(err) => {
//                 log::error!(
//                     "[FETCH ERROR] Attempt {}: Failed to fetch series: {}",
//                     attempt,
//                     err
//                 );
//             }
//         }

//         let delay = base_delay * attempt;
//         log::info!(
//             "[FETCH RETRY] Retrying in {:?} (attempt {}/{})",
//             delay,
//             attempt,
//             max_attempts
//         );
//         sleep(delay).await;
//     }

//     log::error!("[FETCH FAILURE] Failed to fetch series after multiple attempts");
//     Err("Failed to fetch series".into())
// }

// pub async fn fetch_serie_info_v1(
//     client: &reqwest::Client,
//     api_url: &str,
//     username: &str,
//     password: &str,
//     series_id: &i64,
// ) -> Result<Series, Box<dyn std::error::Error + Send + Sync>> {
//     // Construct the URL
//     let url = format!(
//         "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
//         api_url, username, password, series_id
//     );

//     log::debug!("[FETCH SERIES INFO] Fetching series info from URL: {}", url);

//     // Send the request
//     let response = client.get(&url).send().await?;
//     log::debug!("[FETCH SERIES INFO] Received response: {:?}", response);

//     // Deserialize the response
//     let series_info: Series = response.json().await?;
//     log::debug!("[FETCH SERIES INFO] Fetched series info: {:?}", series_info);

//     log::info!(
//         "[FETCH SERIES INFO] Series Episodes for series ID: {}",
//         series_id
//     );

//     // Return the series info
//     Ok(series_info)
// }

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
        // log::info!("Raw response body: {}", raw_body);

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
        // log::info!("Raw response body: {}", raw_body);

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

    // if response.status().is_success() {
    //     response.json::<Vec<SeriesInfo>>().await.map_err(|e| {
    //         ApiError::InvalidResponse(format!("Failed to parse Series: {}", e))
    //     })
    // } else {
    //     Err(ApiError::Unknown(format!(
    //         "Unexpected status code: {}",
    //         response.status()
    //     )))
    // }
    if response.status().is_success() {
        let raw_body = response.text().map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to read response body: {}", e))
        })?;
        // log::info!("Raw response body: {}", raw_body);

        serde_json::from_str::<Vec<SeriesInfo>>(&raw_body)
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse Series: {}", e)))
    } else {
        Err(ApiError::Unknown(format!(
            "Unexpected status code: {}",
            response.status()
        )))
    }
}

// // I can download using wget.
