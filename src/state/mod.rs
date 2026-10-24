//! Application state types for navigation, catalog feeds, search, title details,
//! OMDb enrichment, bookmarks, and settings.
#![allow(dead_code)]

use gpui_kit::component::IconName;

use crate::{
    i18n::Dictionary,
    models::{ApiGenre, ApiMedia, ApiSeason, BrowseSort, MediaCategory, OmdbDetails, OmdbSeason},
};

/// Top-level sidebar navigation sections required by the desktop specification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavSection {
    #[default]
    Home,
    Movies,
    Series,
    Search,
    Bookmarks,
    Settings,
}

impl NavSection {
    pub const PRIMARY: [Self; 4] = [Self::Home, Self::Movies, Self::Series, Self::Search];
    pub const LIBRARY: [Self; 2] = [Self::Bookmarks, Self::Settings];

    pub fn label(self, dict: &Dictionary) -> &'static str {
        match self {
            Self::Home => dict.nav.home,
            Self::Movies => dict.nav.movies,
            Self::Series => dict.nav.series,
            Self::Search => dict.nav.search,
            Self::Bookmarks => dict.nav.bookmarks,
            Self::Settings => dict.nav.settings,
        }
    }

    pub fn subtitle(self, dict: &Dictionary) -> &'static str {
        match self {
            Self::Home => dict.home.trending_subtitle,
            Self::Movies => dict.home.movies_subtitle,
            Self::Series => dict.home.series_subtitle,
            Self::Search => dict.search.hint,
            Self::Bookmarks => dict.bookmarks.subtitle,
            Self::Settings => dict.settings.description,
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Self::Home => IconName::LayoutDashboard,
            Self::Movies => IconName::GalleryVerticalEnd,
            Self::Series => IconName::Frame,
            Self::Search => IconName::Search,
            Self::Bookmarks => IconName::BookOpen,
            Self::Settings => IconName::Settings2,
        }
    }
}

/// Filter tab on the Search and Bookmarks screens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaFilterTab {
    #[default]
    All,
    Movies,
    Series,
}

impl MediaFilterTab {
    pub const ALL_TABS: [Self; 3] = [Self::All, Self::Movies, Self::Series];

    pub fn index(self) -> usize {
        match self {
            Self::All => 0,
            Self::Movies => 1,
            Self::Series => 2,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Movies,
            2 => Self::Series,
            _ => Self::All,
        }
    }

    pub fn matches(self, kind: MediaCategory) -> bool {
        match self {
            Self::All => true,
            Self::Movies => kind == MediaCategory::Movie,
            Self::Series => kind == MediaCategory::Series,
        }
    }
}

/// State for the Home view feeds.
#[derive(Clone, Debug, Default)]
pub struct HomeFeedState {
    pub loading: bool,
    pub latest_movies_error: Option<String>,
    pub latest_series_error: Option<String>,
    pub imdb_movies_error: Option<String>,
    pub imdb_series_error: Option<String>,
    pub latest_movies: Vec<ApiMedia>,
    pub latest_series: Vec<ApiMedia>,
    pub imdb_movies: Vec<ApiMedia>,
    pub imdb_series: Vec<ApiMedia>,
}

/// State for the paginated Movies or Series catalog grid.
#[derive(Clone, Debug)]
pub struct CatalogGridState {
    pub category: MediaCategory,
    pub sort: BrowseSort,
    pub genre_id: u64,
    pub page: usize,
    pub loading: bool,
    pub loading_more: bool,
    pub has_more: bool,
    pub error: Option<String>,
    pub items: Vec<ApiMedia>,
}

impl CatalogGridState {
    pub fn new(category: MediaCategory) -> Self {
        Self {
            category,
            sort: BrowseSort::Created,
            genre_id: 0,
            page: 0,
            loading: false,
            loading_more: false,
            has_more: true,
            error: None,
            items: Vec::new(),
        }
    }
}

/// State for the live catalog search screen.
#[derive(Clone, Debug, Default)]
pub struct SearchState {
    pub query: String,
    pub generation: u64,
    pub filter_tab: MediaFilterTab,
    pub loading: bool,
    pub error: Option<String>,
    pub results: Vec<ApiMedia>,
}

/// State for the active Title Details screen (movie or series).
#[derive(Clone, Debug)]
pub struct DetailsState {
    pub media: ApiMedia,
    pub category: MediaCategory,
    pub return_section: NavSection,
    pub seasons: Vec<ApiSeason>,
    pub seasons_loading: bool,
    pub seasons_error: Option<String>,
    pub selected_season_index: usize,
    pub episode_guide_open: bool,
    pub autoplay_when_ready: bool,
    pub omdb_details: Option<OmdbDetails>,
    pub omdb_loading: bool,
    pub omdb_error: Option<String>,
    pub omdb_season: Option<OmdbSeason>,
    pub omdb_season_loading: bool,
}

impl DetailsState {
    pub fn new(media: ApiMedia, category: MediaCategory, return_section: NavSection) -> Self {
        Self {
            media,
            category,
            return_section,
            seasons: Vec::new(),
            seasons_loading: category == MediaCategory::Series,
            seasons_error: None,
            selected_season_index: 0,
            episode_guide_open: true,
            autoplay_when_ready: false,
            omdb_details: None,
            omdb_loading: true,
            omdb_error: None,
            omdb_season: None,
            omdb_season_loading: false,
        }
    }
}

/// Shared catalog metadata cache (genres list).
#[derive(Clone, Debug, Default)]
pub struct CatalogMetadataState {
    pub genres: Vec<ApiGenre>,
}
