use anyhow::{bail, Result};

use crate::commands::connect::do_attach;
use crate::core::{
    env::is_inside_tmux,
    output::hint,
    tmux::{has_tmux, list_sessions, switch_to_last, TmuxSession},
};

/// Inside tmux: the client's previous session (tmux tracks it per client).
/// Outside tmux: the most recently attached session, per tmux's own
/// `session_last_attached`, so switches made with tmux keybindings count too.
pub fn run() -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    if is_inside_tmux() {
        if !switch_to_last() {
            bail!("no previous session");
        }
        return Ok(());
    }

    match most_recent(&list_sessions()) {
        Some(name) => do_attach(name, false),
        None => {
            hint("use 'muxx <name>' to attach to a session first");
            bail!("no last session");
        }
    }
}

fn most_recent(sessions: &[TmuxSession]) -> Option<&str> {
    sessions
        .iter()
        .filter(|s| s.last_attached > 0)
        .max_by_key(|s| s.last_attached)
        .map(|s| s.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(name: &str, last_attached: u64) -> TmuxSession {
        TmuxSession {
            name: name.to_string(),
            windows: 1,
            attached: false,
            created: 0,
            last_attached,
        }
    }

    #[test]
    fn most_recent_picks_highest_last_attached() {
        let s = [session("a", 100), session("b", 300), session("c", 200)];
        assert_eq!(most_recent(&s), Some("b"));
    }

    #[test]
    fn most_recent_ignores_never_attached() {
        assert_eq!(most_recent(&[session("a", 0)]), None);
        assert_eq!(most_recent(&[]), None);
    }
}
