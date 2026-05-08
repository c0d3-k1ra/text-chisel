use std::sync::mpsc;

use tao::dpi::LogicalSize;
use tao::event_loop::EventLoop;
use tao::window::{Window, WindowBuilder};
use wry::WebViewBuilder;

use crate::config::{self, Config};

pub enum SettingsEvent {
    Hide,
    Saved(Config), // carries the full saved config so main can reinit the form and recheck status
    TestResult { ok: bool, msg: String },
}

pub struct SettingsWindow {
    webview: wry::WebView,
    window: Window,
    pub rx: mpsc::Receiver<SettingsEvent>,
}

impl SettingsWindow {
    pub fn show(&self, config: &Config) {
        let api_key_json = serde_json::to_string(&config.api_key).unwrap_or_default();
        let model_json = serde_json::to_string(&config.model).unwrap_or_default();
        let _ = self
            .webview
            .evaluate_script(&format!("init({}, {})", api_key_json, model_json));
        self.window.set_visible(true);
        self.window.set_focus();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    /// Drains pending events. Returns the new API key if settings were saved,
    /// so the caller can trigger a connection status re-check.
    pub fn poll(&self) -> Option<Config> {
        let mut saved = None;
        while let Ok(event) = self.rx.try_recv() {
            match event {
                SettingsEvent::Hide => self.hide(),
                SettingsEvent::Saved(config) => {
                    self.hide();
                    saved = Some(config);
                }
                SettingsEvent::TestResult { ok, msg } => {
                    let ok_js = if ok { "true" } else { "false" };
                    let msg_json = serde_json::to_string(&msg).unwrap_or_default();
                    let _ = self
                        .webview
                        .evaluate_script(&format!("showTestResult({}, {})", ok_js, msg_json));
                }
            }
        }
        saved
    }
}

pub fn build(event_loop: &EventLoop<()>, config: &Config) -> SettingsWindow {
    let window = WindowBuilder::new()
        .with_title("text-chisel — Settings")
        .with_inner_size(LogicalSize::new(400, 320))
        .with_resizable(false)
        .with_visible(false)
        .build(event_loop)
        .expect("failed to create settings window");

    let html = build_html(config);
    let config_clone = config.clone();
    let (tx, rx) = mpsc::channel();

    let webview = WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |msg| {
            handle_ipc(msg.body(), &config_clone, &tx);
        })
        .build(&window)
        .expect("failed to create settings webview");

    SettingsWindow {
        webview,
        window,
        rx,
    }
}

fn config_from_save_payload(value: &serde_json::Value, original: &Config) -> Config {
    Config {
        api_key: value["apiKey"].as_str().unwrap_or("").to_string(),
        model: value["model"]
            .as_str()
            .unwrap_or(&original.model)
            .to_string(),
    }
}

fn build_html(config: &Config) -> String {
    let template = include_str!("../assets/settings.html");
    let api_key_json = serde_json::to_string(&config.api_key).unwrap_or_default();
    let model_json = serde_json::to_string(&config.model).unwrap_or_default();
    let init_script = format!(
        "<script>window.addEventListener('DOMContentLoaded', () => init({}, {}));</script>",
        api_key_json, model_json
    );
    template.replace("</body>", &format!("{}</body>", init_script))
}

fn handle_ipc(msg: &str, original_config: &Config, tx: &mpsc::Sender<SettingsEvent>) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(msg) else {
        return;
    };

    match value["action"].as_str() {
        Some("save") => {
            log::info!("settings: saving config");
            let new_config = config_from_save_payload(&value, original_config);
            config::save(&new_config);
            // SAFETY: called from the wry IPC callback, which runs on the main
            // thread before any concurrent rewrite is in flight.
            unsafe {
                if !new_config.api_key.is_empty() {
                    std::env::set_var("ANTHROPIC_API_KEY", &new_config.api_key);
                }
                if !new_config.model.is_empty() {
                    std::env::set_var("ANTHROPIC_MODEL", &new_config.model);
                }
            }
            let _ = tx.send(SettingsEvent::Saved(new_config));
        }
        Some("test") => {
            log::info!("settings: testing connection");
            let api_key = value["apiKey"].as_str().unwrap_or("").to_string();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let result = test_connection(&api_key);
                let _ = tx.send(result);
            });
        }
        Some("cancel") => {
            let _ = tx.send(SettingsEvent::Hide);
        }
        _ => {}
    }
}

