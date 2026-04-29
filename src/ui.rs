pub struct App;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.label("Hello, World!");

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            std::process::exit(0);
        }
    }
}

pub fn show() {
    let options = eframe::NativeOptions::default();
    eframe::run_native("text-chisel", options, Box::new(|_cc| Ok(Box::new(App)))).unwrap();
}
