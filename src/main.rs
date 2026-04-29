use global_hotkey::GlobalHotKeyEvent;
use objc2::MainThreadMarker;
use std::thread;

mod clipboard;
mod hotkey;
fn main() {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let ns_app = objc2_app_kit::NSApplication::sharedApplication(mtm);

    let _manager = hotkey::register_hotkey().unwrap();
    thread::spawn(|| {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.try_recv() {
                handle_hotkey_event(event);
            }
        }
    });
    ns_app.run();
}

fn handle_hotkey_event(event: GlobalHotKeyEvent) {
    println!("Hotkey event received: {:?}", event);
    match clipboard::get_selected_text() {
        Ok(text) => println!("Copied text is {text}"),
        Err(e) => eprintln!("Error occurred: {e}"),
    }
}
