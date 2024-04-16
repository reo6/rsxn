#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod launcher;
pub mod gui;
use gui::gui::LauncherUI;
use std::sync::mpsc;


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