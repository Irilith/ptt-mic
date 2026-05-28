# ptt-mic

> **Disclaimer:** This project was built for personal use and tied to a specific setup. It probably won't work as expected for you.

`ptt-mic` is a push-to-talk daemon for Linux, written in Rust. It reads raw input events from an evdev device and runs commands on button press and release. Supports runtime mode switching and IPC via a Unix socket.

## Table of contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [How to build](#how-to-build)
- [Setup](#setup)
- [Usage](#usage)
- [Uninstalling](#uninstalling)
- [Design](#design)
- [Compatibility](#compatibility)
- [Contributing](#contributing)

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

## How to build

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
device = "/dev/input/event6"    # run `evtest` to find your desired device
notify = true
default_mode = "desktop"

[mode.obs]
[[mode.obs.binds]]
button = "BTN_SIDE"  # run `evtest` to find your button event code
press = ["obs-cmd", "audio", "unmute", "Mic/Aux"]
release = ["obs-cmd", "audio", "mute", "Mic/Aux"]

[mode.desktop]
device = "/dev/input/event11" 
[[mode.desktop.binds]]
button = "KEY_V"
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

## Design

There are no restrictions on what actions you can trigger. Press and release are just shell commands, so you can wire up anything that has a CLI:

```toml
press = ["pactl", "set-source-mute", "@DEFAULT_SOURCE@", "0"]
release = ["pactl", "set-source-mute", "@DEFAULT_SOURCE@", "1"]
```

Any tool that can be called from a terminal works here.

## Compatibility

Only tested on CachyOS with the Niri compositor. Never tested on other distros or window managers.

For audio I use PipeWire, but since actions are just shell commands you can use whatever backend you want. `pactl` should cover most setups.

## Contributing

There are plenty of things in my mind to improve but this (project) isn't my main focus right now. If you feel like something a PTT daemon should have is missing, feel free to contribute. I'll check it when I can.
