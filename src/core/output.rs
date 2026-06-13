use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

/// Set by the `--no-color` global flag. When true, color is suppressed
/// regardless of TTY detection.
static NO_COLOR_FLAG: AtomicBool = AtomicBool::new(false);

/// Disable colored output for the rest of the process (called when `--no-color` is passed).
pub fn set_no_color(value: bool) {
    NO_COLOR_FLAG.store(value, Ordering::Relaxed);
}

/// Whether to emit ANSI color. False if `--no-color` was passed, the `NO_COLOR`
/// env var is set, or the target stream is not a terminal.
fn color_enabled(is_tty: bool) -> bool {
    if NO_COLOR_FLAG.load(Ordering::Relaxed) {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    is_tty
}

/// Whether stdout should be colored. For callers that build ANSI sequences
/// directly (e.g. the `list` table) instead of using the helpers below.
pub fn stdout_color() -> bool {
    color_enabled(std::io::stdout().is_terminal())
}

pub fn success(msg: &str) {
    if color_enabled(std::io::stdout().is_terminal()) {
        println!("\x1b[32m✓\x1b[0m {msg}");
    } else {
        println!("✓ {msg}");
    }
}

pub fn info(msg: &str) {
    if color_enabled(std::io::stdout().is_terminal()) {
        println!("\x1b[36m→\x1b[0m {msg}");
    } else {
        println!("→ {msg}");
    }
}

pub fn error(msg: &str) {
    if color_enabled(std::io::stderr().is_terminal()) {
        eprintln!("\x1b[31m✗\x1b[0m {msg}");
    } else {
        eprintln!("✗ {msg}");
    }
}

pub fn hint(msg: &str) {
    if color_enabled(std::io::stdout().is_terminal()) {
        println!("\x1b[2m  {msg}\x1b[0m");
    } else {
        println!("  {msg}");
    }
}

pub fn warn(msg: &str) {
    if color_enabled(std::io::stderr().is_terminal()) {
        eprintln!("\x1b[33m!\x1b[0m {msg}");
    } else {
        eprintln!("! {msg}");
    }
}
