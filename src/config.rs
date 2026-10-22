//! Environment and runtime configuration for Streamovie macOS.
//!
//! Reads `.env` / process environment variables and falls back to the required
//! Streamovie catalog API key and OMDb API key.

use std::{collections::HashMap, env, fs, path::Path};

pub const DEFAULT_STREAMOVIE_API_KEY: &str = "4F5A9C3D9A86FA54EACEDDD635185";
pub const DEFAULT_OMDB_API_KEY: &str = "41031c12";

pub const DEFAULT_API_HOSTS: [&str; 3] = [
    "https://server-hi-speed-iran.info",
    "https://hostinnegar.com",
    "https://windowsdiba.info",
];

pub const OMDB_BASE_URL: &str = "https://www.omdbapi.com/";

#[derive(Clone, Debug)]
pub struct AppConfig {
    streamovie_api_key: String,
    omdb_api_key: String,
    api_hosts: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::load()
    }
}

impl AppConfig {
    /// Loads configuration from `.env.local`, `.env`, and process environment variables,
    /// falling back to the configured Streamovie and OMDb keys.
    pub fn load() -> Self {
        let mut dotenv_vars = HashMap::new();
        for candidate in [".env", ".env.local"] {
            if let Ok(content) = fs::read_to_string(Path::new(candidate)) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = trimmed.split_once('=') {
                        let key = key.trim().to_string();
                        let value = value
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        dotenv_vars.insert(key, value);
                    }
                }
            }
        }

        let get_var = |name: &str| -> Option<String> {
            env::var(name)
                .ok()
                .or_else(|| dotenv_vars.get(name).cloned())
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };

        let streamovie_api_key = get_var("NEXT_PUBLIC_STREAMOVIE_API_KEY")
            .unwrap_or_else(|| DEFAULT_STREAMOVIE_API_KEY.to_string());

        let omdb_api_key =
            get_var("OMDB_API_KEY").unwrap_or_else(|| DEFAULT_OMDB_API_KEY.to_string());

        let host_0 = get_var("NEXT_PUBLIC_STREAMOVIE_API_HOST")
            .unwrap_or_else(|| DEFAULT_API_HOSTS[0].to_string());
        let host_1 = get_var("NEXT_PUBLIC_STREAMOVIE_HELPER_HOST_1")
            .unwrap_or_else(|| DEFAULT_API_HOSTS[1].to_string());
        let host_2 = get_var("NEXT_PUBLIC_STREAMOVIE_HELPER_HOST_2")
            .unwrap_or_else(|| DEFAULT_API_HOSTS[2].to_string());

        let api_hosts = vec![host_0, host_1, host_2]
            .into_iter()
            .map(|h| h.trim_end_matches('/').to_string())
            .collect();

        Self {
            streamovie_api_key,
            omdb_api_key,
            api_hosts,
        }
    }

    pub fn streamovie_api_key(&self) -> &str {
        &self.streamovie_api_key
    }

    pub fn omdb_api_key(&self) -> &str {
        &self.omdb_api_key
    }

    pub fn api_hosts(&self) -> &[String] {
        &self.api_hosts
    }
}
