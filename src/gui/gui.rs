use eframe::egui;
use std::sync::mpsc::Receiver;
use crate::launcher::ServerLauncher;
use std::sync::{Arc, Mutex};

pub struct RsxnGUI {
    command_input: String,
    log_stream_receiver: Receiver<String>,
    logs: Vec<String>,
    launcher: Arc<Mutex<ServerLauncher>>,
}

impl RsxnGUI {
    pub fn new_with_receiver(log_stream_receiver: Receiver<String>, launcher: Arc<Mutex<ServerLauncher>>) -> RsxnGUI {
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
                        ui.add(egui::TextEdit::multiline(&mut logs_for_textedit.clone())
                            .font(egui::FontId::new(13.0, egui::FontFamily::Monospace))
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                        );
                    }
                );

                let mut launcher = self.launcher.lock().unwrap();

                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.command_input)
                        .hint_text("Enter a Command...")
                    );
                    if ui.button("Send").clicked() {
                        launcher.send_command(self.command_input.clone());
                        self.command_input.clear();
                    }
                });


                ui.horizontal(|ui| {

                    if ui.button("Start").clicked() {
                        self.logs.clear();
                        launcher.launch();
                    } else if (launcher.state == crate::launcher::ServerState::RUNNING) && ui.button("Stop").clicked() {
                        launcher.stop();
                    }
                })
            })

        });
    }
}