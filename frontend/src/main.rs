mod app;
mod painter;
mod transfer;

use eframe::NativeOptions;
use egui::Color32;

use app::App;

fn main() {
    eframe::run_native(
        "slither",
        NativeOptions::default(),
        Box::new(|_| {
            Ok(Box::new(App::Launcher {
                server_ip: "".into(),
                nickname: "".into(),
                color: Color32::WHITE,
            }))
        }),
    )
    .unwrap();
}
