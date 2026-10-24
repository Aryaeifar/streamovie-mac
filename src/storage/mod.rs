//! Local disk storage for bookmarks (`bookmarks.json`) and user preferences (`settings.json`)
//! inside `~/Library/Application Support/com.streamovie.mac/`.

pub mod bookmarks;
pub mod history;
pub mod settings;

use std::{fs, path::PathBuf};

pub use bookmarks::BookmarkStorage;
pub use history::{ContinueEntry, ContinueStorage, ContinueWatchingData};
pub use settings::{AccentPalette, PersistedSettings, SettingsStorage};

const APP_DATA_FOLDER: &str = "com.streamovie.mac";

/// Resolves and creates the application data directory on macOS:
/// `~/Library/Application Support/com.streamovie.mac/`.
pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join(APP_DATA_FOLDER);
    let _ = fs::create_dir_all(&dir);
    dir
}
