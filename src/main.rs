mod clipboard;
mod config;
mod hotkey;
mod login_item;
mod prompts;
mod rewrite;
mod settings_window;
mod tray;

use hotkey::HotKeyEvent;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};

fn notify_error(message: &str) {
    let safe = message
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace(['\n', '\r'], " ");
    let safe = if safe.len() > 150 {
        &safe[..150]
    } else {
        &safe
    };
    let script =
        format!("display notification \"{safe}\" with title \"Text Chisel\" sound name \"Basso\"");
    if let Err(e) = std::process::Command::new("osascript")
        .args(["-e", &script])
        .spawn()
    {
        log::warn!("osascript spawn failed: {}", e);
    }
}

fn spawn_connection_check(
    rt: &tokio::runtime::Runtime,
    api_key: String,
    tx: std::sync::mpsc::Sender<&'static str>,
) {
    rt.spawn(async move {
        let status = match rewrite::rewrite_with_key("hi", "Concise", &api_key).await {
            Ok(_) => "🟢 Connected",
            Err(e) => {
                log::warn!("connection check failed: {}", e);
                "🔴 Not connected"
            }
        };
        let _ = tx.send(status);
    });
}

#[cfg(target_os = "macos")]
fn setup_edit_menu() {
    // SAFETY: called once on the main thread from StartCause::Init.
    // Without an NSMenu containing Edit items (cut:, copy:, paste:, selectAll:),
    // WKWebView cannot route standard keyboard shortcuts through the responder
    // chain in apps that have no menu bar (NSApplicationActivationPolicyAccessory).
    unsafe {
        use std::os::raw::c_char;

        use objc::runtime::{Object, Sel};
        use objc::{class, msg_send, sel, sel_impl};

        let nsstr = |s: &[u8]| -> *mut Object {
            msg_send![class!(NSString), stringWithUTF8String: s.as_ptr() as *const c_char]
        };

        let menu_item = |title: &[u8], action: &str, key: &[u8]| -> *mut Object {
            let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
            msg_send![
                item,
                initWithTitle: nsstr(title)
                action: Sel::register(action)
                keyEquivalent: nsstr(key)
            ]
        };

        let main_menu: *mut Object = msg_send![class!(NSMenu), new];

        // First item must be the app menu (can be empty)
        let app_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![main_menu, addItem: app_item];

        // Edit menu
        let edit_menu: *mut Object = msg_send![class!(NSMenu), new];
        let _: () = msg_send![edit_menu, addItem: menu_item(b"Undo\0",       "undo:",      b"z\0")];
        let _: () = msg_send![edit_menu, addItem: menu_item(b"Redo\0",       "redo:",      b"Z\0")];
        let sep: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![edit_menu, addItem: sep];
        let _: () = msg_send![edit_menu, addItem: menu_item(b"Cut\0",        "cut:",       b"x\0")];
        let _: () = msg_send![edit_menu, addItem: menu_item(b"Copy\0",       "copy:",      b"c\0")];
        let _: () = msg_send![edit_menu, addItem: menu_item(b"Paste\0",      "paste:",     b"v\0")];
        let _: () = msg_send![edit_menu, addItem: menu_item(b"Select All\0", "selectAll:", b"a\0")];

        let edit_item: *mut Object = msg_send![class!(NSMenuItem), new];
        let _: () = msg_send![edit_item, setSubmenu: edit_menu];
        let _: () = msg_send![main_menu, addItem: edit_item];

        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, setMainMenu: main_menu];
    }
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
            notify_error(
                "Text was rewritten but could not be pasted. Check Accessibility access in System Settings.",
            );
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

    let rx = hotkey::run().unwrap_or_else(|e| {
        log::error!("failed to register hotkey: {}", e);
        std::process::exit(1);
    });
    log::info!("hotkey registered: Cmd+Option+R");

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let mut selected_tone: &'static str = "Professional";
    let mut current_config = cfg.clone();

    let event_loop = EventLoop::new();

    let tray = tray::build(login_item::is_enabled());
    log::info!("tray icon created");

    let settings_win = settings_window::build(&event_loop, &cfg);

    let (status_tx, status_rx) = std::sync::mpsc::channel::<&'static str>();

    if cfg.api_key.is_empty() {
        log::warn!("no API key configured — opening settings");
        tray.set_status("🔴 No API key");
        settings_win.show(&cfg);
    } else {
        spawn_connection_check(&rt, cfg.api_key.clone(), status_tx.clone());
    }

    log::info!("text-chisel running");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        #[cfg(target_os = "macos")]
        if let Event::NewEvents(StartCause::Init) = event {
            #[allow(unexpected_cfgs)]
            // SAFETY: called once on the main thread inside StartCause::Init,
            // before any other windows or threads interact with NSApplication.
            unsafe {
                use objc::{class, msg_send, sel, sel_impl};
                let app: *mut objc::runtime::Object =
                    msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![app, setActivationPolicy: 1_isize];
            }
            setup_edit_menu();
        }

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            settings_win.hide();
        }

        if let Some(saved) = settings_win.poll() {
            let new_key = saved.api_key.clone();
            current_config = saved;
            if new_key.is_empty() {
                tray.set_status("🔴 No API key");
            } else {
                tray.set_status("⏳ Checking...");
                spawn_connection_check(&rt, new_key, status_tx.clone());
            }
        }

        if let Ok(status) = status_rx.try_recv() {
            tray.set_status(status);
        }

        if let Ok(HotKeyEvent::RewriteTriggered) = rx.try_recv() {
            log::info!("hotkey fired");
            handle_hotkey(&rt, selected_tone);
        }

        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == tray.quit_id {
                log::info!("quit requested");
                std::process::exit(0);
            }
            if event.id == tray.settings_id {
                log::debug!("opening settings window");
                settings_win.show(&current_config);
            }
            if event.id == tray.login_id {
                let enabling = !login_item::is_enabled();
                let result = if enabling {
                    login_item::enable()
                } else {
                    login_item::disable()
                };
                match result {
                    Ok(_) => tray.set_launch_at_login(enabling),
                    Err(e) => {
                        log::error!("launch at login toggle failed: {}", e);
                        notify_error(
                            "Could not update launch at login setting. Check logs for details.",
                        );
                    }
                }
            }
            if let Some((_, tone)) = tray.tone_ids.iter().find(|(id, _)| *id == event.id) {
                log::info!("tone changed to: {}", tone);
                selected_tone = tone;
                tray.set_tone(tone);
            }
        }
    });
}
