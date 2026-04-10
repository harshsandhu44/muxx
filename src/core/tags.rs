use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TagsStore {
    #[serde(default)]
    pub tags: HashMap<String, Vec<String>>,
}

impl TagsStore {
    pub fn get_tags(&self, session: &str) -> Vec<String> {
        self.tags.get(session).cloned().unwrap_or_default()
    }

    pub fn add_tags(&mut self, session: &str, tags: &[String]) {
        let entry = self.tags.entry(session.to_string()).or_default();
        for tag in tags {
            let normalized = tag.trim().to_lowercase();
            if !normalized.is_empty() {
                entry.push(normalized);
            }
        }
        entry.sort();
        entry.dedup();
        if entry.is_empty() {
            self.tags.remove(session);
        }
    }

    pub fn remove_tags(&mut self, session: &str, tags: &[String]) {
        if let Some(entry) = self.tags.get_mut(session) {
            let to_remove: Vec<String> = tags.iter().map(|t| t.trim().to_lowercase()).collect();
            entry.retain(|t| !to_remove.contains(t));
            if entry.is_empty() {
                self.tags.remove(session);
            }
        }
    }

    pub fn clear_tags(&mut self, session: &str) {
        self.tags.remove(session);
    }

    pub fn rename_session(&mut self, old: &str, new: &str) {
        if let Some(tags) = self.tags.remove(old) {
            self.tags.insert(new.to_string(), tags);
        }
    }

    /// Removes a tag from every session that has it. Sessions with no remaining
    /// tags are cleaned up. Returns the number of sessions that were affected.
    pub fn delete_tag(&mut self, tag: &str) -> usize {
        let normalized = tag.trim().to_lowercase();
        let mut affected = 0;
        let sessions: Vec<String> = self.tags.keys().cloned().collect();
        for session in sessions {
            if let Some(entry) = self.tags.get_mut(&session) {
                let before = entry.len();
                entry.retain(|t| t != &normalized);
                if entry.len() < before {
                    affected += 1;
                    if entry.is_empty() {
                        self.tags.remove(&session);
                    }
                }
            }
        }
        affected
    }

    /// Returns a sorted, deduplicated list of every tag used across all sessions.
    pub fn all_known_tags(&self) -> Vec<String> {
        let mut all: Vec<String> = self.tags.values().flat_map(|v| v.iter().cloned()).collect();
        all.sort();
        all.dedup();
        all
    }
}

pub fn tags_path() -> PathBuf {
    if let Ok(p) = std::env::var("MUXX_TAGS_PATH") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".config")
        .join("muxx")
        .join("tags.toml")
}

pub fn load_tags() -> TagsStore {
    load_tags_from(&tags_path())
}

