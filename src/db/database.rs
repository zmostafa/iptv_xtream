use crate::models::live::{Category, LiveStream};
use crate::models::movies::Movie;
use crate::models::series::{Series, SeriesInfo};
use serde::{Deserialize, Serialize};
use serde_json;
use sled::Db;

#[derive(Clone, Debug)]
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
        self.db
            .get(key)
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_slice(&v).ok())
    }

    pub fn save_live_categories(&self, categories: Vec<Category>) {
        log::info!("[DB] Save Live Categories\n.");
        let key = "live_categories";
        let serialized = bincode::serialize(&categories).expect("Failed to serialize categories");
        self.db
            .insert(key, serialized)
            .expect("Failed to save categories");
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

    pub fn save_movies_categories(&self, categories: Vec<Category>) {
        log::info!("[DB] Save Movies Categories.");
        let key = "movies_categories";
        let serialized = bincode::serialize(&categories).expect("Failed to serialize categories");
        self.db
            .insert(key, serialized)
            .expect("Failed to save categories");
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

    pub fn save_series_categories(&self, categories: Vec<Category>) {
        log::info!("[DB] Save Series Categories.");
        let key = "series_categories";
        let serialized = bincode::serialize(&categories).expect("Failed to serialize categories");
        self.db
            .insert(key, serialized)
            .expect("Failed to save categories");
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

    pub fn save_series_info(&self, category_id: &str, serie_id: &i64, series: &Series) {
        let key = format!("serie_info_{}_{}", category_id, serie_id);
        let serialized = bincode::serialize(series).expect("Failed to serialize series info");
        self.db
            .insert(key, serialized)
            .expect("Failed to save series info");
        log::info!("[DB] Save Series Info.");
    }

    pub fn get_series_info(&self, category_id: &str, serie_id: &i64) -> Option<Series> {
        log::info!("[DB] Get Series Info.");
        let key = format!("serie_info_{}_{}", category_id, serie_id);
        match self.db.get(&key) {
            Ok(Some(data)) => bincode::deserialize(&data).ok(), // Return deserialized data if successful
            Ok(None) => {
                log::warn!("No series info found for ID: {}", serie_id);
                None
            }
            Err(err) => {
                log::error!(
                    "Failed to retrieve series info for ID {}: {}",
                    serie_id,
                    err
                );
                None
            }
        }
    }

    pub fn save_individual_live_stream(&self, category_id: &str, stream: &LiveStream) {
        let key = format!("live_stream_{}_{}", category_id, stream.stream_id);
        let serialized = bincode::serialize(&stream).expect("Failed to serialize live stream");
        self.db
            .insert(key, serialized)
            .expect("Failed to save live stream");
        log::info!(
            "Saved live stream {} under category {}",
            stream.name,
            category_id
        );
    }

    pub fn get_live_streams(&self, category_id: &str) -> Vec<LiveStream> {
        log::info!("[DB] Get Live Streams for category {}", category_id);
        let prefix = format!("live_stream_{}_", category_id);

        self.db
            .scan_prefix(prefix)
            .filter_map(|item| {
                if let Ok((_, value)) = item {
                    bincode::deserialize::<LiveStream>(&value).ok()
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn save_individual_movie(&self, category_id: &str, movie: &Movie) {
        let key = format!("movie_{}_{}", category_id, movie.stream_id);
        let serialized = bincode::serialize(movie).expect("Failed to serialize movie");
        self.db
            .insert(key, serialized)
            .expect("Failed to save movie");
        log::info!("Saved movie {} under category {}", movie.name, category_id);
    }

    pub fn get_movies_streams(&self, category_id: &str) -> Vec<Movie> {
        log::info!("[DB] Get Movies Streams for category {}", category_id);
        let prefix = format!("movie_{}_", category_id);

        self.db
            .scan_prefix(prefix)
            .filter_map(|item| {
                if let Ok((_, value)) = item {
                    bincode::deserialize::<Movie>(&value).ok()
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn save_serie_info_for_all_series(&self, category_id: &str, serie: &SeriesInfo) {
        let key = format!("serie_{}_{:?}", category_id, serie.series_id);
        let serialized = bincode::serialize(serie).expect("Failed to serialize movie");
        self.db
            .insert(key, serialized)
            .expect("Failed to save movie");
        log::info!("Saved movie {} under category {}", serie.name, category_id);
    }

    pub fn get_serie_info_for_all_series(&self, category_id: &str) -> Vec<SeriesInfo> {
        log::info!("[DB] Get Series Streams for category {}", category_id);
        let prefix = format!("serie_{}_", category_id);

        self.db
            .scan_prefix(prefix)
            .filter_map(|item| {
                if let Ok((_, value)) = item {
                    bincode::deserialize::<SeriesInfo>(&value).ok()
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn save_download(&self, stream_id: u32, file_path: &str) {
        let key = format!("download_{}", stream_id);
        self.save(&key, &file_path);
    }

    pub fn get_download_path(&self, stream_id: &u32) -> Option<String> {
        let key = format!("download_{}", stream_id);
        self.get(&key)
    }

    pub fn remove_download_path(&self, stream_id: &u32) -> Option<String> {
        let key = format!("download_{}", stream_id);
        self.db.remove(&key).ok().flatten().and_then(|v| serde_json::from_slice(&v).ok())
    }

    pub fn is_downloaded(&self, stream_id: &u32) -> Option<String> {
        let key = format!("download_{}", stream_id);
        self.get::<String>(&key)
    }

    pub fn save_watched(&self, stream_id: &u32) {
        let key = format!("watched_{}", stream_id);
        self.save(&key, &true);
    }

    pub fn is_watched(&self, stream_id: &u32) -> Option<bool> {
        let key = format!("watched_{}", stream_id);
        self.get::<bool>(&key)
    }
}
