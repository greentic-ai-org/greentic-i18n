use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CACHE_FILE_SUFFIX: &str = ".json";

#[derive(Debug, Clone)]
pub struct CacheStore {
    dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    translation: String,
}

impl CacheStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                return Path::new(&local).join("greentic").join("i18n-translator");
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return Path::new(&home)
                    .join("Library")
                    .join("Caches")
                    .join("greentic")
                    .join("i18n-translator");
            }
        }

        if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
            return Path::new(&xdg).join("greentic").join("i18n-translator");
        }
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home)
                .join(".cache")
                .join("greentic")
                .join("i18n-translator");
        }
        PathBuf::from(".i18n/cache")
    }

    pub fn cache_key(
        lang: &str,
        english_text: &str,
        glossary_version: &str,
        rules_version: &str,
    ) -> String {
        let seed = format!("{lang}\n{english_text}\n{glossary_version}\n{rules_version}");
        blake3::hash(seed.as_bytes()).to_hex().to_string()
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        let path = self.entry_path(key);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path)
            .map_err(|err| format!("failed reading cache entry {}: {err}", path.display()))?;
        let entry: CacheEntry = serde_json::from_str(&raw)
            .map_err(|err| format!("invalid cache entry {}: {err}", path.display()))?;
        Ok(Some(entry.translation))
    }

    pub fn put(&self, key: &str, translation: &str) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|err| {
            format!(
                "failed creating cache directory {}: {err}",
                self.dir.display()
            )
        })?;
        let path = self.entry_path(key);
        let raw = serde_json::to_string(&CacheEntry {
            translation: translation.to_string(),
        })
        .map_err(|err| format!("failed serializing cache entry: {err}"))?;
        fs::write(&path, raw)
            .map_err(|err| format!("failed writing cache entry {}: {err}", path.display()))
    }

    fn entry_path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}{CACHE_FILE_SUFFIX}"))
    }
}
