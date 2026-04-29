mod hotkey;
mod ui;

fn main() {
    ui::show(hotkey::run());
}
