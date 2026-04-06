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

    #[test]
    fn resolve_dir_none_returns_current_directory() {
        let result = resolve_dir(None);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_dir());
    }

    #[test]
    fn resolve_dir_empty_string_falls_back_to_cwd() {
        let result = resolve_dir(Some(""));
        assert!(result.is_ok());
        assert!(result.unwrap().is_dir());
    }

    #[test]
    fn resolve_dir_whitespace_only_falls_back_to_cwd() {
        let result = resolve_dir(Some("   "));
        assert!(result.is_ok());
        assert!(result.unwrap().is_dir());
    }

    #[test]
    fn resolve_dir_valid_directory() {
        // /tmp always exists
        let result = resolve_dir(Some("/tmp"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_dir());
    }

    #[test]
    fn resolve_dir_nonexistent_path_errors() {
        let result = resolve_dir(Some("/tmp/muxx-nonexistent-dir-xyz-987654"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not exist"));
    }

    #[test]
    fn resolve_dir_file_path_errors() {
        // /etc/hosts is a file, not a directory — canonicalize succeeds but is_dir() fails
        let result = resolve_dir(Some("/etc/hosts"));
        assert!(result.is_err(), "expected error for file path");
    }

    #[test]
    fn resolve_dir_tilde_expands_to_home() {
        let result = resolve_dir(Some("~"));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        let home = dirs::home_dir().unwrap();
        assert_eq!(resolved, std::fs::canonicalize(home).unwrap());
    }
}
