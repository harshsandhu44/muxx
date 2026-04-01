use anyhow::{bail, Result};

use crate::core::{env::is_inside_tmux, output, tmux};

pub fn run(session: &str) -> Result<()> {
    if !tmux::has_tmux() {
        bail!("tmux is not installed or not in PATH");
    }

    if !tmux::has_session(session) {
        output::error(&format!("session '{}' does not exist", session));
        output::hint("run 'muxx list' to see active sessions");
        std::process::exit(1);
    }

    let ok = if is_inside_tmux() {
        tmux::switch_client(session)
    } else {
        tmux::attach_session(session)
    };

    if !ok {
        bail!("failed to attach to session '{}'", session);
    }

    Ok(())
}
