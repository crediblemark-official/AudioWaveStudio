//! Minimal cross-platform diagnostic log.
//!
//! Windows release builds run with `windows_subsystem = "windows"`, so there
//! is no attached console and every `eprintln!` is silently dropped. That made
//! it impossible to answer "is the preview on GPU or CPU?" on Windows — the
//! log simply never existed.
//!
//! This module mirrors every diagnostic line to a real file on all platforms:
//!   • Windows: %LOCALAPPDATA%\Audiowave\audiowave-panic.log
//!   • Unix:    /tmp/audiowave-panic.log
//! The `AUDIOWAVE_PANIC_LOG` env var overrides the location everywhere.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serializes appends so lines from several threads (UI thread, export thread,
/// audio thread) never interleave mid-line. Poisoned locks fall back to the
/// inner guard — logging must never be able to take the app down.
static LOG_LOCK: Mutex<()> = Mutex::new(());

/// Where the diagnostic log lives on this platform.
pub fn log_path() -> PathBuf {
    if let Ok(p) = std::env::var("AUDIOWAVE_PANIC_LOG") {
        return PathBuf::from(p);
    }
    #[cfg(windows)]
    {
        if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
            let mut p = PathBuf::from(dir);
            p.push("Audiowave");
            p.push("audiowave-panic.log");
            return p;
        }
        PathBuf::from("audiowave-panic.log")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/tmp/audiowave-panic.log")
    }
}

/// Print one diagnostic line to stderr (visible in dev builds / a terminal)
/// AND append it to the log file. Never panics; a failure to write the file is
/// swallowed so logging can't break rendering.
pub fn write(msg: &str) {
    eprintln!("{msg}");
    let _guard = match LOG_LOCK.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    let path = log_path();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "{msg}");
    }
}

/// `crate::logline!("[Render] ... {}", x)` — same call shape as `eprintln!`,
/// but the line also lands in the on-disk diagnostic log.
#[macro_export]
macro_rules! logline {
    ($($arg:tt)*) => {
        $crate::logging::write(&format!($($arg)*))
    };
}
