pub mod launcher;
pub mod gui;

use launcher::ServerLauncher;
use gui::gui::RsxnGUI;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};


fn main() {
    env_logger::init();

    let (sender, receiver) = mpsc::channel();

    let server_launcher = Arc::new(Mutex::new(ServerLauncher::new(
        std::env::var("SERVER_JAR_PATH").expect("SERVER_JAR_PATH not set"),
        std::env::var("JAVA_EXE_PATH").expect("JAVA_EXE_PATH not set"),
        std::env::var("RSXN_SERVER_PATH").expect("RSXN_SERVER_PATH not set"),
        vec![],
        "Test Server".to_string(),
        2048,
        Arc::new(sender),
    )));

    let server_launcher_clone = Arc::clone(&server_launcher);

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "rsxn",
        native_options,
        Box::new(move |_cc| Box::new(RsxnGUI::new_with_receiver(receiver, server_launcher_clone.clone())))
    );
}