//! API client layer for Streamovie catalog endpoints and OMDb metadata enrichment.

pub mod omdb;
pub mod streamovie;

pub use omdb::OmdbClient;
pub use streamovie::StreamovieClient;
