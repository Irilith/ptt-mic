use crate::config::Config;
use crate::evdev;
use crate::ipc;
use crate::state::State;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

pub fn run(config: Config, socket_path: PathBuf) {
    let state = Arc::new(RwLock::new(State::new(config)));

    println!("Starting ptt-mic daemon...");
    {
        let s = state.read().unwrap();
        println!("Device: {:?}", s.config.general.device);
        println!("Mode: {}", s.mode);
        println!("Socket: {:?}", socket_path);
    }

    let evdev_state = Arc::clone(&state);
    let evdev_handle = thread::spawn(move || evdev::run_evdev_loop(evdev_state));

    let ipc_state = Arc::clone(&state);
    let ipc_handle = thread::spawn(move || ipc::run_ipc_server(&socket_path, ipc_state));

    evdev_handle.join().unwrap();
    ipc_handle.join().unwrap();
}
