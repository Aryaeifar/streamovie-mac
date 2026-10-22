//! Local bookmark and watchlist models matching `Streamovie/src/stores/use-library-store.ts`.
#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::media::{ApiMedia, MediaCategory};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedMediaEntry {
    pub media: ApiMedia,
    pub kind: MediaCategory,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LibraryStoreData {
    #[serde(default)]
    pub watchlist: Vec<u64>,
    #[serde(default)]
    pub saved_media: HashMap<u64, SavedMediaEntry>,
    #[serde(default)]
    pub likes: Vec<u64>,
}

impl LibraryStoreData {
    pub fn is_bookmarked(&self, media_id: u64) -> bool {
        self.watchlist.contains(&media_id)
    }

    pub fn is_liked(&self, media_id: u64) -> bool {
        self.likes.contains(&media_id)
    }

    pub fn add_bookmark(&mut self, media: ApiMedia, kind: MediaCategory) {
        let id = media.id;
        if !self.watchlist.contains(&id) {
            self.watchlist.push(id);
        }
        self.saved_media.insert(id, SavedMediaEntry { media, kind });
    }

    pub fn remove_bookmark(&mut self, media_id: u64) {
        self.watchlist.retain(|id| *id != media_id);
        self.saved_media.remove(&media_id);
    }

    pub fn toggle_bookmark(&mut self, media: ApiMedia, kind: MediaCategory) -> bool {
        let id = media.id;
        if self.is_bookmarked(id) {
            self.remove_bookmark(id);
            false
        } else {
            self.add_bookmark(media, kind);
            true
        }
    }

    pub fn toggle_like(&mut self, media_id: u64) -> bool {
        if let Some(pos) = self.likes.iter().position(|id| *id == media_id) {
            self.likes.remove(pos);
            false
        } else {
            self.likes.push(media_id);
            true
        }
    }

    /// Returns ordered bookmarked items in insertion order.
    pub fn ordered_bookmarks(&self) -> Vec<SavedMediaEntry> {
        self.watchlist
            .iter()
            .filter_map(|id| self.saved_media.get(id).cloned())
            .collect()
    }
}
