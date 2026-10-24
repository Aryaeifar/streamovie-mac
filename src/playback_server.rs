//! Local player origin. WKWebView will not play a catalog file from an opaque
//! page, so the player document and the media bytes are both served on
//! `127.0.0.1`. Matroska files are remuxed to fragmented MP4 on the way
//! through, because WebKit does not decode MKV. HLS playlists are rewritten
//! so every segment comes back through the same proxy.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context as _, Result, anyhow};
use reqwest::blocking::Client;

use crate::playback_dns::PlaybackDns;
use crate::{config::DEFAULT_API_HOSTS, playback::player_document, storage::app_data_dir};

struct PlayerState {
    port: u16,
    media_url: Mutex<String>,
    status: Mutex<String>,
    hls: Mutex<bool>,
    client: Client,
}

static PLAYER: OnceLock<PlayerState> = OnceLock::new();
static PLAYER_START: Mutex<()> = Mutex::new(());
static HLS_CHILD: Mutex<Option<std::process::Child>> = Mutex::new(None);

pub fn player_page(media_url: &str, title: &str) -> Result<String> {
    let state = player_state()?;
    *state
        .media_url
        .lock()
        .map_err(|_| anyhow!("Player state lock was poisoned"))? = media_url.to_string();
    let hls = url_needs_remux(media_url);
    if hls {
        start_hls(&state.client, media_url)?;
    } else {
        stop_hls();
    }
    if let Ok(mut guard) = state.hls.lock() {
        *guard = hls;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let _ = title;
    Ok(format!(
        "http://127.0.0.1:{}/player?t={stamp}",
        state.port
    ))
}

fn player_state() -> Result<&'static PlayerState> {
    let _guard = PLAYER_START
        .lock()
        .map_err(|_| anyhow!("Player start lock was poisoned"))?;
    if let Some(state) = PLAYER.get() {
        return Ok(state);
    }
    let listener = TcpListener::bind("127.0.0.1:0").context("Could not bind the local player")?;
    let port = listener
        .local_addr()
        .context("Could not read the local player port")?
        .port();
    let resolver = PlaybackDns::new().context("Could not build the player DNS resolver")?;
    let client = Client::builder()
        .timeout(None)
        .connect_timeout(Duration::from_secs(20))
        .dns_resolver(std::sync::Arc::new(resolver))
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
        )
        .build()
        .context("Could not build the player HTTP client")?;
    let state = PlayerState {
        port,
        media_url: Mutex::new(String::new()),
        status: Mutex::new(String::new()),
        hls: Mutex::new(false),
        client,
    };
    let _ = PLAYER.set(state);
    let state = PLAYER.get().expect("player state was just installed");
    thread::Builder::new()
        .name("streamovie-player".to_string())
        .spawn(move || accept_loop(listener))
        .context("Could not start the local player")?;
    Ok(state)
}

fn accept_loop(listener: TcpListener) {
    for connection in listener.incoming() {
        let Ok(socket) = connection else {
            continue;
        };
        thread::spawn(move || {
            let _ = handle_connection(socket);
        });
    }
}

fn handle_connection(mut socket: std::net::TcpStream) -> Result<()> {
    let _ = socket.set_read_timeout(Some(Duration::from_secs(15)));
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 2048];
    while buffer.len() < 16_384 && !buffer.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = socket.read(&mut chunk).unwrap_or(0);
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
    let _ = socket.set_read_timeout(None);
    let request = String::from_utf8_lossy(&buffer);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let range = request.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("range") {
            Some(value.trim().to_string())
        } else {
            None
        }
    });

    if path.starts_with("/status") {
        let message = PLAYER
            .get()
            .and_then(|state| state.status.lock().ok().map(|guard| guard.clone()))
            .filter(|message| !message.is_empty())
            .unwrap_or_else(|| "Playback has not reported an error.".to_string());
        return write_text(&mut socket, 200, "text/plain; charset=utf-8", &message);
    }

    let bare = path.split('?').next().unwrap_or(path);
    if let Some(name) = bare.strip_prefix("/hls/") {
        return serve_hls(name, range.as_deref(), &mut socket);
    }

    if path.starts_with("/stream") {
        let Some(state) = PLAYER.get() else {
            return write_text(&mut socket, 500, "text/plain", "Player is not ready");
        };
        let target = target_from_path(path, &state.media_url);
        proxy_media(&state.client, &target, range.as_deref(), state.port, &mut socket)
    } else {
        let hls = PLAYER
            .get()
            .and_then(|state| state.hls.lock().ok().map(|guard| *guard))
            .unwrap_or(false);
        let source = if hls { "/hls/index.m3u8" } else { "/stream" };
        let html = player_document("Streamovie", source);
        write_text(&mut socket, 200, "text/html; charset=utf-8", &html)
    }
}

