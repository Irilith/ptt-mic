use std::process::Command;

pub fn notify(summary: &str, body: &str) {
    let _ = Command::new("notify-send").arg(summary).arg(body).spawn();
}
