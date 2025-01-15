use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Movie {
    pub num: u32,
    pub name: String,
    pub stream_type: String,
    pub stream_id: u32,
    pub stream_icon: String,
    // Rating is sometimes float not a string which breaks serialzation
    // pub rating: StringOrFloat,
    pub rating_5based: f32,
    pub added: Option<String>,
    pub is_adult: String,
    pub category_id: String,
    pub container_extension: String,
    pub custom_sid: String,
    pub direct_source: String,
}

#[derive(Debug)]
pub enum StringOrFloat {
    String(String),
    Float(f64),
}

impl Serialize for StringOrFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            StringOrFloat::String(s) => serializer.serialize_str(s),
            StringOrFloat::Float(f) => serializer.serialize_f64(*f),
        }
    }
}

impl<'de> Deserialize<'de> for StringOrFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        if let Some(s) = value.as_str() {
            Ok(StringOrFloat::String(s.to_string()))
        } else if let Some(f) = value.as_f64() {
            Ok(StringOrFloat::Float(f))
        } else {
            Err(serde::de::Error::custom("Expected string or float"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecentlyWatchedMovie {
    pub category_id: String,
    pub stream_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieResponse {
    pub info: MovieInfo,
    pub movie_data: MovieData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieInfo {
    pub movie_image: String,
    pub tmdb_id: String,
    pub backdrop: String,
    pub youtube_trailer: String,
    pub genre: String,
    pub plot: String,
    pub cast: String,
    pub rating: String,
    pub director: String,
    pub releasedate: String,
    pub backdrop_path: Vec<String>,
    pub duration_secs: u64,
    pub duration: String,
    pub video: VideoInfo,
    pub audio: AudioInfo,
    pub bitrate: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub index: u64,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: String,
    pub codec_type: String,
    pub codec_time_base: String,
    pub codec_tag_string: String,
    pub codec_tag: String,
    pub width: u64,
    pub height: u64,
    pub coded_width: u64,
    pub coded_height: u64,
    pub has_b_frames: u64,
    pub pix_fmt: String,
    pub level: u64,
    pub color_range: String,
    pub chroma_location: String,
    pub field_order: String,
    pub refs: u64,
    pub is_avc: String,
    pub nal_length_size: String,
    pub r_frame_rate: String,
    pub avg_frame_rate: String,
    pub time_base: String,
    pub start_pts: u64,
    pub start_time: String,
    pub bits_per_raw_sample: String,
    pub disposition: Disposition,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioInfo {
    pub index: u64,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: String,
    pub codec_type: String,
    pub codec_time_base: String,
    pub codec_tag_string: String,
    pub codec_tag: String,
    pub sample_fmt: String,
    pub sample_rate: String,
    pub channels: u64,
    pub channel_layout: String,
    pub bits_per_sample: u64,
    pub r_frame_rate: String,
    pub avg_frame_rate: String,
    pub time_base: String,
    pub start_pts: u64,
    pub start_time: String,
    pub disposition: Disposition,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disposition {
    pub default: u64,
    pub dub: u64,
    pub original: u64,
    pub comment: u64,
    pub lyrics: u64,
    pub karaoke: u64,
    pub forced: u64,
    pub hearing_impaired: u64,
    pub visual_impaired: u64,
    pub clean_effects: u64,
    pub attached_pic: u64,
    pub timed_thumbnails: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieData {
    pub stream_id: u64,
    pub name: String,
    pub added: String,
    pub category_id: String,
    pub container_extension: String,
    pub custom_sid: String,
    pub direct_source: String,
}