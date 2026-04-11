use anyhow::{bail, Result};

use crate::core::{
    env::is_inside_tmux,
    output::success,
    tmux::{current_session, has_session, has_tmux, kill_session},
};

pub fn run(name: &str, force: bool) -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    if !has_session(name) {
        bail!("session not found: {name}");
    }

    if !force && is_inside_tmux() {
        if let Some(current) = current_session() {
            if current == name {
                bail!("refusing to kill current session '{name}' (use --force to override)");
            }
        }
    }

    if !kill_session(name) {
        bail!("failed to kill session: {name}");
    }

    success(&format!("killed: {name}"));
    Ok(())
}
