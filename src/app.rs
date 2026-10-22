//! Root GPUI application view (`StreamovieApp`) combining `uni-kit`'s `TitleBar`,
//! a Netflix + Linear vertical dark sidebar (`Home`, `Movies`, `Series`, `Search`,
//! `Bookmarks`, `Settings`), top glassmorphic toolbar, `ImageCache` remote image loader,
//! horizontal `ScrollHandle` carousels, and reactive async data loading for Streamovie + OMDb.

use std::sync::Arc;

use gpui_kit::{
    AnyElement, App, AppContext as _, BoxShadow, Context, Entity, Image, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, div, hsla, point,
    prelude::FluentBuilder as _, px, white,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, TitleBar,
    button::Button,
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement as _,
    sidebar::SidebarToggleButton,
    tag::Tag,
    v_flex,
};

use crate::{
    api::{OmdbClient, StreamovieClient},
    config::AppConfig,
    i18n::{Locale, directional_row},
    models::{ApiGenre, ApiMedia, BrowseSort, LibraryStoreData, MediaCategory, SavedMediaEntry},
    state::{
        CatalogGridState, CatalogMetadataState, DetailsState, HomeFeedState, MediaFilterTab,
        NavSection, SearchState,
    },
    storage::{
        AccentPalette, BookmarkStorage, PersistedSettings, SettingsStorage, ThemeModePreference,
        app_data_dir,
    },
    ui::{
        image_cache::ImageCache,
        views::{
            render_bookmarks_view, render_catalog_view, render_details_view, render_home_view,
            render_search_view, render_settings_view,
        },
    },
};

pub struct StreamovieApp {
    config: AppConfig,
    streamovie_api: StreamovieClient,
    omdb_api: OmdbClient,
    bookmark_storage: BookmarkStorage,
    settings_storage: SettingsStorage,
    image_cache: ImageCache,

    settings: PersistedSettings,
    library: LibraryStoreData,

    section: NavSection,
    sidebar_collapsed: bool,

    home: HomeFeedState,
    home_row_scroll_handles: [ScrollHandle; 4],
    movies_catalog: CatalogGridState,
    series_catalog: CatalogGridState,
    metadata: CatalogMetadataState,

    search: SearchState,
    search_input: Entity<InputState>,

    bookmarks_filter_tab: MediaFilterTab,
    active_details: Option<DetailsState>,

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
                    if val.trim().is_empty() {
                        this.search.results.clear();
                        this.search.error = None;
                        this.search.loading = false;
                        cx.notify();
                    } else if val.trim().chars().count() >= 2 {
                        this.section = NavSection::Search;
                        this.active_details = None;
                        this.trigger_search_now(cx);
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
            sidebar_collapsed: false,
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
        if let Some(details) = self.active_details.take() {
            self.section = details.return_section;
        }
        cx.notify();
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

        self.fetch_omdb_season_guide(
            media_id,
            lookup_title,
            season_index + 1,
            year,
            imdb_id,
            cx,
        );
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

    // --- Paramount+ Left Dock Navigation ---

    fn render_paramount_nav_row(&self, item: NavSection, cx: &mut Context<Self>) -> AnyElement {
        let locale = self.locale();
        let dict = locale.dict();
        let rtl = locale.is_rtl();
        let collapsed = self.sidebar_collapsed;
        let is_active = self.active_details.is_none() && self.section == item;
        let bookmark_count = self.library.watchlist.len();

        let primary = cx.theme().primary; // #0064FF Paramount Blue
        let active_bg = primary.opacity(0.18);
        let hover_bg = hsla(216.0 / 360.0, 0.40, 0.85, 0.08);
        let fg_active = white();
        let fg_muted = hsla(215.0 / 360.0, 0.18, 0.70, 1.0);

        let row_id = SharedString::from(format!("paramount-nav-{}", item.label(dict)));

        div()
            .id(row_id)
            .relative()
            .w_full()
            .px_3()
            .py_2p5()
            .rounded(px(8.))
            .cursor_pointer()
            .when(is_active, |el| el.bg(active_bg))
            .hover(move |s| s.bg(if is_active { active_bg } else { hover_bg }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_section(item, cx);
            }))
            // Paramount+ vertical active indicator bar with luminous blue glow
            .when(is_active, |el| {
                el.child(
                    div()
                        .absolute()
                        .top(px(8.))
                        .bottom(px(8.))
                        .when(!rtl, |bar| bar.left_0())
                        .when(rtl, |bar| bar.right_0())
                        .w(px(3.5))
                        .rounded_full()
                        .bg(primary)
                        .shadow(vec![BoxShadow {
                            offset: point(px(0.), px(0.)),
                            blur_radius: px(10.),
                            spread_radius: px(1.),
                            inset: false,
                            color: primary.opacity(0.85),
                        }]),
                )
            })
            .child(
                directional_row(h_flex(), rtl)
                    .w_full()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .when(collapsed, |row| row.justify_center())
                    .child(
                        directional_row(h_flex(), rtl)
                            .items_center()
                            .gap_3()
                            .min_w_0()
                            .child(
                                Icon::new(item.icon())
                                    .small()
                                    .text_color(if is_active { primary } else { fg_muted }),
                            )
                            .when(!collapsed, |row| {
                                row.child(
                                    div()
                                        .text_xs()
                                        .when(is_active, |t| t.font_extrabold().text_color(fg_active))
                                        .when(!is_active, |t| t.font_semibold().text_color(fg_muted))
                                        .whitespace_nowrap()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(item.label(dict).to_uppercase()),
                                )
                            }),
                    )
                    .when(
                        !collapsed && item == NavSection::Bookmarks && bookmark_count > 0,
                        |row| {
                            row.child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(primary)
                                    .text_xs()
                                    .font_bold()
                                    .text_color(white())
                                    .shadow(vec![BoxShadow {
                                        offset: point(px(0.), px(2.)),
                                        blur_radius: px(6.),
                                        spread_radius: px(0.),
                                        inset: false,
                                        color: primary.opacity(0.60),
                                    }])
                                    .child(bookmark_count.to_string()),
                            )
                        },
                    ),
            )
            .into_any_element()
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let locale = self.locale();
        let _dict = locale.dict();
        let rtl = locale.is_rtl();
        let collapsed = self.sidebar_collapsed;