fn target_from_path(path: &str, slot: &Mutex<String>) -> String {
    if let Some(query) = path.split_once('?').map(|(_, query)| query) {
        for pair in query.split('&') {
            let Some(value) = pair.strip_prefix("u=") else {
                continue;
            };
            if let Ok(decoded) = urlencoding::decode(value) {
                let decoded = decoded.trim();
                if !decoded.is_empty() {
                    return decoded.to_string();
                }
            }
        }
    }
    slot.lock().map(|guard| guard.clone()).unwrap_or_default()
}

fn set_status(message: impl Into<String>) {
    if let Some(state) = PLAYER.get() {
        if let Ok(mut guard) = state.status.lock() {
            *guard = message.into();
        }
    }
}

fn proxy_media(
    client: &Client,
    target: &str,
    range: Option<&str>,
    port: u16,
    socket: &mut std::net::TcpStream,
) -> Result<()> {
    if !(target.starts_with("http://") || target.starts_with("https://")) {
        set_status("This title has no playable address.");
        return write_text(socket, 400, "text/plain", "Missing media address");
    }
    let mut request = client
        .get(target)
        .header("Referer", format!("{}/", DEFAULT_API_HOSTS[0]))
        .header("Accept", "*/*");
    // A byte range of an MKV is not a decodable file. Fetch it from the start
    // and remux the whole stream.
    if !url_needs_remux(target) {
        if let Some(range) = range {
            request = request.header("Range", range);
        }
    }
    let mut response = request
        .send()
        .context("Catalog media request failed")
        .inspect_err(|err| set_status(err.to_string()))?;
    if !response.status().is_success() {
        let message = format!(
            "The catalog file returned {}.",
            response.status().as_u16()
        );
        set_status(message.clone());
        return write_text(socket, 502, "text/plain; charset=utf-8", &message);
    }
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let final_url = response.url().to_string();
    let mut prefix = [0_u8; 16];
    let prefix_len = response.read(&mut prefix).unwrap_or(0);
    let prefix_text = String::from_utf8_lossy(&prefix[..prefix_len]);
    if is_playlist(&content_type, &final_url, &prefix_text) {
        let mut body = prefix[..prefix_len].to_vec();
        response
            .read_to_end(&mut body)
            .context("Could not read playlist")?;
        let text = String::from_utf8_lossy(&body);
        let rewritten = rewrite_playlist(&text, &final_url, port);
        return write_text(
            socket,
            200,
            "application/vnd.apple.mpegurl",
            &rewritten,
        );
    }

    let content_range = response
        .headers()
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let content_length = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let mut head = format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nAccept-Ranges: bytes\r\nConnection: close\r\n");
    if let Some(content_range) = content_range {
        head.push_str(&format!("Content-Range: {content_range}\r\n"));
    }
    if let Some(content_length) = content_length {
        head.push_str(&format!("Content-Length: {content_length}\r\n"));
    }
    head.push_str("\r\n");
    socket.write_all(head.as_bytes())?;
    socket.write_all(&prefix[..prefix_len])?;
    std::io::copy(&mut response, socket)?;
    let _ = socket.flush();
    Ok(())
}

fn write_text(socket: &mut std::net::TcpStream, status: u16, content_type: &str, body: &str) -> Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        500 => "Error",
        _ => "OK",
    };
    let bytes = body.as_bytes();
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        bytes.len()
    );
    socket.write_all(head.as_bytes())?;
    socket.write_all(bytes)?;
    let _ = socket.flush();
    Ok(())
}

fn url_needs_remux(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url).to_ascii_lowercase();
    [".mkv", ".avi", ".wmv", ".flv"]
        .iter()
        .any(|ext| path.ends_with(ext))
}

fn hls_dir() -> PathBuf {
    app_data_dir().join("playback-hls")
}

