use crate::config::{Bind, Config};

pub struct State {
    pub mode: String,
    pub enabled: bool,
    pub config: Config,
}

impl State {
    pub fn new(config: Config) -> Self {
        let mode = config
            .general
            .default_mode
            .clone()
            .unwrap_or_else(|| config.modes.keys().next().cloned().unwrap_or_default());
        Self {
            mode,
            enabled: true,
            config,
        }
    }

    pub fn current_binds(&self) -> &[Bind] {
        self.config
            .modes
            .get(&self.mode)
            .map(|m| m.binds.as_slice())
            .unwrap_or(&[])
    }

    pub fn is_notify_enabled(&self) -> bool {
        self.config.general.notify
    }
}
