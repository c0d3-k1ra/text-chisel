use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_key: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "claude-haiku-4-5-20251001".to_string(),
        }
    }
}

pub fn path() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("text-chisel").join("config.toml")
}

pub(crate) fn load_from(p: &Path) -> Config {
    match std::fs::read_to_string(p) {
        Ok(s) => match toml::from_str(&s) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::warn!(
                    "config: failed to parse {}: {} — using defaults",
                    p.display(),
                    e
                );
                Config::default()
            }
        },
        Err(_) => {
            log::debug!("config: no file at {} — using defaults", p.display());
            Config::default()
        }
    }
}

pub(crate) fn save_to(config: &Config, p: &Path) {
    if let Some(parent) = p.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        log::error!("config: failed to create directory: {}", e);
        return;
    }
    match toml::to_string(config) {
        Ok(s) => match std::fs::write(p, &s) {
            Ok(_) => log::info!("config saved to {}", p.display()),
            Err(e) => log::error!("config: failed to write {}: {}", p.display(), e),
        },
        Err(e) => log::error!("config: failed to serialize: {}", e),
    }
}

pub fn load() -> Config {
    load_from(&path())
}

pub fn save(config: &Config) {
    save_to(config, &path())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("text-chisel-test-{}.toml", tag))
    }

    #[test]
    fn round_trip() {
        let p = tmp_path("round-trip");
        let original = Config {
            api_key: "sk-ant-test123".to_string(),
            model: "claude-haiku-4-5-20251001".to_string(),
        };
        save_to(&original, &p);
        let restored = load_from(&p);
        assert_eq!(restored.api_key, original.api_key);
        assert_eq!(restored.model, original.model);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn missing_file_returns_default() {
        let p = tmp_path("does-not-exist-xyzzy");
        let _ = std::fs::remove_file(&p); // ensure absent
        let cfg = load_from(&p);
        assert!(cfg.api_key.is_empty());
        assert_eq!(cfg.model, Config::default().model);
    }

    #[test]
    fn malformed_toml_returns_default() {
        let p = tmp_path("malformed");
        std::fs::write(&p, "not valid toml !!!").unwrap();
        let cfg = load_from(&p);
        assert!(cfg.api_key.is_empty());
        let _ = std::fs::remove_file(&p);
    }
}
