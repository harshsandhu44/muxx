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
            stdout: String::from_utf8(out.stdout)
                .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
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

pub fn rename_session(old: &str, new: &str) -> bool {
    run(&["rename-session", "-t", old, new]).exit_code == 0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sessions_empty_input() {
        assert!(parse_sessions("").is_empty());
    }

    #[test]
    fn parse_sessions_whitespace_only() {
        assert!(parse_sessions("   \n  \n").is_empty());
    }

    #[test]
    fn parse_sessions_single_detached() {
        let raw = "mysession:2:0:1700000000\n";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "mysession");
        assert_eq!(sessions[0].windows, 2);
        assert!(!sessions[0].attached);
        assert_eq!(sessions[0].created, 1700000000);
    }

    #[test]
    fn parse_sessions_single_attached() {
        let raw = "work:1:1:1700000001\n";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].attached);
        assert_eq!(sessions[0].name, "work");
        assert_eq!(sessions[0].windows, 1);
    }

    #[test]
    fn parse_sessions_name_with_colon() {
        // rsplitn from the right: name field absorbs embedded colons
        let raw = "my:session:3:0:1700000002\n";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "my:session");
        assert_eq!(sessions[0].windows, 3);
        assert!(!sessions[0].attached);
        assert_eq!(sessions[0].created, 1700000002);
    }

    #[test]
    fn parse_sessions_multiple() {
        let raw = "alpha:1:0:100\nbeta:2:1:200\n";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].name, "alpha");
        assert!(!sessions[0].attached);
        assert_eq!(sessions[1].name, "beta");
        assert!(sessions[1].attached);
    }

    #[test]
    fn parse_sessions_skips_blank_lines() {
        let raw = "\nalpha:1:0:100\n\nbeta:2:1:200\n\n";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn parse_sessions_skips_malformed_line() {
        let raw = "bad-line\ngood:1:0:100\n";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "good");
    }

    #[test]
    fn parse_sessions_windows_count_preserved() {
        let raw = "multi:5:0:9999\n";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions[0].windows, 5);
    }

    #[test]
    fn parse_sessions_no_trailing_newline() {
        let raw = "solo:1:0:42";
        let sessions = parse_sessions(raw);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "solo");
    }
}
