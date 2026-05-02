use anyhow::{bail, Result};

use crate::core::{
    output::success,
    session_name::sanitize_session_name,
    state,
    tmux::{has_session, has_tmux, rename_session},
};

pub fn run(from: &str, to: &str) -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    if !has_session(from) {
        bail!("session not found: {from}");
    }

    let raw_to = to;
    let to = sanitize_session_name(to);

    if to.is_empty() {
        bail!("invalid name: \"{raw_to}\" produces an empty session name after sanitization");
    }

    if has_session(&to) {
        bail!("session already exists: {to}");
    }

    if !rename_session(from, &to) {
        bail!("failed to rename session: {from}");
    }

    state::update_last_session_if(from, &to);

    // Migrate tags to the new session name (best-effort).
    let mut tag_store = crate::core::tags::load_tags();
    tag_store.rename_session(from, &to);
    let _ = crate::core::tags::save_tags(&tag_store);

    // Migrate note to the new session name (best-effort).
    let mut notes_store = crate::core::notes::load_notes();
    notes_store.rename_session(from, &to);
    let _ = crate::core::notes::save_notes(&notes_store);

    success(&format!("renamed: {from} -> {to}"));
    Ok(())
}
