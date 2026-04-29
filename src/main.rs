mod clipboard;
mod hotkey;
mod rewrite;

use hotkey::HotKeyEvent;
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoop};

fn main() {
    dotenvy::dotenv().ok();
    let rx = hotkey::run();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let event_loop = EventLoop::new();
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(50),
        );

        if let Ok(HotKeyEvent::RewriteTriggered) = rx.try_recv() {
            eprintln!("hotkey fired");
            match clipboard::get_selected_text() {
                Ok(text) => {
                    eprintln!("copied: {:?}", text);
                    eprintln!("sending to API...");
                    match rt.block_on(rewrite::rewrite(&text, "Professional")) {
                        Ok(rewritten) => {
                            eprintln!("received: {:?}", rewritten);
                            match clipboard::paste_text(&rewritten) {
                                Ok(_) => eprintln!("pasted"),
                                Err(e) => eprintln!("paste error: {}", e),
                            }
                        }
                        Err(e) => eprintln!("API error: {}", e),
                    }
                }
                Err(e) => eprintln!("clipboard error: {}", e),
            }
        }

        if let Event::WindowEvent { .. } = _event {
        }
    });
}
