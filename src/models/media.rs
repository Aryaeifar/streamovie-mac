//! Media domain models and parsing utilities ported directly from
//! `Streamovie/src/lib/api/types.ts`, `src/lib/media.ts`, and `src/lib/media-exclusions.ts`.
#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Media category (`movie` or `series`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaCategory {
    #[default]
    Movie,
    Series,
}

impl MediaCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Series => "series",
        }
    }
}

/// Catalog sort order (`created` or `imdb`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowseSort {
    #[default]
    Created,
    Imdb,
}

impl BrowseSort {
    pub fn as_api_param(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Imdb => "imdb",
        }
    }
}

/// Raw catalog media item returned by the Streamovie API.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiMedia {
    #[serde(default)]
    pub id: u64,
    pub title: Option<String>,
    pub name: Option<String>,
    pub poster: Option<String>,
    pub poster_url: Option<String>,
    pub image: Option<String>,
    pub thumbnail: Option<String>,
    pub cover: Option<String>,
    pub backdrop: Option<String>,
    pub backdrop_url: Option<String>,
    pub banner: Option<String>,
    pub wallpaper: Option<String>,
    pub year: Option<Value>,
    pub rate: Option<Value>,
    pub rating: Option<Value>,
    pub imdb: Option<Value>,
    pub duration: Option<String>,
    pub runtime: Option<String>,
    pub classification: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub genres: Option<Value>,
    pub genre: Option<Value>,
    pub country: Option<Value>,
    pub countries: Option<Value>,
    pub sources: Option<Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Season entry returned by `/api/season/by/serie/{seriesId}/{key}/`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiSeason {
    #[serde(default)]
    pub id: u64,
    pub title: Option<String>,
    pub name: Option<String>,
    pub episodes: Option<Value>,
    pub parts: Option<Value>,
    pub items: Option<Value>,
    pub sources: Option<Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Genre item returned by `/api/genre/all/{key}/`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiGenre {
    #[serde(default)]
    pub id: u64,
    pub title: Option<String>,
    pub name: Option<String>,
}

impl ApiGenre {
    pub fn display_name(&self) -> String {
        first_non_empty_str(&[self.title.as_deref(), self.name.as_deref()])
            .unwrap_or_else(|| format!("#{}", self.id))
    }
}

/// Country item returned by `/api/country/all/{key}/`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiCountry {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub image: String,
}

/// Wrapper returned by `/api/search/{query}/{key}/`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub posters: Vec<ApiMedia>,
}

/// Stream or download source link attached to a movie or episode.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLink {
    pub id: usize,
    pub quality: String,
    pub format_type: Option<String>,
    pub url: String,
    pub is_trailer: bool,
}

impl SourceLink {
    /// Checks whether the source points to a direct video stream (`mp4`, `webm`, `ogg`, `ogv`).
    pub fn is_playable_video(&self) -> bool {
        if self.is_trailer {
            return false;
        }
        let lower_url = self.url.to_ascii_lowercase();
        let has_video_ext = [".mp4", ".webm", ".ogg", ".ogv"]
            .iter()
            .any(|ext| lower_url.ends_with(ext) || lower_url.contains(&format!("{ext}?")));
        let combined = format!(
            "{} {}",
            self.format_type.as_deref().unwrap_or_default(),
            self.quality
        )
        .to_ascii_lowercase();
        has_video_ext
            || ["mp4", "webm", "ogg", "ogv"]
                .iter()
                .any(|fmt| combined.contains(fmt))
    }

    /// HLS playlist. WKWebView can play these, but they are not a single downloadable file.
    pub fn is_hls(&self) -> bool {
        url_path_lower(&self.url).contains(".m3u8")
    }

    /// A single media file the app can save without treating a playlist or web page as a download.
    pub fn is_direct_download(&self) -> bool {
        if self.is_trailer || self.is_hls() {
            return false;
        }
        let path = url_path_lower(&self.url);
        [".mp4", ".m4v", ".mov", ".webm", ".mkv", ".ogg", ".ogv"]
            .iter()
            .any(|ext| path.ends_with(ext))
    }

