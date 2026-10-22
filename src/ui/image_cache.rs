//! Asynchronous remote image loader, magic-byte format detector, and two-tier
//! (in-memory + local disk) cache for GPUI.
//!
//! Solves GPUI's remote image loading issues by:
//! 1. Fetching images on `cx.background_executor()` via `reqwest::blocking::Client`
//!    (which manages its own internal runtime and never fails with "no Tokio reactor").
//! 2. Sending browser-compatible `User-Agent` and `Accept` headers and following redirects.
//! 3. Retrying relative or Streamovie mirror paths across all configured catalog hosts,
//!    plus supporting an optional fallback image URL (e.g. falling back from an external
//!    wallpaper URL to the catalog's own cover/poster URL).
//! 4. Detecting `gpui_kit::ImageFormat` (`Jpeg`, `Png`, `Webp`, `Gif`, `Bmp`, `Svg`, `Tiff`, `Ico`)
//!    directly from magic bytes so `.webp` images served under `.jpg` URLs decode properly.
//! 5. Rendering via `img(Arc<Image>).absolute().inset_0().size_full().object_fit(ObjectFit::Cover)`
//!    inside a `.relative().flex().overflow_hidden()` container so Taffy always gives the
//!    image definite non-zero bounds.

use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use gpui_kit::{
    AnyElement, Context, Image, ImageFormat, IntoElement, ObjectFit, ParentElement as _, Pixels,
    Styled as _, StyledImage as _, div, img, prelude::FluentBuilder as _,
};
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, Sizable as _, h_flex, v_flex};
use reqwest::blocking::Client;

use crate::{app::StreamovieApp, config::DEFAULT_API_HOSTS, storage::app_data_dir};

const IMAGE_TIMEOUT: Duration = Duration::from_secs(20);
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";

#[derive(Clone)]
pub struct ImageCache {
    inner: Arc<Mutex<ImageCacheInner>>,
    http: Client,
    cache_dir: PathBuf,
}

