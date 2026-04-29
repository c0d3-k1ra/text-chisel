use tray_icon::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};

pub const TONES: &[&str] = &["Professional", "Polite", "Assertive", "Concise", "Gen Z"];
const DEFAULT_TONE: &str = "Professional";

pub struct Tray {
    pub quit_id: tray_icon::menu::MenuId,
    pub settings_id: tray_icon::menu::MenuId,
    pub tone_ids: Vec<(tray_icon::menu::MenuId, &'static str)>,
    tone_items: Vec<CheckMenuItem>,
    _icon: tray_icon::TrayIcon,
}

impl Tray {
    pub fn set_tone(&self, tone: &str) {
        for (item, t) in self.tone_items.iter().zip(TONES.iter()) {
            item.set_checked(*t == tone);
        }
    }
}

pub fn build() -> Tray {
    let icon = load_icon();
    let hotkey_item = MenuItem::new("⌘⌥R  Rewrite selected text", false, None);
    let settings_item = MenuItem::new("Settings", true, None);
    let settings_id = settings_item.id().clone();
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

    let menu = Menu::new();
    menu.append(&hotkey_item)
        .expect("failed to append hotkey item");
    menu.append(&PredefinedMenuItem::separator())
        .expect("failed to append separator");
    for item in &tone_items {
        menu.append(item).expect("failed to append tone item");
    }
    menu.append(&PredefinedMenuItem::separator())
        .expect("failed to append separator");
    menu.append(&settings_item)
        .expect("failed to append settings item");
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
        tone_ids,
        tone_items,
        _icon,
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
