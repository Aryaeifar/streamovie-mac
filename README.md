# Streamovie for macOS (`streamovie-mac`)

Native macOS desktop application for **[Streamovie](https://github.com/Aryaeifar/Streamovie)** built with **Rust**, **GPUI**, and **[uni-kit](https://github.com/MohsenDastaran/uni-kit)** (`gpui-kit`).

## Highlights

- **Native GPUI + `uni-kit` Design System**: Uses `gpui-kit` components and blocks (`TitleBar`, `Sidebar`, `Settings`, `Card`, `TabBar`, `Input`, `DescriptionList`, `Tag`, `Badge`, `Empty`, `Skeleton`, `Spinner`, `Alert`, `Dialog`, and `Notification`).
- **Live Streamovie Catalog + OMDb Enrichment**:
  - Connects to the official Streamovie catalog mirrors (`server-hi-speed-iran.info`, `hostinnegar.com`, `windowsdiba.info`) with automatic host fallback.
  - Enriches Movie and Series detail screens and episode guides with live [OMDb API](https://www.omdbapi.com/) metadata, IMDb ratings, vote counts, awards, and episode release dates.
- **Bilingual (`English` + `فارسی`) with Full RTL Support**:
  - Instant switching between English (LTR) and Persian (RTL).
  - Mirrors the `Sidebar` to the right edge (`Side::Right`), reverses horizontal flex stacks (`flex_row_reverse()`), right-aligns Persian typography, and formats localized counts with Persian numerals (`۰-۹`).
- **Dark-First Theming + Brand Accent Palettes**:
  - Supports **Dark** (default), **Light**, and macOS **System** appearance modes, plus all 7 Streamovie brand accent palettes (`Netflix`, `Midnight`, `Aurora`, `Ocean`, `Sunset`, `Candy`, `Noir`).
- **Local Watchlist & Preferences Persistence**:
  - Saves bookmarked titles (`bookmarks.json`) and user settings (`settings.json`) locally inside `~/Library/Application Support/com.streamovie.mac/`.

## Environment Configuration

The project includes `.env` and `.env.example` pre-configured with the required keys:

```env
NEXT_PUBLIC_STREAMOVIE_API_KEY=4F5A9C3D9A86FA54EACEDDD635185
OMDB_API_KEY=41031c12
```

## Running on macOS

```bash
cargo run
```
