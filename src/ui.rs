use crate::hotkey;
use std::sync::mpsc;

enum AppState {
    PickingTone,
    Loading(String),
    Done(String),
    Error(String),
}

pub struct App {
    receiver: mpsc::Receiver<hotkey::HotKeyEvent>,
    state: AppState,
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
            self.state = AppState::PickingTone;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }

        let mut selected_tone: Option<String> = None;

        match &self.state {
            AppState::PickingTone => {
                ui.label("Rewrite as:");
                ui.add_space(8.0);
                for (key, tone) in TONES {
                    if ui.button(format!("[{}] {}", key, tone)).clicked() {
                        selected_tone = Some(tone.to_string());
                    }
                }
            }
            AppState::Loading(tone) => {
                ui.label(format!("Rewriting as {}...", tone));
            }
            AppState::Done(text) => {
                ui.label(text);
            }
            AppState::Error(e) => {
                ui.label(format!("Error: {}", e));
            }
        }

        if let Some(tone) = selected_tone {
            self.state = AppState::Loading(tone);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.state = AppState::PickingTone;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Visible(false));
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
                state: AppState::PickingTone,
            }))
        }),
    )
    .unwrap();
}