fn test_connection(api_key: &str) -> SettingsEvent {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            return SettingsEvent::TestResult {
                ok: false,
                msg: e.to_string(),
            };
        }
    };

    match rt.block_on(crate::rewrite::rewrite_with_key("hi", "Concise", api_key)) {
        Ok(_) => {
            log::info!("settings: connection test passed");
            SettingsEvent::TestResult {
                ok: true,
                msg: "Connection successful".to_string(),
            }
        }
        Err(e) => {
            log::warn!("settings: connection test failed: {}", e);
            let msg = e.to_string();
            let short = if msg.len() > 80 {
                msg[..80].to_string()
            } else {
                msg
            };
            SettingsEvent::TestResult {
                ok: false,
                msg: short,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn default_config() -> Config {
        Config {
            api_key: "sk-ant-test".to_string(),
            model: "claude-haiku-4-5-20251001".to_string(),
        }
    }

    // --- build_html ---

    #[test]
    fn build_html_injects_api_key_and_model() {
        let html = build_html(&default_config());
        assert!(html.contains("sk-ant-test"));
        assert!(html.contains("claude-haiku-4-5-20251001"));
    }

    #[test]
    fn build_html_contains_init_call() {
        let html = build_html(&default_config());
        assert!(html.contains("DOMContentLoaded"));
        assert!(html.contains("init("));
    }

    #[test]
    fn build_html_empty_api_key_does_not_panic() {
        let cfg = Config {
            api_key: String::new(),
            model: String::new(),
        };
        let html = build_html(&cfg);
        assert!(!html.is_empty());
    }

    // --- handle_ipc ---

    // --- config_from_save_payload ---

    #[test]
    fn config_from_save_payload_uses_provided_values() {
        let original = default_config();
        let value = serde_json::json!({ "apiKey": "new-key", "model": "claude-sonnet-4-6" });
        let cfg = config_from_save_payload(&value, &original);
        assert_eq!(cfg.api_key, "new-key");
        assert_eq!(cfg.model, "claude-sonnet-4-6");
    }

    #[test]
    fn config_from_save_payload_falls_back_to_original_model() {
        let original = default_config();
        let value = serde_json::json!({ "apiKey": "new-key" });
        let cfg = config_from_save_payload(&value, &original);
        assert_eq!(cfg.model, original.model);
    }

    #[test]
    fn config_from_save_payload_empty_api_key() {
        let original = default_config();
        let value = serde_json::json!({ "apiKey": "", "model": "claude-haiku-4-5-20251001" });
        let cfg = config_from_save_payload(&value, &original);
        assert!(cfg.api_key.is_empty());
    }

    #[test]
    fn config_from_save_payload_missing_api_key_defaults_to_empty() {
        let original = default_config();
        let value = serde_json::json!({ "model": "claude-haiku-4-5-20251001" });
        let cfg = config_from_save_payload(&value, &original);
        assert!(cfg.api_key.is_empty());
    }

    // --- handle_ipc ---

    #[test]
    fn handle_ipc_cancel_sends_hide() {
        let cfg = default_config();
        let (tx, rx) = std::sync::mpsc::channel();
        handle_ipc(r#"{"action":"cancel"}"#, &cfg, &tx);
        assert!(matches!(rx.try_recv().unwrap(), SettingsEvent::Hide));
    }

    #[test]
    fn handle_ipc_invalid_json_no_panic_no_event() {
        let cfg = default_config();
        let (tx, rx) = std::sync::mpsc::channel();
        handle_ipc("not json at all", &cfg, &tx);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn handle_ipc_unknown_action_no_event() {
        let cfg = default_config();
        let (tx, rx) = std::sync::mpsc::channel();
        handle_ipc(r#"{"action":"unknown"}"#, &cfg, &tx);
        assert!(rx.try_recv().is_err());
    }
}