fn start_hls(client: &Client, url: &str) -> Result<()> {
    stop_hls();
    let dir = hls_dir();
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).context("Could not prepare playback storage")?;
    let ffmpeg = ensure_ffmpeg()?;
    let playlist = dir.join("index.m3u8");
    let segment = dir.join("seg%05d.ts");
    let mut child = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-fflags",
            "+genpts",
            "-probesize",
            "5000000",
            "-analyzeduration",
            "5000000",
            "-i",
            "pipe:0",
            "-map",
            "0:v:0",
            "-map",
            "0:a:0?",
            "-sn",
            "-dn",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-b:a",
            "160k",
            "-ac",
            "2",
            "-f",
            "hls",
            "-hls_time",
            "4",
            "-hls_list_size",
            "0",
            "-hls_playlist_type",
            "event",
            "-hls_flags",
            "independent_segments",
            "-hls_segment_filename",
        ])
        .arg(&segment)
        .arg(&playlist)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Could not start the video decoder")?;
    let mut stdin = child.stdin.take().context("Decoder input was unavailable")?;
    let stderr = child.stderr.take();
    if let Ok(mut guard) = HLS_CHILD.lock() {
        *guard = Some(child);
    }
    thread::spawn(move || {
        let Some(mut stderr) = stderr else {
            return;
        };
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        if !text.trim().is_empty() {
            set_status(text.trim().to_string());
        }
    });
    let client = client.clone();
    let url = url.to_string();
    let referer = format!("{}/", DEFAULT_API_HOSTS[0]);
    thread::spawn(move || {
        let downloaded = client
            .get(&url)
            .header("Referer", referer)
            .header("Accept", "*/*")
            .send();
        match downloaded {
            Ok(mut response) if response.status().is_success() => {
                let _ = std::io::copy(&mut response, &mut stdin);
            }
            Ok(response) => {
                set_status(format!(
                    "The catalog file returned {}.",
                    response.status().as_u16()
                ));
            }
            Err(err) => set_status(err.to_string()),
        }
        drop(stdin);
    });
    Ok(())
}

pub fn stop_active() {
    stop_hls();
}

fn stop_hls() {
    if let Ok(mut guard) = HLS_CHILD.lock() {
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        *guard = None;
    }
}

fn serve_hls(
    name: &str,
    range: Option<&str>,
    socket: &mut std::net::TcpStream,
) -> Result<()> {
    if name.is_empty() || name.contains('/') || name.contains("..") {
        return write_text(socket, 400, "text/plain", "Bad segment name");
    }
    let path = hls_dir().join(name);
    if name == "index.m3u8" {
        let ready = wait_for_playlist(&path);
        if ready.is_err() {
            let message = PLAYER
                .get()
                .and_then(|state| state.status.lock().ok().map(|guard| guard.clone()))
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| "The video is still preparing.".to_string());
            set_status(message.clone());
            return write_text(socket, 502, "text/plain; charset=utf-8", &message);
        }
    } else if !wait_for_file(&path) {
        return write_text(socket, 404, "text/plain", "Segment is not ready");
    }
    let content_type = if name.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else {
        "video/mp2t"
    };
    write_file(socket, &path, content_type, range)
}

