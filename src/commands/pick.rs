use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};

use crate::commands::connect::{self, idle_aliases};
use crate::core::{
    output::hint,
    tags::load_tags,
    tmux::{has_tmux, list_sessions},
};

pub fn run(no_attach: bool, filter_tags: &[String]) -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    let mut sessions = list_sessions();
    let tags_store = load_tags();
    // Tags live on sessions, so a tag filter hides not-yet-running aliases.
    let aliases = if filter_tags.is_empty() {
        idle_aliases(&sessions)
    } else {
        Vec::new()
    };

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

    if sessions.is_empty() && aliases.is_empty() {
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
        .chain(
            aliases
                .iter()
                .map(|(name, cwd)| format!("{name}\t(new) {cwd}")),
        )
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
    let line = raw.lines().next().unwrap_or("");
    let session_name = line.split('\t').next().unwrap_or("").trim().to_string();

    if session_name.is_empty() {
        return Ok(());
    }

    // Same path as `muxx <name>`: attaches a live session or creates from an alias.
    connect::run(Some(&session_name), None, None, no_attach, None, false)
}
