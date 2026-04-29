mod clipboard;
fn main() {
    println!("Select some text, you have 3 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(3));
    match clipboard::get_selected_text() {
        Ok(text) => println!("Copied text is {text}"),
        Err(e) => eprintln!("Error occurred: {e}"),
    }
}