fn wait_for_playlist(path: &std::path::Path) -> Result<()> {
    for _ in 0..150 {
        if let Ok(text) = fs::read_to_string(path) {
            if text.contains("#EXTINF") {
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    Err(anyhow!("Playlist was not ready"))
}

fn wait_for_file(path: &std::path::Path) -> bool {
    for _ in 0..25 {
        if path.is_file() {
            return true;
        }
        thread::sleep(Duration::from_millis(80));
    }
    path.is_file()
}

fn write_file(
    socket: &mut std::net::TcpStream,
    path: &std::path::Path,
    content_type: &str,
    range: Option<&str>,
) -> Result<()> {
    let mut file = fs::File::open(path)?;
    let length = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    let (start, end) = parse_range(range, length).unwrap_or((0, length.saturating_sub(1)));
    let status = if range.is_some() && length > 0 { 206 } else { 200 };
    let body_len = if length == 0 { 0 } else { end - start + 1 };
    let mut head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {body_len}\r\nAccept-Ranges: bytes\r\nCache-Control: no-store\r\nConnection: close\r\n",
        if status == 206 { "Partial Content" } else { "OK" }
    );
    if status == 206 {
        head.push_str(&format!("Content-Range: bytes {start}-{end}/{length}\r\n"));
    }
    head.push_str("\r\n");
    socket.write_all(head.as_bytes())?;
    if start > 0 {
        let _ = std::io::Seek::seek(&mut file, std::io::SeekFrom::Start(start));
    }
    let mut remaining = body_len;
    let mut buf = [0_u8; 64 * 1024];
    while remaining > 0 {
        let want = remaining.min(buf.len() as u64) as usize;
        let read = file.read(&mut buf[..want])?;
        if read == 0 {
            break;
        }
        socket.write_all(&buf[..read])?;
        remaining -= read as u64;
    }
    let _ = socket.flush();
    Ok(())
}

fn parse_range(range: Option<&str>, length: u64) -> Option<(u64, u64)> {
    if length == 0 {
        return None;
    }
    let range = range?.strip_prefix("bytes=")?;
    let (start, end) = range.split_once('-')?;
    let start = if start.is_empty() { 0 } else { start.parse().ok()? };
    let end = if end.is_empty() {
        length - 1
    } else {
        end.parse::<u64>().ok()?.min(length - 1)
    };
    if start > end || start >= length {
        return None;
    }
    Some((start, end))
}

fn ensure_ffmpeg() -> Result<PathBuf> {
    let bundled = app_data_dir().join("bin").join("ffmpeg");
    if ffmpeg_runs(&bundled) {
        return Ok(bundled);
    }
    for candidate in ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"] {
        let path = PathBuf::from(candidate);
        if ffmpeg_runs(&path) {
            return Ok(path);
        }
    }
    if let Some(path) = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path).find_map(|dir| {
            let candidate = dir.join("ffmpeg");
            ffmpeg_runs(&candidate).then_some(candidate)
        })
    }) {
        return Ok(path);
    }
    download_ffmpeg(&bundled)
}

fn ffmpeg_runs(path: &std::path::Path) -> bool {
    path.is_file()
        && Command::new(path)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
}

fn download_ffmpeg(dest: &std::path::Path) -> Result<PathBuf> {
    let asset = match std::env::consts::ARCH {
        "x86_64" => "ffmpeg-darwin-x64.gz",
        _ => "ffmpeg-darwin-arm64.gz",
    };
    let url = format!(
        "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/{asset}"
    );
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let gz_path = dest.with_extension("gz");
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .user_agent("Streamovie-macOS/0.1.0")
        .build()
        .context("Could not start the decoder download")?;
    let mut response = client
        .get(url)
        .send()
        .context("Could not download the video decoder")?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "Could not download the video decoder ({})",
            response.status()
        ));
    }
    let mut file = fs::File::create(&gz_path).context("Could not save the video decoder")?;
    std::io::copy(&mut response, &mut file)?;
    drop(file);
    let unpacked = Command::new("/usr/bin/gzip")
        .args(["-dc"])
        .arg(&gz_path)
        .output()
        .context("Could not unpack the video decoder")?;
    if !unpacked.status.success() {
        return Err(anyhow!("Could not unpack the video decoder"));
    }
    fs::write(dest, unpacked.stdout).context("Could not install the video decoder")?;
    let _ = fs::remove_file(gz_path);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(dest, fs::Permissions::from_mode(0o755));
    }
    let _ = Command::new("/usr/bin/xattr")
        .args(["-d", "com.apple.quarantine"])
        .arg(dest)
        .status();
    let _ = Command::new("/usr/bin/codesign")
        .args(["--force", "--sign", "-"])
        .arg(dest)
        .status();
    if !ffmpeg_runs(dest) {
        return Err(anyhow!("The video decoder could not be started"));
    }
    Ok(dest.to_path_buf())
}

fn is_playlist(content_type: &str, url: &str, prefix: &str) -> bool {
    let content_type = content_type.to_ascii_lowercase();
    let path = url.split(['?', '#']).next().unwrap_or(url).to_ascii_lowercase();
    content_type.contains("mpegurl")
        || content_type.contains("mpeg-url")
        || path.ends_with(".m3u8")
        || prefix.starts_with("#EXTM3U")
}

pub(crate) fn rewrite_playlist(body: &str, playlist_url: &str, port: u16) -> String {
    let base = playlist_directory(playlist_url);
    let mut out = String::new();
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push_str(line);
            continue;
        }
        if trimmed.starts_with('#') {
            out.push_str(&rewrite_tag_uris(line, &base, port));
        } else {
            let newline = if line.ends_with('\n') { "\n" } else { "" };
            let absolute = resolve_media_ref(&base, trimmed);
            out.push_str(&proxy_link(port, &absolute));
            out.push_str(newline);
        }
    }
    out
}

