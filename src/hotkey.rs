use global_hotkey::{
    GlobalHotKeyManager,
    GlobalHotKeyEvent,
    GlobalHotKeyEventReceiver,
    hotkey::{Code, HotKey, Modifiers},
};

pub fn run() -> (GlobalHotKeyManager, &'static GlobalHotKeyEventReceiver) {
    let manager = register_hotkey().unwrap();
    let receiver = GlobalHotKeyEvent::receiver();
    (manager, receiver)
}

fn register_hotkey() -> anyhow::Result<GlobalHotKeyManager> {
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
