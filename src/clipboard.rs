use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings};
pub fn get_selected_text() -> anyhow::Result<String> {
    simulate_copy_shortcut()?;

    let mut clipboard = Clipboard::new()?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    let text = clipboard.get_text()?;
    validate_text(&text)?;
    Ok(text)
}

fn simulate_copy_shortcut() -> anyhow::Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.key(Key::Meta, enigo::Direction::Press)?;
    enigo.key(Key::Unicode('c'), enigo::Direction::Click)?;
    enigo.key(Key::Meta, enigo::Direction::Release)?;
    Ok(())
}

fn validate_text(text: &str) -> anyhow::Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("Clipboard is empty or contains only whitespace");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "This test requires user interaction to select text before running. Run it manually to verify functionality."]
    fn test_get_selected_text_returns_something() {
        let text = get_selected_text().unwrap();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_empty_clipboard_returns_error() {
        assert!(validate_text("").is_err());
    }

    #[test]
    fn test_validate_text_returns_ok_for_non_empty_string() {
        assert!(validate_text("Hello").is_ok());
    }
}
