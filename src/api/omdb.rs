//! OMDb API client ported from `Streamovie/src/app/api/omdb/route.ts` and
//! `Streamovie/src/lib/api/omdb-api.ts`.

use std::time::Duration;

use anyhow::{Context as _, Result};
use reqwest::blocking::Client;

use crate::{
    config::{AppConfig, OMDB_BASE_URL},
    models::{MediaCategory, OmdbDetails, OmdbSeason},
};

const OMDB_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Clone)]
pub struct OmdbClient {
    config: AppConfig,
    http: Client,
}

impl OmdbClient {
    pub fn new(config: AppConfig) -> Self {
        let http = Client::builder()
            .timeout(OMDB_TIMEOUT)
            .user_agent("Streamovie-macOS/0.1.0")
            .build()
            .expect("Failed to construct OMDb HTTP client");

        Self { config, http }
    }

    /// Fetches title enrichment details from `https://www.omdbapi.com/`.
    pub fn fetch_details(
        &self,
        title: &str,
        year: Option<&str>,
        imdb_id: Option<&str>,
        category: MediaCategory,
    ) -> Result<OmdbDetails> {
        let mut query: Vec<(&str, String)> = vec![
            ("apikey", self.config.omdb_api_key().to_string()),
            ("type", category.as_str().to_string()),
            ("plot", "full".to_string()),
            ("r", "json".to_string()),
        ];

        if let Some(imdb) = imdb_id.filter(|s| !s.trim().is_empty()) {
            query.push(("i", imdb.trim().to_string()));
        } else {
            query.push(("t", title.trim().to_string()));
        }

        if let Some(y) = year.and_then(extract_four_digit_year) {
            query.push(("y", y));
        }

        let response = self
            .http
            .get(OMDB_BASE_URL)
            .query(&query)
            .send()
            .context("Failed to reach OMDb API")?;

        let body = response
            .text()
            .context("Failed to read OMDb details response text")?;
        serde_json::from_str::<OmdbDetails>(&body).context("Failed to parse OMDb details JSON")
    }

    /// Fetches season episode guide from `https://www.omdbapi.com/` (`Season={season}`).
    pub fn fetch_season(
        &self,
        title: &str,
        season: usize,
        year: Option<&str>,
        imdb_id: Option<&str>,
    ) -> Result<OmdbSeason> {
        let mut query: Vec<(&str, String)> = vec![
            ("apikey", self.config.omdb_api_key().to_string()),
            ("type", "series".to_string()),
            ("Season", season.to_string()),
            ("r", "json".to_string()),
        ];

        if let Some(imdb) = imdb_id.filter(|s| !s.trim().is_empty()) {
            query.push(("i", imdb.trim().to_string()));
        } else {
            query.push(("t", title.trim().to_string()));
        }

        if let Some(y) = year.and_then(extract_four_digit_year) {
            query.push(("y", y));
        }

        let response = self
            .http
            .get(OMDB_BASE_URL)
            .query(&query)
            .send()
            .context("Failed to reach OMDb season API")?;

        let body = response
            .text()
            .context("Failed to read OMDb season response text")?;
        serde_json::from_str::<OmdbSeason>(&body).context("Failed to parse OMDb season JSON")
    }
}

fn extract_four_digit_year(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 4 {
        return None;
    }
    for window in bytes.windows(4) {
        if window.iter().all(|b| b.is_ascii_digit()) {
            return String::from_utf8(window.to_vec()).ok();
        }
    }
    None
}
