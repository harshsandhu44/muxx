use anyhow::Result;

use crate::core::{
    notes::{load_notes, save_notes},
    output::{error, hint, success},
    tmux::has_tmux,
};

pub fn run(session: &str, text: Option<&str>, clear: bool) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    let mut store = load_notes();

    if clear {
        store.clear_note(session);
        save_notes(&store)?;
        success(&format!("note cleared for: {session}"));
        return Ok(());
    }

    if let Some(t) = text {
        store.set_note(session, t);
        save_notes(&store)?;
        success(&format!("note set for: {session}"));
        return Ok(());
    }

    // No text and no --clear: print the current note.
    match store.get_note(session) {
        Some(note) => println!("{note}"),
        None => hint(&format!("no note for: {session}")),
    }

    Ok(())
}
