use anyhow::Result;

use crate::core::{
    env::is_inside_tmux, notes::load_notes, output::error, tags::load_tags, tmux::current_session,
};

pub fn run() -> Result<()> {
    if !is_inside_tmux() {
        error("not inside a tmux session");
        std::process::exit(1);
    }

    let name = match current_session() {
        Some(n) => n,
        None => {
            error("could not determine current session");
            std::process::exit(1);
        }
    };

    let tags = load_tags().get_tags(&name);
    let notes_store = load_notes();
    let note = notes_store.get_note(&name);

    let mut out = name.clone();

    if !tags.is_empty() {
        out.push_str(&format!(" [{}]", tags.join(",")));
    }

    if let Some(n) = note {
        out.push_str(&format!(" — {n}"));
    }

    println!("{out}");
    Ok(())
}
