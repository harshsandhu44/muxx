use std::fmt::Write;

use anyhow::{bail, Result};

use crate::core::{env::is_inside_tmux, notes::load_notes, tags::load_tags, tmux::current_session};

pub fn run() -> Result<()> {
    if !is_inside_tmux() {
        bail!("not inside a tmux session");
    }

    let name =
        current_session().ok_or_else(|| anyhow::anyhow!("could not determine current session"))?;

    let tags = load_tags().get_tags(&name);
    let notes_store = load_notes();
    let note = notes_store.get_note(&name);

    let mut out = name.clone();

    if !tags.is_empty() {
        write!(out, " [{}]", tags.join(",")).unwrap();
    }

    if let Some(n) = note {
        write!(out, " \u{2014} {n}").unwrap();
    }

    println!("{out}");
    Ok(())
}
