use anyhow::Result;

use crate::core::{
    notes::{load_notes, save_notes},
    output::{hint, success},
    tags::{load_tags, save_tags},
    tmux::{has_tmux, list_sessions},
};

pub fn run() -> Result<()> {
    if !has_tmux() {
        crate::core::output::error("tmux not found in PATH");
        std::process::exit(1);
    }

    let live: Vec<String> = list_sessions().into_iter().map(|s| s.name).collect();

    let mut tags_store = load_tags();
    let mut notes_store = load_notes();

    // GC tags
    let dead_tag_sessions: Vec<String> = tags_store
        .tags
        .keys()
        .filter(|s| !live.contains(s))
        .cloned()
        .collect();
    let tags_removed = dead_tag_sessions.len();
    for s in &dead_tag_sessions {
        tags_store.tags.remove(s);
    }

    // GC notes — collect names before mutating
    let dead_note_sessions: Vec<String> = notes_store
        .notes
        .keys()
        .filter(|s| !live.contains(s))
        .cloned()
        .collect();
    notes_store.gc(&live);

    if tags_removed == 0 && dead_note_sessions.is_empty() {
        hint("nothing to clean up");
        return Ok(());
    }

    if tags_removed > 0 {
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
