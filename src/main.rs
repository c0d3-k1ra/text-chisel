mod hotkey;
mod ui;
mod clipboard;
fn main() {
    ui::show(hotkey::run());
}
