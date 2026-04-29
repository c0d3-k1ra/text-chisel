mod clipboard;
mod hotkey;
mod rewrite;
mod ui;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    ui::show(hotkey::run());
}
