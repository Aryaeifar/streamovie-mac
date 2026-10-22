//! OMDb API response models ported from `Streamovie/src/lib/api/omdb-api.ts`.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmdbRating {
    #[serde(rename = "Source", default)]
    pub source: String,
    #[serde(rename = "Value", default)]
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmdbEpisode {
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Released")]
    pub released: Option<String>,
    #[serde(rename = "Episode")]
    pub episode: Option<String>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: Option<String>,
    #[serde(rename = "imdbID")]
    pub imdb_id: Option<String>,
    #[serde(rename = "Season")]
    pub season: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmdbDetails {
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Year")]
    pub year: Option<String>,
    #[serde(rename = "Rated")]
    pub rated: Option<String>,
    #[serde(rename = "Released")]
    pub released: Option<String>,
    #[serde(rename = "Runtime")]
    pub runtime: Option<String>,
    #[serde(rename = "Genre")]
    pub genre: Option<String>,
    #[serde(rename = "Director")]
    pub director: Option<String>,
    #[serde(rename = "Writer")]
    pub writer: Option<String>,
    #[serde(rename = "Actors")]
    pub actors: Option<String>,
    #[serde(rename = "Plot")]
    pub plot: Option<String>,
    #[serde(rename = "Awards")]
    pub awards: Option<String>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: Option<String>,
    #[serde(rename = "imdbVotes")]
    pub imdb_votes: Option<String>,
    #[serde(rename = "imdbID")]
    pub imdb_id: Option<String>,
    #[serde(rename = "totalSeasons")]
    pub total_seasons: Option<String>,
    #[serde(rename = "Type")]
    pub media_type: Option<String>,
    #[serde(rename = "Country")]
    pub country: Option<String>,
    #[serde(rename = "Language")]
    pub language: Option<String>,
    #[serde(rename = "Ratings", default)]
    pub ratings: Vec<OmdbRating>,
    #[serde(rename = "Response")]
    pub response: Option<String>,
    #[serde(rename = "Error")]
    pub error: Option<String>,
}

impl OmdbDetails {
    pub fn is_ok(&self) -> bool {
        self.response.as_deref() == Some("True")
    }

    pub fn non_na(field: &Option<String>) -> Option<&str> {
        field
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty() && *v != "N/A")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OmdbSeason {
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Season")]
    pub season: Option<String>,
    #[serde(rename = "totalSeasons")]
    pub total_seasons: Option<String>,
    #[serde(rename = "Episodes", default)]
    pub episodes: Vec<OmdbEpisode>,
    #[serde(rename = "Response")]
    pub response: Option<String>,
    #[serde(rename = "Error")]
    pub error: Option<String>,
}

impl OmdbSeason {
    pub fn is_ok(&self) -> bool {
        self.response.as_deref() == Some("True")
    }
}
