//! Streamovie native macOS desktop application entry point built on `uni-kit` (`gpui-kit`).

mod api;
mod app;
mod config;
mod download;
mod i18n;
mod models;
mod playback;
mod state;
mod storage;
mod ui;

use std::sync::Arc;

use app::StreamovieApp;
use gpui_kit::assets::AllAssets;
use gpui_kit::component::TitleBar;
use gpui_kit::*;

actions!(streamovie, [Quit, ClosePlayer]);

fn main() {
    let app = gpui_kit::application().with_assets(AllAssets);

    app.run(|cx| {
        // Initialize uni-kit (gpui-base + gpui-component + themes + overlays)
        gpui_kit::init(cx);
        i18n::install_fonts(cx);

        // Register HTTP client so GPUI `img()` can fetch remote movie/series posters & backdrops
        if let Ok(http_client) = reqwest_client::ReqwestClient::user_agent("Streamovie-macOS/0.1.0")
        {
            cx.set_http_client(Arc::new(http_client));
        }

        cx.set_app_identity("com.streamovie.mac", "Streamovie");
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("escape", ClosePlayer, None),
        ]);
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
        cx.activate(true);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1360.), px(860.)), cx)),
            window_min_size: Some(Size {
                width: px(960.),
                height: px(640.),
            }),
            ..TitleBar::window_options()
        };

        gpui_kit::open_window(window_options, cx, |window, cx| {
            StreamovieApp::view(window, cx)
        })
        .expect("Failed to open Streamovie macOS window");
    });
}
