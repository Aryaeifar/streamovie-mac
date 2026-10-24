//! Official IMDb Top 250 order.
//!
//! The catalog genre named “250 برتر تاریخ” is not stored in chart order, and
//! a few unrelated titles sit at the top because their rating field is `10`.
//! Titles are matched against a snapshot of the real charts and then sorted
//! by that rank.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

use crate::models::{ApiMedia, MediaCategory};

const CHART_JSON: &str = include_str!("../../assets/imdb-top250.json");

#[derive(Deserialize)]
struct ChartFile {
    movies: Vec<RawEntry>,
    series: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawEntry {
    title: String,
    #[serde(rename = "originalTitle")]
    original_title: String,
    year: i32,
}

#[derive(Clone, Copy)]
struct ChartHit {
    rank: u16,
    year: i32,
}

struct ChartIndex {
    movies: HashMap<String, Vec<ChartHit>>,
    series: HashMap<String, Vec<ChartHit>>,
}

fn charts() -> &'static ChartIndex {
    static CHARTS: OnceLock<ChartIndex> = OnceLock::new();
    CHARTS.get_or_init(|| {
        let file: ChartFile = serde_json::from_str(CHART_JSON).expect("IMDb Top 250 snapshot");
        ChartIndex {
            movies: index_entries(&file.movies),
            series: index_entries(&file.series),
        }
    })
}

fn index_entries(entries: &[RawEntry]) -> HashMap<String, Vec<ChartHit>> {
    let mut index: HashMap<String, Vec<ChartHit>> = HashMap::new();
    for (offset, entry) in entries.iter().enumerate() {
        let hit = ChartHit {
            rank: (offset + 1) as u16,
            year: entry.year,
        };
        for title in [&entry.title, &entry.original_title] {
            let normalized = normalize_title(title);
            if normalized.is_empty() {
                continue;
            }
            let hits = index.entry(normalized).or_insert_with(Vec::new);
            if !hits.iter().any(|existing| existing.rank == hit.rank) {
                hits.push(hit);
            }
        }
    }
    index
}

fn normalize_title(title: &str) -> String {
    title
        .nfkd()
        .collect::<String>()
        .to_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn strip_trailing_year(normalized: &str) -> String {
    let bytes = normalized.as_bytes();
    if bytes.len() < 4 {
        return normalized.to_string();
    }
    let tail = &normalized[normalized.len() - 4..];
    if tail.chars().all(|ch| ch.is_ascii_digit())
        && matches!(&tail[..2], "18" | "19" | "20")
    {
        return normalized[..normalized.len() - 4].to_string();
    }
    normalized.to_string()
}

fn catalog_titles(media: &ApiMedia) -> Vec<String> {
    let mut titles = Vec::new();
    for title in [media.title.as_deref(), media.name.as_deref()]
        .into_iter()
        .flatten()
    {
        let trimmed = title.trim();
        if !trimmed.is_empty() {
            titles.push(trimmed.to_string());
        }
    }
    let description = media
        .description
        .as_deref()
        .or(media.summary.as_deref())
        .unwrap_or("");
    for line in description.lines() {
        let Some((label, value)) = line.split_once([':', '：']) else {
            continue;
        };
        if label.contains("نام اصلی") || label.contains("نام انگلیسی") {
            let value = value.trim();
            if !value.is_empty() {
                titles.push(value.to_string());
            }
        }
    }
    titles
}

fn catalog_year(media: &ApiMedia, title: &str) -> Option<i32> {
    if let Some(year) = media.year.as_ref().and_then(year_number) {
        if (1880..=2100).contains(&year) {
            return Some(year);
        }
    }
    let trimmed = title.trim();
    let digits: String = trimmed
        .chars()
        .rev()
        .skip_while(|ch| ch.is_whitespace())
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if digits.len() == 4 && matches!(&digits[..2], "18" | "19" | "20") {
        let boundary = trimmed.trim_end().len().saturating_sub(4);
        if boundary == 0 || !trimmed[..boundary].ends_with(|ch: char| ch.is_ascii_alphanumeric()) {
            return digits.parse().ok();
        }
    }
    None
}

fn year_number(value: &Value) -> Option<i32> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .map(|year| year as i32)
            .or_else(|| number.as_f64().map(|year| year as i32)),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

/// Chart position, starting at 1, when this catalog title is on the snapshot.
pub fn chart_rank(media: &ApiMedia, category: MediaCategory) -> Option<u16> {
    let index = match category {
        MediaCategory::Movie => &charts().movies,
        MediaCategory::Series => &charts().series,
    };
    for title in catalog_titles(media) {
        let normalized = normalize_title(&title);
        let stripped = strip_trailing_year(&normalized);
        let Some(candidates) = index.get(&normalized).or_else(|| index.get(&stripped)) else {
            continue;
        };
        let year = catalog_year(media, &title);
        if let Some(hit) = candidates.iter().find(|hit| {
            year.is_none_or(|year| hit.year == 0 || year == hit.year)
        }) {
            return Some(hit.rank);
        }
        if candidates.len() == 1 {
            let hit = candidates[0];
            if year.is_none_or(|year| (hit.year - year).abs() <= 1) {
                return Some(hit.rank);
            }
        }
    }
    None
}

/// Drops titles that are not on the chart and orders the rest by rank.
pub fn in_chart_order(items: Vec<ApiMedia>, category: MediaCategory) -> Vec<ApiMedia> {
    let mut best: HashMap<u64, (u16, ApiMedia)> = HashMap::new();
    let mut unkeyed = Vec::new();
    for item in items {
        let Some(rank) = chart_rank(&item, category) else {
            continue;
        };
        if item.id == 0 {
            unkeyed.push((rank, item));
            continue;
        }
        match best.get(&item.id) {
            Some((existing, _)) if *existing <= rank => {}
            _ => {
                best.insert(item.id, (rank, item));
            }
        }
    }
    let mut ranked: Vec<_> = best.into_values().chain(unkeyed).collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::{chart_rank, in_chart_order};
    use crate::models::{ApiMedia, MediaCategory};

    fn media(id: u64, title: &str, year: i32) -> ApiMedia {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "title": title,
            "year": year
        }))
        .unwrap()
    }

    #[test]
    fn chart_order_starts_with_the_official_number_one() {
        assert_eq!(
            chart_rank(
                &media(1, "The Shawshank Redemption", 1994),
                MediaCategory::Movie
            ),
            Some(1)
        );
        assert_eq!(
            chart_rank(&media(2, "The Godfather", 1972), MediaCategory::Movie),
            Some(2)
        );
        assert_eq!(
            chart_rank(&media(3, "Breaking Bad", 2008), MediaCategory::Series),
            Some(1)
        );
    }

    #[test]
    fn inflated_catalog_ratings_are_left_off_the_chart() {
        let ranked = in_chart_order(
            vec![
                media(9, "Melody Box", 2026),
                media(4, "The Dark Knight", 2008),
                media(1, "The Shawshank Redemption", 1994),
            ],
            MediaCategory::Movie,
        );
        let titles: Vec<_> = ranked
            .iter()
            .map(|item| item.title.clone().unwrap_or_default())
            .collect();
        assert_eq!(
            titles,
            vec![
                "The Shawshank Redemption".to_string(),
                "The Dark Knight".to_string()
            ]
        );
    }

    #[test]
    fn a_one_year_catalog_mismatch_still_matches_a_unique_title() {
        assert_eq!(
            chart_rank(&media(7, "Schindler's List", 1994), MediaCategory::Movie),
            Some(7)
        );
    }
}
