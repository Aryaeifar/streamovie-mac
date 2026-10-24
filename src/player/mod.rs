//! In-app video player session API.
//!
//! On macOS, progressive playable sources open in a native AVKit `AVPlayerView` window.
//! On other platforms (and in CI), a stub reports that playback requires macOS.

mod avkit;

use crate::models::SourceLink;

/// Active in-app playback session metadata held on `StreamovieApp`.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PlayerSession {
    pub title: String,
    pub source: SourceLink,
    pub media_id: u64,
    pub episode_key: Option<String>,
}

/// Why a source could not be launched in the in-app player.
#[derive(Clone, Debug)]
pub enum PlayerError {
    /// URL is not a progressive playable video (and is not claimed as HLS-capable).
    NotPlayable,
    /// HLS / adaptive streams are not supported by this player build.
    HlsUnsupported,
    /// Native window / AVKit launch failed.
    LaunchFailed(String),
    /// Current OS has no AVKit backend (Linux CI stubs, etc.).
    PlatformUnsupported,
}

impl PlayerError {
    pub fn user_message(&self) -> String {
        match self {
            Self::NotPlayable => {
                "This source cannot play in-app. Only progressive video files (MP4, WebM, OGG) are supported."
                    .to_string()
            }
            Self::HlsUnsupported => {
                "HLS streams (.m3u8) are not supported in the in-app player yet. No fake playback is available."
                    .to_string()
            }
            Self::LaunchFailed(detail) => format!("Could not open the in-app player: {detail}"),
            Self::PlatformUnsupported => {
                "In-app playback requires macOS with AVKit. This build cannot open a video window."
                    .to_string()
            }
        }
    }
}

impl PlayerSession {
    pub fn new(
        title: impl Into<String>,
        source: SourceLink,
        media_id: u64,
        episode_key: Option<String>,
    ) -> Self {
        Self {
            title: title.into(),
            source,
            media_id,
            episode_key,
        }
    }
}

/// Pick the best non-trailer progressive playable source, preferring higher listed quality.
pub fn best_playable_source(sources: &[SourceLink]) -> Option<&SourceLink> {
    sources
        .iter()
        .find(|s| !s.is_trailer && s.is_playable_video())
}

/// Validate and launch the in-app player for `session`.
pub fn play_session(session: &PlayerSession) -> Result<(), PlayerError> {
    if session.source.is_hls() {
        return Err(PlayerError::HlsUnsupported);
    }
    if !session.source.is_playable_video() {
        return Err(PlayerError::NotPlayable);
    }
    avkit::open_player_window(&session.title, &session.source.url)
}

/// Open a URL with the system handler as an explicit user-chosen fallback only.
pub fn open_externally(url: &str) -> Result<(), PlayerError> {
    avkit::open_externally(url)
}
