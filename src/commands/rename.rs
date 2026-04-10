use anyhow::Result;

use crate::core::{
    output::{error, success},
    session_name::sanitize_session_name,
    state,
    tmux::{has_session, has_tmux, rename_session},
};

pub fn run(from: &str, to: &str) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    if !has_session(from) {
        error(&format!("session not found: {from}"));
        std::process::exit(1);
    }

    let to = sanitize_session_name(to);

    if has_session(&to) {
        error(&format!("session already exists: {to}"));
        std::process::exit(1);
    }

    if !rename_session(from, &to) {
        error(&format!("failed to rename session: {from}"));
        std::process::exit(1);
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
