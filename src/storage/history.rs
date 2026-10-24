//! Recently opened titles for the Continue Watching row (`continue.json`).

use std::{fs, path::PathBuf};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use crate::{
    models::{ApiMedia, MediaCategory},
    storage::app_data_dir,
};

const MAX_ITEMS: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContinueEntry {
    pub media: ApiMedia,
    pub kind: MediaCategory,
    pub progress: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ContinueWatchingData {
    #[serde(default)]
    pub items: Vec<ContinueEntry>,
}

impl ContinueWatchingData {
    /// Moves a title to the front and raises its progress up to the given floor.
    pub fn touch(&mut self, media: ApiMedia, kind: MediaCategory, floor: f32) {
        let progress = self
            .items
            .iter()
            .find(|entry| same_title(entry, &media, kind))
            .map(|entry| entry.progress.max(floor))
            .unwrap_or(floor)
            .clamp(0.08, 0.92);
        self.items.retain(|entry| !same_title(entry, &media, kind));
        self.items.insert(
            0,
            ContinueEntry {
                media,
                kind,
                progress,
            },
        );
        self.items.truncate(MAX_ITEMS);
    }
}

fn same_title(entry: &ContinueEntry, media: &ApiMedia, kind: MediaCategory) -> bool {
    if media.id != 0 {
        entry.media.id == media.id && entry.kind == kind
    } else {
        entry.kind == kind && entry.media.display_title() == media.display_title()
    }
}

#[derive(Clone, Debug)]
pub struct ContinueStorage {
    file_path: PathBuf,
}

impl Default for ContinueStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinueStorage {
    pub fn new() -> Self {
        Self {
            file_path: app_data_dir().join("continue.json"),
        }
    }

    pub fn load(&self) -> ContinueWatchingData {
        let Ok(content) = fs::read_to_string(&self.file_path) else {
            return ContinueWatchingData::default();
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self, data: &ContinueWatchingData) -> Result<()> {
        let serialized =
            serde_json::to_string_pretty(data).context("Failed to serialize continue watching")?;
        fs::write(&self.file_path, serialized).context("Failed to write continue.json")
    }
}
