mod clipboard;
mod hotkey;
mod rewrite;
mod ui;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    //ui::show(hotkey::run());
    let rewrite = rewrite::rewrite("Yo bro what's up?", "Professional")
        .await
        .unwrap();
    println!("{}", rewrite);
}
