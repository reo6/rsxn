use crate::launcher::ServerLauncher;
use eframe::egui;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use crate::launcher::ServerState;
use std::sync::mpsc::Sender;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::env;

const UI_LOG_PREFIX: &str = "[RSXN] ";
const CONFIG_FILE_NAME: &str = "rsxn_config.json";

fn get_home_dir() -> String {
    env::var("HOME").or_else(|_| env::var("USERPROFILE")).unwrap_or_else(|_| String::from("."))
}

pub enum Page {
    LAUNCHER,
    START,
}


#[derive(Serialize, Deserialize)]
struct LauncherConfig {
    server_jar_path: String,
    java_exe_path: String,
    rsxn_server_path: String,
    memory: String,
}


pub struct LauncherUI {
    command_input: String,
    log_stream_receiver: Receiver<String>,
    log_stream_sender: Arc<Sender<String>>,
    logs: Vec<String>,
    launcher: Option<Arc<Mutex<ServerLauncher>>>,
    current_page: Page,
    server_jar_path: String,
    java_exe_path: String,
    rsxn_server_path: String,
    memory: String,
}

impl LauncherUI {
    pub fn new(
        log_stream_receiver: Receiver<String>,
        log_stream_sender: Sender<String>,
    ) -> LauncherUI {
        let mut ui = LauncherUI {
            command_input: String::new(),
            log_stream_receiver,
            log_stream_sender: Arc::new(log_stream_sender),
            logs: Vec::new(),
            launcher: None,
            current_page: Page::START,
            server_jar_path: String::new(),
            java_exe_path: String::new(),
            rsxn_server_path: String::new(),
            memory: String::new(),
        };
        ui.load_config();
        ui
    }

    fn load_config(&mut self) {
        let home_dir = get_home_dir();
        let config_path = Path::new(&home_dir).join(CONFIG_FILE_NAME);
        if config_path.exists() {
            let config: LauncherConfig = serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
            self.server_jar_path = config.server_jar_path;
            self.java_exe_path = config.java_exe_path;
            self.rsxn_server_path = config.rsxn_server_path;
            self.memory = config.memory;
        }
    }

    fn save_config(&self) {
        let home_dir = get_home_dir();
        let config_path = Path::new(&home_dir).join(CONFIG_FILE_NAME);
        let config = LauncherConfig {
            server_jar_path: self.server_jar_path.clone(),
            java_exe_path: self.java_exe_path.clone(),
            rsxn_server_path: self.rsxn_server_path.clone(),
            memory: self.memory.clone(),
        };
        fs::write(&config_path, serde_json::to_string(&config).unwrap()).unwrap();
    }

    fn start_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::TextEdit::singleline(&mut self.server_jar_path).hint_text("Server JAR Path"));
            ui.add(egui::TextEdit::singleline(&mut self.java_exe_path).hint_text("Java Path"));
            ui.add(egui::TextEdit::singleline(&mut self.rsxn_server_path).hint_text("Server Directory Path"));
            ui.add(egui::TextEdit::singleline(&mut self.memory).hint_text("Memory (MB)"));
    
            if ui.button("Start").clicked() {
                let server_name = Path::new(&self.rsxn_server_path)
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                    .unwrap_or("Unknown Server")
                    .to_string();
    
                let launcher = ServerLauncher::new(
                    self.server_jar_path.clone(),
                    self.java_exe_path.clone(),
                    self.rsxn_server_path.clone(),
                    vec![],
                    server_name,
                    self.memory.parse().unwrap(),
                    self.log_stream_sender.clone(),
                );
                self.launcher = Some(Arc::new(Mutex::new(launcher)));
                self.current_page = Page::LAUNCHER;
                self.save_config();
            }
        });
    }
    
    fn launcher_page(&mut self, ctx: &egui::Context) {
        let launcher_arc = self.launcher.as_ref().unwrap();
        let mut launcher = launcher_arc.lock().unwrap();
        let launcher_state = launcher.state.lock().unwrap().clone();
        
        // Side Panel
        egui::SidePanel::right("sidebar")
            .resizable(true)
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {

                    if ((launcher_state == ServerState::STOPPED) || (launcher_state == ServerState::CRASHED)) && ui.button("Start").clicked() {
                        self.logs.clear();
                        self.logs.push(format!("{}Starting server...", UI_LOG_PREFIX));
                        launcher.launch();
                    }
                    
                    if (launcher_state == ServerState::RUNNING) && ui.button("Stop").clicked() {
                        launcher.stop();
                        self.logs.push(format!("{}Stopped server.", UI_LOG_PREFIX));
                    }

                    if (launcher_state == ServerState::RUNNING) && ui.button("Shutdown").clicked() {
                        launcher.shutdown();
                        self.logs.push(format!("{}Shutting down server...", UI_LOG_PREFIX));
                    }

                    if ui.button("Clear Logs").clicked() {
                        self.logs.clear();
                    }

                    // Button to open the server directory
                    if ui.button("Open Server Directory").clicked() {
                        if let Err(e) = opener::open(&launcher.server_dir) {
                            self.logs.push(format!("{}Failed to open server directory: {}", UI_LOG_PREFIX, e));
                        }
                    }
                });
            });




        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(launcher.server_name.clone());
                ui.separator();

                while let Ok(log) = self.log_stream_receiver.try_recv() {
                    let log = String::from_utf8(strip_ansi_escapes::strip(log.as_bytes())).unwrap();
                    self.logs.push(log);
                }

                let logs_for_textedit = self.logs.join("\n");

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut logs_for_textedit.clone())
                                .font(egui::FontId::new(13.0, egui::FontFamily::Monospace))
                                .desired_width(f32::INFINITY)
                                .desired_rows(10),
                        );
                    });


                ui.horizontal(|ui| {
                    let command_input = egui::TextEdit::singleline(&mut self.command_input)
                        .desired_width(f32::INFINITY)
                        .hint_text("Enter a Command...");

                    let command_input_response = ui.add(command_input);

                    if ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && command_input_response.lost_focus()
                        && !self.command_input.is_empty()
                        && (launcher_state == crate::launcher::ServerState::RUNNING)
                    {
                        launcher.send_command(self.command_input.clone());
                        self.command_input.clear();
                        command_input_response.request_focus();
                    }
                });

            })
        });
    }


}



impl eframe::App for LauncherUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_page {
            Page::LAUNCHER => self.launcher_page(ctx),
            Page::START => self.start_page(ctx),
        }
    }
}
