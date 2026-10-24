//! Platform player backends.
//!
//! macOS: same-process AVKit `AVPlayerView` window titled "Streamovie".
//! Other targets: honest stub so `cargo check` / Linux CI can compile.

use super::PlayerError;

#[cfg(target_os = "macos")]
mod macos {
    use super::PlayerError;
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::ClassType;
    use objc2_foundation::{MainThreadMarker, NSString, NSURL};

    /// Opens a titled Streamovie window hosting `AVPlayerView` for a progressive media URL.
    ///
    /// Uses dynamic Objective-C messaging so we stay compatible across objc2 framework crate
    /// revisions while still owning a real AVKit window (not `open` / Safari).
    pub fn open_player_window(title: &str, url: &str) -> Result<(), PlayerError> {
        let _mtm = MainThreadMarker::new().ok_or_else(|| {
            PlayerError::LaunchFailed("AVKit player must run on the main thread".into())
        })?;

        unsafe {
            let ns_url_str = NSString::from_str(url);
            let ns_url: Option<Retained<NSURL>> = NSURL::URLWithString(&ns_url_str);
            let ns_url = ns_url
                .ok_or_else(|| PlayerError::LaunchFailed("Invalid media URL".into()))?;

            let av_player_cls = AnyClass::get("AVPlayer").ok_or_else(|| {
                PlayerError::LaunchFailed("AVPlayer class unavailable".into())
            })?;
            let player: *mut AnyObject = msg_send![av_player_cls, playerWithURL: &*ns_url];
            if player.is_null() {
                return Err(PlayerError::LaunchFailed(
                    "AVPlayer failed to initialize".into(),
                ));
            }

            let av_view_cls = AnyClass::get("AVPlayerView").ok_or_else(|| {
                PlayerError::LaunchFailed("AVPlayerView class unavailable".into())
            })?;
            let player_view: *mut AnyObject = msg_send![av_view_cls, new];
            if player_view.is_null() {
                return Err(PlayerError::LaunchFailed(
                    "AVPlayerView failed to initialize".into(),
                ));
            }
            let _: () = msg_send![player_view, setPlayer: player];
            // AVPlayerViewControlsStyleInline == 1
            let _: () = msg_send![player_view, setControlsStyle: 1_i64];
            let _: () = msg_send![player_view, setShowsFullScreenToggleButton: true];

            let ns_window_cls = AnyClass::get("NSWindow").ok_or_else(|| {
                PlayerError::LaunchFailed("NSWindow class unavailable".into())
            })?;

            // NSMakeRect(120, 120, 960, 540) as four doubles via NSValue is awkward;
            // allocate window then set frame / content view.
            let style_mask: u64 = 15; // titled|closable|miniaturizable|resizable
            let backing: u64 = 2; // NSBackingStoreBuffered
            let window: *mut AnyObject = msg_send![
                msg_send![ns_window_cls, alloc],
                initWithContentRect: objc2_foundation::NSRect::new(
                    objc2_foundation::NSPoint::new(120.0, 120.0),
                    objc2_foundation::NSSize::new(960.0, 540.0),
                )
                styleMask: style_mask
                backing: backing
                defer: false
            ];
            if window.is_null() {
                return Err(PlayerError::LaunchFailed(
                    "NSWindow failed to initialize".into(),
                ));
            }

            let window_title = NSString::from_str(&format!("Streamovie — {title}"));
            let _: () = msg_send![window, setTitle: &*window_title];
            let _: () = msg_send![window, setContentView: player_view];
            let _: () = msg_send![window, center];
            let _: () = msg_send![window, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
            let _: () = msg_send![player, play];

            // Retain for process lifetime of this playback session.
            std::mem::forget(Retained::from_raw(player).ok_or_else(|| {
                PlayerError::LaunchFailed("Failed to retain AVPlayer".into())
            })?);
            std::mem::forget(Retained::from_raw(player_view).ok_or_else(|| {
                PlayerError::LaunchFailed("Failed to retain AVPlayerView".into())
            })?);
            std::mem::forget(Retained::from_raw(window).ok_or_else(|| {
                PlayerError::LaunchFailed("Failed to retain NSWindow".into())
            })?);
            let _ = ClassType::class;
        }

        Ok(())
    }

    pub fn open_externally(url: &str) -> Result<(), PlayerError> {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| PlayerError::LaunchFailed(e.to_string()))
    }
}

#[cfg(target_os = "macos")]
pub use macos::{open_externally, open_player_window};

#[cfg(not(target_os = "macos"))]
pub fn open_player_window(_title: &str, _url: &str) -> Result<(), PlayerError> {
    Err(PlayerError::PlatformUnsupported)
}

#[cfg(not(target_os = "macos"))]
pub fn open_externally(url: &str) -> Result<(), PlayerError> {
    let opener = if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        "open"
    };
    std::process::Command::new(opener)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| PlayerError::LaunchFailed(e.to_string()))
}
