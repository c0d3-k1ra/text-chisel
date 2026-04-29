mod clipboard;
mod hotkey;
mod prompts;
mod rewrite;
mod tray;

use std::sync::{Arc, Mutex};

use hotkey::HotKeyEvent;
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
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("error: ANTHROPIC_API_KEY is not set");
        std::process::exit(1);
    }

    let rx = hotkey::run();
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let tray = tray::build();
    let selected_tone: Arc<Mutex<&'static str>> = Arc::new(Mutex::new("Professional"));

    let event_loop = EventLoop::new();
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(HotKeyEvent::RewriteTriggered) = rx.try_recv() {
            eprintln!("hotkey fired");
            let tone = *selected_tone.lock().unwrap();
            handle_hotkey(&rt, tone);
        }

        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == tray.quit_id {
                std::process::exit(0);
            }
            if let Some((_, tone)) = tray.tone_ids.iter().find(|(id, _)| *id == event.id) {
                eprintln!("tone selected: {}", tone);
                *selected_tone.lock().unwrap() = tone;
                tray.set_tone(tone);
            }
        }
    });
}
