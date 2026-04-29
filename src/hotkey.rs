use global_hotkey::{
    GlobalHotKeyManager,
    GlobalHotKeyEvent,
    hotkey::{Code, HotKey, Modifiers},
};
use std::sync::mpsc;
use std::thread;

pub enum HotKeyEvent {
    RewriteTriggered,
}

pub fn run() -> mpsc::Receiver<HotKeyEvent> {
    let (tx, rx) = mpsc::channel();
    let manager = register_hotkey().unwrap();
    Box::leak(Box::new(manager));

    thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.state == global_hotkey::HotKeyState::Pressed {
                    let _ = tx.send(HotKeyEvent::RewriteTriggered);
                }
            }
        }
    });
    
    rx
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
