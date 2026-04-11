use anyhow::{bail, Result};

use crate::core::{
    notes::{load_notes, save_notes},
    output::{hint, success},
    tags::{load_tags, save_tags},
    tmux::{has_tmux, list_sessions},
};

pub fn run() -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    let live: Vec<String> = list_sessions().into_iter().map(|s| s.name).collect();

    let mut tags_store = load_tags();
    let mut notes_store = load_notes();

    let dead_tag_sessions = tags_store.gc(&live);
    let dead_note_sessions = notes_store.gc(&live);

    if dead_tag_sessions.is_empty() && dead_note_sessions.is_empty() {
        hint("nothing to clean up");
        return Ok(());
    }

    if !dead_tag_sessions.is_empty() {
        save_tags(&tags_store)?;
        for s in &dead_tag_sessions {
            success(&format!("removed tags for dead session: {s}"));
        }
    }

    if !dead_note_sessions.is_empty() {
        save_notes(&notes_store)?;
        for s in &dead_note_sessions {
            success(&format!("removed note for dead session: {s}"));
        }
    }

    Ok(())
}
