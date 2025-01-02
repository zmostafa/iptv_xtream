use sled::Db;
use std::io;

#[derive(Debug, Clone)]
pub struct ImageCache {
    db: Db,
}

impl ImageCache {
    pub fn new(path: &str) -> Self {
        log::info!("[DB] Start Image Cache DB.");
        let db = sled::open(path).expect("Failed to open database");
        ImageCache { db }
    }

    pub fn save_image(&self, key: &str, image_data: &[u8]) -> io::Result<()> {
        log::info!("Storing image in cache: {}", key);
        self.db.insert(key, image_data)?;
        Ok(())
    }

    pub fn load_image(&self, key: &str) -> io::Result<Vec<u8>> {
        log::info!("Retrieving image from cache: {}", key);
        if let Some(image_data) = self.db.get(key)? {
            Ok(image_data.to_vec())
        } else {
            Ok(vec![])
        }
    }

    pub fn is_cached(&self, key: &str) -> bool {
        log::info!("Checking if image is cached: {}", key);
        self.db.contains_key(key).unwrap_or(false)
    }
}
