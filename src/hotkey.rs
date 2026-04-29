use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};

pub fn register_hotkey() -> anyhow::Result<GlobalHotKeyManager> {
    let manager = GlobalHotKeyManager::new()?;
    let hotkey = create_hotkey();
    manager.register(hotkey)?;
    Ok(manager)
}

fn create_hotkey() -> HotKey {
    HotKey::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyR)
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hotkey_has_correct_modifiers_and_key() {
        let hotkey = create_hotkey();
        assert!(hotkey.matches(Modifiers::SUPER | Modifiers::ALT, Code::KeyR));
    }
}