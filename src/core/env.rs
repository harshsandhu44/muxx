use std::path::PathBuf;

use anyhow::{bail, Context, Result};

pub fn is_inside_tmux() -> bool {
    is_inside_tmux_with(std::env::var("TMUX").ok())
}

fn is_inside_tmux_with(val: Option<String>) -> bool {
    val.map(|v| !v.is_empty()).unwrap_or(false)
}

pub fn expand_home(input: &str) -> String {
    if input == "~" {
        return dirs::home_dir()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|| input.to_string());
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    input.to_string()
}

pub fn resolve_dir(target: Option<&str>) -> Result<PathBuf> {
    let raw = match target {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => std::env::current_dir()
            .context("failed to get current directory")?
            .to_string_lossy()
            .into_owned(),
    };

    let expanded = expand_home(&raw);
    let resolved = std::fs::canonicalize(&expanded)
        .with_context(|| format!("directory does not exist: {expanded}"))?;

    if !resolved.is_dir() {
        bail!("path is not a directory: {}", resolved.display());
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_home_tilde_alone() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_home("~"), home.to_string_lossy());
    }

    #[test]
    fn expand_home_tilde_slash_foo() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_home("~/foo"), home.join("foo").to_string_lossy());
    }

    #[test]
    fn expand_home_nested_path() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_home("~/a/b/c"), home.join("a/b/c").to_string_lossy());
    }

    #[test]
    fn expand_home_absolute_unchanged() {
        assert_eq!(expand_home("/usr/local/bin"), "/usr/local/bin");
    }

    #[test]
    fn expand_home_relative_unchanged() {
        assert_eq!(expand_home("some/relative/path"), "some/relative/path");
    }

    #[test]
    fn expand_home_tilde_word_unchanged() {
        assert_eq!(expand_home("~user/path"), "~user/path");
    }

    #[test]
    fn is_inside_tmux_unset() {
        assert!(!is_inside_tmux_with(None));
    }

    #[test]
    fn is_inside_tmux_empty() {
        assert!(!is_inside_tmux_with(Some(String::new())));
    }

    #[test]
    fn is_inside_tmux_set() {
        assert!(is_inside_tmux_with(Some(
            "/tmp/tmux-1000/default,12345,0".to_string()
        )));
    }
}
