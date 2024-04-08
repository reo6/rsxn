use crate::launcher::ServerLauncher;
use eframe::egui;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

const UI_LOG_PREFIX: &str = "[RSXN] ";

pub struct RsxnGUI {
    command_input: String,
    log_stream_receiver: Receiver<String>,
    logs: Vec<String>,
    launcher: Arc<Mutex<ServerLauncher>>,
}

impl RsxnGUI {
    pub fn new_with_receiver(
        log_stream_receiver: Receiver<String>,
        launcher: Arc<Mutex<ServerLauncher>>,
    ) -> RsxnGUI {
        RsxnGUI {
            command_input: String::new(),
            log_stream_receiver,
            logs: Vec::new(),
            launcher,
        }
    }
}



impl eframe::App for RsxnGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut launcher = self.launcher.lock().unwrap();

        // Side Panel
        egui::SidePanel::right("sidebar")
            .resizable(true)
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    if (launcher.state == crate::launcher::ServerState::STOPPED) && ui.button("Start").clicked() {
                        self.logs.clear();
                        self.logs.push(format!("{}Starting server...", UI_LOG_PREFIX));
                        launcher.launch();
                    }
                    
                    if (launcher.state == crate::launcher::ServerState::RUNNING) && ui.button("Stop").clicked() {
                        launcher.stop();
                        self.logs.push(format!("{}Stopped server.", UI_LOG_PREFIX));
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
                ui.heading("rsxn");
                ui.separator();

                while let Ok(log) = self.log_stream_receiver.try_recv() {
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
                        && (launcher.state == crate::launcher::ServerState::RUNNING)
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
