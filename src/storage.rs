use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Default)]
pub struct Credentials {
    pub server: String,
    pub username: String,
    pub password: String,
}

pub fn save_credentials(creds: &Credentials) -> Result<(), Box<dyn std::error::Error>> {
    let dir = dirs::config_dir()
        .ok_or("Failed to find config directory")?
        .join("iptv_app");
    fs::create_dir_all(&dir)?;
    let file_path = dir.join("credentials.json");
    let serialized = serde_json::to_string(creds)?;
    fs::write(file_path, serialized)?;
    Ok(())
}

pub fn load_credentials() -> Option<Credentials> {
    let file_path = dirs::config_dir()?.join("iptv_app/credentials.json");
    let content = fs::read_to_string(file_path).ok()?;
    serde_json::from_str(&content).ok()
}
