//! In-app player. The page itself is served on localhost and the catalog file is
//! proxied through that same origin, so WKWebView can play MP4 and HLS.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::{App, Entity, Window};
use gpui_wry::WebView;
use raw_window_handle::HasWindowHandle;

/// Formats supported for real playback inside the embedded WebKit video element.
pub struct PlaybackSession {
    pub heading: String,
    pub message: String,
    pub stream_url: Option<String>,
    pub webview: Option<Entity<WebView>>,
    pub mount_error: Option<String>,
    pub close_requested: Arc<AtomicBool>,
}

impl PlaybackSession {
    pub fn unavailable(heading: String, message: String) -> Self {
        Self {
            heading,
            message,
            stream_url: None,
            webview: None,
            mount_error: None,
            close_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn for_url(heading: String, url: String) -> Self {
        Self {
            heading,
            message: String::new(),
            stream_url: Some(url),
            webview: None,
            mount_error: None,
            close_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn consume_close(&self) -> bool {
        self.close_requested.swap(false, Ordering::Relaxed)
    }
}

pub fn mount_player(
    window: &mut Window,
    cx: &mut impl gpui_kit::AppContext,
    url: &str,
    title: &str,
    _close_requested: Arc<AtomicBool>,
) -> Result<Entity<WebView>, String> {
    let page = crate::playback_server::player_page(url, title).map_err(|err| err.to_string())?;
    let handle = HasWindowHandle::window_handle(window).map_err(|err| err.to_string())?;
    let built = wry::WebViewBuilder::new()
        .with_autoplay(true)
        .with_url(page)
        .build_as_child(&handle)
        .map_err(|err| err.to_string())?;

    Ok(cx.new(|cx| {
        let app: &mut App = cx;
        WebView::new(built, window, app)
    }))
}

pub fn player_document(title: &str, media_url: &str) -> String {
    let title_json = serde_json::to_string(title).unwrap_or_else(|_| "\"Streamovie\"".to_string());
    let url_json = serde_json::to_string(media_url).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Streamovie</title>
<style>
  :root {{ color-scheme: dark; }}
  * {{ box-sizing: border-box; }}
  html, body {{
    margin: 0; height: 100%; background: #050506; color: #f5f5f4;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif;
  }}
  body {{ display: flex; }}
  .stage {{
    position: relative; flex: 1; min-height: 0; background: #000; overflow: hidden;
  }}
  video {{
    width: 100%; height: 100%; object-fit: contain; background: #000; display: block;
  }}
  .veil {{
    position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
    background: rgba(0,0,0,.45); pointer-events: none;
  }}
  .veil[hidden], .bar[hidden] {{ display: none !important; }}
  .spinner {{
    width: 42px; height: 42px; border-radius: 50%;
    border: 2px solid rgba(255,255,255,.2); border-top-color: #fff;
    animation: spin .8s linear infinite;
  }}
  @keyframes spin {{ to {{ transform: rotate(360deg); }} }}
  .message {{
    position: absolute; left: 24px; right: 24px; top: 28px;
    display: none; padding: 14px 16px; border-radius: 12px;
    background: rgba(12,12,14,.88); border: 1px solid rgba(255,255,255,.08);
  }}
  .message strong {{ display: block; margin-bottom: 4px; font-size: 14px; }}
  .message p {{ margin: 0; color: #c8c8c6; font-size: 13px; line-height: 1.45; }}
  .center-play {{
    display: none;
    position: absolute; inset: 0; align-items: center; justify-content: center;
    background: transparent; border: 0; cursor: pointer;
  }}
  .center-play[hidden] {{ display: none; }}
  .center-play span {{
    width: 68px; height: 68px; border-radius: 999px; display: grid; place-items: center;
    background: rgba(255,255,255,.94); color: #111;
  }}
  .bar {{
    display: none;
    position: absolute; left: 0; right: 0; bottom: 0;
    padding: 28px 18px 14px;
    background: linear-gradient(to top, rgba(0,0,0,.82), rgba(0,0,0,0));
    display: flex; flex-direction: column; gap: 8px;
  }}
  .seek {{
    -webkit-appearance: none; appearance: none; width: 100%; height: 4px; border-radius: 99px;
    background: rgba(255,255,255,.28); outline: none; cursor: pointer;
  }}
  .seek::-webkit-slider-thumb {{
    -webkit-appearance: none; width: 12px; height: 12px; border-radius: 99px; background: #fff; border: 0;
  }}
  .row {{ display: flex; align-items: center; gap: 10px; }}
  button.ctrl {{
    appearance: none; border: 0; background: transparent; color: #fff;
    font: 600 13px/1 -apple-system, sans-serif; padding: 6px 8px; border-radius: 8px; cursor: pointer;
  }}
  button.ctrl:hover {{ background: rgba(255,255,255,.1); }}
  .time {{ font-variant-numeric: tabular-nums; font-size: 12px; color: #ececea; min-width: 92px; }}
  .spacer {{ flex: 1; }}
  input[type="range"].vol {{ width: 84px; }}
</style>
</head>
<body>
  <div class="stage">
    <video id="video" playsinline preload="auto" controls></video>
    <div class="veil" id="loading"><div class="spinner"></div></div>
    <div class="message" id="message"><strong id="message-title"></strong><p id="message-body"></p></div>
    <button class="center-play" id="center" hidden aria-label="Play"><span>▶</span></button>
    <div class="bar" id="bar">
      <input class="seek" id="seek" type="range" min="0" max="1000" value="0" />
      <div class="row">
        <button class="ctrl" id="play" type="button">Play</button>
        <span class="time" id="time">0:00 / 0:00</span>
        <span class="spacer"></span>
        <button class="ctrl" id="mute" type="button">Mute</button>
        <input class="vol" id="volume" type="range" min="0" max="100" value="100" />
        <button class="ctrl" id="full" type="button">Full screen</button>
      </div>
    </div>
  </div>
<script>
const title = {title_json};
const mediaUrl = {url_json};
const video = document.getElementById('video');
const loading = document.getElementById('loading');
const message = document.getElementById('message');
const center = document.getElementById('center');
const play = document.getElementById('play');
const seek = document.getElementById('seek');
const time = document.getElementById('time');
const mute = document.getElementById('mute');
const volume = document.getElementById('volume');
const full = document.getElementById('full');
let seeking = false;
let failed = false;

function fmt(seconds) {{
  if (!Number.isFinite(seconds) || seconds < 0) return '0:00';
  const s = Math.floor(seconds % 60).toString().padStart(2, '0');
  const m = Math.floor(seconds / 60) % 60;
  const h = Math.floor(seconds / 3600);
  return h > 0 ? `${{h}}:${{m.toString().padStart(2,'0')}}:${{s}}` : `${{m}}:${{s}}`;
}}

function showError(text) {{
  failed = true;
  loading.hidden = true;
  center.hidden = true;
  message.style.display = 'block';
  document.getElementById('message-title').textContent = 'Playback unavailable';
  document.getElementById('message-body').textContent = text;
}}

function sync() {{
  const duration = video.duration;
  const known = Number.isFinite(duration) && duration > 0;
  if (!seeking && known) seek.value = String(Math.round((video.currentTime / duration) * 1000));
  time.textContent = `${{fmt(video.currentTime)}} / ${{known ? fmt(duration) : '—'}}`;
  play.textContent = video.paused ? 'Play' : 'Pause';
  center.hidden = failed || !video.paused || video.readyState < 2;
  mute.textContent = video.muted || video.volume === 0 ? 'Unmute' : 'Mute';
}}

video.src = mediaUrl;
video.volume = 1;
const start = video.play();
if (start && start.catch) start.catch(() => {{ center.hidden = false; loading.hidden = true; }});

video.addEventListener('waiting', () => {{ if (!failed) loading.hidden = false; }});
video.addEventListener('playing', () => {{ loading.hidden = true; message.style.display = 'none'; }});
video.addEventListener('canplay', () => {{ loading.hidden = true; }});
video.addEventListener('timeupdate', sync);
video.addEventListener('play', sync);
video.addEventListener('pause', sync);
video.addEventListener('volumechange', sync);
video.addEventListener('error', () => {{
  fetch('/status').then((res) => res.text()).then((text) => {{
    const detail = (text || '').trim();
    showError(detail || 'Streamovie could not decode this file.');
  }}).catch(() => showError('Streamovie could not decode this file.'));
}});

function toggle() {{
  if (failed) return;
  if (video.paused) video.play(); else video.pause();
}}
play.addEventListener('click', toggle);
center.addEventListener('click', toggle);
mute.addEventListener('click', () => {{ video.muted = !video.muted; }});
volume.addEventListener('input', () => {{
  video.volume = Number(volume.value) / 100;
  video.muted = video.volume === 0;
}});
seek.addEventListener('pointerdown', () => {{ seeking = true; }});
seek.addEventListener('change', () => {{
  if (Number.isFinite(video.duration)) {{
    video.currentTime = (Number(seek.value) / 1000) * video.duration;
  }}
  seeking = false;
}});
full.addEventListener('click', () => {{
  if (video.webkitEnterFullscreen) video.webkitEnterFullscreen();
  else if (video.requestFullscreen) video.requestFullscreen();
}});
document.addEventListener('keydown', (event) => {{
  if (event.key === 'Escape') {{
    if (window.ipc && window.ipc.postMessage) window.ipc.postMessage('close');
    return;
  }}
  if (event.target && event.target.tagName === 'INPUT' && event.target.type === 'range' && event.key !== ' ') return;
  if (event.key === ' ' || event.key === 'k') {{ event.preventDefault(); toggle(); }}
  if (event.key === 'm') video.muted = !video.muted;
  if (event.key === 'f') full.click();
  if (event.key === 'ArrowRight') video.currentTime = Math.min((video.duration || video.currentTime + 10), video.currentTime + 10);
  if (event.key === 'ArrowLeft') video.currentTime = Math.max(0, video.currentTime - 10);
  if (event.key === 'ArrowUp') {{ video.volume = Math.min(1, video.volume + 0.05); volume.value = String(Math.round(video.volume * 100)); }}
  if (event.key === 'ArrowDown') {{ video.volume = Math.max(0, video.volume - 0.05); volume.value = String(Math.round(video.volume * 100)); }}
}});
document.title = title;
</script>
</body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::player_document;

    #[test]
    fn player_document_embeds_the_media_url() {
        let html = player_document("Night Drive", "https://example.com/night.mp4");
        assert!(html.contains("https://example.com/night.mp4"));
        assert!(html.contains("function fmt"));
        assert!(html.contains("Playback unavailable"));
    }
}