fn rewrite_tag_uris(line: &str, base: &str, port: u16) -> String {
    let mut result = line.to_string();
    let mut search_from = 0;
    while let Some(relative) = result[search_from..].find("URI=\"") {
        let start = search_from + relative;
        let value_start = start + 5;
        let Some(end) = result[value_start..].find('"') else {
            break;
        };
        let raw = result[value_start..value_start + end].to_string();
        let replacement = proxy_link(port, &resolve_media_ref(base, &raw));
        result.replace_range(value_start..value_start + end, &replacement);
        search_from = value_start + replacement.len() + 1;
    }
    result
}

fn proxy_link(port: u16, absolute: &str) -> String {
    format!(
        "http://127.0.0.1:{port}/stream?u={}",
        urlencoding::encode(absolute)
    )
}

fn playlist_directory(url: &str) -> String {
    let without_query = url.split(['?', '#']).next().unwrap_or(url);
    match without_query.rfind('/') {
        Some(index) if index > 8 => without_query[..=index].to_string(),
        _ => format!("{without_query}/"),
    }
}

fn resolve_media_ref(base_dir: &str, reference: &str) -> String {
    let reference = reference.trim();
    if reference.starts_with("https://") || reference.starts_with("http://") {
        return reference.to_string();
    }
    if let Some(rest) = reference.strip_prefix("//") {
        return format!("https://{rest}");
    }
    if reference.starts_with('/') {
        return format!("{}{reference}", origin_of(base_dir));
    }
    let mut directory = base_dir.trim_end_matches('/').to_string();
    let mut relative = reference;
    while let Some(rest) = relative.strip_prefix("../") {
        if let Some(slash) = directory.rfind('/') {
            let scheme = directory.find("://").map(|index| index + 3).unwrap_or(0);
            if slash > scheme {
                directory.truncate(slash);
            }
        }
        relative = rest;
    }
    format!("{directory}/{relative}")
}

fn origin_of(url: &str) -> String {
    let Some(scheme) = url.find("://") else {
        return String::new();
    };
    let host_start = scheme + 3;
    match url[host_start..].find('/') {
        Some(index) => url[..host_start + index].to_string(),
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::{proxy_link, rewrite_playlist};

    #[test]
    fn playlist_segments_are_proxied() {
        let body = "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:6,\nseg.ts\n../audio/a.m3u8\nhttps://cdn.example/abs.ts\n";
        let rewritten = rewrite_playlist(body, "https://cdn.example/hls/master.m3u8", 9);
        assert!(rewritten.contains(&proxy_link(9, "https://cdn.example/hls/init.mp4")));
        assert!(rewritten.contains(&proxy_link(9, "https://cdn.example/hls/seg.ts")));
        assert!(rewritten.contains(&proxy_link(9, "https://cdn.example/audio/a.m3u8")));
        assert!(rewritten.contains(&proxy_link(9, "https://cdn.example/abs.ts")));
    }

    #[test]
    #[ignore = "downloads a catalog file"]
    fn catalog_mkv_is_served_as_hls() {
        let page = super::player_page(
            "http://peer.tci.cdn.146.dl150m.info/film1/Be.Yourself.2026.480p-BaroMovie.mkv",
            "Be Yourself",
        )
        .expect("player page");
        let base = page.split("/player").next().unwrap();
        let mut socket = std::net::TcpStream::connect(base.trim_start_matches("http://")).expect("connect");
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(40)))
            .ok();
        socket
            .write_all(b"GET /hls/index.m3u8 HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .unwrap();
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(40)))
            .ok();
        let mut buf = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            match socket.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(err) => panic!("read failed: {err}"),
            }
        }
        let text = String::from_utf8_lossy(&buf);
        assert!(
            text.contains("HTTP/1.1 200") && text.contains("#EXTINF") && text.contains(".ts"),
            "expected an HLS playlist, got {text}"
        );
        if std::env::var("STREAMOVIE_HOLD").is_ok() {
            eprintln!("holding {base}");
            std::thread::sleep(std::time::Duration::from_secs(50));
        }
    }
}
