//! Asynchronous episode downloads for direct media files.
//!
//! Playlists (HLS) and ordinary web pages are not treated as downloadable files.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadPhase {
    Running,
    Finished,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct DownloadSnapshot {
    pub key: String,
    #[allow(dead_code)]
    pub title: String,
    pub received: u64,
    pub total: Option<u64>,
    pub phase: DownloadPhase,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
}

struct DownloadShared {
    snapshot: Mutex<DownloadSnapshot>,
    cancel: AtomicBool,
}

#[derive(Clone)]
pub struct DownloadHandle {
    inner: Arc<DownloadShared>,
}

impl DownloadHandle {
    pub fn snapshot(&self) -> DownloadSnapshot {
        self.inner
            .snapshot
            .lock()
            .expect("download snapshot")
            .clone()
    }

    pub fn is_running(&self) -> bool {
        self.snapshot().phase == DownloadPhase::Running
    }

    pub fn cancel(&self) {
        self.inner.cancel.store(true, Ordering::Relaxed);
    }

    pub fn key(&self) -> String {
        self.snapshot().key
    }
}

pub fn downloads_dir() -> PathBuf {
    let dir = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Streamovie");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn reveal_path(path: &Path) {
    let _ = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn();
}

pub fn start_download(
    key: String,
    title: String,
    url: String,
    extension: &str,
) -> Result<DownloadHandle> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(anyhow!("Only direct http(s) media URLs can be downloaded."));
    }

    let file_name = format!("{}.{}", sanitize_file_name(&title), extension);
    let path = unique_path(&downloads_dir().join(file_name));
    let snapshot = DownloadSnapshot {
        key,
        title,
        received: 0,
        total: None,
        phase: DownloadPhase::Running,
        path: Some(path.clone()),
        error: None,
    };
    let inner = Arc::new(DownloadShared {
        snapshot: Mutex::new(snapshot),
        cancel: AtomicBool::new(false),
    });
    let handle = DownloadHandle {
        inner: inner.clone(),
    };
    let download_url = url;

    std::thread::Builder::new()
        .name("streamovie-download".into())
        .spawn(move || {
            let result = run_download(&inner, &download_url, &path);
            let mut snap = inner.snapshot.lock().expect("download snapshot");
            if inner.cancel.load(Ordering::Relaxed) {
                snap.phase = DownloadPhase::Cancelled;
                drop(snap);
                let _ = fs::remove_file(&path);
                return;
            }
            match result {
                Ok(()) => snap.phase = DownloadPhase::Finished,
                Err(err) => {
                    snap.phase = DownloadPhase::Failed;
                    snap.error = Some(err.to_string());
                    let _ = fs::remove_file(&path);
                    snap.path = None;
                }
            }
        })
        .context("failed to start download thread")?;

    Ok(handle)
}

fn run_download(shared: &DownloadShared, url: &str, path: &Path) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(None)
        .user_agent("Streamovie-macOS/0.1.0")
        .build()
        .context("failed to build download client")?;

    let mut response = client.get(url).send().context("download request failed")?;
    if !response.status().is_success() {
        return Err(anyhow!("Download failed with HTTP {}", response.status()));
    }

    let total = response.content_length();
    {
        let mut snap = shared.snapshot.lock().expect("download snapshot");
        snap.total = total;
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    if content_type.contains("text/html") || content_type.contains("application/xhtml") {
        return Err(anyhow!(
            "This address is a web page, not a direct media file."
        ));
    }

    let mut file = File::create(path).context("could not create the download file")?;
    let mut buffer = [0_u8; 64 * 1024];
    let mut received = 0_u64;

    loop {
        if shared.cancel.load(Ordering::Relaxed) {
            return Err(anyhow!("cancelled"));
        }
        let read = response.read(&mut buffer).context("download interrupted")?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .context("could not write the download")?;
        received += read as u64;
        let mut snap = shared.snapshot.lock().expect("download snapshot");
        snap.received = received;
    }

    file.flush().ok();
    if let Some(total) = total {
        if received > 0 && received + 1024 < total {
            return Err(anyhow!("Download ended before the full file arrived."));
        }
    }
    if received == 0 {
        return Err(anyhow!("The server returned an empty file."));
    }
    Ok(())
}

fn sanitize_file_name(title: &str) -> String {
    let mut cleaned: String = title
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            other if other.is_control() => '-',
            other => other,
        })
        .collect();
    cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        "Streamovie episode".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn unique_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("episode");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("mp4");
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    for index in 2..1000 {
        let candidate = parent.join(format!("{stem} ({index}).{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}
