# Streamovie for macOS

Native macOS desktop application for browsing and discovering movies & series.  
Built with **Rust**, **GPUI**, and **[uni-kit](https://github.com/MohsenDastaran/uni-kit)**.

Inspired by the architecture and experience of **[CCloud](https://github.com/code3-dev/CCloud)**.

## Features

- **Native GPUI + uni-kit UI** — Built with gpui-kit components (TitleBar, Sidebar, Cards, Tabs, Inputs, Dialogs, and more)
- **Live Catalog** — Connects to Streamovie catalog mirrors with automatic fallback
- **OMDb Enrichment** — Movie & series details enriched with IMDb ratings, awards, and metadata
- **Bilingual + Full RTL** — Instant switch between English and Persian (فارسی) with proper RTL layout and Persian numerals
- **Themes** — Dark (default), Light, and System appearance + multiple brand accent palettes
- **Local Persistence** — Watchlist (bookmarks) and user settings saved locally
- **Fast & Lightweight** — Native performance with no Electron overhead

## Tech Stack

- **Rust**
- **GPUI** (from Zed)
- **uni-kit** / gpui-kit
- **OMDb API** for metadata enrichment

## Getting Started

### Prerequisites

- Rust (latest stable)
- macOS

### Run

```bash
git clone https://github.com/Aryaeifar/streamovie-mac.git
cd streamovie-mac
cargo run