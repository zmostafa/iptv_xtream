use sled::Db;
use serde::{Serialize, Deserialize};
use serde_json;
use crate::models::live::{Category, LiveStream};
use crate::models::series::{Series, SeriesInfo};
use crate::models::movies::Movie;

#[derive(Clone)]
pub struct Database {
    db: Db,
}

impl Database {
    pub fn new(path: &str) -> Self {
        log::info!("[DB] Start DB.");
        let db = sled::open(path).expect("Failed to open database");
        Database { db }
    }

    pub fn save<T: Serialize>(&self, key: &str, value: &T) {
        let serialized = serde_json::to_vec(value).unwrap();
        self.db.insert(key, serialized).unwrap();
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.db.get(key).ok().flatten().and_then(|v| serde_json::from_slice(&v).ok())
    }

    // pub fn save_live_categories(&self, categories: Vec<Category>) {
    //     self.save("live_categories", &categories);
    // }

    // pub fn get_live_categories(&self) -> Vec<Category> {
    //     self.get("live_categories").unwrap_or_default()
    // }

    // pub fn save_vod_categories(&self, categories: Vec<Category>) {
    //     self.save("vod_categories", &categories);
    // }

    // pub fn get_vod_categories(&self) -> Vec<Category> {
    //     self.get("vod_categories").unwrap_or_default()
    // }

    // pub fn save_series_categories(&self, categories: Vec<Category>) {
    //     self.save("series_categories", &categories);
    // }

    // pub fn get_series_categories(&self) -> Vec<Category> {
    //     self.get("series_categories").unwrap_or_default()
    // }

    // pub fn save_streams(&self, key: &str, streams: Vec<Series>) {
    //     self.save(key, &streams);
    // }

    // pub fn get_streams(&self, key: &str) -> Vec<Series> {
    //     self.get(key).unwrap_or_default()
    // }
    pub fn save_live_categories(&self, categories: Vec<Category>) {
        log::info!("[DB] Save Live Categories\n.");
        let key = "live_categories";
        let serialized = bincode::serialize(&categories).expect("Failed to serialize categories");
        self.db.insert(key, serialized).expect("Failed to save categories");
    }

    pub fn get_live_categories(&self) -> Vec<Category> {
        log::info!("[DB] Get Live Categories.");
        let key = "live_categories";
        if let Ok(Some(data)) = self.db.get(key) {
            bincode::deserialize(&data).expect("Failed to deserialize categories")
        } else {
            vec![] // Return an empty vector if no data is found
        }
    }

    pub fn save_live_streams(&self, category_id: &str, streams: Vec<LiveStream>) {
        let key = format!("live_streams_{}", category_id);
        let serialized = bincode::serialize(&streams).expect("Failed to serialize streams");
        self.db.insert(key, serialized).expect("Failed to save streams");
        log::info!("[DB] Save Live Streams.");
    }

    pub fn get_live_streams(&self, category_id: &str) -> Vec<LiveStream> {
        log::info!("[DB] Get Live Streams.");
        let key = format!("live_streams_{}", category_id);
        if let Ok(Some(data)) = self.db.get(key) {
            bincode::deserialize(&data).expect("Failed to deserialize streams")
        } else {
            vec![]
        }
    }

    pub fn save_movies_categories(&self, categories: Vec<Category>) {
        log::info!("[DB] Save Movies Categories.");
        let key = "movies_categories";
        let serialized = bincode::serialize(&categories).expect("Failed to serialize categories");
        self.db.insert(key, serialized).expect("Failed to save categories");
    }

    pub fn get_movies_categories(&self) -> Vec<Category> {
        log::info!("[DB] Get Movies Categories.");
        let key = "movies_categories";
        if let Ok(Some(data)) = self.db.get(key) {
            bincode::deserialize(&data).expect("Failed to deserialize categories")
        } else {
            vec![] // Return an empty vector if no data is found
        }
    }

    pub fn save_movies_streams(&self, category_id: &str, streams: Vec<Movie>) {
        let key = format!("movies_streams_{}", category_id);
        let serialized = bincode::serialize(&streams).expect("Failed to serialize streams");
        self.db.insert(key, serialized).expect("Failed to save streams");
        log::info!("[DB] Save Movies Streams.");
    }

    pub fn get_movies_streams(&self, category_id: &str) -> Vec<Movie> {
        log::info!("[DB] Get Movies Streams.");
        let key = format!("movies_streams_{}", category_id);
        if let Ok(Some(data)) = self.db.get(key) {
            bincode::deserialize(&data).expect("Failed to deserialize streams")
        } else {
            vec![]
        }
    }

    pub fn save_series_categories(&self, categories: Vec<Category>) {
        log::info!("[DB] Save Series Categories.");
        let key = "series_categories";
        let serialized = bincode::serialize(&categories).expect("Failed to serialize categories");
        self.db.insert(key, serialized).expect("Failed to save categories");
    }

    pub fn get_series_categories(&self) -> Vec<Category> {
        log::info!("[DB] Get Series Categories.");
        let key = "series_categories";
        if let Ok(Some(data)) = self.db.get(key) {
            bincode::deserialize(&data).expect("Failed to deserialize categories")
        } else {
            vec![] // Return an empty vector if no data is found
        }
    }

    pub fn save_series_streams(&self, category_id: &str, streams: Vec<SeriesInfo>) {
        let key = format!("series_streams_{}", category_id);
        let serialized = bincode::serialize(&streams).expect("Failed to serialize streams");
        self.db.insert(key, serialized).expect("Failed to save streams");
        log::info!("[DB] Save Series Streams.");
    }

    pub fn get_series_streams(&self, category_id: &str) -> Vec<SeriesInfo> {
        log::info!("[DB] Get Series Streams.");
        let key = format!("series_streams_{}", category_id);
        if let Ok(Some(data)) = self.db.get(key) {
            bincode::deserialize(&data).expect("Failed to deserialize streams")
        } else {
            vec![]
        }
    }

    pub fn save_series_info(&self, serie_id: &i64, series: &Series) {
        let key = format!("serie_info_{}", serie_id);
        let serialized = bincode::serialize(series).expect("Failed to serialize series info");
        self.db.insert(key, serialized).expect("Failed to save series info");
        log::info!("[DB] Save Series Info.");
    }
    
    pub fn get_series_info(&self, serie_id: &i64) -> Option<Series> {
        log::info!("[DB] Get Series Info.");
        let key = format!("serie_info_{}", serie_id);
        match self.db.get(&key) {
            Ok(Some(data)) => bincode::deserialize(&data).ok(), // Return deserialized data if successful
            Ok(None) => {
                log::warn!("No series info found for ID: {}", serie_id);
                None
            }
            Err(err) => {
                log::error!("Failed to retrieve series info for ID {}: {}", serie_id, err);
                None
            }
        }
    }
}
