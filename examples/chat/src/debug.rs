//! Simple debug logging to file.
//!
//! Enable with: TELEX_DEBUG=1
//! Logs to: ~/.config/telex-ai/debug.log

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

/// Initialize the debug logger.
pub fn init() {
    if std::env::var("TELEX_DEBUG").is_ok() {
        if let Some(path) = log_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            if let Ok(file) = OpenOptions::new().create(true).append(true).open(&path) {
                if let Ok(mut guard) = LOG_FILE.lock() {
                    *guard = Some(file);
                }
                log("=== telex-ai started ===");
            }
        }
    }
}

/// Log a message to the debug file.
pub fn log(msg: &str) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(file) = guard.as_mut() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() % 1_000_000)
                .unwrap_or(0);
            let _ = writeln!(file, "[{:06}] {}", timestamp, msg);
            let _ = file.flush();
        }
    }
}

/// Log a separator for user input.
pub fn log_user_input(prompt: &str) {
    log("────────────────────────────────────────────────────────────────────────────────");
    log(&format!(">>> USER: {}", prompt));
    log("────────────────────────────────────────────────────────────────────────────────");
}

/// Log end of response.
pub fn log_response_end() {
    log("════════════════════════════════════════════════════════════════════════════════");
    log("<<< RESPONSE COMPLETE");
    log("");
}

fn log_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(home).join(".config/telex-ai/debug.log"))
}
