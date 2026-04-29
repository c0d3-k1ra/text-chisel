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
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(config: &Config) {
    let p = path();
    if let Some(parent) = p.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("config: failed to create directory: {}", e);
            return;
        }
    }
    match toml::to_string(config) {
        Ok(s) => {
            if let Err(e) = std::fs::write(&p, s) {
                eprintln!("config: failed to write {}: {}", p.display(), e);
            }
        }
        Err(e) => eprintln!("config: failed to serialize: {}", e),
    }
}
