pub mod launcher;
pub mod gui;

use launcher::ServerLauncher;
use gui::gui::LauncherUI;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};


fn main() {
    env_logger::init();

    let (sender, receiver) = mpsc::channel();

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "rsxn",
        native_options,
        Box::new(move |_cc| Box::new(LauncherUI::new(receiver, sender)))
    );
}