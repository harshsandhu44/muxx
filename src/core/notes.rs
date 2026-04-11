use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NotesStore {
    #[serde(default)]
    pub notes: HashMap<String, String>,
}

impl NotesStore {
    pub fn get_note(&self, session: &str) -> Option<&str> {
        self.notes.get(session).map(String::as_str)
    }

    pub fn set_note(&mut self, session: &str, note: &str) {
        let trimmed = note.trim();
        if trimmed.is_empty() {
            self.notes.remove(session);
        } else {
            self.notes.insert(session.to_string(), trimmed.to_string());
        }
    }

    pub fn clear_note(&mut self, session: &str) {
        self.notes.remove(session);
    }

    pub fn rename_session(&mut self, old: &str, new: &str) {
        if let Some(note) = self.notes.remove(old) {
            self.notes.insert(new.to_string(), note);
        }
    }

    /// Remove notes for sessions not in `live`. Returns the names of removed sessions.
    pub fn gc(&mut self, live: &[String]) -> Vec<String> {
        let live_set: std::collections::HashSet<&str> = live.iter().map(String::as_str).collect();
        let dead: Vec<String> = self
            .notes
            .keys()
            .filter(|s| !live_set.contains(s.as_str()))
            .cloned()
            .collect();
        for s in &dead {
            self.notes.remove(s);
        }
        dead
    }
}

pub fn notes_path() -> PathBuf {
    if let Ok(p) = std::env::var("MUXX_NOTES_PATH") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".config")
        .join("muxx")
        .join("notes.toml")
}

pub fn load_notes() -> NotesStore {
    load_notes_from(&notes_path())
}

fn load_notes_from(path: &std::path::Path) -> NotesStore {
    match std::fs::read_to_string(path) {
        Ok(raw) => {
            if raw.trim().is_empty() {
                return NotesStore::default();
            }
            match toml::from_str::<NotesStore>(&raw) {
                Ok(store) => store,
                Err(e) => {
                    crate::core::output::error(&format!(
                        "invalid TOML in {}: {}",
                        path.display(),
                        e
                    ));
                    std::process::exit(1);
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => NotesStore::default(),
        Err(e) => {
            crate::core::output::error(&format!("failed to read notes {}: {}", path.display(), e));
            std::process::exit(1);
        }
    }
}

fn save_notes_to(store: &NotesStore, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let toml = toml::to_string(store)?;
    std::fs::write(path, toml)?;
    Ok(())
}

pub fn save_notes(store: &NotesStore) -> Result<()> {
    save_notes_to(store, &notes_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_note() {
        let mut store = NotesStore::default();
        store.set_note("proj", "working on auth");
        assert_eq!(store.get_note("proj"), Some("working on auth"));
    }

    #[test]
    fn set_note_trims_whitespace() {
        let mut store = NotesStore::default();
        store.set_note("proj", "  hello  ");
        assert_eq!(store.get_note("proj"), Some("hello"));
    }

    #[test]
    fn set_note_empty_removes_entry() {
        let mut store = NotesStore::default();
        store.set_note("proj", "hello");
        store.set_note("proj", "   ");
        assert!(store.get_note("proj").is_none());
        assert!(!store.notes.contains_key("proj"));
    }

    #[test]
    fn clear_note_removes_entry() {
        let mut store = NotesStore::default();
        store.set_note("proj", "hello");
        store.clear_note("proj");
        assert!(store.get_note("proj").is_none());
    }

    #[test]
    fn clear_note_noop_for_unknown_session() {
        let mut store = NotesStore::default();
        store.clear_note("nonexistent");
        assert!(store.notes.is_empty());
    }

    #[test]
    fn get_note_returns_none_for_unknown() {
        let store = NotesStore::default();
        assert!(store.get_note("unknown").is_none());
    }

    #[test]
    fn rename_session_migrates_note() {
        let mut store = NotesStore::default();
        store.set_note("old", "my note");
        store.rename_session("old", "new");
        assert!(store.get_note("old").is_none());
        assert_eq!(store.get_note("new"), Some("my note"));
    }

    #[test]
    fn rename_session_noop_when_no_note() {
        let mut store = NotesStore::default();
        store.rename_session("old", "new");
        assert!(store.notes.is_empty());
    }

    #[test]
    fn gc_removes_dead_sessions() {
        let mut store = NotesStore::default();
        store.set_note("alive", "keep");
        store.set_note("dead", "remove");
        let removed = store.gc(&["alive".to_string()]);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "dead");
        assert!(store.get_note("alive").is_some());
        assert!(store.get_note("dead").is_none());
    }

    #[test]
    fn gc_noop_when_all_live() {
        let mut store = NotesStore::default();
        store.set_note("a", "note");
        let removed = store.gc(&["a".to_string()]);
        assert!(removed.is_empty());
    }

    #[test]
    fn roundtrip_save_and_load() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_path_buf();

        let mut store = NotesStore::default();
        store.set_note("proj", "my note");

        save_notes_to(&store, &path).unwrap();
        let loaded = load_notes_from(&path);
        assert_eq!(loaded.get_note("proj"), Some("my note"));

        drop(f);
    }
}
