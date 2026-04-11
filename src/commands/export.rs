use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use crate::core::{notes::load_notes, output::success, tags::load_tags};

#[derive(Serialize)]
struct Export {
    tags: HashMap<String, Vec<String>>,
    notes: HashMap<String, String>,
}

pub fn run(path: Option<&str>) -> Result<()> {
    let tags = load_tags();
    let notes = load_notes();

    let export = Export {
        tags: tags.tags,
        notes: notes.notes,
    };

    let toml = toml::to_string_pretty(&export)?;

    match path {
        Some(p) => {
            std::fs::write(p, &toml)?;
            success(&format!("exported to {p}"));
        }
        None => print!("{toml}"),
    }

    Ok(())
}
