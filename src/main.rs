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

fn handle_hotkey(rt: &tokio::runtime::Runtime, tone: &str) {
    log::info!("rewriting with tone: {}", tone);

    let text = match clipboard::get_selected_text() {
        Ok(t) => {
            log::debug!("copied text: {:?}", t);
            t
        }
        Err(e) => {
            log::error!("clipboard error: {}", e);
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
            return;
        }
    };

    match clipboard::paste_text(&rewritten) {
        Ok(_) => log::info!("pasted successfully"),
        Err(e) => log::error!("paste error: {}", e),
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
