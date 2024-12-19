use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_info: UserInfo,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub status: String,
    pub exp_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub url: String,
    pub port: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stream {
    pub stream_id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_adult: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_sid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VOD {
    pub stream_id: i64,
    pub name: String,
    pub container_extension: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Series {
    pub series_id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cast: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub director: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube_trailer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesInfoResponse {
    pub seasons: Vec<Season>,
    pub info: SeriesInfo,
    pub episodes: HashMap<String, Vec<Episode>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Season {
    pub season_number: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesInfo {
    pub name: String,
    pub plot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub title: String,
    pub container_extension: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamVariant {
    Live(Stream),
    VOD(VOD),
    Series(Series),
}

impl StreamVariant {
    pub fn get_id(&self) -> String {
        match self {
            StreamVariant::Live(stream) => stream.stream_id.to_string(),
            StreamVariant::VOD(vod) => vod.stream_id.to_string(),
            StreamVariant::Series(series) => series.series_id.to_string(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            StreamVariant::Live(stream) => stream.name.clone(),
            StreamVariant::VOD(vod) => vod.name.clone(),
            StreamVariant::Series(series) => series.name.clone(),
        }
    }

    pub fn get_extension(&self) -> String {
        match self {
            StreamVariant::Live(stream) => "ts".to_string(),
            StreamVariant::VOD(vod) => vod.container_extension.clone(),
            StreamVariant::Series(_) => "ts".to_string(),
        }
    }
}
