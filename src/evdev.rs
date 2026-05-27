use crate::state::State;
use evdev::{Device, EventType};
use std::process::Command;
use std::sync::{Arc, RwLock};

pub fn run_evdev_loop(state: Arc<RwLock<State>>) {
    let device_path = {
        let s = state.read().unwrap();
        s.config.general.device.clone()
    };

    let mut device = Device::open(&device_path).unwrap_or_else(|e| {
        eprintln!("Cannot open device {:?}: {}", device_path, e);
        eprintln!("Hint: make sure you're in the 'input' group: sudo usermod -aG input $USER");
        std::process::exit(1);
    });

    loop {
        let events = match device.fetch_events() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading events: {}", e);
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
