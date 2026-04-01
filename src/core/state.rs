use std::path::PathBuf;

fn state_file() -> Option<PathBuf> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("muxx");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("last_session"))
}

/// Returns the last recorded session name, or `None` if not set.
pub fn load_last_session() -> Option<String> {
    let s = std::fs::read_to_string(state_file()?).ok()?;
    let s = s.trim().to_owned();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Persists the session name as the last attached session (best-effort).
pub fn save_last_session(name: &str) {
    if let Some(path) = state_file() {
        let _ = std::fs::write(path, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        save_last_session("my-session");
        assert_eq!(load_last_session().as_deref(), Some("my-session"));
    }
}
