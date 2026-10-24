//! Streamovie catalog API client ported directly from
//! `Streamovie/src/lib/api/client.ts` and `Streamovie/src/lib/api/media-api.ts`.

use std::time::Duration;

use anyhow::{Context as _, Result, anyhow};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;

use crate::{
    config::AppConfig,
    models::{
        ApiCountry, ApiGenre, ApiMedia, ApiSeason, BrowseSort, MediaCategory, SearchResponse,
        exclude_hidden_media, imdb_top250,
    },
};

/// Catalog genre “250 برتر تاریخ”. It holds the site’s Top 250 bucket, not chart order.
const TOP250_GENRE_ID: u64 = 32;
const TOP250_PAGE_SIZE: usize = 30;

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
                                last_error = Some(anyhow!("Invalid JSON from {}: {}", host, err));
                            }
                        },
                        Err(err) => {
                            last_error = Some(anyhow!(
                                "Failed reading response body from {}: {}",
                                host,
                                err
                            ));
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

    /// Every page of the Top 250 genre, kept only when it is on the official chart
    /// and ordered by that rank.
    pub fn fetch_imdb_chart(&self, category: MediaCategory) -> Result<Vec<ApiMedia>> {
        let mut collected = Vec::new();
        for page in 0..8 {
            let batch = match category {
                MediaCategory::Movie => self.fetch_movies(page, TOP250_GENRE_ID, BrowseSort::Imdb),
                MediaCategory::Series => self.fetch_series(page, TOP250_GENRE_ID, BrowseSort::Imdb),
            };
            match batch {
                Ok(items) => {
                    let done = items.len() < TOP250_PAGE_SIZE;
                    collected.extend(items);
                    if done {
                        break;
                    }
                }
                Err(err) if collected.is_empty() => return Err(err),
                Err(_) => break,
            }
        }
        Ok(imdb_top250::in_chart_order(collected, category))
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
}
