use crate::state::State;
use evdev::{Device, EventType};
use std::process::Command;
use std::sync::{Arc, RwLock};

use std::collections::HashSet;
use std::thread;

pub fn run_evdev_loop(state: Arc<RwLock<State>>) {
    let devices = {
        let s = state.read().unwrap();
        let mut devs = HashSet::new();
        for mode in s.config.modes.values() {
            devs.insert(s.config.resolve_device(mode));
        }
        devs
    };

    let mut handles = vec![];
    for device_path in devices {
        let st = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            run_single_evdev_loop(st, device_path);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

fn run_single_evdev_loop(state: Arc<RwLock<State>>, device_path: std::path::PathBuf) {
    let mut device = Device::open(&device_path).unwrap_or_else(|e| {
        eprintln!("Cannot open device {:?}: {}", device_path, e);
        eprintln!("Hint: make sure you're in the 'input' group: sudo usermod -aG input $USER");
        std::process::exit(1);
    });

    loop {
        let events = match device.fetch_events() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading events on {:?}: {}", device_path, e);
                continue;
            }
        };

        for event in events {
            if event.event_type() != EventType::KEY {
                continue;
            }

            let s = state.read().unwrap();
            if !s.enabled {
                continue;
            }

            if let Some(current_mode_config) = s.config.modes.get(&s.mode) {
                let effective_device = s.config.resolve_device(current_mode_config);
                if effective_device != device_path {
                    continue;
                }
            } else {
                continue;
            }

            for bind in s.current_binds() {
                if event.code() == bind.button.code() {
                    let cmd = if event.value() == 1 {
                        &bind.press
                    } else if event.value() == 0 {
                        &bind.release
                    } else {
                        &None
                    };

                    if let Some(cmd_args) = cmd
                        && !cmd_args.is_empty()
                    {
                        run_cmd(cmd_args);
                    }
                }
            }
        }
    }
}

fn run_cmd(args: &[String]) {
    if let Err(e) = Command::new(&args[0]).args(&args[1..]).spawn() {
        eprintln!("Failed to run {:?}: {}", args, e);
    }
}
