use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ImageCache {
    cache_dir: PathBuf,
}

impl ImageCache {
    pub fn new(cache_dir: &str) -> Self {
        let cache_path = Path::new(cache_dir);
        if !cache_path.exists() {
            fs::create_dir_all(cache_path).expect("Failed to create cache directory");
        }
        Self {
            cache_dir: cache_path.to_path_buf(),
        }
    }

    // Check if an image is already cached
    pub fn is_cached(&self, url: &str) -> bool {
        self.get_cache_path(url).exists()
    }

    // Get the cached file path for a URL
    pub fn get_cache_path(&self, url: &str) -> PathBuf {
        let hash = self.hash_url(url);
        self.cache_dir.join(hash)
    }

    // Save an image to the cache
    pub fn save_image(&self, url: &str, data: &[u8]) -> io::Result<()> {
        log::info!("Saving image to cache : {}", url);
        let cache_path = self.get_cache_path(url);
        let mut file = fs::File::create(cache_path)?;
        file.write_all(data)
    }

    // Load an image from the cache
    pub fn load_image(&self, url: &str) -> io::Result<Vec<u8>> {
        let cache_path = self.get_cache_path(url);
        fs::read(cache_path)
    }

    // Hash the URL to create a unique filename
    fn hash_url(&self, url: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(url);
        format!("{:x}", hasher.finalize())
    }
}
