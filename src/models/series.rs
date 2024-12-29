use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VideoInfo {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: Option<String>,
    pub profile: Option<String>,
    pub codec_type: String,
    pub codec_time_base: String,
    pub width: u32,
    pub height: u32,
    pub bit_rate: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AudioInfo {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: Option<String>,
    pub profile: Option<String>,
    pub sample_rate: String,
    pub channels: u32,
    pub channel_layout: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EpisodeInfo {
    pub movie_image: Option<String>,
    pub plot: Option<String>,
    // pub rating: Option<f32>,
    pub releasedate: Option<String>,
    pub duration_secs: Option<u32>,
    // pub video: Option<VideoInfo>,
    // pub audio: Option<AudioInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Episode {
    pub id: String,
    pub episode_num: u32,
    pub title: String,
    pub container_extension: String,
    pub info: EpisodeInfo,
    pub season: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SeriesInfo {
    pub num: Option<i64>,
    pub name: String,
    pub series_id: Option<i64>,
    pub cover: String,
    pub plot: String,
    pub cast: String,
    pub director: String,
    pub genre: String,
    pub releaseDate: String,
    pub last_modified: String,
    // pub rating: String,
    // pub rating_5based: f32,
    // pub backdrop_path: Option<Vec<String>>,
    pub youtube_trailer: String,
    pub episode_run_time: String,
    pub category_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    pub air_date: Option<String>,          // Nullable or missing field, use Option
    pub episode_count: u32,               // Integer value
    pub id: u32,                          // Integer value
    pub name: String,                     // String
    pub overview: String,                 // String
    pub season_number: u32,               // Integer value
    pub vote_average: f64,                // Floating point value
    pub cover: Option<String>,            // Nullable or missing field, use Option
    pub cover_big: Option<String>,        // Nullable or missing field, use Option
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Series {
    pub seasons: Vec<Season>,
    pub info: SeriesInfo,
    pub episodes: HashMap<String, Vec<Episode>>, // Map of seasons to episodes
}