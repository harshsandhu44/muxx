use anyhow::Result;

use crate::core::{
    env::is_inside_tmux,
    output::{error, success},
    tmux::{current_session, has_session, has_tmux, kill_session},
};

pub fn run(name: &str, force: bool) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    if !has_session(name) {
        error(&format!("session not found: {name}"));
        std::process::exit(1);
    }

    if !force && is_inside_tmux() {
        if let Some(current) = current_session() {
            if current == name {
                error(&format!(
                    "refusing to kill current session '{name}' (use --force to override)"
                ));
                std::process::exit(1);
            }
        }
    }

    if !kill_session(name) {
        error(&format!("failed to kill session: {name}"));
        std::process::exit(1);
    }

    success(&format!("killed: {name}"));
    Ok(())
}
