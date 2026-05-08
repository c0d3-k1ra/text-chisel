use tray_icon::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};

pub const TONES: &[&str] = &["Professional", "Polite", "Assertive", "Concise", "Gen Z"];
const DEFAULT_TONE: &str = "Professional";

pub struct Tray {
    pub quit_id: tray_icon::menu::MenuId,
    pub settings_id: tray_icon::menu::MenuId,
    pub login_id: tray_icon::menu::MenuId,
    pub tone_ids: Vec<(tray_icon::menu::MenuId, &'static str)>,
    tone_items: Vec<CheckMenuItem>,
    tone_submenu: Submenu,
    login_item: CheckMenuItem,
    status_item: MenuItem,
    _icon: tray_icon::TrayIcon,
}

impl Tray {
    pub fn set_tone(&self, tone: &str) {
        debug_assert!(
            TONES.contains(&tone),
            "set_tone called with unknown tone: {}",
            tone
        );
        for (item, t) in self.tone_items.iter().zip(TONES.iter()) {
            item.set_checked(*t == tone);
        }
        self.tone_submenu.set_text(format!("Tone: {}", tone));
    }

    pub fn set_launch_at_login(&self, enabled: bool) {
        self.login_item.set_checked(enabled);
    }

    pub fn set_status(&self, text: &str) {
        self.status_item.set_text(text);
    }
}

pub fn build(launch_at_login: bool) -> Tray {
    let icon = load_icon();
    let hotkey_item = MenuItem::new("⌘⌥R  Rewrite selected text", false, None);
    let status_item = MenuItem::new("⏳ Checking...", false, None);
    let settings_item = MenuItem::new("Settings", true, None);
    let settings_id = settings_item.id().clone();
    let login_item = CheckMenuItem::new("Launch at Login", true, launch_at_login, None);
    let login_id = login_item.id().clone();
    let quit_item = MenuItem::new("Quit", true, None);
    let quit_id = quit_item.id().clone();

    let tone_items: Vec<CheckMenuItem> = TONES
        .iter()
        .map(|t| CheckMenuItem::new(*t, true, *t == DEFAULT_TONE, None))
        .collect();

    let tone_ids: Vec<(tray_icon::menu::MenuId, &'static str)> = tone_items
        .iter()
        .zip(TONES.iter())
        .map(|(item, t)| (item.id().clone(), *t))
        .collect();

    let tone_submenu = Submenu::new(format!("Tone: {}", DEFAULT_TONE), true);
    for item in &tone_items {
        tone_submenu
            .append(item)
            .expect("failed to append tone item");
    }

    let menu = Menu::new();
    menu.append(&hotkey_item)
        .expect("failed to append hotkey item");
    menu.append(&status_item)
        .expect("failed to append status item");
    menu.append(&PredefinedMenuItem::separator())
        .expect("failed to append separator");
    menu.append(&tone_submenu)
        .expect("failed to append tone submenu");
    menu.append(&PredefinedMenuItem::separator())
        .expect("failed to append separator");
    menu.append(&settings_item)
        .expect("failed to append settings item");
    menu.append(&login_item)
        .expect("failed to append login item");
    menu.append(&PredefinedMenuItem::separator())
        .expect("failed to append separator");
    menu.append(&quit_item).expect("failed to append quit item");

    let _icon = tray_icon::TrayIconBuilder::new()
        .with_icon(icon)
        .with_icon_as_template(true)
        .with_tooltip("text-chisel")
        .with_menu(Box::new(menu))
        .build()
        .expect("failed to create tray icon");

    Tray {
        quit_id,
        settings_id,
        login_id,
        tone_ids,
        tone_items,
        tone_submenu,
        login_item,
        status_item,
        _icon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tones_contains_all_expected_values() {
        assert!(TONES.contains(&"Professional"));
        assert!(TONES.contains(&"Polite"));
        assert!(TONES.contains(&"Assertive"));
        assert!(TONES.contains(&"Concise"));
        assert!(TONES.contains(&"Gen Z"));
        assert_eq!(TONES.len(), 5);
    }

    #[test]
    fn default_tone_is_in_tones() {
        assert!(TONES.contains(&DEFAULT_TONE));
    }
}

fn load_icon() -> tray_icon::Icon {
    let svg = include_str!("../assets/icon.svg");
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg, &opt).expect("failed to parse icon SVG");
    let mut pixmap = resvg::tiny_skia::Pixmap::new(44, 44).expect("failed to create pixmap");
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    tray_icon::Icon::from_rgba(pixmap.data().to_vec(), 44, 44).expect("failed to create tray icon")
}