    /// Online playback candidate for the in-app player (progressive file or HLS).
    pub fn is_streamable(&self) -> bool {
        if self.is_trailer {
            return false;
        }
        self.is_playable_video() || self.is_hls() || self.is_direct_download()
    }

    pub fn file_extension(&self) -> &'static str {
        let path = url_path_lower(&self.url);
        for ext in [
            ".mp4", ".m4v", ".mov", ".webm", ".mkv", ".ogg", ".ogv", ".m3u8",
        ] {
            if path.ends_with(ext) {
                return match ext {
                    ".m4v" => "m4v",
                    ".mov" => "mov",
                    ".webm" => "webm",
                    ".mkv" => "mkv",
                    ".ogg" => "ogg",
                    ".ogv" => "ogv",
                    ".m3u8" => "m3u8",
                    _ => "mp4",
                };
            }
        }
        "mp4"
    }
}

fn url_path_lower(url: &str) -> String {
    url.split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase()
}

fn quality_rank(quality: &str) -> i32 {
    let label = quality.to_ascii_lowercase();
    if label.contains("2160") || label.contains("4k") {
        50
    } else if label.contains("1440") {
        40
    } else if label.contains("1080") {
        30
    } else if label.contains("720") {
        20
    } else if label.contains("480") {
        10
    } else {
        0
    }
}

/// Highest-quality source the in-app player can attempt.
pub fn best_stream_source(sources: &[SourceLink]) -> Option<&SourceLink> {
    sources
        .iter()
        .filter(|source| source.is_streamable())
        .max_by_key(|source| quality_rank(&source.quality))
}

/// Highest-quality direct file suitable for a local download.
pub fn best_download_source(sources: &[SourceLink]) -> Option<&SourceLink> {
    sources
        .iter()
        .filter(|source| source.is_direct_download())
        .max_by_key(|source| quality_rank(&source.quality))
}

/// Structured key-value block parsed from Persian catalog descriptions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptionBlock {
    pub label: String,
    pub value: String,
}

/// Parsed overview and structured fields extracted from `ApiMedia::description`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedDescription {
    pub overview: Option<String>,
    pub fields: Vec<DescriptionBlock>,
}

/// Normalized episode item within a season.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    pub key: String,
    pub id: Option<u64>,
    pub title: String,
    pub sources: Vec<SourceLink>,
}

/// Spotlight hero item with curated backdrop and Persian localized copy.
#[derive(Clone, Debug)]
pub struct HeroSpotlightItem {
    pub media: ApiMedia,
    pub category: MediaCategory,
    pub artwork: Option<String>,
    pub title_fa: Option<String>,
    pub description_fa: Option<String>,
}

/// Curated spotlight hero metadata from `Streamovie/src/app/page.tsx`.
pub const HERO_SPOTLIGHT_SPECS: [(&str, &str, &str, &str); 6] = [
    (
        "Friends",
        "https://wallpapercrafter.com/desktop1/520373-Chandler-Bing-David-Schwimmer-Friends-TV-Series.jpg",
        "فرندز",
        "شش دوست در نیویورک با لحظه‌های خنده‌دار، رابطه‌های پیچیده و ماجراهای روزمره کنار هم بزرگ می‌شوند.",
    ),
    (
        "Attack on Titan",
        "https://global-img.gamergen.com/shingeki-no-kyojin-attack-on-titan-final-season-29-05-2020_0900953328.jpg",
        "حمله به تایتان",
        "انسان‌ها پشت دیوارهای عظیم از تایتان‌ها پناه گرفته‌اند؛ کشف حقیقت، سرنوشتشان را دگرگون می‌کند.",
    ),
    (
        "Berserk",
        "https://server.wallpaperalchemy.com/storage/wallpapers/423/berserk-guts-moon-wallpaper-4k.jpg",
        "برزرک",
        "گاتس، جنگجویی تنها در دنیایی تاریک، با سرنوشتی خونین و نیروهای اهریمنی روبه‌روست.",
    ),
    (
        "Dark",
        "https://img.zeit.de/kultur/film/2017-11/dark-netflix-louis-hofmann-teaser/wide__1300x731",
        "دارک",
        "ناپدیدشدن یک کودک در شهری کوچک، رازهای چند نسل و پیوند پیچیده زمان را آشکار می‌کند.",
    ),
    (
        "Breaking Bad",
        "https://img.apmcdn.org/ef7946f7c9a4e263229a4b3e0fb182618ab18f73/normal/baca2e-20131004-breaking-bad.jpg",
        "بریکینگ بد",
        "یک معلم شیمی پس از تشخیص بیماری، وارد دنیای تولید مواد مخدر می‌شود و زندگی‌اش را به مسیری خطرناک می‌کشاند.",
    ),
    (
        "Game of Thrones",
        "https://media.assettype.com/thequint/2019-04/6ce6797f-d7d9-4fec-897c-8a167105d56e/Got_Hero_Image.jpg",
        "بازی تاج‌وتخت",
        "خاندان‌های قدرتمند برای تخت آهنین می‌جنگند، در حالی که تهدیدی فراسوی دیوار در حال نزدیک‌شدن است.",
    ),
];

