//! Root GPUI application view for Streamovie.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Placement, Sizable as _, StyledExt as _, TitleBar,
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, Entity, Image, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, ScrollHandle, SharedString, Styled as _, Subscription, Window, div,
    point, px,
};

use crate::{
    api::{OmdbClient, StreamovieClient},
    config::AppConfig,
    download::{self, DownloadHandle, DownloadSnapshot},
    i18n::{Locale, directional_row},
    models::{
        ApiGenre, ApiMedia, BrowseSort, LibraryStoreData, MediaCategory, SavedMediaEntry,
        SourceLink, best_stream_source,
    },
    playback::{self, PlaybackSession},
    state::{
        CatalogGridState, CatalogMetadataState, DetailsState, HomeFeedState, MediaFilterTab,
        NavSection, SearchState,
    },
    storage::{
        AccentPalette, BookmarkStorage, PersistedSettings, SettingsStorage, ThemeModePreference,
        app_data_dir,
    },
    ui::{
        components::render_player_view,
        image_cache::ImageCache,
        views::{
            render_bookmarks_view, render_catalog_view, render_details_view, render_home_view,
            render_search_view, render_settings_view,
        },
    },
};

pub struct StreamovieApp {
    #[allow(dead_code)]
    config: AppConfig,
    streamovie_api: StreamovieClient,
    omdb_api: OmdbClient,
    bookmark_storage: BookmarkStorage,
    settings_storage: SettingsStorage,
    image_cache: ImageCache,

    settings: PersistedSettings,
    library: LibraryStoreData,

    section: NavSection,

    home: HomeFeedState,
    home_row_scroll_handles: [ScrollHandle; 4],
    movies_catalog: CatalogGridState,
    series_catalog: CatalogGridState,
    metadata: CatalogMetadataState,

    search: SearchState,
    search_input: Entity<InputState>,

    bookmarks_filter_tab: MediaFilterTab,
    active_details: Option<DetailsState>,
    playback: Option<PlaybackSession>,
    downloads: Vec<DownloadHandle>,
    download_polling: bool,
    player_polling: bool,
    hovered_card: Option<SharedString>,

    _subscriptions: Vec<Subscription>,
}

