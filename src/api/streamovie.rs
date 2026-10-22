//! Streamovie catalog API client ported directly from
//! `Streamovie/src/lib/api/client.ts` and `Streamovie/src/lib/api/media-api.ts`.

use std::time::Duration;

use anyhow::{Context as _, Result, anyhow};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use crate::{
    config::AppConfig,
    models::{
        ApiCountry, ApiGenre, ApiMedia, ApiSeason, BrowseSort, HeroSpotlightItem,
        HERO_SPOTLIGHT_SPECS, MediaCategory, SearchResponse, exclude_hidden_media,
    },
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct StreamovieClient {
    config: AppConfig,
    http: Client,
}

impl StreamovieClient {
    pub fn new(config: AppConfig) -> Self {
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent("Streamovie-macOS/0.1.0")
            .build()
            .expect("Failed to construct Streamovie HTTP client");

        Self { config, http }
    }

    fn encoded_key(&self) -> String {
        urlencoding::encode(self.config.streamovie_api_key()).into_owned()
    }

    /// Requests `path` across the configured Streamovie mirror hosts in order,
    /// returning the first valid deserialized response (`requestWithFallback`).
    pub fn request_with_fallback<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let mut last_error = None;

        for host in self.config.api_hosts() {
            let url = format!("{host}{path}");
            match self.http.get(&url).send() {
                Ok(response) => {
                    if !response.status().is_success() {
                        last_error = Some(anyhow!("HTTP {} from {}", response.status(), host));
                        continue;
                    }
                    match response.text() {
                        Ok(body) => match serde_json::from_str::<T>(&body) {
                            Ok(parsed) => return Ok(parsed),
                            Err(err) => {
                                last_error =
                                    Some(anyhow!("Invalid JSON from {}: {}", host, err));
                            }
                        },
                        Err(err) => {
                            last_error =
                                Some(anyhow!("Failed reading response body from {}: {}", host, err));
                        }
                    }
                }
                Err(err) => {
                    last_error = Some(anyhow!("Network error reaching {}: {}", host, err));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Unable to reach any Streamovie API host")))
    }

    /// `GET /api/movie/by/filtres/{genreId}/{sort}/{page}/{key}`
    pub fn fetch_movies(
        &self,
        page: usize,
        genre_id: u64,
        sort: BrowseSort,
    ) -> Result<Vec<ApiMedia>> {
        let sort_param = urlencoding::encode(sort.as_api_param());
        let path = format!(
            "/api/movie/by/filtres/{genre_id}/{sort_param}/{page}/{}",
            self.encoded_key()
        );
        let items: Vec<ApiMedia> = self
            .request_with_fallback(&path)
            .context("Failed to fetch movies")?;
        Ok(exclude_hidden_media(items))
    }

    /// `GET /api/serie/by/filtres/{genreId}/{sort}/{page}/{key}`
    pub fn fetch_series(
        &self,
        page: usize,
        genre_id: u64,
        sort: BrowseSort,
    ) -> Result<Vec<ApiMedia>> {
        let sort_param = urlencoding::encode(sort.as_api_param());
        let path = format!(
            "/api/serie/by/filtres/{genre_id}/{sort_param}/{page}/{}",
            self.encoded_key()
        );
        let items: Vec<ApiMedia> = self
            .request_with_fallback(&path)
            .context("Failed to fetch series")?;
        Ok(exclude_hidden_media(items))
    }

    /// `GET /api/search/{query}/{key}/`
    pub fn search(&self, query: &str) -> Result<Vec<ApiMedia>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let encoded_query = urlencoding::encode(trimmed);
        let path = format!("/api/search/{encoded_query}/{}/", self.encoded_key());
        let response: SearchResponse = self
            .request_with_fallback(&path)
            .context("Failed to search catalog")?;
        Ok(exclude_hidden_media(response.posters))
    }

    /// `GET /api/season/by/serie/{seriesId}/{key}/`
    pub fn fetch_seasons(&self, series_id: u64) -> Result<Vec<ApiSeason>> {
        let path = format!("/api/season/by/serie/{series_id}/{}/", self.encoded_key());
        self.request_with_fallback(&path)
            .context("Failed to fetch seasons")
    }

    /// `GET /api/genre/all/{key}/`
    pub fn fetch_genres(&self) -> Result<Vec<ApiGenre>> {
        let path = format!("/api/genre/all/{}/", self.encoded_key());
        self.request_with_fallback(&path)
            .context("Failed to fetch genres")
    }

    /// `GET /api/country/all/{key}/`
    #[allow(dead_code)]
    pub fn fetch_countries(&self) -> Result<Vec<ApiCountry>> {
        let path = format!("/api/country/all/{}/", self.encoded_key());
        self.request_with_fallback(&path)
            .context("Failed to fetch countries")
    }

    /// `GET /api/poster/by/filtres/0/{countryId}/{sort}/{page}/{key}`
    #[allow(dead_code)]
    pub fn fetch_media_by_country(
        &self,
        country_id: u64,
        page: usize,
        sort: BrowseSort,
    ) -> Result<Vec<ApiMedia>> {
        let sort_param = urlencoding::encode(sort.as_api_param());
        let path = format!(
            "/api/poster/by/filtres/0/{country_id}/{sort_param}/{page}/{}",
            self.encoded_key()
        );
        let items: Vec<ApiMedia> = self
            .request_with_fallback(&path)
            .context("Failed to fetch media by country")?;
        Ok(exclude_hidden_media(items))
    }

    /// Resolves the curated featured spotlight series from `Streamovie/src/app/page.tsx`
    /// (`Friends`, `Attack on Titan`, `Berserk`, `Dark`, `Breaking Bad`, `Game of Thrones`),
    /// falling back to top catalog items if any query does not match.
    pub fn fetch_featured_spotlight(
        &self,
        fallback_series: &[ApiMedia],
    ) -> Vec<HeroSpotlightItem> {
        let mut spotlight = Vec::new();

        for (title, backdrop, title_fa, description_fa) in HERO_SPOTLIGHT_SPECS {
            let needle = title.to_lowercase();
            if let Ok(results) = self.search(title) {
                if let Some(matched) = results.into_iter().find(|media| {
                    let candidates = [
                        media.title.as_deref(),
                        media.name.as_deref(),
                        media.extra.get("original_title").and_then(|v| v.as_str()),
                        media.extra.get("original_name").and_then(|v| v.as_str()),
                        media.extra.get("en_title").and_then(|v| v.as_str()),
                    ];
                    candidates.into_iter().flatten().any(|name| {
                        let lower = name.to_lowercase();
                        lower == needle || lower.contains(&needle)
                    })
                }) {
                    spotlight.push(HeroSpotlightItem {
                        media: matched,
                        category: MediaCategory::Series,
                        artwork: Some(backdrop.to_string()),
                        title_fa: Some(title_fa.to_string()),
                        description_fa: Some(description_fa.to_string()),
                    });
                }
            }
        }

        if spotlight.is_empty() {
            for media in fallback_series.iter().take(6).cloned() {
                let artwork = media.hero_image();
                spotlight.push(HeroSpotlightItem {
                    media,
                    category: MediaCategory::Series,
                    artwork,
                    title_fa: None,
                    description_fa: None,
                });
            }
        }

        spotlight
    }
}
