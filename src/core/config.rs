use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub cwd: String,
    pub startup: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct MuxxConfig {
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

fn config_path() -> PathBuf {
    // Allow override via env var for testing
    if let Ok(p) = std::env::var("MUXX_CONFIG_PATH") {
        return PathBuf::from(p);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("muxx")
        .join("config.json")
}

pub fn load_config() -> MuxxConfig {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(raw) => match serde_json::from_str::<MuxxConfig>(&raw) {
            Ok(cfg) => cfg,
            Err(e) => {
                crate::core::output::error(&format!("invalid JSON in {}: {}", path.display(), e));
                std::process::exit(1);
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => MuxxConfig::default(),
        Err(e) => {
            crate::core::output::error(&format!("failed to read config {}: {}", path.display(), e));
            std::process::exit(1);
        }
    }
}

pub fn resolve_project<'a>(config: &'a MuxxConfig, key: &str) -> Option<&'a ProjectConfig> {
    config.projects.get(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> MuxxConfig {
        let raw = r#"
        {
            "projects": {
                "myapp": { "cwd": "/home/user/myapp" },
                "api": { "cwd": "/home/user/api", "startup": "npm run dev" }
            }
        }
        "#;
        serde_json::from_str(raw).unwrap()
    }

    #[test]
    fn resolve_returns_config_for_existing_key() {
        let cfg = base_config();
        let p = resolve_project(&cfg, "myapp").unwrap();
        assert_eq!(p.cwd, "/home/user/myapp");
        assert!(p.startup.is_none());
    }

    #[test]
    fn resolve_returns_config_with_startup() {
        let cfg = base_config();
        let p = resolve_project(&cfg, "api").unwrap();
        assert_eq!(p.cwd, "/home/user/api");
        assert_eq!(p.startup.as_deref(), Some("npm run dev"));
    }

    #[test]
    fn resolve_returns_none_for_missing_key() {
        let cfg = base_config();
        assert!(resolve_project(&cfg, "nonexistent").is_none());
    }

    #[test]
    fn resolve_returns_none_on_empty_projects() {
        let cfg = MuxxConfig::default();
        assert!(resolve_project(&cfg, "anything").is_none());
    }

    #[test]
    fn load_config_returns_default_when_file_missing() {
        // Point to a path that doesn't exist
        std::env::set_var("MUXX_CONFIG_PATH", "/tmp/muxx-test-nonexistent-config.json");
        let cfg = load_config();
        std::env::remove_var("MUXX_CONFIG_PATH");
        assert!(cfg.projects.is_empty());
    }

    #[test]
    fn load_config_parses_valid_file() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, r#"{{"projects":{{"foo":{{"cwd":"/tmp/foo"}}}}}}"#).unwrap();
        std::env::set_var("MUXX_CONFIG_PATH", f.path());
        let cfg = load_config();
        std::env::remove_var("MUXX_CONFIG_PATH");
        assert!(resolve_project(&cfg, "foo").is_some());
    }
}
