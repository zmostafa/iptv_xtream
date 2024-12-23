use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Movie {
    pub num: u32,
    pub name: String,
    pub stream_type: String,
    pub stream_id: u32,
    pub stream_icon: String,
    pub rating: String,
    pub rating_5based: f32,
    pub added: String,
    pub is_adult: String,
    pub category_id: String,
    pub container_extension: String,
    pub custom_sid: String,
    pub direct_source: String,
}