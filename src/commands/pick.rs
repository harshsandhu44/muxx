use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::core::{
    env::is_inside_tmux,
    output::hint,
    state,
    tags::load_tags,
    tmux::{attach_session, has_tmux, list_sessions, switch_client},
};

pub fn run(no_attach: bool, filter_tags: &[String]) -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    let mut sessions = list_sessions();
    let tags_store = load_tags();

    // Apply tag filter: keep sessions that have ALL of the requested tags.
    if !filter_tags.is_empty() {
        let normalized: Vec<String> = filter_tags
            .iter()
            .map(|t| t.trim().to_lowercase())
            .collect();
        sessions.retain(|s| {
            let session_tags = tags_store.get_tags(&s.name);
            normalized.iter().all(|ft| session_tags.contains(ft))
        });
    }

    if sessions.is_empty() {
        hint("no sessions");
        return Ok(());
    }

    // Format lines as "session_name\ttag1, tag2" so fzf shows tags and they
    // are fuzzy-searchable alongside the session name.
    let input: String = sessions
        .iter()
        .map(|s| {
            let tags = tags_store.get_tags(&s.name);
            if tags.is_empty() {
                s.name.clone()
            } else {
                format!("{}\t{}", s.name, tags.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = match Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!("fzf not found in PATH — install it to use pick");
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

    let raw = String::from_utf8_lossy(&output.stdout);
    // Extract session name — it's the part before the tab (tags follow after \t).
    let session_name = raw.split('\t').next().unwrap_or("").trim().to_string();

    if session_name.is_empty() || no_attach {
        return Ok(());
    }

    state::save_last_session(&session_name);

    if is_inside_tmux() {
        if !switch_client(&session_name) {
            bail!("failed to switch to session: {session_name}");
        }
    } else if !attach_session(&session_name) {
        bail!("failed to attach to session: {session_name}");
    }

    Ok(())
}
