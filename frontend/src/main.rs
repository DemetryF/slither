mod app;
mod mutex_ext;
mod painter;
mod transfer;

use eframe::NativeOptions;

use app::{App, Launcher};

fn main() {
    eframe::run_native(
        "slither",
        NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App::Launcher(Launcher::default())))),
    )
    .unwrap();
}
