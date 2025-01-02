use crate::app;
use crate::app::state::IPTVApp;
use eframe::egui;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub fn render_playback(app: &mut IPTVApp, ui: &mut egui::Ui, stream_url: String) {
    ui.heading("Now Playing:");
    ui.label(format!("Stream URL: {}", stream_url));

    // Socket path
    let temp_dir = std::env::current_dir().unwrap().join("iptv_cache");
    let socket_path = temp_dir.join("mpvsocket");

    if app.mpv_process.is_none() {
        app.mpv_process = Some(
            Command::new("mpv")
                .arg(&stream_url)
                .arg("--no-terminal")
                .arg("--force-window=yes")
                .arg(format!(
                    "--input-ipc-server={}",
                    socket_path.to_string_lossy()
                ))
                .arg("--resume-playback")
                .arg("--save-position-on-quit")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("Failed to start MPV"),
        );
    }

    if ui.button("Stop").clicked() {
        if let Some(mut process) = app.mpv_process.take() {
            // let _ = process.kill();
            let _ = futures::executor::block_on(send_quit_command(&socket_path.to_string_lossy()));
            if let Err(e) = process.wait() {
                log::error!("Failed to wait for the process: {}", e);
            }
        }

        if let Some(view) = app.view_stack.pop() {
            app.current_view = view;
        }
    }
}
pub fn create_playlist_file(playlist: &[String]) -> Result<PathBuf, std::io::Error> {
    // Create a temporary file
    let temp_dir = std::env::current_dir().unwrap().join("iptv_cache");
    let playlist_path = temp_dir.join("playlist.txt");

    // Write the playlist URLs to the file
    let mut file = File::create(&playlist_path)?;
    for url in playlist {
        writeln!(file, "{}", url)?;
    }

    Ok(playlist_path)
}

pub async fn render_playlist(app: &mut IPTVApp, ui: &mut egui::Ui, playlist: &[String]) {
    ui.heading("Now Playing:");
    if playlist.is_empty() {
        return;
    }

    // Create a playlist file
    let playlist_path = match create_playlist_file(playlist) {
        Ok(path) => {
            log::info!("Playlist file created at: {:?}", &path);
            path
        }
        Err(e) => {
            log::error!("Failed to create playlist file: {}", e);
            return;
        }
    };

    // Query the currently playing episode
    let temp_dir = std::env::current_dir().unwrap().join("iptv_cache");
    let socket_path = temp_dir.join("mpvsocket");

    // Start mpv with the playlist file
    if app.mpv_process.is_none() {
        app.mpv_process = Some(
            Command::new("mpv")
                .arg(format!("--playlist={}", playlist_path.to_string_lossy()))
                .arg("--no-terminal")
                .arg("--force-window=yes")
                .arg(format!(
                    "--input-ipc-server={}",
                    socket_path.to_string_lossy()
                ))
                .arg("--resume-playback")
                .arg("--save-position-on-quit")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("Failed to start MPV"),
        );
    }

    // let (tx, mut rx) = mpsc::channel(32);
    // tokio::spawn(async move {
    //     match get_currently_playing_episode(&socket_path.to_string_lossy()).await {
    //         Ok(Some(episode)) => {
    //             log::info!("Currently playing: {}", episode);
    //             if let Err(_) = tx.send(Some(episode)).await {
    //                 log::error!("Failed to send episode to main thread");
    //             }
    //         }
    //         Ok(None) => {
    //             log::info!("No file is currently playing.");
    //             tx.send(Some("None".to_string())).await.unwrap();
    //         }
    //         Err(e) => {
    //             log::error!("Error: {}", e);
    //             tx.send(Some("None".to_string())).await.unwrap();
    //         }
    //     }
    //     // Add a delay to avoid busy-waiting
    //     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // });

    // if let Some(episode) = rx.recv().await {
    //     let episode_clone = episode.clone();
    //     if episode_clone.unwrap() != "None".to_string() {
    //         log::info!(
    //             "Episode: {} is added to watch list",
    //             episode.clone().unwrap()
    //         );
    //         app.db
    //             .save_watched(&episode.unwrap().parse::<u32>().unwrap());
    //     }
    // }

    // Store the process in the app state
    // app.mpv_process = Some(process);

    if ui.button("Stop").clicked() {
        if let Some(mut process) = app.mpv_process.take() {
            // let _ = process.kill();
            let _ = futures::executor::block_on(send_quit_command(&socket_path.to_string_lossy()));
            if let Err(e) = process.wait() {
                log::error!("Failed to wait for the process: {}", e);
            }
        }

        if let Some(view) = app.view_stack.pop() {
            app.current_view = view;
        }
    }
}

use regex::Regex;
use serde_json::json;
use serde_json::Value;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

async fn send_quit_command(
    socket_path: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    // Connect to the mpv IPC socket
    let mut stream = match UnixStream::connect(socket_path).await {
        Ok(stream) => stream,
        Err(e) => {
            log::error!("Failed to connect to MPV socket: {}", e);
            return Ok(None);
        }
    };

    // Create the JSON quit command
    let quit_command = json!({
        "command": ["quit"]
    });

    // Send the command to mpv
    stream
        .write_all(quit_command.to_string().as_bytes())
        .await?;
    stream.write_all(b"\n").await?;

    // Wait for a response (optional)
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await?;
    println!("MPV response: {}", String::from_utf8_lossy(&buffer));
    Ok(Some("Quit command sent".to_string()))
}

async fn get_currently_playing_episode(
    ipc_socket_path: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    log::info!(
        "Getting currently playing episode... Socket path: {}",
        ipc_socket_path
    );
    let mut stream = match UnixStream::connect(ipc_socket_path).await {
        Ok(stream) => stream,
        Err(e) => {
            log::error!("Failed to connect to MPV socket: {}", e);
            return Ok(None);
        }
    };

    // Send the command to get the currently playing file
    let command = json!({
        "command": ["get_property", "path"]
    });
    let command_str = command.to_string() + "\n";
    stream.write_all(command_str.as_bytes()).await?;

    // Read the response into a buffer
    log::info!("Reading response...");
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    // Parse the response
    // let response_json: serde_json::Value = serde_json::from_str(&response)?;
    // Ok(response_json["data"].as_str().map(|s| s.to_string()))
    // Parse the response
    // Parse the response
    let response_json: Value = serde_json::from_str(&response)?;
    if let Some(url) = response_json["data"].as_str() {
        // Extract the episode ID from the URL using a regular expression
        let re = Regex::new(r"/(\d+)\.\w+$").unwrap();
        if let Some(captures) = re.captures(url) {
            if let Some(episode_id) = captures.get(1) {
                return Ok(Some(episode_id.as_str().to_string()));
            }
        }
    }

    Ok(None)
}
