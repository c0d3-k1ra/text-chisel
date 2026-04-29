use objc2::MainThreadMarker;
use std::thread;

mod clipboard;
mod hotkey;
fn main() {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let ns_app = objc2_app_kit::NSApplication::sharedApplication(mtm);

    let receiver = hotkey::run();
    thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            handle_hotkey_event(event);
        }
    });
    ns_app.run();
}

fn handle_hotkey_event(_event: hotkey::HotKeyEvent) {
    match clipboard::get_selected_text() {
        Ok(text) => println!("Copied text is {text}"),
        Err(e) => eprintln!("Error occurred: {e}"),
    }
}
