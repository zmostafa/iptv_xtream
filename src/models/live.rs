use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LiveStream {
    pub num: u32,
    pub name: String,
    pub stream_type: String,
    pub stream_id: u32,
    pub stream_icon: String,
    pub epg_channel_id: Option<String>,
    pub added: String,
    pub is_adult: String,
    pub category_id: String,
    pub custom_sid: String,
    pub tv_archive: u8,
    pub direct_source: String,
    pub tv_archive_duration: u32,
}