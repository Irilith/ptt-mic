# ptt-mic

> **Disclaimer:** This project was built for personal use and tied to a specific setup. It probably won't work as expected for you.

`ptt-mic` is a push-to-talk daemon for Linux, written in Rust. It reads raw input events from an evdev device and runs commands on button press and release. Supports runtime mode switching and IPC via a Unix socket.

## Prerequisites

- Rust toolchain (`cargo`)
- Your user needs to be in the `input` group to read evdev devices without root:
```bash
  sudo usermod -aG input $USER
```
  Log out and back in after running this.
- `obs-cmd` if you want OBS mode.
- `notify-send` for desktop notifications (optional).

## Installation

Download the latest binary from the [release page](https://codeberg.org/Cinders/ptt-mic/releases) or build from source.

## How to Build

```bash
cargo build --release
```

## Setup

**1. Find your device path**

Figure out which `/dev/input/eventX` is your mouse or keyboard.

**2. Create a config file**

Default location: `~/.config/ptt-mic/config.toml`

```toml
[general]
device = "/dev/input/event6"    # replace with your actual device
notify = true
default_mode = "desktop"

[mode.obs]
[[mode.obs.binds]]
button = "BTN_SIDE"  # run `evtest` to find your button event code
press = ["obs-cmd", "audio", "unmute", "Mic/Aux"]
release = ["obs-cmd", "audio", "mute", "Mic/Aux"]

[mode.desktop]
[[mode.desktop.binds]]
button = "BTN_SIDE"
press = ["pactl", "set-source-mute", "@DEFAULT_SOURCE@", "0"]
release = ["pactl", "set-source-mute", "@DEFAULT_SOURCE@", "1"]
```

**3. Install the systemd user service**

```bash
./target/release/ptt-mic install
```

This generates the service file, reloads the systemd daemon, and enables and starts `ptt-mic`.

## Usage

Once the daemon is running, use the CLI to talk to it.

```bash
# Switch mode
ptt-mic mode <name>
ptt-mic mode desktop

# Enable or disable listening
ptt-mic enable
ptt-mic disable
ptt-mic toggle

# Check status
ptt-mic status
```

## Uninstalling

```bash
ptt-mic uninstall
```

Stops the service and removes the systemd file.
