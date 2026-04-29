use std::path::PathBuf;

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

pub fn load() -> Config {
    let p = path();
    match std::fs::read_to_string(&p) {
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

pub fn save(config: &Config) {
    let p = path();
    if let Some(parent) = p.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("config: failed to create directory: {}", e);
            return;
        }
    }
    match toml::to_string(config) {
        Ok(s) => match std::fs::write(&p, &s) {
            Ok(_) => log::info!("config saved to {}", p.display()),
            Err(e) => log::error!("config: failed to write {}: {}", p.display(), e),
        },
        Err(e) => log::error!("config: failed to serialize: {}", e),
    }
}
