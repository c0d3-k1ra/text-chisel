use std::thread;

mod clipboard;
mod hotkey;
mod ui;
fn main() {
    let receiver = hotkey::run();
    thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            handle_hotkey_event(event);
        }
    });
    ui::show();
}

fn handle_hotkey_event(_event: hotkey::HotKeyEvent) {
    match clipboard::get_selected_text() {
        Ok(text) => println!("Copied text is {text}"),
        Err(e) => eprintln!("Error occurred: {e}"),
    }
}
