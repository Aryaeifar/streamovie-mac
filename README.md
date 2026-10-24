# Streamovie for macOS

Native macOS desktop application for browsing and discovering movies and series.  
Built with **Rust**, **GPUI**, and **[uni-kit](https://github.com/MohsenDastaran/uni-kit)**.

Inspired by the architecture and experience of **[CCloud](https://github.com/code3-dev/CCloud)**.

## Features

- **Native GPUI + uni-kit UI** — Built with gpui-kit components (TitleBar, Sidebar, Cards, Tabs, Inputs, Dialogs, and more)
- **Live Catalog** — Connects to Streamovie catalog mirrors with automatic fallback
- **OMDb Enrichment** — Movie and series details enriched with IMDb ratings, awards, and metadata
- **Bilingual + Full RTL** — Instant switch between English and Persian (فارسی) with proper RTL layout and Persian numerals
- **Themes** — Dark (default), Light, and System appearance, plus multiple brand accent palettes
- **Local Persistence** — Watchlist (bookmarks) and user settings saved locally
- **Fast and Lightweight** — Native performance with no Electron overhead

## Tech Stack

- **Rust**
- **GPUI** (from Zed)
- **uni-kit** / gpui-kit
- **OMDb API** for metadata enrichment

## Download

Every push builds a disk image with GitHub Actions (`.github/workflows/macos.yml`).

1. Open the **Actions** tab and the latest successful **macOS build**.
2. Download the **Streamovie-mac** artifact. Inside it is `Streamovie-mac.dmg`.
3. Open the disk image and drag **Streamovie** into **Applications**.

The app is Apple Silicon (`arm64`) and needs macOS 13 or later. The build is ad-hoc signed and not notarized. After a browser download, macOS can say **“Streamovie” is damaged and can’t be opened.** The file is intact. Clear the quarantine flag, then open it:

```bash
xattr -cr /Applications/Streamovie.app
open /Applications/Streamovie.app
```

You can also Control-click the app and choose **Open**.

## Getting Started

### Prerequisites

- Rust (latest stable)
- macOS 13 or later

### Run

```bash
git clone https://github.com/Aryaeifar/streamovie-mac.git
cd streamovie-mac
cargo run
```

### Release build

```bash
cargo build --release
```

