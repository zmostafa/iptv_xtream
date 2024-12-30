use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LiveStream {
    pub num: u32,
    pub name: String,
    pub stream_type: String,
    pub stream_id: u32,
    pub stream_icon: String,
    pub epg_channel_id: Option<String>,
    pub added: Option<String>,
    pub is_adult: Option<String>,
    pub category_id: Option<String>,
    pub custom_sid: Option<String>,
    pub tv_archive: Option<u8>,
    pub direct_source: Option<String>,
    pub tv_archive_duration: Option<u32>,
}
