pub mod bookmarks_view;
pub mod catalog_view;
pub mod details_view;
pub mod home_view;
pub mod search_view;
pub mod settings_view;

pub use bookmarks_view::render_bookmarks_view;
pub use catalog_view::render_catalog_view;
pub use details_view::render_details_view;
pub use home_view::render_home_view;
pub use search_view::render_search_view;
pub use settings_view::render_settings_view;
