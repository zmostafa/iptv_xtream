use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait StreamTrait: Debug {
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_type(&self) -> String;
    fn get_extension(&self) -> String;
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub user_info: UserInfo,
    pub server_info: ServerInfo,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub status: String,
    pub exp_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerInfo {
    pub url: String,
    pub port: String,
}

#[derive(Debug, Deserialize)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Stream {
    pub stream_id: i64,
    pub name: String,
    pub stream_type: Option<String>,
    pub num: i64,
    pub stream_icon: Option<String>,
    pub epg_channel_id: Option<i64>,
    pub added: String,
    pub is_adult: String,
    pub category_id: String,
    pub custom_sid: String,
    pub tv_archive: i64,
    pub direct_source: String,
    pub tv_archive_duration: i64,
}

impl StreamTrait for Stream {
    fn get_id(&self) -> String {
        self.stream_id.to_string()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> String {
        "live".to_string()
    }

    fn get_extension(&self) -> String {
        "ts".to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct VOD {
    pub num: i64,
    pub stream_id: i64,
    pub name: String,
    pub stream_type: String,
    pub stream_icon: String,
    pub rating: String,
    pub category_id: String,
    pub container_extension: String,
    pub is_adult: String,
    pub added: String,
}

impl StreamTrait for VOD {
    fn get_id(&self) -> String {
        self.stream_id.to_string()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> String {
        "Movie".to_string()
    }

    fn get_extension(&self) -> String {
        self.container_extension.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Series {
    pub series_id: i64,
    pub name: String,
    pub cover: String,
    pub plot: String,
    pub cast: String,
    pub director: String,
    pub genre: String,
    pub category_id: String,
    pub last_modified: String,
    pub rating: String,
    pub rating_5based: f32,
    pub youtube_trailer: String,
    pub episode_run_time: String,
    pub num: i64,
}

impl StreamTrait for Series {
    fn get_id(&self) -> String {
        self.series_id.to_string()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> String {
        "Series".to_string()
    }

    fn get_extension(&self) -> String {
        "ts".to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct SeriesInfoResponse {
    pub seasons: Vec<Season>,
    pub info: SeriesInfo,
    pub episodes: HashMap<String, Vec<Episode>>,
}

#[derive(Debug, Deserialize)]
pub struct Season {
    pub air_date: String,
    pub episode_count: u32,
    pub id: u64,
    pub name: String,
    pub overview: String,
    pub season_number: u32,
    pub vote_average: f32,
    pub cover: String,
    pub cover_big: String,
}

#[derive(Debug, Deserialize)]
pub struct SeriesInfo {
    pub name: String,
    pub cover: String,
    pub plot: String,
    pub cast: String,
    pub director: Option<String>,
    pub genre: String,
    pub releaseDate: String,
    pub last_modified: String,
    pub rating: String,
    pub rating_5based: f32,
    pub backdrop_path: Vec<String>,
    pub youtube_trailer: Option<String>,
    pub episode_run_time: String,
    pub category_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Episode {
    pub id: String,
    pub episode_num: u32,
    pub title: String,
    pub container_extension: String,
    // pub info: Option<EpisodeInfo>,
    pub season: u32,
    pub added: String,
    pub direct_source: String,
}

#[derive(Debug, Deserialize)]
pub struct EpisodeInfo {
    pub video: Option<VideoInfo>,
    pub audio: Option<AudioInfo>,
}

#[derive(Debug, Deserialize)]
pub struct VideoInfo {
    pub codec_name: String,
    pub width: u32,
    pub height: u32,
    pub pix_fmt: String,
    pub level: u32,
    pub avg_frame_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct AudioInfo {
    pub codec_name: String,
    pub sample_rate: String,
    pub channels: u32,
    pub channel_layout: String,
}
