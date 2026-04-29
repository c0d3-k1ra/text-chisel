mod clipboard;
mod hotkey;
mod prompts;
mod rewrite;

use hotkey::HotKeyEvent;
use tao::event_loop::{ControlFlow, EventLoop};
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};

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

fn main() {
    dotenvy::dotenv().ok();
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("error: ANTHROPIC_API_KEY is not set");
        std::process::exit(1);
    }

    let rx = hotkey::run();
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let hotkey_item = MenuItem::new("⌘⌥R  Rewrite selected text", false, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let menu = Menu::new();
    menu.append_items(&[&hotkey_item, &PredefinedMenuItem::separator(), &quit_item])
        .expect("failed to build tray menu");

    let _tray = tray_icon::TrayIconBuilder::new()
        .with_icon(load_icon())
        .with_icon_as_template(true)
        .with_tooltip("text-chisel")
        .with_menu(Box::new(menu))
        .build()
        .expect("failed to create tray icon");

    let quit_id = quit_item.id().clone();

    let event_loop = EventLoop::new();
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(HotKeyEvent::RewriteTriggered) = rx.try_recv() {
            eprintln!("hotkey fired");
            match clipboard::get_selected_text() {
                Ok(text) => {
                    eprintln!("copied: {:?}", text);
                    eprintln!("sending to API...");
                    match rt.block_on(rewrite::rewrite(&text, "Professional")) {
                        Ok(rewritten) => {
                            eprintln!("received: {:?}", rewritten);
                            match clipboard::paste_text(&rewritten) {
                                Ok(_) => eprintln!("pasted"),
                                Err(e) => eprintln!("paste error: {}", e),
                            }
                        }
                        Err(e) => eprintln!("API error: {}", e),
                    }
                }
                Err(e) => eprintln!("clipboard error: {}", e),
            }
        }

        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == quit_id {
                std::process::exit(0);
            }
        }
    });
}
