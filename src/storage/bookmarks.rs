//! Persistent JSON storage for Streamovie bookmarks and likes (`bookmarks.json`).
#![allow(dead_code)]

use std::{fs, path::PathBuf};

use anyhow::{Context as _, Result};

use crate::{models::LibraryStoreData, storage::app_data_dir};

#[derive(Clone, Debug)]
pub struct BookmarkStorage {
    file_path: PathBuf,
}

impl Default for BookmarkStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl BookmarkStorage {
    pub fn new() -> Self {
        Self {
            file_path: app_data_dir().join("bookmarks.json"),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn load(&self) -> LibraryStoreData {
        let Ok(content) = fs::read_to_string(&self.file_path) else {
            return LibraryStoreData::default();
        };
        serde_json::from_str::<LibraryStoreData>(&content).unwrap_or_default()
    }

    pub fn save(&self, data: &LibraryStoreData) -> Result<()> {
        let serialized =
            serde_json::to_string_pretty(data).context("Failed to serialize bookmarks")?;
        fs::write(&self.file_path, serialized).context("Failed to write bookmarks.json")
    }
}