fn first_non_empty_str(candidates: &[Option<&str>]) -> Option<String> {
    for candidate in candidates.iter().flatten() {
        let trimmed = candidate.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn value_to_string(val: &Value) -> Option<String> {
    match val {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn value_to_f64(val: &Value) -> f64 {
    match val {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

impl ApiMedia {
    /// Display title (`title` or `name` or `"Untitled"`).
    pub fn display_title(&self) -> String {
        first_non_empty_str(&[self.title.as_deref(), self.name.as_deref()])
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Primary poster image URL.
    pub fn poster_image(&self) -> Option<String> {
        first_non_empty_str(&[
            self.image.as_deref(),
            self.poster.as_deref(),
            self.poster_url.as_deref(),
            self.thumbnail.as_deref(),
        ])
    }

    /// Cover / backdrop artwork URL.
    pub fn cover_image(&self) -> Option<String> {
        first_non_empty_str(&[
            self.cover.as_deref(),
            self.backdrop.as_deref(),
            self.backdrop_url.as_deref(),
            self.banner.as_deref(),
            self.wallpaper.as_deref(),
            self.image.as_deref(),
        ])
    }

    /// High-resolution hero image URL, upgrading TMDB `w...` paths to `original`.
    pub fn hero_image(&self) -> Option<String> {
        let extra_str = |key: &str| self.extra.get(key).and_then(value_to_string);
        let wide = self
            .backdrop_url
            .clone()
            .or_else(|| self.backdrop.clone())
            .or_else(|| extra_str("backdrop_image"))
            .or_else(|| extra_str("backdropImage"))
            .or_else(|| self.banner.clone())
            .or_else(|| self.wallpaper.clone())
            .or_else(|| extra_str("fanart"))
            .or_else(|| extra_str("background_image"))
            .or_else(|| extra_str("landscape_url"))
            .or_else(|| self.cover_image())
            .or_else(|| self.poster_image())?;

        Some(upgrade_tmdb_original(&wide))
    }

    /// Release year string.
    pub fn release_year(&self) -> Option<String> {
        self.year
            .as_ref()
            .and_then(value_to_string)
            .or_else(|| self.extra.get("release_year").and_then(value_to_string))
            .or_else(|| self.extra.get("release_date").and_then(value_to_string))
    }

    /// IMDb or catalog numeric rating (`0.0` if absent).
    pub fn rating_value(&self) -> f64 {
        if let Some(imdb) = self.imdb.as_ref() {
            let v = value_to_f64(imdb);
            if v > 0.0 {
                return v;
            }
        }
        if let Some(rating) = self.rating.as_ref() {
            let v = value_to_f64(rating);
            if v > 0.0 {
                return v;
            }
        }
        if let Some(rate) = self.rate.as_ref() {
            return value_to_f64(rate);
        }
        0.0
    }

    /// Runtime / duration string.
    pub fn duration_label(&self) -> Option<String> {
        first_non_empty_str(&[
            self.duration.as_deref(),
            self.runtime.as_deref(),
            self.extra.get("length").and_then(|v| v.as_str()),
        ])
    }

    /// Age classification string.
    pub fn classification_label(&self) -> Option<String> {
        first_non_empty_str(&[
            self.classification.as_deref(),
            self.extra.get("age_rating").and_then(|v| v.as_str()),
        ])
    }

    /// Extracted genre names.
    pub fn genre_list(&self) -> Vec<String> {
        extract_titles(self.genres.as_ref().or(self.genre.as_ref()))
    }

    /// Extracted genre IDs for filtering.
    pub fn genre_ids(&self) -> Vec<u64> {
        let Some(Value::Array(items)) = self.genres.as_ref().or(self.genre.as_ref()) else {
            return Vec::new();
        };
        items
            .iter()
            .filter_map(|item| item.get("id").and_then(|id| id.as_u64()))
            .collect()
    }

    /// Extracted country names.
    pub fn country_list(&self) -> Vec<String> {
        extract_titles(self.country.as_ref().or(self.countries.as_ref()))
    }

    /// Extracted stream/download sources.
    pub fn source_links(&self) -> Vec<SourceLink> {
        extract_sources(self.sources.as_ref())
    }

    /// Parsed description with overview and labeled metadata blocks.
    pub fn parsed_description(&self) -> ParsedDescription {
        let raw = self
            .description
            .as_deref()
            .or(self.summary.as_deref())
            .unwrap_or_default();
        parse_description(raw)
    }

    /// Inferred `MediaCategory` (`Movie` or `Series`).
    pub fn inferred_category(&self) -> MediaCategory {
        let raw = self
            .media_type
            .as_deref()
            .or_else(|| self.extra.get("kind").and_then(|v| v.as_str()))
            .or_else(|| self.extra.get("media_type").and_then(|v| v.as_str()))
            .unwrap_or_default()
            .to_ascii_lowercase();
        if raw.contains("serie") || raw.contains("tv") {
            MediaCategory::Series
        } else {
            MediaCategory::Movie
        }
    }

    /// Extracts an IMDb ID (`tt1234567`) if present in the payload.
    pub fn imdb_id(&self) -> Option<String> {
        for key in ["imdb_id", "imdbId", "imdbID", "imdb_url", "imdb_link"] {
            if let Some(val) = self.extra.get(key).and_then(|v| v.as_str()) {
                let trimmed = val.trim();
                if trimmed.starts_with("tt")
                    && trimmed.len() >= 7
                    && trimmed[2..].chars().all(|c| c.is_ascii_digit())
                {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    /// Extracts an English lookup title for OMDb queries (checking `original_title`,
    /// `en_title`, description lines, or the main title).
    pub fn english_lookup_title(&self) -> String {
        for key in ["original_title", "original_name", "en_title"] {
            if let Some(val) = self.extra.get(key).and_then(|v| v.as_str()) {
                let trimmed = val.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        let main = self.display_title();
        // Strip trailing 4-digit year if embedded in title (e.g. "Inception 2010")
        let words: Vec<&str> = main.split_whitespace().collect();
        if words.len() > 1 {
            if let Some(last) = words.last() {
                if last.len() == 4 && last.chars().all(|c| c.is_ascii_digit()) {
                    return words[..words.len() - 1].join(" ");
                }
            }
        }
        main
    }
}

impl ApiSeason {
    pub fn display_title(&self, index: usize) -> String {
        first_non_empty_str(&[self.title.as_deref(), self.name.as_deref()])
            .unwrap_or_else(|| format!("Season {}", index + 1))
    }

    pub fn episode_list(&self) -> Vec<Episode> {
        let raw = self
            .episodes
            .as_ref()
            .or(self.parts.as_ref())
            .or(self.items.as_ref());
        let Some(Value::Array(items)) = raw else {
            return Vec::new();
        };
        items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .or_else(|| item.get("name").and_then(|v| v.as_str()))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| format!("Episode {}", index + 1));
                let id = item.get("id").and_then(|v| v.as_u64());
                let sources = extract_sources(item.get("sources"));
                Episode {
                    key: format!("{}-{index}", id.unwrap_or(index as u64)),
                    id,
                    title,
                    sources,
                }
            })
            .collect()
    }
}

fn upgrade_tmdb_original(url: &str) -> String {
    for prefix in ["https://image.tmdb.org/t/p/", "http://image.tmdb.org/t/p/"] {
        if let Some(rest) = url.strip_prefix(prefix) {
            if let Some((_, tail)) = rest.split_once('/') {
                return format!("{prefix}original/{tail}");
            }
        }
    }
    url.to_string()
}

fn extract_titles(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
                Value::Object(map) => map
                    .get("title")
                    .or_else(|| map.get("name"))
                    .and_then(value_to_string),
                _ => None,
            })
            .collect(),
        Value::String(s) => s
            .split([',', '،'])
            .map(|part| part.trim().to_string())
            .filter(|part| !part.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

pub fn extract_sources(value: Option<&Value>) -> Vec<SourceLink> {
    let Some(Value::Array(items)) = value else {
        return Vec::new();
    };
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let obj = item.as_object()?;
            let url = obj
                .get("url")
                .or_else(|| obj.get("link"))
                .or_else(|| obj.get("src"))
                .and_then(value_to_string)?;
            let quality = obj
                .get("quality")
                .or_else(|| obj.get("resolution"))
                .or_else(|| obj.get("label"))
                .and_then(value_to_string)
                .unwrap_or_else(|| "Source".to_string());
            let format_type = obj
                .get("type")
                .or_else(|| obj.get("format"))
                .and_then(value_to_string);
            let id = obj
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(index);
            let q_lower = quality.to_lowercase();
            let is_trailer = q_lower.contains("تیزر")
                || q_lower.contains("trailer")
                || q_lower.contains("teaser");

            Some(SourceLink {
                id,
                quality: format_quality(&quality),
                format_type,
                url,
                is_trailer,
            })
        })
        .collect()
}

pub fn format_quality(quality: &str) -> String {
    quality.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parses a Streamovie description string into an overview and canonical Persian metadata blocks,
/// matching `Streamovie/src/lib/media.ts`.
pub fn parse_description(description: &str) -> ParsedDescription {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return ParsedDescription::default();
    }

    let normalized = trimmed.replace("\r\n", "\n");
    let mut fields = Vec::new();
    let mut overview = None;

    for block in normalized.split("\n\n") {
        let text = block.trim();
        if text.is_empty() {
            continue;
        }
        let Some((raw_label, raw_value)) = text.split_once(':') else {
            continue;
        };
        let label = raw_label.split_whitespace().collect::<Vec<_>>().join(" ");
        let value = raw_value.trim().to_string();
        if value.is_empty() {
            continue;
        }

        if label.contains("خلاصه") {
            overview = Some(value);
            continue;
        }

        let canonical = match label.as_str() {
            "نام پارسی" => Some("نام پارسی"),
            "سایر عناوین" => Some("سایر عناوین"),
            "کارگردان" => Some("کارگردان"),
            "ستارگان" => Some("بازیگران"),
            "پخش از شبکه" => Some("شبکه"),
            "وضعیت سریال" => Some("وضعیت"),
            _ => None,
        };

        if let Some(canonical_label) = canonical {
            fields.push(DescriptionBlock {
                label: canonical_label.to_string(),
                value,
            });
        }
    }

    // If no labeled `خلاصه:` block existed, treat the plain text as the overview.
    if overview.is_none() && fields.is_empty() {
        overview = Some(normalized);
    }

    ParsedDescription { overview, fields }
}

/// Normalizes a Persian/English title for exclusion checks (`src/lib/media-exclusions.ts`).
fn normalize_exclusion_title(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'ي' | 'ى' => 'ی',
            'ك' => 'ک',
            other => other,
        })
        .filter(|c| !('\u{064B}'..='\u{065F}').contains(c) && *c != '\u{0670}')
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn is_excluded_media(media: &ApiMedia) -> bool {
    let excluded = [
        normalize_exclusion_title("نسخه جدید ملودی باکس"),
        normalize_exclusion_title("نسخه جدید هورتون"),
    ];
    [media.title.as_deref(), media.name.as_deref()]
        .into_iter()
        .flatten()
        .any(|title| {
            let norm = normalize_exclusion_title(title);
            excluded.iter().any(|ex| ex == &norm)
        })
}

pub fn exclude_hidden_media(items: Vec<ApiMedia>) -> Vec<ApiMedia> {
    items
        .into_iter()
        .filter(|item| !is_excluded_media(item))
        .collect()
}
