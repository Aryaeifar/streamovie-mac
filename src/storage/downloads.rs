//! Episode / source file download manager.
//!
//! Saves progressive media URLs under the app data `downloads/` directory with
//! per-episode progress tracking. Never reports success for failed or stream-only URLs.

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context as _, Result};

use crate::storage::app_data_dir;

/// Lifecycle of a single download keyed by episode / source identity.
#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub enum DownloadStatus {
    #[default]
    Idle,
    Downloading { progress: f32 },
    Done { path: PathBuf },
    Failed { message: String },
}

#[derive(Clone, Debug, Default)]
struct DownloadStateMap {
    by_key: HashMap<String, DownloadStatus>,
}

/// Thread-safe download coordinator shared with background workers.
#[derive(Clone, Debug, Default)]
pub struct DownloadManager {
    inner: Arc<Mutex<DownloadStateMap>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn downloads_dir() -> PathBuf {
        let dir = app_data_dir().join("downloads");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn status(&self, key: &str) -> DownloadStatus {
        self.inner
            .lock()
            .ok()
            .and_then(|guard| guard.by_key.get(key).cloned())
            .unwrap_or_default()
    }

    pub fn is_in_flight(&self, key: &str) -> bool {
        matches!(self.status(key), DownloadStatus::Downloading { .. })
    }

    fn set_status(&self, key: &str, status: DownloadStatus) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.by_key.insert(key.to_string(), status);
        }
    }

    /// Start downloading `url` for `key` if no in-flight job exists for that key.
    /// Returns `Err` immediately when the URL looks like HLS / non-file stream.
    pub fn start_download(
        &self,
        key: impl Into<String>,
        url: impl Into<String>,
        suggested_name: impl Into<String>,
    ) -> Result<()> {
        let key = key.into();
        let url = url.into();
        let suggested_name = suggested_name.into();

        if self.is_in_flight(&key) {
            return Err(anyhow!("A download for this episode is already in progress."));
        }

        let lower = url.to_ascii_lowercase();
        if lower.contains(".m3u8") {
            let msg = "This source is an HLS stream (.m3u8) and cannot be saved as a file download.";
            self.set_status(
                &key,
                DownloadStatus::Failed {
                    message: msg.to_string(),
                },
            );
            return Err(anyhow!(msg));
        }

        self.set_status(&key, DownloadStatus::Downloading { progress: 0.0 });

        let manager = self.clone();
        let key_bg = key.clone();
        std::thread::Builder::new()
            .name(format!("streamovie-dl-{key}"))
            .spawn(move || {
                let result = download_file(&manager, &key_bg, &url, &suggested_name);
                match result {
                    Ok(path) => manager.set_status(&key_bg, DownloadStatus::Done { path }),
                    Err(err) => manager.set_status(
                        &key_bg,
                        DownloadStatus::Failed {
                            message: err.to_string(),
                        },
                    ),
                }
            })
            .context("Failed to spawn download thread")?;

        Ok(())
    }
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        "streamovie-download".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn extension_from_url(url: &str) -> &'static str {
    let lower = url.to_ascii_lowercase();
    let path = lower.split('?').next().unwrap_or(&lower);
    if path.ends_with(".webm") {
        "webm"
    } else if path.ends_with(".ogg") || path.ends_with(".ogv") {
        "ogg"
    } else {
        "mp4"
    }
}

fn download_file(
    manager: &DownloadManager,
    key: &str,
    url: &str,
    suggested_name: &str,
) -> Result<PathBuf> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Streamovie-macOS/0.1.0")
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .context("Failed to build HTTP client")?;

    let mut response = client
        .get(url)
        .send()
        .context("Failed to start download request")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Download failed with HTTP status {}.",
            response.status()
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    if content_type.contains("mpegurl")
        || content_type.contains("application/vnd.apple.mpegurl")
        || content_type.contains("application/x-mpegurl")
    {
        return Err(anyhow!(
            "Server returned an HLS playlist, which cannot be saved as a single video file."
        ));
    }

    let total = response.content_length();
    let ext = extension_from_url(url);
    let filename = format!("{}.{}", sanitize_filename(suggested_name), ext);
    let dest = DownloadManager::downloads_dir().join(filename);
    let tmp = dest.with_extension(format!("{ext}.partial"));

    let mut file = File::create(&tmp).context("Failed to create download file")?;
    let mut written: u64 = 0;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let n = {
            use std::io::Read;
            response
                .read(&mut buffer)
                .context("Failed while reading download stream")?
        };
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])
            .context("Failed while writing download file")?;
        written += n as u64;
        let progress = match total {
            Some(t) if t > 0 => (written as f32 / t as f32).clamp(0.0, 1.0),
            _ => 0.0,
        };
        manager.set_status(key, DownloadStatus::Downloading { progress });
    }

    file.flush().ok();
    drop(file);

    if written == 0 {
        let _ = fs::remove_file(&tmp);
        return Err(anyhow!(
            "Download produced an empty file. The URL may be stream-only or unavailable."
        ));
    }

    fs::rename(&tmp, &dest).context("Failed to finalize download")?;
    Ok(dest)
}
