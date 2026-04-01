use std::process::{Command, Stdio};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TmuxSession {
    pub name: String,
    pub windows: u32,
    pub attached: bool,
    /// Unix timestamp (seconds since epoch)
    pub created: u64,
}

// ---------------------------------------------------------------------------
// Internal process helpers
// ---------------------------------------------------------------------------

struct Output {
    stdout: String,
    exit_code: i32,
}

fn run(args: &[&str]) -> Output {
    let result = Command::new("tmux").args(args).output();

    match result {
        Ok(out) => Output {
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            exit_code: out.status.code().unwrap_or(1),
        },
        Err(_) => Output {
            stdout: String::new(),
            exit_code: 1,
        },
    }
}

/// Runs tmux with stdio inherited — used for attach-session and switch-client.
fn run_interactive(args: &[&str]) -> i32 {
    Command::new("tmux")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map(|s| s.code().unwrap_or(1))
        .unwrap_or(1)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn has_tmux() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn list_sessions() -> Vec<TmuxSession> {
    let out = run(&[
        "list-sessions",
        "-F",
        "#{session_name}:#{session_windows}:#{session_attached}:#{session_created}",
    ]);
    if out.exit_code != 0 {
        return Vec::new();
    }
    parse_sessions(&out.stdout)
}

fn parse_sessions(raw: &str) -> Vec<TmuxSession> {
    raw.trim()
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            // Split from the right: the last 3 fields are always numeric.
            // This handles session names that contain colons.
            let mut parts = line.rsplitn(4, ':');
            let created: u64 = parts.next()?.parse().ok()?;
            let attached = parts.next()? == "1";
            let windows: u32 = parts.next()?.parse().ok()?;
            let name = parts.next()?.to_string();
            Some(TmuxSession {
                name,
                windows,
                attached,
                created,
            })
        })
        .collect()
}

pub fn has_session(name: &str) -> bool {
    run(&["has-session", "-t", name]).exit_code == 0
}

pub fn create_session(name: &str, cwd: &str) -> bool {
    run(&["new-session", "-d", "-s", name, "-c", cwd]).exit_code == 0
}

/// Attaches to a session (used outside tmux). Inherits the terminal.
pub fn attach_session(name: &str) -> bool {
    run_interactive(&["attach-session", "-t", name]) == 0
}

/// Switches the current tmux client to another session (used inside tmux).
pub fn switch_client(name: &str) -> bool {
    run_interactive(&["switch-client", "-t", name]) == 0
}

/// Switches the current tmux client to the previously active session.
pub fn switch_to_last() -> bool {
    run_interactive(&["switch-client", "-l"]) == 0
}

/// Sends a command to the first pane of a session, followed by Enter.
/// Used only on new session creation — no escaping is performed.
pub fn send_keys(session: &str, cmd: &str) -> bool {
    let target = format!("{session}:");
    run(&["send-keys", "-t", &target, cmd, "Enter"]).exit_code == 0
}

pub fn kill_session(name: &str) -> bool {
    run(&["kill-session", "-t", name]).exit_code == 0
}

pub fn current_session() -> Option<String> {
    let out = run(&["display-message", "-p", "#{session_name}"]);
    if out.exit_code != 0 {
        return None;
    }
    let name = out.stdout.trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}
