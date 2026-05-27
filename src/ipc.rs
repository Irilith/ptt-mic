use crate::notify::notify;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "cmd")]
pub enum IpcCommand {
    #[serde(rename = "mode")]
    Mode { name: String },
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "toggle")]
    Toggle,
    #[serde(rename = "status")]
    Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IpcResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

impl IpcResponse {
    fn ok() -> Self {
        Self {
            ok: true,
            error: None,
            mode: None,
            enabled: None,
            device: None,
        }
    }
    fn error(msg: String) -> Self {
        Self {
            ok: false,
            error: Some(msg),
            mode: None,
            enabled: None,
            device: None,
        }
    }
}

pub fn run_ipc_server(socket_path: &Path, state: Arc<RwLock<State>>) {
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }

    let listener = match UnixListener::bind(socket_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind socket {:?}: {}", socket_path, e);
            std::process::exit(1);
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let s = Arc::clone(&state);
                std::thread::spawn(move || handle_client(stream, s));
            }
            Err(e) => eprintln!("Accept failed: {}", e),
        }
    }
}

fn handle_client(mut stream: UnixStream, state: Arc<RwLock<State>>) {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() {
        let response = match serde_json::from_str::<IpcCommand>(&line) {
            Ok(cmd) => process_command(cmd, state),
            Err(e) => IpcResponse::error(format!("Invalid command: {}", e)),
        };

        if let Ok(resp_str) = serde_json::to_string(&response) {
            let _ = stream.write_all(resp_str.as_bytes());
            let _ = stream.write_all(b"\n");
        }
    }
}

fn process_command(cmd: IpcCommand, state: Arc<RwLock<State>>) -> IpcResponse {
    let mut s = state.write().unwrap();
    let notify_enabled = s.is_notify_enabled();

    match cmd {
        IpcCommand::Mode { name } => {
            if s.config.modes.contains_key(&name) {
                s.mode = name.clone();
                if notify_enabled {
                    notify("ptt-mic", &format!("Mode: {}", name));
                }
                IpcResponse::ok()
            } else {
                IpcResponse::error(format!("unknown mode: {}", name))
            }
        }
        IpcCommand::Enable => {
            s.enabled = true;
            if notify_enabled {
                notify("ptt-mic", "Enabled");
            }
            IpcResponse::ok()
        }
        IpcCommand::Disable => {
            s.enabled = false;
            if notify_enabled {
                notify("ptt-mic", "Disabled");
            }
            IpcResponse::ok()
        }
        IpcCommand::Toggle => {
            s.enabled = !s.enabled;
            if notify_enabled {
                let status = if s.enabled { "Enabled" } else { "Disabled" };
                notify("ptt-mic", status);
            }
            IpcResponse::ok()
        }
        IpcCommand::Status => IpcResponse {
            ok: true,
            error: None,
            mode: Some(s.mode.clone()),
            enabled: Some(s.enabled),
            device: Some(s.config.general.device.to_string_lossy().into_owned()),
        },
    }
}

pub fn send_command(
    socket_path: &Path,
    cmd: &IpcCommand,
) -> Result<IpcResponse, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path)?;
    let req = serde_json::to_string(cmd)?;
    stream.write_all(req.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let resp: IpcResponse = serde_json::from_str(&line)?;
    Ok(resp)
}
