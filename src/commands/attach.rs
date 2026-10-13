use anyhow::{bail, Result};

use crate::core::{env::is_inside_tmux, output, state, tmux};

pub fn run(session: &str) -> Result<()> {
    if !tmux::has_tmux() {
        bail!("tmux is not installed or not in PATH");
    }

    if session == "-" {
        return attach_last();
    }

    let name = crate::commands::connect::resolve_live_session(session)?;
    do_attach(&name)
}

fn do_attach(session: &str) -> Result<()> {
    state::save_last_session(session);

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

fn attach_last() -> Result<()> {
    if is_inside_tmux() {
        if !tmux::switch_to_last() {
            bail!("no previous session");
        }
        return Ok(());
    }

    match state::load_last_session() {
        Some(name) => do_attach(&name),
        None => {
            output::hint("use 'muxx <name>' to attach to a session first");
            bail!("no last session recorded");
        }
    }
}
