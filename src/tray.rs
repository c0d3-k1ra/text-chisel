use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};

pub struct Tray {
    pub quit_id: tray_icon::menu::MenuId,
    _icon: tray_icon::TrayIcon,
}

pub fn build() -> Tray {
    let icon = load_icon();
    let (menu, quit_id) = build_menu();

    let _icon = tray_icon::TrayIconBuilder::new()
        .with_icon(icon)
        .with_icon_as_template(true)
        .with_tooltip("text-chisel")
        .with_menu(Box::new(menu))
        .build()
        .expect("failed to create tray icon");

    Tray { quit_id, _icon }
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

fn build_menu() -> (Menu, tray_icon::menu::MenuId) {
    let hotkey_item = MenuItem::new("⌘⌥R  Rewrite selected text", false, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let quit_id = quit_item.id().clone();
    let menu = Menu::new();
    menu.append_items(&[&hotkey_item, &PredefinedMenuItem::separator(), &quit_item])
        .expect("failed to build tray menu");
    (menu, quit_id)
}