impl StreamovieApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = AppConfig::load();
        let streamovie_api = StreamovieClient::new(config.clone());
        let omdb_api = OmdbClient::new(config.clone());
        let bookmark_storage = BookmarkStorage::new();
        let settings_storage = SettingsStorage::new();
        let image_cache = ImageCache::new();

        let settings = settings_storage.load();
        let library = bookmark_storage.load();

        // Apply persisted theme (Dark default, Light, or System) + brand accent palette
        settings.apply_to_uni_kit(Some(window), cx);

        let placeholder = settings.locale.dict().search.placeholder;
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));

        let search_sub = cx.subscribe_in(&search_input, window, {
            let search_input = search_input.clone();
            move |this, _, event: &InputEvent, _window, cx| match event {
                InputEvent::Change => {
                    let val = search_input.read(cx).value().to_string();
                    this.search.query = val.clone();
                    this.search.generation = this.search.generation.wrapping_add(1);
                    let generation = this.search.generation;
                    if val.trim().is_empty() {
                        this.search.results.clear();
                        this.search.error = None;
                        this.search.loading = false;
                        cx.notify();
                    } else if val.trim().chars().count() >= 2 {
                        this.section = NavSection::Search;
                        this.active_details = None;
                        this.close_playback(cx);
                        cx.spawn(async move |this, cx| {
                            cx.background_executor()
                                .timer(Duration::from_millis(280))
                                .await;
                            let _ = this.update(cx, |this, cx| {
                                if this.search.generation == generation {
                                    this.trigger_search_now(cx);
                                }
                            });
                        })
                        .detach();
                    }
                }
                InputEvent::PressEnter { .. } => {
                    this.section = NavSection::Search;
                    this.active_details = None;
                    this.trigger_search_now(cx);
                }
                _ => {}
            }
        });

        let mut app = Self {
            config,
            streamovie_api,
            omdb_api,
            bookmark_storage,
            settings_storage,
            image_cache,
            settings,
            library,
            section: NavSection::Home,
            home: HomeFeedState {
                loading: true,
                ..Default::default()
            },
            home_row_scroll_handles: [
                ScrollHandle::new(),
                ScrollHandle::new(),
                ScrollHandle::new(),
                ScrollHandle::new(),
            ],
            movies_catalog: CatalogGridState::new(MediaCategory::Movie),
            series_catalog: CatalogGridState::new(MediaCategory::Series),
            metadata: CatalogMetadataState::default(),
            search: SearchState::default(),
            search_input,
            bookmarks_filter_tab: MediaFilterTab::All,
            active_details: None,
            playback: None,
            downloads: Vec::new(),
            download_polling: false,
            player_polling: false,
            hovered_card: None,
            _subscriptions: vec![search_sub],
        };

        // Kick off initial background catalog fetches
        app.reload_home_feeds(cx);
        app.load_genres(cx);
        app.reload_catalog(MediaCategory::Movie, cx);
        app.reload_catalog(MediaCategory::Series, cx);

        app
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    // --- Accessors for Views ---

    #[allow(dead_code)]
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn settings(&self) -> &PersistedSettings {
        &self.settings
    }

    pub fn locale(&self) -> Locale {
        self.settings.locale
    }

    pub fn home_state(&self) -> &HomeFeedState {
        &self.home
    }

    pub fn home_row_scroll_handle(&self, index: usize) -> &ScrollHandle {
        &self.home_row_scroll_handles[index.min(self.home_row_scroll_handles.len() - 1)]
    }

    pub fn card_is_hovered(&self, id: &SharedString) -> bool {
        self.hovered_card.as_ref() == Some(id)
    }

    pub fn set_card_hovered(&mut self, id: &SharedString, hovered: bool, cx: &mut Context<Self>) {
        if hovered {
            if self.hovered_card.as_ref() == Some(id) {
                return;
            }
            self.hovered_card = Some(id.clone());
        } else if self.hovered_card.as_ref() == Some(id) {
            self.hovered_card = None;
        } else {
            return;
        }
        cx.notify();
    }

    pub fn scroll_home_row(&mut self, index: usize, direction: isize, cx: &mut Context<Self>) {
        let handle =
            &self.home_row_scroll_handles[index.min(self.home_row_scroll_handles.len() - 1)];
        let current = handle.offset();
        let step = px(620.0 * (direction as f32));
        let next_x = (current.x - step).min(px(0.0));
        handle.set_offset(point(next_x, current.y));
        cx.notify();
    }

    pub fn resolve_image(
        &self,
        primary_url: &str,
        fallback_url: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Option<Arc<Image>> {
        self.image_cache.get_or_load(primary_url, fallback_url, cx)
    }

    pub fn catalog_state(&self, category: MediaCategory) -> &CatalogGridState {
        match category {
            MediaCategory::Movie => &self.movies_catalog,
            MediaCategory::Series => &self.series_catalog,
        }
    }

    pub fn catalog_state_mut(&mut self, category: MediaCategory) -> &mut CatalogGridState {
        match category {
            MediaCategory::Movie => &mut self.movies_catalog,
            MediaCategory::Series => &mut self.series_catalog,
        }
    }

    pub fn genres(&self) -> &[ApiGenre] {
        &self.metadata.genres
    }

    pub fn search_state(&self) -> &SearchState {
        &self.search
    }

    pub fn search_input(&self) -> &Entity<InputState> {
        &self.search_input
    }

    pub fn bookmarks_filter_tab(&self) -> MediaFilterTab {
        self.bookmarks_filter_tab
    }

    pub fn is_bookmarked(&self, media_id: u64) -> bool {
        self.library.is_bookmarked(media_id)
    }

    pub fn is_liked(&self, media_id: u64) -> bool {
        self.library.is_liked(media_id)
    }

    pub fn ordered_bookmarks(&self) -> Vec<SavedMediaEntry> {
        self.library.ordered_bookmarks()
    }

    pub fn storage_dir_display(&self) -> String {
        app_data_dir().display().to_string()
    }

    // --- Navigation & UI Actions ---

    pub fn select_section(&mut self, section: NavSection, cx: &mut Context<Self>) {
        self.close_playback(cx);
        self.section = section;
        self.active_details = None;
        cx.notify();
    }

    pub fn navigate_to_catalog_with_sort(
        &mut self,
        section: NavSection,
        sort: BrowseSort,
        cx: &mut Context<Self>,
    ) {
        self.section = section;
        self.active_details = None;
        match section {
            NavSection::Movies => self.set_catalog_sort(MediaCategory::Movie, sort, cx),
            NavSection::Series => self.set_catalog_sort(MediaCategory::Series, sort, cx),
            _ => cx.notify(),
        }
    }

    pub fn set_spotlight_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.home.spotlight.len() {
            self.home.spotlight_index = index;
            cx.notify();
        }
    }

    #[allow(dead_code)]
    pub fn step_spotlight(&mut self, delta: isize, cx: &mut Context<Self>) {
        let len = self.home.spotlight.len();
        if len == 0 {
            return;
        }
        let current = self.home.spotlight_index as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        self.home.spotlight_index = next;
        cx.notify();
    }

    pub fn set_bookmarks_filter_tab(&mut self, tab: MediaFilterTab, cx: &mut Context<Self>) {
        self.bookmarks_filter_tab = tab;
        cx.notify();
    }

    pub fn set_search_filter_tab(&mut self, tab: MediaFilterTab, cx: &mut Context<Self>) {
        self.search.filter_tab = tab;
        cx.notify();
    }

    // --- Bookmarks & Likes Persistence ---

    pub fn toggle_bookmark(
        &mut self,
        media: ApiMedia,
        category: MediaCategory,
        cx: &mut Context<Self>,
    ) -> bool {
        let added = self.library.toggle_bookmark(media, category);
        let _ = self.bookmark_storage.save(&self.library);
        cx.notify();
        added
    }

    #[allow(dead_code)]
    pub fn toggle_like(&mut self, media_id: u64, cx: &mut Context<Self>) -> bool {
        let liked = self.library.toggle_like(media_id);
        let _ = self.bookmark_storage.save(&self.library);
        cx.notify();
        liked
    }

    // --- Settings & Theme / Locale Mutations ---

    pub fn set_locale(
        &mut self,
        locale: Locale,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        self.settings.locale = locale;
        let placeholder = locale.dict().search.placeholder;
        if let Some(win) = window {
            self.search_input.update(cx, |input, cx| {
                input.set_placeholder(placeholder, win, cx);
            });
            self.settings.apply_to_uni_kit(Some(win), cx);
        } else {
            self.settings.apply_to_uni_kit(None, cx);
        }
        let _ = self.settings_storage.save(&self.settings);
        cx.notify();
    }

    pub fn toggle_locale(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        let next = self.settings.locale.toggle();
        self.set_locale(next, window, cx);
    }

    pub fn set_theme_mode(
        &mut self,
        mode: ThemeModePreference,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        self.settings.theme_mode = mode;
        self.settings.apply_to_uni_kit(window, cx);
        let _ = self.settings_storage.save(&self.settings);
        cx.notify();
    }

    pub fn cycle_theme_mode(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        let next = match self.settings.theme_mode {
            ThemeModePreference::Dark => ThemeModePreference::Light,
            ThemeModePreference::Light => ThemeModePreference::System,
            ThemeModePreference::System => ThemeModePreference::Dark,
        };
        self.set_theme_mode(next, window, cx);
    }

    pub fn set_accent_palette(
        &mut self,
        palette: AccentPalette,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        self.settings.accent_palette = palette;
        self.settings.apply_to_uni_kit(window, cx);
        let _ = self.settings_storage.save(&self.settings);
        cx.notify();
    }

    pub fn reset_settings(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        self.settings = PersistedSettings::default();
        self.settings.apply_to_uni_kit(window, cx);
        let _ = self.settings_storage.save(&self.settings);
        cx.notify();
    }

    pub fn save_settings_now(&mut self, cx: &mut Context<Self>) {
        let _ = self.settings_storage.save(&self.settings);
        cx.notify();
    }

    // --- Async Data Fetching (Streamovie Catalog + OMDb) ---

    pub fn reload_home_feeds(&mut self, cx: &mut Context<Self>) {
        self.home.loading = true;
        self.home.error = None;
        cx.notify();

        let client = self.streamovie_api.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let latest_movies = client.fetch_movies(0, 0, BrowseSort::Created)?;
                    let latest_series = client.fetch_series(0, 0, BrowseSort::Created)?;
                    let imdb_movies = client
                        .fetch_movies(0, 32, BrowseSort::Imdb)
                        .unwrap_or_default();
                    let imdb_series = client
                        .fetch_series(0, 32, BrowseSort::Imdb)
                        .unwrap_or_default();
                    let spotlight = client.fetch_featured_spotlight(&latest_series);
                    Ok::<_, anyhow::Error>((
                        spotlight,
                        latest_movies,
                        latest_series,
                        imdb_movies,
                        imdb_series,
                    ))
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                this.home.loading = false;
                match result {
                    Ok((spotlight, latest_movies, latest_series, imdb_movies, imdb_series)) => {
                        this.home.spotlight = spotlight;
                        this.home.spotlight_index = 0;
                        this.home.latest_movies = latest_movies;
                        this.home.latest_series = latest_series;
                        this.home.imdb_movies = imdb_movies;
                        this.home.imdb_series = imdb_series;
                        this.home.error = None;
                    }
                    Err(err) => {
                        this.home.error = Some(err.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn load_genres(&mut self, cx: &mut Context<Self>) {
        let client = self.streamovie_api.clone();
        cx.spawn(async move |this, cx| {
            let genres = cx
                .background_executor()
                .spawn(async move { client.fetch_genres() })
                .await;
            if let Ok(list) = genres {
                let _ = this.update(cx, |this, cx| {
                    this.metadata.genres = list;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn set_catalog_sort(
        &mut self,
        category: MediaCategory,
        sort: BrowseSort,
        cx: &mut Context<Self>,
    ) {
        let state = self.catalog_state_mut(category);
        if state.sort == sort && !state.items.is_empty() {
            return;
        }
        state.sort = sort;
        state.page = 0;
        self.reload_catalog(category, cx);
    }

    pub fn set_catalog_genre(
        &mut self,
        category: MediaCategory,
        genre_id: u64,
        cx: &mut Context<Self>,
    ) {
        let state = self.catalog_state_mut(category);
        if state.genre_id == genre_id && !state.items.is_empty() {
            return;
        }
        state.genre_id = genre_id;
        state.page = 0;
        self.reload_catalog(category, cx);
    }

    #[allow(dead_code)]
    pub fn reset_catalog_filters(&mut self, category: MediaCategory, cx: &mut Context<Self>) {
        let state = self.catalog_state_mut(category);
        state.sort = BrowseSort::Created;
        state.genre_id = 0;
        state.page = 0;
        self.reload_catalog(category, cx);
    }

    pub fn reload_catalog(&mut self, category: MediaCategory, cx: &mut Context<Self>) {
        let (genre_id, sort) = {
            let state = self.catalog_state_mut(category);
            state.loading = true;
            state.error = None;
            state.page = 0;
            (state.genre_id, state.sort)
        };
        cx.notify();

        let client = self.streamovie_api.clone();
        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move {
                    match category {
                        MediaCategory::Movie => client.fetch_movies(0, genre_id, sort),
                        MediaCategory::Series => client.fetch_series(0, genre_id, sort),
                    }
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                let state = this.catalog_state_mut(category);
                state.loading = false;
                match res {
                    Ok(items) => {
                        state.has_more = !items.is_empty();
                        state.items = items;
                        state.error = None;
                    }
                    Err(err) => {
                        state.error = Some(err.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn load_more_catalog(&mut self, category: MediaCategory, cx: &mut Context<Self>) {
        let (next_page, genre_id, sort) = {
            let state = self.catalog_state_mut(category);
            if state.loading || state.loading_more || !state.has_more {
                return;
            }
            state.loading_more = true;
            (state.page + 1, state.genre_id, state.sort)
        };
        cx.notify();

        let client = self.streamovie_api.clone();
        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move {
                    match category {
                        MediaCategory::Movie => client.fetch_movies(next_page, genre_id, sort),
                        MediaCategory::Series => client.fetch_series(next_page, genre_id, sort),
                    }
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                let state = this.catalog_state_mut(category);
                state.loading_more = false;
                match res {
                    Ok(more) => {
                        if more.is_empty() {
                            state.has_more = false;
                        } else {
                            state.page = next_page;
                            for item in more {
                                if !state.items.iter().any(|existing| existing.id == item.id) {
                                    state.items.push(item);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        state.error = Some(err.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn trigger_search_now(&mut self, cx: &mut Context<Self>) {
        let query = self.search_input.read(cx).value().trim().to_string();
        self.search.query = query.clone();
        if query.is_empty() {
            self.search.results.clear();
            self.search.loading = false;
            self.search.error = None;
            cx.notify();
            return;
        }

        self.search.loading = true;
        self.search.error = None;
        cx.notify();

        let client = self.streamovie_api.clone();
        let expected_query = query.clone();
        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move { client.search(&query) })
                .await;

            let _ = this.update(cx, |this, cx| {
                if this.search.query != expected_query {
                    return;
                }
                this.search.loading = false;
                match res {
                    Ok(items) => {
                        this.search.results = items;
                        this.search.error = None;
                    }
                    Err(err) => {
                        this.search.error = Some(err.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    // --- Title Details + Seasons + OMDb Enrichment ---

    pub fn open_details(
        &mut self,
        media: ApiMedia,
        category: MediaCategory,
        cx: &mut Context<Self>,
    ) {
        let media_id = media.id;
        let lookup_title = media.english_lookup_title();
        let year = media.release_year();
        let imdb_id = media.imdb_id();

        self.active_details = Some(DetailsState::new(media, category, self.section));
        cx.notify();

        // 1. Fetch OMDb enrichment
        let omdb_client = self.omdb_api.clone();
        let title_for_omdb = lookup_title.clone();
        let year_for_omdb = year.clone();
        let imdb_for_omdb = imdb_id.clone();

        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move {
                    omdb_client.fetch_details(
                        &title_for_omdb,
                        year_for_omdb.as_deref(),
                        imdb_for_omdb.as_deref(),
                        category,
                    )
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                let Some(details) = this.active_details.as_mut() else {
                    return;
                };
                if details.media.id != media_id {
                    return;
                }
                details.omdb_loading = false;
                match res {
                    Ok(omdb) => {
                        details.omdb_details = Some(omdb);
                    }
                    Err(err) => {
                        details.omdb_error = Some(err.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();

        // 2. If series, fetch Streamovie seasons & episodes + Season 1 OMDb guide
        if category == MediaCategory::Series {
            let streamovie_client = self.streamovie_api.clone();
            cx.spawn(async move |this, cx| {
                let seasons_res = cx
                    .background_executor()
                    .spawn(async move { streamovie_client.fetch_seasons(media_id) })
                    .await;

                let _ = this.update(cx, |this, cx| {
                    let Some(details) = this.active_details.as_mut() else {
                        return;
                    };
                    if details.media.id != media_id {
                        return;
                    }
                    details.seasons_loading = false;
                    match seasons_res {
                        Ok(seasons) => {
                            details.seasons = seasons;
                        }
                        Err(err) => {
                            details.seasons_error = Some(err.to_string());
                        }
                    }
                    cx.notify();
                });
            })
            .detach();

            self.fetch_omdb_season_guide(media_id, lookup_title, 1, year, imdb_id, cx);
        }
    }

    pub fn close_details(&mut self, cx: &mut Context<Self>) {
        self.close_playback(cx);
        if let Some(details) = self.active_details.take() {
            self.section = details.return_section;
        }
        cx.notify();
    }

    pub fn watch_title(
        &mut self,
        media: ApiMedia,
        category: MediaCategory,
        cx: &mut Context<Self>,
    ) {
        if category == MediaCategory::Series {
            let ready = self.active_details.as_ref().is_some_and(|details| {
                details.media.id == media.id
                    && !details.seasons_loading
                    && !details.seasons.is_empty()
            });
            if ready {
                self.play_loaded_series(cx);
            } else {
                self.open_details(media, category, cx);
                if let Some(details) = self.active_details.as_mut() {
                    details.autoplay_when_ready = true;
                }
                cx.notify();
            }
            return;
        }
        let title = media.display_title();
        let sources = media.source_links();
        self.watch_sources(title, sources, cx);
    }

    pub fn watch_sources(
        &mut self,
        heading: String,
        sources: Vec<SourceLink>,
        cx: &mut Context<Self>,
    ) {
        self.close_playback(cx);
        let message = self.locale().dict().player.unavailable_body.to_string();
        self.playback = Some(match best_stream_source(&sources) {
            Some(source) => PlaybackSession::for_url(heading, source.url.clone()),
            None => PlaybackSession::unavailable(heading, message),
        });
        self.ensure_player_poll(cx);
        cx.notify();
    }

    pub fn close_playback(&mut self, cx: &mut Context<Self>) {
        if let Some(session) = self.playback.take() {
            if let Some(webview) = session.webview {
                webview.update(cx, |webview, _| webview.hide());
            }
        }
    }

    fn play_loaded_series(&mut self, cx: &mut Context<Self>) {
        let Some(details) = self.active_details.as_ref() else {
            return;
        };
        let series_title = details.media.display_title();
        let episode = details
            .seasons
            .first()
            .and_then(|season| season.episode_list().into_iter().next());
        match episode {
            Some(episode) => {
                let heading = format!("{series_title} · {}", episode.title);
                let sources = episode.sources;
                self.watch_sources(heading, sources, cx);
            }
            None => self.watch_sources(series_title, Vec::new(), cx),
        }
    }

    fn settle_autoplay(&mut self, cx: &mut Context<Self>) {
        let ready = self
            .active_details
            .as_ref()
            .is_some_and(|details| details.autoplay_when_ready && !details.seasons_loading);
        if !ready {
            return;
        }
        if let Some(details) = self.active_details.as_mut() {
            details.autoplay_when_ready = false;
        }
        self.play_loaded_series(cx);
    }

    fn ensure_player_mounted(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let should_mount = self.playback.as_ref().is_some_and(|session| {
            session.stream_url.is_some()
                && session.webview.is_none()
                && session.mount_error.is_none()
        });
        if !should_mount {
            return;
        }
        let (url, title, flag) = {
            let session = self.playback.as_ref().unwrap();
            (
                session.stream_url.clone().unwrap(),
                session.heading.clone(),
                session.close_requested.clone(),
            )
        };
        match playback::mount_player(window, cx, &url, &title, flag) {
            Ok(webview) => {
                if let Some(session) = self.playback.as_mut() {
                    session.webview = Some(webview);
                }
            }
            Err(err) => {
                if let Some(session) = self.playback.as_mut() {
                    session.message = err.clone();
                    session.mount_error = Some(err);
                }
            }
        }
    }

    pub fn download_snapshot(&self, key: &str) -> Option<DownloadSnapshot> {
        self.downloads
            .iter()
            .find(|handle| handle.key() == key)
            .map(DownloadHandle::snapshot)
    }

    pub fn start_episode_download(
        &mut self,
        key: String,
        title: String,
        sources: Vec<SourceLink>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dict = self.locale().dict();
        if self
            .download_snapshot(&key)
            .is_some_and(|job| job.phase == download::DownloadPhase::Running)
        {
            return;
        }
        let Some(source) = crate::models::best_download_source(&sources) else {
            window.push_notification(dict.player.no_download, cx);
            return;
        };
        let extension = source.file_extension();
        match download::start_download(key.clone(), title, source.url.clone(), extension) {
            Ok(handle) => {
                self.downloads.retain(|existing| existing.key() != key);
                self.downloads.push(handle);
                self.ensure_download_poll(cx);
                cx.notify();
            }
            Err(err) => window.push_notification(err.to_string(), cx),
        }
    }

    pub fn cancel_download(&mut self, key: &str, cx: &mut Context<Self>) {
        if let Some(handle) = self.downloads.iter().find(|handle| handle.key() == key) {
            handle.cancel();
        }
        cx.notify();
    }

    pub fn reveal_download(&mut self, key: &str, cx: &mut Context<Self>) {
        if let Some(path) = self
            .download_snapshot(key)
            .and_then(|snapshot| snapshot.path)
        {
            download::reveal_path(&path);
        }
        cx.notify();
    }

    fn ensure_player_poll(&mut self, cx: &mut Context<Self>) {
        if self.player_polling {
            return;
        }
        self.player_polling = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                let alive = this
                    .update(cx, |this, cx| {
                        if this
                            .playback
                            .as_ref()
                            .is_some_and(|session| session.consume_close())
                        {
                            this.close_playback(cx);
                        }
                        let alive = this.playback.is_some();
                        if !alive {
                            this.player_polling = false;
                        }
                        cx.notify();
                        alive
                    })
                    .unwrap_or(false);
                if !alive {
                    break;
                }
            }
        })
        .detach();
    }

    fn ensure_download_poll(&mut self, cx: &mut Context<Self>) {
        if self.download_polling {
            return;
        }
        self.download_polling = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(400))
                    .await;
                let running = this
                    .update(cx, |this, cx| {
                        let running = this.downloads.iter().any(DownloadHandle::is_running);
                        if !running {
                            this.download_polling = false;
                        }
                        cx.notify();
                        running
                    })
                    .unwrap_or(false);
                if !running {
                    break;
                }
            }
        })
        .detach();
    }

    pub fn select_season(&mut self, season_index: usize, cx: &mut Context<Self>) {
        let Some(details) = self.active_details.as_mut() else {
            return;
        };
        if season_index >= details.seasons.len() || details.selected_season_index == season_index {
            return;
        }
        details.selected_season_index = season_index;
        let media_id = details.media.id;
        let lookup_title = details.media.english_lookup_title();
        let year = details.media.release_year();
        let imdb_id = details
            .omdb_details
            .as_ref()
            .and_then(|d| d.imdb_id.clone())
            .or_else(|| details.media.imdb_id());
        cx.notify();

        self.fetch_omdb_season_guide(media_id, lookup_title, season_index + 1, year, imdb_id, cx);
    }

    fn fetch_omdb_season_guide(
        &mut self,
        media_id: u64,
        title: String,
        season_number: usize,
        year: Option<String>,
        imdb_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if let Some(details) = self.active_details.as_mut() {
            details.omdb_season_loading = true;
        }
        let omdb_client = self.omdb_api.clone();
        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move {
                    omdb_client.fetch_season(
                        &title,
                        season_number,
                        year.as_deref(),
                        imdb_id.as_deref(),
                    )
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                let Some(details) = this.active_details.as_mut() else {
                    return;
                };
                if details.media.id != media_id
                    || details.selected_season_index + 1 != season_number
                {
                    return;
                }
                details.omdb_season_loading = false;
                if let Ok(season_data) = res {
                    details.omdb_season = Some(season_data);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn render_nav_row(&self, item: NavSection, cx: &mut Context<Self>) -> AnyElement {
        let locale = self.locale();
        let dict = locale.dict();
        let rtl = locale.is_rtl();
        let is_active =
            self.playback.is_none() && self.active_details.is_none() && self.section == item;
        let label = item.label(dict);
        let row_id = SharedString::from(format!("nav-{label}"));
        let icon_color = if is_active {
            cx.theme().foreground
        } else {
            cx.theme().muted_foreground.opacity(0.72)
        };

        Button::new(row_id)
            .text()
            .with_size(px(34.))
            .w(px(44.))
            .h(px(44.))
            .rounded_full()
            .icon(item.icon())
            .text_color(icon_color)
            .tooltip(label)
            .tooltip_placement(if rtl {
                Placement::Left
            } else {
                Placement::Right
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_section(item, cx);
            }))
            .into_any_element()
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let items = [
            NavSection::Search,
            NavSection::Home,
            NavSection::Movies,
            NavSection::Series,
            NavSection::Bookmarks,
            NavSection::Settings,
        ];

        v_flex()
            .w(px(72.))
            .h_full()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .gap_3()
            .children(items.into_iter().map(|item| self.render_nav_row(item, cx)))
            .into_any_element()
    }

    fn render_top_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let locale = self.locale();
        let dict = locale.dict();
        let rtl = locale.is_rtl();
        let mode_icon = match self.settings.theme_mode {
            ThemeModePreference::Dark => IconName::Moon,
            ThemeModePreference::Light => IconName::Sun,
            ThemeModePreference::System => IconName::Palette,
        };

        directional_row(h_flex(), rtl)
            .w_full()
            .px_6()
            .h(px(64.))
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                div().w(px(460.)).max_w_full().child(
                    Input::new(&self.search_input).cleanable(true).prefix(
                        Icon::new(IconName::Search)
                            .small()
                            .text_color(cx.theme().muted_foreground),
                    ),
                ),
            )
            .child(
                directional_row(h_flex(), rtl)
                    .gap_1()
                    .child(
                        Button::new("toolbar-locale")
                            .small()
                            .outline()
                            .label(locale.short_badge())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.toggle_locale(Some(window), cx);
                            })),
                    )
                    .child(
                        Button::new("toolbar-theme")
                            .small()
                            .outline()
                            .icon(mode_icon)
                            .tooltip(self.settings.theme_mode.label(dict))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cycle_theme_mode(Some(window), cx);
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl Render for StreamovieApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let locale = self.locale();
        let dict = locale.dict();
        let rtl = locale.is_rtl();

        if self
            .playback
            .as_ref()
            .is_some_and(|session| session.consume_close())
        {
            self.close_playback(cx);
        }
        self.settle_autoplay(cx);
        self.ensure_player_mounted(window, cx);

        let playing = self.playback.is_some();
        let player = self
            .playback
            .as_ref()
            .map(|session| render_player_view(self, session, cx));
        let sidebar = if playing {
            None
        } else {
            Some(self.render_sidebar(cx))
        };
        let toolbar = if playing {
            None
        } else {
            Some(self.render_top_toolbar(cx))
        };

        let content = if playing {
            player.unwrap_or_else(|| div().into_any_element())
        } else if let Some(details) = self.active_details.clone() {
            render_details_view(self, &details, cx)
        } else {
            match self.section {
                NavSection::Home => render_home_view(self, cx),
                NavSection::Movies => render_catalog_view(self, MediaCategory::Movie, cx),
                NavSection::Series => render_catalog_view(self, MediaCategory::Series, cx),
                NavSection::Search => render_search_view(self, cx),
                NavSection::Bookmarks => render_bookmarks_view(self, cx),
                NavSection::Settings => render_settings_view(self, cx).into_any_element(),
            }
        };

        div()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .size_full()
                    .child(
                        TitleBar::new().child(
                            div()
                                .pl_2()
                                .text_xs()
                                .font_semibold()
                                .text_color(cx.theme().muted_foreground)
                                .child(dict.brand.name),
                        ),
                    )
                    .child(
                        directional_row(h_flex(), rtl)
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .overflow_hidden()
                            .children(sidebar)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .h_full()
                                    .overflow_hidden()
                                    .children(toolbar)
                                    .child(if playing {
                                        div()
                                            .flex_1()
                                            .min_h_0()
                                            .overflow_hidden()
                                            .child(content)
                                            .into_any_element()
                                    } else if self.section == NavSection::Settings
                                        && self.active_details.is_none()
                                    {
                                        div()
                                            .flex_1()
                                            .min_h_0()
                                            .px_8()
                                            .py_6()
                                            .overflow_hidden()
                                            .child(content)
                                            .into_any_element()
                                    } else {
                                        div()
                                            .id("streamovie-main-scroll")
                                            .flex_1()
                                            .min_h_0()
                                            .overflow_y_scrollbar()
                                            .child(content)
                                            .into_any_element()
                                    }),
                            ),
                    ),
            )
    }
}
