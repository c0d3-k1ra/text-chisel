mod clipboard;
mod config;
mod hotkey;
mod prompts;
mod rewrite;
mod settings_window;
mod tray;

use std::sync::{Arc, Mutex};

use hotkey::HotKeyEvent;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};

fn notify_error(message: &str) {
    let safe = message
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace(['\n', '\r'], " ");
    let safe = if safe.len() > 150 { &safe[..150] } else { &safe };
    let script = format!("display notification \"{safe}\" with title \"Text Chisel\" sound name \"Basso\"");
    let _ = std::process::Command::new("osascript")
        .args(["-e", &script])
        .spawn();
}

fn handle_hotkey(rt: &tokio::runtime::Runtime, tone: &str) {
    log::info!("rewriting with tone: {}", tone);

    let text = match clipboard::get_selected_text() {
        Ok(t) => {
            log::debug!("copied text: {:?}", t);
            t
        }
        Err(e) => {
            log::error!("clipboard error: {}", e);
            let msg = if e.to_string().contains("empty") || e.to_string().contains("whitespace") {
                "Select some text first, then press \u{2318}\u{2325}R."
            } else {
                "Text Chisel needs Accessibility access. Enable it in System Settings > Privacy > Accessibility."
            };
            notify_error(msg);
            return;
        }
    };

    log::info!("sending to API ({} chars)", text.len());
    let rewritten = match rt.block_on(rewrite::rewrite(&text, tone)) {
        Ok(r) => {
            log::debug!("API response: {:?}", r);
            r
        }
        Err(e) => {
            log::error!("API error: {}", e);
            let s = e.to_string();
            let msg = if s.contains("environment variable not found") {
                "Add your Anthropic API key in Settings to get started.".to_string()
            } else if s.contains("401") {
                "API key not accepted. Open Settings to update it.".to_string()
            } else if s.contains("too long") {
                "Selection is too long. Try again with under 8,000 characters.".to_string()
            } else if s.contains("429") {
                "Too many requests. Wait a moment and try again.".to_string()
            } else if s.contains("529") || s.contains("overloaded") {
                "Claude is busy right now. Give it a moment and try again.".to_string()
            } else if s.contains("timed out") || s.contains("timeout") {
                "Claude took too long to respond. Try again in a moment.".to_string()
            } else {
                "Something went wrong. Try again or check the logs.".to_string()
            };
            notify_error(&msg);
            return;
        }
    };

    match clipboard::paste_text(&rewritten) {
        Ok(_) => log::info!("pasted successfully"),
        Err(e) => {
            log::error!("paste error: {}", e);
            notify_error("Text was rewritten but could not be pasted. Check Accessibility access in System Settings.");
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    dotenvy::dotenv().ok();

    let cfg = config::load();
    log::info!("config loaded from {}", config::path().display());

    // Safety: called before any threads are spawned that read these vars.
    // hotkey::run() is called after this block.
    unsafe {
        if !cfg.api_key.is_empty() {
            std::env::set_var("ANTHROPIC_API_KEY", &cfg.api_key);
        }
        if !cfg.model.is_empty() {
            std::env::set_var("ANTHROPIC_MODEL", &cfg.model);
            log::info!("model: {}", cfg.model);
        }
    }

    let rx = hotkey::run();
    log::info!("hotkey registered: Cmd+Option+R");

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let selected_tone: Arc<Mutex<&'static str>> = Arc::new(Mutex::new("Professional"));

    let event_loop = EventLoop::new();

    let tray = tray::build();
    log::info!("tray icon created");

    let settings_win = settings_window::build(&event_loop, &cfg);

    if cfg.api_key.is_empty() {
        log::warn!("no API key configured — opening settings");
        settings_win.show();
    }

    log::info!("text-chisel running");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        #[cfg(target_os = "macos")]
        if let Event::NewEvents(StartCause::Init) = event {
            #[allow(unexpected_cfgs)]
            unsafe {
                use objc::{class, msg_send, sel, sel_impl};
                let app: *mut objc::runtime::Object =
                    msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![app, setActivationPolicy: 1_isize];
            }
        }

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            settings_win.hide();
        }

        settings_win.poll();

        if let Ok(HotKeyEvent::RewriteTriggered) = rx.try_recv() {
            log::info!("hotkey fired");
            let tone = *selected_tone.lock().unwrap();
            handle_hotkey(&rt, tone);
        }

        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == tray.quit_id {
                log::info!("quit requested");
                std::process::exit(0);
            }
            if event.id == tray.settings_id {
                log::debug!("opening settings window");
                settings_win.show();
            }
            if let Some((_, tone)) = tray.tone_ids.iter().find(|(id, _)| *id == event.id) {
                log::info!("tone changed to: {}", tone);
                *selected_tone.lock().unwrap() = tone;
                tray.set_tone(tone);
            }
        }
    });
}
