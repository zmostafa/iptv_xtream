use crate::models::{Category, LoginResponse, Series, Stream, StreamTrait, VOD};
use reqwest::Client;
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
) -> Result<Vec<Box<dyn StreamTrait>>, Box<dyn Error>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action={}&category_id={}",
        server, username, password, action, category_id
    );

    let client = Client::new();
    let response = client.get(&url).send().await?;
    let body = response.text().await?;

    println!("Raw Response: {}", body); // Log the raw response for debugging

    let streams: Vec<Box<dyn StreamTrait>> = match action {
        "get_live_streams" => {
            let data: Vec<Stream> = serde_json::from_str(&body)?;
            data.into_iter()
                .map(|s| Box::new(s) as Box<dyn StreamTrait>)
                .collect()
        }
        "get_vod_streams" => {
            let data: Vec<VOD> = serde_json::from_str(&body)?;
            data.into_iter()
                .map(|v| Box::new(v) as Box<dyn StreamTrait>)
                .collect()
        }
        "get_series" => {
            let data: Vec<Series> = serde_json::from_str(&body)?;
            data.into_iter()
                .map(|s| Box::new(s) as Box<dyn StreamTrait>)
                .collect()
        }
        _ => return Err("Unsupported action".into()),
    };

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

use crate::models::{SeriesInfoResponse};

pub async fn fetch_series_info(
    server: &str,
    username: &str,
    password: &str,
    series_id: &str,
) -> Result<SeriesInfoResponse, Box<dyn Error>> {
    let url = format!(
        "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
        server, username, password, series_id
    );

    let response = reqwest::get(&url).await?;
    let body = response.text().await?;

    println!("Raw Series Info Response: {}", body); // Log the response for debugging

    let series_info = serde_json::from_str::<SeriesInfoResponse>(&body)?;
    Ok(series_info)
}

// {
//     "num": 2283,
//     "name": "How to Date Billy Walsh (2024)",
//     "stream_type": "movie",
//     "stream_id": 647115,
//     "stream_icon": "https:\/\/lionzmg.com\/images2024\/5qcHcoJL3otMwJPKnbV5M3a1jQd_big.jpg",
//     "rating": "5.6",
//     "rating_5based": 2.8,
//     "added": "1712441495",
//     "is_adult": "0",
//     "category_id": "940",
//     "container_extension": "mkv",
//     "custom_sid": "",
//     "direct_source": ""
// },
// {
//     "num": 2286,
//     "name": "Dune: Part Two ( 2024 ) 4K",
//     "stream_type": "movie",
//     "stream_id": 646938,
//     "stream_icon": "https:\/\/lionzmg.com\/uploads\/171236046688351.jpg",
//     "rating": "8.36",
//     "rating_5based": 4.2,
//     "added": "1712360489",
//     "is_adult": "0",
//     "category_id": "482",
//     "container_extension": "mkv",
//     "custom_sid": "",
//     "direct_source": ""
// }


// {
//     "num": 2874,
//     "name": "Af | DSTV: Mzansi Magic HD",
//     "stream_type": "live",
//     "stream_id": 353841,
//     "stream_icon": "https:\/\/lionzmg.com\/uploads\/172210700119793.jpeg",
//     "epg_channel_id": null,
//     "added": "1647781689",
//     "is_adult": "0",
//     "category_id": "820",
//     "custom_sid": "",
//     "tv_archive": 0,
//     "direct_source": "",
//     "tv_archive_duration": 0
// },
// {
//     "num": 2875,
//     "name": "Af | DSTV: Mzansi Bioskop",
//     "stream_type": "live",
//     "stream_id": 353840,
//     "stream_icon": "https:\/\/lionzmg.com\/uploads\/172210795064343.jpeg",
//     "epg_channel_id": null,
//     "added": "1647781688",
//     "is_adult": "0",
//     "category_id": "820",
//     "custom_sid": "",
//     "tv_archive": 0,
//     "direct_source": "",
//     "tv_archive_duration": 0
// }


// {
//     "backdrop_path": "",
//     "category_id": "652",
//     "cover": "",
//     "episode_run_time": "0",
//     "last_modified": "1598242931",
//     "name": "",
//     "num": 649,
//     "plot": "",
//     "rating": "7",
//     "rating_5based": 3.5,
//     "releaseDate": "",
//     "series_id": 1281,
//     "youtube_trailer": ""
// }

// {
//     "category_id": "636",
//     "category_name": "Pakistan - \u0628\u0627\u0643\u0633\u062a\u0627\u0646",
//     "parent_id": 0
// },
// {
//     "category_id": "933",
//     "category_name": "China - \u0627\u0644\u0635\u064a\u0646",
//     "parent_id": 0
// }
