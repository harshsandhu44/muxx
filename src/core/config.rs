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

pub fn config_path() -> PathBuf {
    // Allow override via env var for testing
    if let Ok(p) = std::env::var("MUXX_CONFIG_PATH") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".config")
        .join("muxx")
        .join("config.toml")
}

pub fn load_config() -> MuxxConfig {
    load_config_from(&config_path())
}

fn load_config_from(path: &std::path::Path) -> MuxxConfig {
    match std::fs::read_to_string(path) {
        Ok(raw) => match toml::from_str::<MuxxConfig>(&raw) {
            Ok(cfg) => cfg,
            Err(e) => {
                crate::core::output::error(&format!("invalid TOML in {}: {}", path.display(), e));
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
[projects.myapp]
cwd = "/home/user/myapp"

[projects.api]
cwd = "/home/user/api"
startup = "npm run dev"
        "#;
        toml::from_str(raw).unwrap()
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
        let cfg = load_config_from(std::path::Path::new(
            "/tmp/muxx-test-nonexistent-config.json",
        ));
        assert!(cfg.projects.is_empty());
    }

    #[test]
    fn load_config_parses_valid_file() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "[projects.foo]\ncwd = \"/tmp/foo\"\n").unwrap();
        let cfg = load_config_from(f.path());
        assert!(resolve_project(&cfg, "foo").is_some());
    }

    #[test]
    fn parse_config_empty_object_is_valid() {
        let cfg: MuxxConfig = toml::from_str("").unwrap();
        assert!(cfg.projects.is_empty());
    }

    #[test]
    fn parse_config_multiple_projects() {
        let raw = r#"
[projects.a]
cwd = "/a"

[projects.b]
cwd = "/b"
startup = "npm start"

[projects.c]
cwd = "/c"
"#;
        let cfg: MuxxConfig = toml::from_str(raw).unwrap();
        assert_eq!(cfg.projects.len(), 3);
        assert_eq!(
            resolve_project(&cfg, "b").unwrap().startup.as_deref(),
            Some("npm start")
        );
        assert!(resolve_project(&cfg, "c").unwrap().startup.is_none());
    }

    #[test]
    fn parse_config_project_cwd_is_preserved() {
        let raw = "[projects.proj]\ncwd = \"/home/user/project\"\n";
        let cfg: MuxxConfig = toml::from_str(raw).unwrap();
        assert_eq!(
            resolve_project(&cfg, "proj").unwrap().cwd,
            "/home/user/project"
        );
    }

    #[test]
    fn load_config_from_file_with_env_var_path() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "[projects.envtest]\ncwd = \"/tmp/envtest\"\n").unwrap();
        // load_config_from accepts a path directly — no env var manipulation needed
        let cfg = load_config_from(f.path());
        assert!(resolve_project(&cfg, "envtest").is_some());
        assert_eq!(
            resolve_project(&cfg, "envtest").unwrap().cwd,
            "/tmp/envtest"
        );
    }

    #[test]
    fn resolve_project_key_is_case_sensitive() {
        let cfg = base_config();
        // "MyApp" vs "myapp" — keys are case-sensitive
        assert!(resolve_project(&cfg, "MyApp").is_none());
        assert!(resolve_project(&cfg, "myapp").is_some());
    }
}
