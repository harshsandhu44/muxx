use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::core::{
    notes::{load_notes, save_notes, NotesStore},
    output::success,
    tags::{load_tags, save_tags, TagsStore},
};

#[derive(Deserialize)]
struct Import {
    #[serde(default)]
    tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    notes: HashMap<String, String>,
}

pub fn run(path: &str, merge: bool) -> Result<()> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {path}: {e}"))?;

    let data: Import = toml::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("invalid TOML in {path}: {e}"))?;

    let mut tags_store = if merge {
        load_tags()
    } else {
        TagsStore::default()
    };
    let mut notes_store = if merge {
        load_notes()
    } else {
        NotesStore::default()
    };

    for (session, tags) in &data.tags {
        tags_store.add_tags(session, tags);
    }
    for (session, note) in &data.notes {
        notes_store.set_note(session, note);
    }

    save_tags(&tags_store)?;
    save_notes(&notes_store)?;

    success(&format!(
        "imported {} sessions with tags, {} notes ({})",
        data.tags.len(),
        data.notes.len(),
        if merge { "merged" } else { "replaced" },
    ));

    Ok(())
}
