use std::process::Command;
use std::path::Path;
use std::thread;
use std::fs;
use log::{info, debug, warn, error};
use std::sync::{Arc, Mutex};
use std::io::BufReader;
use std::io::BufRead;
use std::sync::mpsc::Sender;
use std::io::Write;
use std::time::Duration;


#[derive(PartialEq)]
#[derive(Clone)]
pub enum ServerState {
    RUNNING,
    STOPPED,
    CRASHED,
}


pub struct ServerLauncher {
    jarfile: String,
    java_path: String,
    pub server_dir: String,
    server_args: Vec<String>,
    pub server_name: String,
    memory: i32,
    pub state: Arc<Mutex<ServerState>>,
    process: Option<Arc<Mutex<std::process::Child>>>,
    log_stream_sender: Arc<Sender<String>>,
}

impl ServerLauncher {
    pub fn new(jarfile: String, java_path: String, server_dir: String, server_args: Vec<String>, server_name: String, memory: i32, log_stream_sender: Arc<Sender<String>>) -> ServerLauncher {
        ServerLauncher {
            jarfile,
            java_path,
            server_dir,
            server_args,
            server_name,
            memory,
            state: Arc::new(Mutex::new(ServerState::STOPPED)),
            process: None,
            log_stream_sender,
        }
    }

    pub fn launch(&mut self) {
        info!("Launching {}...", self.server_name);

        let mut cmd = Command::new(&self.java_path);

        self.check_server_dir();
        cmd.current_dir(&self.server_dir);

        cmd.arg(format!("-Xmx{}M", self.memory));
        cmd.arg("-jar");
        cmd.arg(&self.jarfile);
        for arg in &self.server_args {
            cmd.arg(arg);
        }
        cmd.arg("-nogui");
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        debug!("Generated command: {:?}", cmd);

        let mut _state_lock = self.state.lock().unwrap();
        *_state_lock = ServerState::RUNNING;

        let process = cmd.spawn().expect("Failed to launch server");
        info!("{} launched with PID {}", self.server_name, process.id());

        let process = Arc::new(Mutex::new(process));
        self.process = Some(Arc::clone(&process));

        let process_clone = Arc::clone(&process);
        let sender_clone = self.log_stream_sender.clone();
        thread::spawn(move || {
            let output = process_clone.lock().unwrap().stdout.take().expect("Failed to capture stdout");
            let reader = BufReader::new(output);
            for line in reader.lines() {
                let line = line.expect("Failed to read line");
                sender_clone.send(line).expect("Failed to send line to GUI");
            }
        });

        let process_clone = Arc::clone(&process);
        let server_name = self.server_name.clone();
        let state_clone = Arc::clone(&self.state);
        thread::spawn(move || {
            loop {
                let maybe_status = {
                    let mut process_lock = process_clone.lock().unwrap();
                    process_lock.try_wait().expect("Failed to wait on child")
                };
        
                match maybe_status {
                    Some(status) => {
                        let mut state_lock = state_clone.lock().unwrap();
                        if status.success() {
                            info!("{} has stopped with code 0.", server_name);
                            *state_lock = ServerState::STOPPED;
                        } else {
                            warn!("{} has crashed with code {}.", server_name, status.code().expect("Failed to get the exit code"));
                            *state_lock = ServerState::CRASHED;
                        }
                        break;
                    }
                    None => {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });
    }

    pub fn shutdown(&mut self) {
        info!("Shutting Down {}...", self.server_name);

        if let Some(process) = &self.process {
            process.lock().unwrap().kill().expect("Failed to kill server");
        } else {
            error!("Failed to stop server: no process found");
        }
    }

    pub fn stop(&mut self) {
        // Send the stop command to the server
        self.send_command("stop".to_string());
    }

    pub fn send_command(&mut self, mut command: String) {
        command.push('\n');
        if let Some(process) = &self.process {
            process.lock().unwrap().stdin.as_mut().map_or_else(|| {
                panic!("Failed to capture stdin");
            }, |stdin| {
                stdin.write_all(command.as_bytes()).expect("Failed to write to stdin");
                stdin.flush().expect("Failed to flush stdin");
            });
        } else {
            error!("Failed to send command to server: no process found");
        }
    }

    fn check_server_dir(&self) {
        if Path::new(&self.server_dir).exists() {
            info!("Server directory {} exists.", self.server_dir);
        } else {
            info!("Server directory {} does not exist. Creating the folder.", self.server_dir);
            fs::create_dir_all(&self.server_dir).expect("Failed to create server directory");
        }
    }
}
