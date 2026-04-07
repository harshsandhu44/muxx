use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;

use crate::core::{
    env::is_inside_tmux,
    output::{error, hint},
    state,
    tmux::{attach_session, has_tmux, list_sessions, switch_client},
};

pub fn run(no_attach: bool) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    let sessions = list_sessions();

    if sessions.is_empty() {
        hint("no sessions");
        return Ok(());
    }

    let input: String = sessions
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = match Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            error("fzf not found in PATH — install it to use pick");
            std::process::exit(1);
        }
        Err(e) => return Err(e.into()),
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        // User cancelled (Ctrl-C / Escape) — exit cleanly
        return Ok(());
    }

    let session_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if session_name.is_empty() || no_attach {
        return Ok(());
    }

    state::save_last_session(&session_name);

    if is_inside_tmux() {
        if !switch_client(&session_name) {
            error(&format!("failed to switch to session: {session_name}"));
            std::process::exit(1);
        }
    } else if !attach_session(&session_name) {
        error(&format!("failed to attach to session: {session_name}"));
        std::process::exit(1);
    }

    Ok(())
}
