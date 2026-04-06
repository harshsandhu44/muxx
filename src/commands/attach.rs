use anyhow::{bail, Result};

use crate::core::{env::is_inside_tmux, fuzzy, output, state, tmux};

pub fn run(session: &str) -> Result<()> {
    if !tmux::has_tmux() {
        bail!("tmux is not installed or not in PATH");
    }

    if session == "-" {
        return attach_last();
    }

    if tmux::has_session(session) {
        return do_attach(session);
    }

    // Exact match failed — try fuzzy matching.
    let sessions = tmux::list_sessions();
    let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
    let matches = fuzzy::find_matches(session, &names);

    match matches.len() {
        0 => {
            output::error(&format!("session '{}' does not exist", session));
            output::hint("run 'muxx list' to see active sessions");
            std::process::exit(1);
        }
        1 => {
            output::info(&format!("matched session '{}'", matches[0]));
            do_attach(matches[0])
        }
        _ => {
            output::error(&format!(
                "ambiguous session name '{}', did you mean:",
                session
            ));
            for m in &matches {
                output::hint(&format!("  {m}"));
            }
            std::process::exit(1);
        }
    }
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
            output::error("no last session recorded");
            output::hint("use 'muxx attach <name>' to attach to a session first");
            std::process::exit(1);
        }
    }
}
