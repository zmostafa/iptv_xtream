use serde::Deserialize;

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
    pub stream_type: Option<String>, // e.g., live or VOD
    pub container_extension: String,
}

#[derive(Debug, Deserialize)]
pub struct VOD {
    pub stream_id: i64,
    pub name: String,
    pub stream_icon: String,
    pub rating: String,
    pub category_id: String,
    pub container_extension: String,
}

#[derive(Debug, Deserialize)]
pub struct Series {
    pub series_id: i64,
    pub name: String,
    pub cover: String,
    pub plot: String,
    pub category_id: String,
}
