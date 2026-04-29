mod clipboard;
mod config;
mod hotkey;
mod prompts;
mod rewrite;
mod settings_window;
mod tray;

use std::sync::{Arc, Mutex};

use hotkey::HotKeyEvent;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};

fn handle_hotkey(rt: &tokio::runtime::Runtime, tone: &str) {
    let text = match clipboard::get_selected_text() {
        Ok(t) => {
            eprintln!("copied: {:?}", t);
            t
        }
        Err(e) => {
            eprintln!("clipboard error: {}", e);
            return;
        }
    };

    let rewritten = match rt.block_on(rewrite::rewrite(&text, tone)) {
        Ok(r) => {
            eprintln!("received: {:?}", r);
            r
        }
        Err(e) => {
            eprintln!("API error: {}", e);
            return;
        }
    };

    match clipboard::paste_text(&rewritten) {
        Ok(_) => eprintln!("pasted"),
        Err(e) => eprintln!("paste error: {}", e),
    }
}

fn main() {
    dotenvy::dotenv().ok();

    let cfg = config::load();
    // Safety: called before any threads are spawned that read these vars.
    // hotkey::run() is called after this block.
    unsafe {
        if !cfg.api_key.is_empty() {
            std::env::set_var("ANTHROPIC_API_KEY", &cfg.api_key);
        }
        if !cfg.model.is_empty() {
            std::env::set_var("ANTHROPIC_MODEL", &cfg.model);
        }
    }

    let rx = hotkey::run();
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let selected_tone: Arc<Mutex<&'static str>> = Arc::new(Mutex::new("Professional"));

    let event_loop = EventLoop::new();

    let tray = tray::build();
    let settings_win = settings_window::build(&event_loop, &cfg);

    if cfg.api_key.is_empty() {
        settings_win.show();
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            settings_win.hide();
        }

        settings_win.poll();

        if let Ok(HotKeyEvent::RewriteTriggered) = rx.try_recv() {
            eprintln!("hotkey fired");
            let tone = *selected_tone.lock().unwrap();
            handle_hotkey(&rt, tone);
        }

        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == tray.quit_id {
                std::process::exit(0);
            }
            if event.id == tray.settings_id {
                settings_win.show();
            }
            if let Some((_, tone)) = tray.tone_ids.iter().find(|(id, _)| *id == event.id) {
                eprintln!("tone selected: {}", tone);
                *selected_tone.lock().unwrap() = tone;
                tray.set_tone(tone);
            }
        }
    });
}
