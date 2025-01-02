use crate::app::{AppView, IPTVApp};
use crate::db::ImageCache;
use crate::models::Movie;
use crate::models::SeriesInfo;
use arabic_reshaper::arabic_reshape;
use egui::FontFamily::{Monospace, Proportional};
use egui::TextBuffer;
use isahc::{AsyncReadResponseExt, HttpClient};
use rustybuzz::{Face, UnicodeBuffer};
use std::io::BufRead;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{fs, io};
use unicode_bidi::BidiInfo;

pub fn configure_fonts(ctx: &egui::Context) {
    log::info!("Configuring fonts.");

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "custom_arabic".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/Amiri-Regular.ttf")).into(),
    );

    fonts
        .families
        .entry(Proportional)
        .or_default()
        .insert(0, "custom_arabic".to_owned());
    fonts
        .families
        .entry(Monospace)
        .or_default()
        .insert(0, "custom_arabic".to_owned());

    ctx.set_fonts(fonts);
}

pub fn search_movies(app: &IPTVApp, query: &str) -> Vec<Movie> {
    let all_movies = app
        .db
        .get_movies_categories()
        .iter()
        .flat_map(|category| app.db.get_movies_streams(&category.category_id))
        .collect::<Vec<_>>();

    all_movies
        .into_iter()
        .filter(|movie| movie.name.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

pub fn search_series(app: &IPTVApp, query: &str) -> Vec<SeriesInfo> {
    let all_series = app
        .db
        .get_series_categories()
        .iter()
        .flat_map(|category| app.db.get_serie_info_for_all_series(&category.category_id))
        .collect::<Vec<_>>();

    all_series
        .into_iter()
        .filter(|serie| serie.name.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

pub fn preprocess_arabic_text_v1(input: &str) -> String {
    // 1. Perform bidirectional text processing
    let bidi_info = BidiInfo::new(input, None);
    let para = &bidi_info.paragraphs[0];
    let line = para.range.clone();
    let reordered = bidi_info.reorder_line(para, line.clone());

    arabic_reshape(reordered.as_str())
}

pub fn preprocess_arabic_text_v2(input: &str) -> String {
    // 1. Split the text using '-' as a delimiter
    let segments: Vec<&str> = input.split('-').collect();

    // 2. Load the font for shaping Arabic text
    let font_data = include_bytes!("/home/zmostafa/github/xtream/assets/Amiri-Regular.ttf");
    let face = Face::from_slice(font_data, 0).expect("Failed to create Rustybuzz Face");

    // 3. Process each segment
    let mut processed_segments = vec![];
    for segment in segments {
        // Check if the segment contains Arabic characters
        if segment.chars().any(|c| c >= '\u{0600}' && c <= '\u{06FF}') {
            // Process Arabic text: Shape and reorder
            let bidi_info = BidiInfo::new(segment, None);
            let para = &bidi_info.paragraphs[0];
            let reordered = bidi_info.reorder_line(para, para.range.clone());

            let mut unicode_buffer = UnicodeBuffer::new();
            unicode_buffer.push_str(&reordered);
            unicode_buffer.set_direction(rustybuzz::Direction::RightToLeft);

            let glyph_buffer = rustybuzz::shape(&face, &[], unicode_buffer);

            let mut shaped_text = String::new();
            for glyph in glyph_buffer.glyph_infos() {
                if let Some(ch) = char::from_u32(glyph.cluster) {
                    shaped_text.push(ch);
                }
            }
            // processed_segments.push(
            //     glyph_buffer
            //         .serialize(&face, SerializeFlags::default())
            //         .to_string(),
            // );
            processed_segments.push(shaped_text);
        } else {
            // Append English or non-Arabic segments as-is
            processed_segments.push(segment.to_string());
        }
    }

    // 4. Rejoin the segments with the separator
    processed_segments.join(" - ")
}

pub fn play_media(app: &mut IPTVApp, stream_id: &u32, stream_url: &str) {
    if let Some(local_path) = app.db.get_download_path(stream_id) {
        log::info!("Found media at {}", local_path);
        if std::path::Path::new(&local_path).exists() {
            log::info!("Playing local file: {}", local_path);
            app.current_view = AppView::Playback(local_path);
            return;
        }
    }
    log::info!("Playing online stream: {}", stream_url);
    app.current_view = AppView::Playback(stream_url.to_string());
}

pub async fn download_with_wget_async(
    url: &str,
    save_path: &str,
    progress: Arc<Mutex<f32>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Spawn the wget process
    let mut child = Command::new("wget")
        .arg("-O")
        .arg(save_path)
        .arg(url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Capture stderr for progress updates (wget outputs progress on stderr)
    if let Some(stderr) = child.stderr.take() {
        let reader = std::io::BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Some(line) = lines.next() {
            // Parse progress from wget's stderr output
            if let Some(progress_value) = parse_wget_progress(&line.unwrap()) {
                let mut progress_guard = progress.lock().unwrap();
                *progress_guard = progress_value;
            }

            // log::info!("wget: {:?}", line);
        }
    }

    // Wait for the process to finish
    let status = child.wait()?;
    if status.success() {
        log::info!("Download completed successfully: {}", save_path);
        Ok(())
    } else {
        Err(format!("wget failed with status: {}", status).into())
    }
}

// Helper function to parse wget progress from a line of output
pub fn parse_wget_progress(line: &str) -> Option<f32> {
    // Example wget progress line:
    // 108850K .......... .......... .......... .......... ..........  8%  785M 5m27s
    log::debug!("Parsing wget progress: {}", line);
    // Split the line into whitespace-separated segments
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts.iter() {
        if part.ends_with('%') {
            log::debug!("Found percentage: {}", part);
            if let Ok(value) = part.trim_end_matches('%').parse::<f32>() {
                return Some(value / 100.0); // Convert to 0.0-1.0 range
            }
        }
    }
    None
}

pub async fn fetch_and_cache_image(
    client: HttpClient,
    cache: ImageCache,
    image_url: &str,
) -> Result<(), String> {
    if cache.is_cached(image_url) {
        log::info!("Image already cached: {}", image_url);
        return Ok(());
    }

    log::info!("Downloading image: {}", image_url);

    match client.get_async(image_url).await {
        Ok(mut response) => {
            if response.status().is_success() {
                let image_data = response
                    .bytes()
                    .await
                    .map_err(|e| format!("Failed to read image data: {}", e))?;
                cache
                    .save_image(image_url, &image_data)
                    .map_err(|e| format!("Failed to save image to cache: {}", e))?;
                Ok(())
            } else {
                Err(format!("Unexpected status code: {}", response.status()))
            }
        }
        Err(err) => Err(format!("Failed to fetch image: {}", err)),
    }
}

pub fn remove_downloaded_episode(
    app: &mut IPTVApp,
    episode_id: &u32,
    download_path: &str,
) -> io::Result<()> {
    // Delete the file from the file system
    fs::remove_file(download_path)?;

    // Remove the episode's entry from the database
    app.db.remove_download_path(episode_id);

    Ok(())
}