        // Paramount+ ordered navigation: Search, Home, Series, Movies
        let primary_items = [
            NavSection::Search,
            NavSection::Home,
            NavSection::Series,
            NavSection::Movies,
        ];
        let mut primary_rows = Vec::with_capacity(primary_items.len());
        for sec in primary_items {
            primary_rows.push(self.render_paramount_nav_row(sec, cx));
        }

        let library_items = [NavSection::Bookmarks, NavSection::Settings];
        let mut library_rows = Vec::with_capacity(library_items.len());
        for sec in library_items {
            library_rows.push(self.render_paramount_nav_row(sec, cx));
        }

        let sidebar_width = if collapsed { px(72.) } else { px(240.) };

        v_flex()
            .w(sidebar_width)
            .h_full()
            .flex_shrink_0()
            .bg(cx.theme().sidebar)
            .when(!rtl, |el| el.border_r_1())
            .when(rtl, |el| el.border_l_1())
            .border_color(cx.theme().sidebar_border)
            .justify_between()
            .p_3()
            // Top Brand Header + Navigation Groups
            .child(
                v_flex()
                    .w_full()
                    .gap_5()
                    // Paramount+ Brand Header
                    .child(
                        directional_row(h_flex(), rtl)
                            .w_full()
                            .px_2()
                            .py_2()
                            .items_center()
                            .gap_3()
                            .when(collapsed, |r| r.justify_center())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size_9()
                                    .flex_shrink_0()
                                    .rounded(px(8.))
                                    .bg(cx.theme().primary)
                                    .text_color(white())
                                    .shadow(vec![BoxShadow {
                                        offset: point(px(0.), px(4.)),
                                        blur_radius: px(14.),
                                        spread_radius: px(0.),
                                        inset: false,
                                        color: cx.theme().primary.opacity(0.60),
                                    }])
                                    .child(Icon::new(IconName::Play)),
                            )
                            .when(!collapsed, |row| {
                                row.child(
                                    v_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .when(rtl, |col| col.items_end().text_right())
                                        .child(
                                            div()
                                                .text_base()
                                                .font_extrabold()
                                                .text_color(cx.theme().foreground)
                                                .child("PARAMOUNT+"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_bold()
                                                .text_color(cx.theme().primary)
                                                .child("STREAMOVIE"),
                                        ),
                                )
                            }),
                    )
                    // Primary Navigation Group (EXPLORE)
                    .child(
                        v_flex()
                            .w_full()
                            .gap_1()
                            .when(!collapsed, |col| {
                                col.child(
                                    div()
                                        .px_3()
                                        .pb_1()
                                        .text_size(px(10.))
                                        .font_extrabold()
                                        .text_color(cx.theme().sidebar_foreground.opacity(0.45))
                                        .when(rtl, |t| t.text_right())
                                        .child("EXPLORE"),
                                )
                            })
                            .children(primary_rows),
                    )
                    // Library & Settings Group (MY STUFF)
                    .child(
                        v_flex()
                            .w_full()
                            .gap_1()
                            .when(!collapsed, |col| {
                                col.child(
                                    div()
                                        .px_3()
                                        .pb_1()
                                        .text_size(px(10.))
                                        .font_extrabold()
                                        .text_color(cx.theme().sidebar_foreground.opacity(0.45))
                                        .when(rtl, |t| t.text_right())
                                        .child("MY STUFF"),
                                )
                            })
                            .children(library_rows),
                    ),
            )
            // Sidebar Footer: Active Locale
            .child(
                div()
                    .w_full()
                    .p_2p5()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(cx.theme().sidebar_border)
                    .bg(hsla(220.0 / 360.0, 0.38, 0.08, 0.60))
                    .child(
                        directional_row(h_flex(), rtl)
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .when(collapsed, |r| r.justify_center())
                            .child(
                                directional_row(h_flex(), rtl)
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Icon::new(IconName::Globe)
                                            .small()
                                            .text_color(cx.theme().primary),
                                    )
                                    .when(!collapsed, |r| {
                                        r.child(
                                            div()
                                                .text_xs()
                                                .font_medium()
                                                .text_color(
                                                    cx.theme().sidebar_foreground.opacity(0.75),
                                                )
                                                .child(locale.label()),
                                        )
                                    }),
                            )
                            .when(!collapsed, |r| {
                                r.child(
                                    Tag::primary()
                                        .small()
                                        .outline()
                                        .child("Paramount+"),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    fn render_top_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let locale = self.locale();
        let dict = locale.dict();
        let rtl = locale.is_rtl();
        let side = locale.sidebar_side();

        let active_title = if let Some(details) = self.active_details.as_ref() {
            details.media.display_title()
        } else {
            self.section.label(dict).to_uppercase()
        };

        let mode_icon = match self.settings.theme_mode {
            ThemeModePreference::Dark => IconName::Moon,
            ThemeModePreference::Light => IconName::Sun,
            ThemeModePreference::System => IconName::Palette,
        };

        directional_row(h_flex(), rtl)
            .w_full()
            .px_6()
            .py_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .justify_between()
            .items_center()
            .gap_4()
            // Left/Start cluster: Sidebar collapse toggle + section heading
            .child(
                directional_row(h_flex(), rtl)
                    .items_center()
                    .gap_3()
                    .min_w_0()
                    .child(
                        SidebarToggleButton::new()
                            .side(side)
                            .collapsed(self.sidebar_collapsed)
                            .accessibility_label(if self.sidebar_collapsed {
                                dict.nav.expand_sidebar
                            } else {
                                dict.nav.collapse_sidebar
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sidebar_collapsed = !this.sidebar_collapsed;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_extrabold()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(active_title),
                    ),
            )
            // Center cluster: Paramount+ Quick Search Input
            .child(
                div().w(px(360.)).child(
                    Input::new(&self.search_input)
                        .small()
                        .cleanable(true)
                        .prefix(
                            Icon::new(IconName::Search)
                                .small()
                                .text_color(cx.theme().primary),
                        ),
                ),
            )
            // Right/End cluster: Refresh, EN/FA Language Toggle, Theme Mode Cycle
            .child(
                directional_row(h_flex(), rtl)
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("paramount-toolbar-refresh")
                            .small()
                            .outline()
                            .icon(IconName::RefreshCw)
                            .accessibility_label(dict.actions.refresh)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.reload_home_feeds(cx);
                                this.reload_catalog(MediaCategory::Movie, cx);
                                this.reload_catalog(MediaCategory::Series, cx);
                            })),
                    )
                    .child(
                        Button::new("paramount-toolbar-locale")
                            .small()
                            .outline()
                            .icon(IconName::Globe)
                            .label(locale.short_badge())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.toggle_locale(Some(window), cx);
                            })),
                    )
                    .child(
                        Button::new("paramount-toolbar-theme-mode")
                            .small()
                            .outline()
                            .icon(mode_icon)
                            .label(self.settings.theme_mode.label(dict))
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

        let sidebar = self.render_sidebar(cx);
        let toolbar = self.render_top_toolbar(cx);

        let content = if let Some(details) = self.active_details.clone() {
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

        let _ = window;

        div()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .size_full()
                    // Native macOS TitleBar
                    .child(
                        TitleBar::new().child(
                            directional_row(h_flex(), rtl)
                                .w_full()
                                .justify_between()
                                .pr_4()
                                .child(
                                    directional_row(h_flex(), rtl)
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_bold()
                                                .text_color(cx.theme().foreground)
                                                .child(dict.brand.name),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("· Paramount+ Cinema Edition"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(if rtl { "فارسی (RTL)" } else { "English (LTR)" }),
                                ),
                        ),
                    )
                    // Main Workspace Body: Netflix Sidebar + Scrollable Viewport
                    .child(
                        directional_row(h_flex(), rtl)
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .overflow_hidden()
                            .child(sidebar)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .h_full()
                                    .overflow_hidden()
                                    .child(toolbar)
                                    .child(if self.section == NavSection::Settings
                                        && self.active_details.is_none()
                                    {
                                        div()
                                            .flex_1()
                                            .min_h_0()
                                            .p_6()
                                            .overflow_hidden()
                                            .child(content)
                                            .into_any_element()
                                    } else {
                                        div()
                                            .id("streamovie-main-scroll")
                                            .flex_1()
                                            .min_h_0()
                                            .p_6()
                                            .overflow_y_scrollbar()
                                            .child(content)
                                            .into_any_element()
                                    }),
                            ),
                    ),
            )
    }
}
