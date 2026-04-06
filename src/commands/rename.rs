use anyhow::Result;

use crate::core::{
    output::{error, success},
    session_name::sanitize_session_name,
    state,
    tmux::{has_session, has_tmux, rename_session},
};

pub fn run(from: &str, to: &str) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    if !has_session(from) {
        error(&format!("session not found: {from}"));
        std::process::exit(1);
    }

    let to = sanitize_session_name(to);

    if has_session(&to) {
        error(&format!("session already exists: {to}"));
        std::process::exit(1);
    }

    if !rename_session(from, &to) {
        error(&format!("failed to rename session: {from}"));
        std::process::exit(1);
    }

    if state::load_last_session().as_deref() == Some(from) {
        state::save_last_session(&to);
    }

    success(&format!("renamed: {from} -> {to}"));
    Ok(())
}
