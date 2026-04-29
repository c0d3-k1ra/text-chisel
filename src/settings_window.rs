use std::sync::mpsc;

use tao::dpi::LogicalSize;
use tao::event_loop::EventLoop;
use tao::window::{Window, WindowBuilder};
use wry::WebViewBuilder;

use crate::config::{self, Config};

pub enum SettingsEvent {
    Hide,
}

pub struct SettingsWindow {
    _webview: wry::WebView,
    window: Window,
    pub rx: mpsc::Receiver<SettingsEvent>,
}

impl SettingsWindow {
    pub fn show(&self) {
        self.window.set_visible(true);
        self.window.set_focus();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn poll(&self) {
        if let Ok(SettingsEvent::Hide) = self.rx.try_recv() {
            self.hide();
        }
    }
}

pub fn build(event_loop: &EventLoop<()>, config: &Config) -> SettingsWindow {
    let window = WindowBuilder::new()
        .with_title("text-chisel — Settings")
        .with_inner_size(LogicalSize::new(400, 300))
        .with_resizable(false)
        .with_visible(false)
        .build(event_loop)
        .expect("failed to create settings window");

    let html = build_html(config);
    let config_clone = config.clone();
    let (tx, rx) = mpsc::channel();

    let _webview = WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |msg| {
            handle_ipc(msg.body(), &config_clone, &tx);
        })
        .build(&window)
        .expect("failed to create settings webview");

    SettingsWindow {
        _webview,
        window,
        rx,
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
            let new_config = Config {
                api_key: value["apiKey"].as_str().unwrap_or("").to_string(),
                model: value["model"]
                    .as_str()
                    .unwrap_or(&original_config.model)
                    .to_string(),
            };
            config::save(&new_config);
            unsafe {
                std::env::set_var("ANTHROPIC_API_KEY", &new_config.api_key);
                std::env::set_var("ANTHROPIC_MODEL", &new_config.model);
            }
            let _ = tx.send(SettingsEvent::Hide);
        }
        Some("cancel") => {
            let _ = tx.send(SettingsEvent::Hide);
        }
        _ => {}
    }
}
