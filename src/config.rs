use evdev::Key;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Config {
    pub general: GeneralConfig,
    pub modes: HashMap<String, ModeConfig>,
}

#[derive(Clone, Debug)]
pub struct GeneralConfig {
    pub device: PathBuf,
    pub notify: bool,
    pub default_mode: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ModeConfig {
    pub device: Option<PathBuf>,
    pub binds: Vec<Bind>,
}

#[derive(Clone, Debug)]
pub struct Bind {
    pub button: Key,
    pub press: Option<Vec<String>>,
    pub release: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawConfig {
    general: RawGeneralConfig,
    #[serde(default)]
    mode: HashMap<String, RawModeConfig>,
}

#[derive(Deserialize)]
struct RawGeneralConfig {
    device: PathBuf,
    #[serde(default = "default_notify")]
    notify: bool,
    default_mode: Option<String>,
}

fn default_notify() -> bool {
    true
}

#[derive(Deserialize)]
struct RawModeConfig {
    device: Option<PathBuf>,
    #[serde(default)]
    binds: Vec<RawBind>,
}

#[derive(Deserialize)]
struct RawBind {
    button: String,
    press: Option<Vec<String>>,
    release: Option<Vec<String>>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let raw: RawConfig = toml::from_str(&content)?;

        if raw.mode.is_empty() {
            return Err("At least one mode must be defined".into());
        }

        if let Some(ref dm) = raw.general.default_mode
            && !raw.mode.contains_key(dm)
        {
            return Err(format!("default_mode '{}' is not defined", dm).into());
        }

        let mut modes = HashMap::new();
        for (name, raw_mode) in raw.mode {
            let mut binds = Vec::new();
            for raw_bind in raw_mode.binds {
                let button = Key::from_str(&raw_bind.button)
                    .map_err(|_| format!("Invalid button: {}", raw_bind.button))?;

                if raw_bind.press.is_none() && raw_bind.release.is_none() {
                    return Err(format!(
                        "Bind for button '{}' must have press or release action",
                        raw_bind.button
                    )
                    .into());
                }

                binds.push(Bind {
                    button,
                    press: raw_bind.press,
                    release: raw_bind.release,
                });
            }
            modes.insert(name, ModeConfig {
                device: raw_mode.device,
                binds,
            });
        }

        Ok(Self {
            general: GeneralConfig {
                device: raw.general.device,
                notify: raw.general.notify,
                default_mode: raw.general.default_mode,
            },
            modes,
        })
    }

    pub fn resolve_device(&self, mode_config: &ModeConfig) -> PathBuf {
        mode_config.device.clone().unwrap_or_else(|| self.general.device.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_device() {
        let global_device = PathBuf::from("/dev/input/event0");
        let mode_device = PathBuf::from("/dev/input/event1");

        let config = Config {
            general: GeneralConfig {
                device: global_device.clone(),
                notify: true,
                default_mode: None,
            },
            modes: HashMap::new(),
        };

        // 1. All modes without device set -> fallback to global (general) device
        let mode_without_device = ModeConfig {
            device: None,
            binds: vec![],
        };
        assert_eq!(config.resolve_device(&mode_without_device), global_device);

        // 2. Mode with specific device set -> uses its own device
        let mode_with_device = ModeConfig {
            device: Some(mode_device.clone()),
            binds: vec![],
        };
        assert_eq!(config.resolve_device(&mode_with_device), mode_device);
    }
}
