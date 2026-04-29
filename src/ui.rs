use std::sync::mpsc;

use crate::hotkey::HotKeyEvent;

enum AppState {
    WaitingForHotkey,
    Loading,
    Error(String),
}

pub struct App {
    receiver: mpsc::Receiver<HotKeyEvent>,
    state: AppState,
    result_tx: mpsc::SyncSender<Result<String, String>>,
    result_rx: mpsc::Receiver<Result<String, String>>,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Ok(HotKeyEvent::RewriteTriggered) = self.receiver.try_recv() {
            match crate::clipboard::get_selected_text() {
                Ok(text) => {
                    let tx = self.result_tx.clone();
                    tokio::runtime::Handle::current().spawn(async move {
                        let result = crate::rewrite::rewrite(&text, "Professional")
                            .await
                            .map_err(|e| e.to_string());
                        let _ = tx.send(result);
                    });
                    self.state = AppState::Loading;
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Visible(true));
                }
                Err(e) => {
                    self.state = AppState::Error(e.to_string());
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Visible(true));
                }
            }
        }

        match &self.state {
            AppState::WaitingForHotkey => {}
            AppState::Loading => {
                ui.label("Rewriting...");
                ui.ctx().request_repaint();
            }
            AppState::Error(e) => {
                ui.label(format!("Error: {}", e));
            }
        }

        if let Ok(result) = self.result_rx.try_recv() {
            match result {
                Ok(rewritten) => {
                    println!("{}", rewritten);
                    self.state = AppState::WaitingForHotkey;
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
                Err(e) => {
                    self.state = AppState::Error(e);
                }
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.state = AppState::WaitingForHotkey;
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }
}

pub fn show(receiver: mpsc::Receiver<HotKeyEvent>) {
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "text-chisel",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(App {
                receiver,
                state: AppState::WaitingForHotkey,
                result_tx,
                result_rx,
            }))
        }),
    )
    .unwrap();
}