fn load_tags_from(path: &std::path::Path) -> TagsStore {
    match std::fs::read_to_string(path) {
        Ok(raw) => {
            if raw.trim().is_empty() {
                return TagsStore::default();
            }
            match toml::from_str::<TagsStore>(&raw) {
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
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => TagsStore::default(),
        Err(e) => {
            crate::core::output::error(&format!("failed to read tags {}: {}", path.display(), e));
            std::process::exit(1);
        }
    }
}

fn save_tags_to(store: &TagsStore, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let toml = toml::to_string(store)?;
    std::fs::write(path, toml)?;
    Ok(())
}

pub fn save_tags(store: &TagsStore) -> Result<()> {
    save_tags_to(store, &tags_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_tags_deduplicates() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["work".to_string(), "work".to_string()]);
        assert_eq!(store.get_tags("proj"), vec!["work"]);
    }

    #[test]
    fn add_tags_sorts() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["z".to_string(), "a".to_string()]);
        assert_eq!(store.get_tags("proj"), vec!["a", "z"]);
    }

    #[test]
    fn add_tags_normalises_case() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["Work".to_string(), "PYTHON".to_string()]);
        assert_eq!(store.get_tags("proj"), vec!["python", "work"]);
    }

    #[test]
    fn add_tags_trims_whitespace() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["  work  ".to_string()]);
        assert_eq!(store.get_tags("proj"), vec!["work"]);
    }

    #[test]
    fn add_tags_ignores_empty() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["".to_string(), "  ".to_string()]);
        assert!(store.get_tags("proj").is_empty());
        assert!(!store.tags.contains_key("proj"));
    }

    #[test]
    fn add_tags_accumulates_across_calls() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["work".to_string()]);
        store.add_tags("proj", &["python".to_string()]);
        assert_eq!(store.get_tags("proj"), vec!["python", "work"]);
    }

    #[test]
    fn remove_tags_partial() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["work".to_string(), "python".to_string()]);
        store.remove_tags("proj", &["python".to_string()]);
        assert_eq!(store.get_tags("proj"), vec!["work"]);
    }

    #[test]
    fn remove_tags_cleans_up_empty_key() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["work".to_string()]);
        store.remove_tags("proj", &["work".to_string()]);
        assert!(!store.tags.contains_key("proj"));
    }

    #[test]
    fn remove_tags_noop_for_unknown_session() {
        let mut store = TagsStore::default();
        store.remove_tags("nonexistent", &["work".to_string()]);
        assert!(store.tags.is_empty());
    }

    #[test]
    fn remove_tags_normalises_case() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["work".to_string()]);
        store.remove_tags("proj", &["WORK".to_string()]);
        assert!(store.get_tags("proj").is_empty());
    }

    #[test]
    fn clear_tags_removes_key() {
        let mut store = TagsStore::default();
        store.add_tags("proj", &["work".to_string()]);
        store.clear_tags("proj");
        assert!(!store.tags.contains_key("proj"));
    }

    #[test]
    fn clear_tags_noop_for_unknown_session() {
        let mut store = TagsStore::default();
        store.clear_tags("nonexistent");
        assert!(store.tags.is_empty());
    }

    #[test]
    fn rename_session_migrates_tags() {
        let mut store = TagsStore::default();
        store.add_tags("old", &["work".to_string()]);
        store.rename_session("old", "new");
        assert!(store.get_tags("old").is_empty());
        assert_eq!(store.get_tags("new"), vec!["work"]);
    }

    #[test]
    fn rename_session_noop_when_no_tags() {
        let mut store = TagsStore::default();
        store.rename_session("old", "new");
        assert!(store.tags.is_empty());
    }

    #[test]
    fn get_tags_returns_empty_for_unknown_session() {
        let store = TagsStore::default();
        assert!(store.get_tags("unknown").is_empty());
    }

    #[test]
    fn delete_tag_removes_from_all_sessions() {
        let mut store = TagsStore::default();
        store.add_tags("a", &["work".to_string(), "rust".to_string()]);
        store.add_tags("b", &["work".to_string(), "personal".to_string()]);
        let affected = store.delete_tag("work");
        assert_eq!(affected, 2);
        assert!(!store.get_tags("a").contains(&"work".to_string()));
        assert!(!store.get_tags("b").contains(&"work".to_string()));
        assert!(store.get_tags("a").contains(&"rust".to_string()));
    }

    #[test]
    fn delete_tag_cleans_up_empty_sessions() {
        let mut store = TagsStore::default();
        store.add_tags("a", &["work".to_string()]);
        store.delete_tag("work");
        assert!(!store.tags.contains_key("a"));
    }

    #[test]
    fn delete_tag_normalises_case() {
        let mut store = TagsStore::default();
        store.add_tags("a", &["work".to_string()]);
        store.delete_tag("WORK");
        assert!(store.get_tags("a").is_empty());
    }

    #[test]
    fn delete_tag_noop_when_tag_not_found() {
        let mut store = TagsStore::default();
        store.add_tags("a", &["work".to_string()]);
        let affected = store.delete_tag("nonexistent");
        assert_eq!(affected, 0);
        assert_eq!(store.get_tags("a"), vec!["work"]);
    }

    #[test]
    fn all_known_tags_returns_union() {
        let mut store = TagsStore::default();
        store.add_tags("a", &["work".to_string(), "rust".to_string()]);
        store.add_tags("b", &["work".to_string(), "personal".to_string()]);
        let all = store.all_known_tags();
        assert_eq!(all, vec!["personal", "rust", "work"]);
    }

    #[test]
    fn all_known_tags_empty_store() {
        let store = TagsStore::default();
        assert!(store.all_known_tags().is_empty());
    }

    #[test]
    fn load_tags_returns_default_when_missing() {
        let store = load_tags_from(std::path::Path::new(
            "/tmp/muxx-test-nonexistent-tags-file.json",
        ));
        assert!(store.tags.is_empty());
    }

    #[test]
    fn roundtrip_save_and_load() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_path_buf();

        let mut store = TagsStore::default();
        store.add_tags("proj", &["work".to_string(), "rust".to_string()]);

        save_tags_to(&store, &path).unwrap();
        let loaded = load_tags_from(&path);
        assert_eq!(loaded.get_tags("proj"), vec!["rust", "work"]);

        drop(f);
    }

    #[test]
    fn roundtrip_empty_store() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_path_buf();

        let store = TagsStore::default();
        save_tags_to(&store, &path).unwrap();
        let loaded = load_tags_from(&path);
        assert!(loaded.tags.is_empty());

        drop(f);
    }
}