struct ImageCacheInner {
    memory: HashMap<String, Arc<Image>>,
    in_flight: HashSet<String>,
    failed: HashSet<String>,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageCache {
    pub fn new() -> Self {
        let cache_dir = app_data_dir().join("image-cache");
        let _ = fs::create_dir_all(&cache_dir);

        let http = Client::builder()
            .timeout(IMAGE_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to construct image HTTP client");

        Self {
            inner: Arc::new(Mutex::new(ImageCacheInner {
                memory: HashMap::new(),
                in_flight: HashSet::new(),
                failed: HashSet::new(),
            })),
            http,
            cache_dir,
        }
    }

    /// Resolves an image from memory/disk cache, or spawns a background download task
    /// that notifies `StreamovieApp` as soon as the image is ready.
    pub fn get_or_load(
        &self,
        primary_url: &str,
        fallback_url: Option<&str>,
        cx: &mut Context<StreamovieApp>,
    ) -> Option<Arc<Image>> {
        let primary = primary_url.trim();
        let fallback = fallback_url.map(str::trim).filter(|s| !s.is_empty());

        let cache_key = if !primary.is_empty() {
            primary.to_string()
        } else if let Some(fb) = fallback {
            fb.to_string()
        } else {
            return None;
        };

        // 1. Check in-memory cache or in-flight/failed status
        {
            let mut guard = self.inner.lock().ok()?;
            if let Some(img) = guard.memory.get(&cache_key) {
                return Some(img.clone());
            }
            if let Some(fb) = fallback {
                if let Some(img) = guard.memory.get(fb) {
                    return Some(img.clone());
                }
            }
            if guard.in_flight.contains(&cache_key) || guard.failed.contains(&cache_key) {
                return None;
            }
            guard.in_flight.insert(cache_key.clone());
        }

        // 2. Spawn background disk check + HTTP fetch task
        let cache = self.clone();
        let bg_cache = cache.clone();
        let primary_owned = primary.to_string();
        let fallback_owned = fallback.map(str::to_string);
        let key_owned = cache_key.clone();

        cx.spawn(async move |this, cx| {
            let fetched = cx
                .background_executor()
                .spawn(async move {
                    bg_cache.load_from_disk_or_network(
                        &key_owned,
                        &primary_owned,
                        fallback_owned.as_deref(),
                    )
                })
                .await;

            if let Ok(mut guard) = cache.inner.lock() {
                guard.in_flight.remove(&cache_key);
                match fetched {
                    Some(arc_img) => {
                        guard.memory.insert(cache_key, arc_img);
                    }
                    None => {
                        guard.failed.insert(cache_key);
                    }
                }
            }

            let _ = this.update(cx, |_, cx| {
                cx.notify();
            });
        })
        .detach();

        None
    }

    fn disk_path_for(&self, key: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        self.cache_dir.join(format!("{hash:016x}.img"))
    }

    fn load_from_disk_or_network(
        &self,
        cache_key: &str,
        primary_url: &str,
        fallback_url: Option<&str>,
    ) -> Option<Arc<Image>> {
        let disk_path = self.disk_path_for(cache_key);

        // Try local disk cache first
        if let Ok(bytes) = fs::read(&disk_path) {
            if let Some(format) = detect_image_format(&bytes, primary_url) {
                return Some(Arc::new(Image::from_bytes(format, bytes)));
            }
        }

        // Build candidate URLs (primary + mirror host variants + fallback + fallback mirrors)
        let mut candidates = Vec::new();
        expand_candidate_urls(primary_url, &mut candidates);
        if let Some(fb) = fallback_url {
            if fb != primary_url {
                expand_candidate_urls(fb, &mut candidates);
            }
        }

        for url in candidates {
            if let Some((bytes, format)) = self.download_one(&url) {
                let _ = fs::write(&disk_path, &bytes);
                return Some(Arc::new(Image::from_bytes(format, bytes)));
            }
        }

        None
    }

    fn download_one(&self, url: &str) -> Option<(Vec<u8>, ImageFormat)> {
        let response = self
            .http
            .get(url)
            .header(
                "Accept",
                "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
            )
            .send()
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let bytes = response.bytes().ok()?.to_vec();
        if bytes.len() < 16 {
            return None;
        }

        let format = detect_image_format(&bytes, url)?;
        Some((bytes, format))
    }
}

/// Expands a single image URL into candidate URLs across Streamovie's mirror hosts
/// (`server-hi-speed-iran.info`, `hostinnegar.com`, `windowsdiba.info`) when applicable.
fn expand_candidate_urls(raw_url: &str, out: &mut Vec<String>) {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return;
    }

    if trimmed.starts_with('/') {
        for host in DEFAULT_API_HOSTS {
            out.push(format!("{host}{trimmed}"));
        }
        return;
    }

    out.push(trimmed.to_string());

    // If the URL belongs to one of the Streamovie catalog domains, add the other mirrors as fallbacks
    for host in DEFAULT_API_HOSTS {
        if let Some(rest) = trimmed.strip_prefix(host) {
            for alt_host in DEFAULT_API_HOSTS {
                if alt_host != host {
                    out.push(format!("{alt_host}{rest}"));
                }
            }
            break;
        }
    }
}

/// Detects `gpui_kit::ImageFormat` from magic bytes first, then URL extension fallback.
pub fn detect_image_format(bytes: &[u8], url: &str) -> Option<ImageFormat> {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);

    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageFormat::Png);
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some(ImageFormat::Jpeg);
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return Some(ImageFormat::Webp);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(ImageFormat::Gif);
    }
    if bytes.starts_with(b"BM") {
        return Some(ImageFormat::Bmp);
    }
    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return Some(ImageFormat::Tiff);
    }
    if bytes.starts_with(b"\0\0\x01\0") || bytes.starts_with(b"\0\0\x02\0") {
        return Some(ImageFormat::Ico);
    }

    if let Ok(prefix) = std::str::from_utf8(&bytes[..bytes.len().min(256)]) {
        let trimmed = prefix.trim_start();
        if trimmed.starts_with("<svg") || trimmed.starts_with("<?xml") {
            return Some(ImageFormat::Svg);
        }
        // Guard against HTML error pages disguised as 200 OK
        if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
            return None;
        }
    }

    // Extension fallback if magic bytes are non-standard
    let clean_url = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    if clean_url.ends_with(".jpg") || clean_url.ends_with(".jpeg") {
        Some(ImageFormat::Jpeg)
    } else if clean_url.ends_with(".png") {
        Some(ImageFormat::Png)
    } else if clean_url.ends_with(".webp") {
        Some(ImageFormat::Webp)
    } else if clean_url.ends_with(".gif") {
        Some(ImageFormat::Gif)
    } else if clean_url.ends_with(".svg") {
        Some(ImageFormat::Svg)
    } else {
        None
    }
}

/// Renders a remote image inside a `.relative().flex().size_full().overflow_hidden()` container
/// with `.absolute().inset_0().size_full().object_fit(ObjectFit::Cover)` so GPUI/Taffy
/// always paints the image across the full bounds of its parent slot.
pub fn render_remote_image(
    app: &StreamovieApp,
    primary_url: Option<&str>,
    fallback_url: Option<&str>,
    radius: Pixels,
    fallback_icon: IconName,
    fallback_label: Option<&str>,
    cx: &mut Context<StreamovieApp>,
) -> AnyElement {
    let resolved = primary_url
        .filter(|s| !s.trim().is_empty())
        .and_then(|url| app.resolve_image(url, fallback_url, cx))
        .or_else(|| {
            fallback_url
                .filter(|s| !s.trim().is_empty())
                .and_then(|fb| app.resolve_image(fb, None, cx))
        });

    div()
        .relative()
        .flex()
        .size_full()
        .rounded(radius)
        .overflow_hidden()
        .bg(cx.theme().muted.opacity(0.35))
        .map(|slot| match resolved {
            Some(arc_image) => slot.child(
                img(arc_image)
                    .absolute()
                    .inset_0()
                    .size_full()
                    .rounded(radius)
                    .object_fit(ObjectFit::Cover),
            ),
            None => slot.child(
                v_flex()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .text_color(cx.theme().muted_foreground.opacity(0.55))
                    .child(Icon::new(fallback_icon).large())
                    .when_some(fallback_label, |col, text| {
                        col.child(
                            h_flex()
                                .px_3()
                                .text_xs()
                                .text_center()
                                .child(text.to_string()),
                        )
                    }),
            ),
        })
        .into_any_element()
}
