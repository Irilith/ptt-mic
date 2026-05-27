mod config;
mod daemon;
mod evdev;
mod ipc;
mod notify;
mod state;

use clap::{Parser, Subcommand};
use config::Config;
use ipc::{IpcCommand, send_command};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "ptt-mic")]
#[command(about = "Push-to-talk mic control daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the push-to-talk daemon
    Run {
        /// Input device path
        #[arg(long)]
        device: Option<PathBuf>,
        /// Config file path
        #[arg(long)]
        config: Option<PathBuf>,
        /// IPC socket path
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Install as a systemd user service and start it
    Install {
        /// Input device path
        #[arg(long)]
        device: Option<PathBuf>,
        /// Config file path
        #[arg(long)]
        config: Option<PathBuf>,
        /// IPC socket path
        #[arg(long)]
        socket: Option<PathBuf>,
    },
    /// Uninstall the systemd user service
    Uninstall,
    /// Switch active mode
    Mode { name: String },
    /// Enable event dispatch
    Enable,
    /// Disable event dispatch
    Disable,
    /// Toggle event dispatch
    Toggle,
    /// Show daemon status
    Status,
}

fn get_default_config_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config/ptt-mic/config.toml")
}

fn get_default_socket_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(dir).join("ptt-mic.sock")
    } else {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(".ptt-mic.sock")
    }
}

fn service_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config/systemd/user")
}

fn service_path() -> PathBuf {
    service_dir().join("ptt-mic.service")
}

fn install(device: Option<PathBuf>, config: Option<PathBuf>, socket: Option<PathBuf>) {
    let binary = std::env::current_exe().expect("Cannot find current executable");

    let mut exec_args = format!("{} run", binary.display());
    if let Some(d) = device {
        exec_args.push_str(&format!(" --device {}", d.display()));
    }
    if let Some(c) = config {
        exec_args.push_str(&format!(" --config {}", c.display()));
    }
    if let Some(s) = socket {
        exec_args.push_str(&format!(" --socket {}", s.display()));
    }

    let service = format!(
        "[Unit]\n\
         Description=Push-to-talk mic via evdev (ptt-mic)\n\
         After=graphical-session.target\n\
         \n\
         [Service]\n\
         ExecStart={exec_args}\n\
         Restart=always\n\
         RestartSec=3\n\
         \n\
         [Install]\n\
         WantedBy=default.target\n",
    );

    fs::create_dir_all(service_dir()).expect("Cannot create systemd user dir");

    let path = service_path();
    let mut file = fs::File::create(&path).expect("Cannot write service file");
    file.write_all(service.as_bytes())
        .expect("Cannot write service content");

    println!("Service file written to {:?}", path);

    for args in [
        vec!["--user", "daemon-reload"],
        vec!["--user", "enable", "ptt-mic"],
        vec!["--user", "start", "ptt-mic"],
    ] {
        let status = Command::new("systemctl").args(&args).status();

        match status {
            Ok(s) if s.success() => {}
            Ok(s) => eprintln!("systemctl {:?} exited with {}", args, s),
            Err(e) => eprintln!("Failed to run systemctl: {}", e),
        }
    }

    println!("ptt-mic installed and started.");
    println!("Check status: systemctl --user status ptt-mic");
}

fn uninstall() {
    for args in [
        vec!["--user", "stop", "ptt-mic"],
        vec!["--user", "disable", "ptt-mic"],
    ] {
        let _ = Command::new("systemctl").args(&args).status();
    }

    let path = service_path();
    if path.exists() {
        fs::remove_file(&path).expect("Cannot remove service file");
        println!("Removed {:?}", path);
    } else {
        println!("Service file not found, nothing to remove.");
    }

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    println!("ptt-mic uninstalled.");
}

fn handle_ipc_command(socket: Option<PathBuf>, cmd: IpcCommand) {
    let socket_path = socket.unwrap_or_else(get_default_socket_path);
    match send_command(&socket_path, &cmd) {
        Ok(resp) => {
            if resp.ok {
                if let IpcCommand::Status = cmd {
                    println!("Daemon is running.");
                    println!("Mode: {:?}", resp.mode.unwrap_or_default());
                    println!("Enabled: {:?}", resp.enabled.unwrap_or(false));
                    println!("Device: {:?}", resp.device.unwrap_or_default());
                } else {
                    println!("Success");
                }
            } else {
                eprintln!("Error: {}", resp.error.unwrap_or_default());
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!(
                "Is the daemon running? Check socket path: {:?}",
                socket_path
            );
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            device,
            config,
            socket,
        } => {
            let config_path = config.unwrap_or_else(get_default_config_path);
            let mut cfg = Config::load(&config_path).unwrap_or_else(|e| {
                eprintln!("Failed to load config {:?}: {}", config_path, e);
                std::process::exit(1);
            });

            if let Some(d) = device {
                cfg.general.device = d;
            }

            let socket_path = socket.unwrap_or_else(get_default_socket_path);
            daemon::run(cfg, socket_path);
        }
        Commands::Install {
            device,
            config,
            socket,
        } => install(device, config, socket),
        Commands::Uninstall => uninstall(),
        Commands::Mode { name } => handle_ipc_command(None, IpcCommand::Mode { name }),
        Commands::Enable => handle_ipc_command(None, IpcCommand::Enable),
        Commands::Disable => handle_ipc_command(None, IpcCommand::Disable),
        Commands::Toggle => handle_ipc_command(None, IpcCommand::Toggle),
        Commands::Status => handle_ipc_command(None, IpcCommand::Status),
    }
}
