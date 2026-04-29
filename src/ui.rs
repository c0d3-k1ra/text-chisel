use crate::hotkey;
use std::sync::mpsc;
pub struct App {
    receiver: mpsc::Receiver<hotkey::HotKeyEvent>,
    visible: bool,
}

const TONES: &[(&str, &str)] = &[
    ("1", "Professional"),
    ("2", "Casual"),
    ("3", "Concise"),
    ("4", "Friendly"),
    ("5", "Formal"),
    ("6", "Fix Grammar"),
];

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Ok(_) = self.receiver.try_recv() {
            self.visible = true;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }

        if self.visible {
            ui.label("Rewrite as:");
            ui.add_space(8.0);
            for (key, tone) in TONES {
                if ui.button(format!("[{}] {}", key, tone)).clicked() {
                    println!("Selected tone: {}", tone);
                }
            }

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.visible = false;
            }
        }
    }
}

pub fn show(receiver: mpsc::Receiver<hotkey::HotKeyEvent>) {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "text-chisel",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(App {
                receiver,
                visible: true, // Start with the UI visible for testing; change to false for production
            }))
        }),
    )
    .unwrap();
}
