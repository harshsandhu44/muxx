use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::core::{
    config::{config_path, MuxxConfig},
    env::expand_home,
    output::{error, hint, success},
    session_name::sanitize_session_name,
    tmux::has_tmux,
};

pub fn run() -> Result<()> {
    let mut issues: u32 = 0;

    // --- Check 1: tmux ---
    if has_tmux() {
        success("tmux is installed");
    } else {
        error("tmux not found in PATH");
        hint("install tmux: brew install tmux  (macOS) or apt install tmux  (Linux)");
        issues += 1;
    }

    // --- Check 2: config file ---
    let path = config_path();
    if !path.exists() {
        hint(&format!(
            "no config file at {} — using defaults",
            path.display()
        ));
    } else {
        match std::fs::read_to_string(&path) {
            Err(e) => {
                error(&format!("cannot read config {}: {e}", path.display()));
                issues += 1;
            }
            Ok(raw) => match toml::from_str::<MuxxConfig>(&raw) {
                Err(e) => {
                    error(&format!("invalid TOML in config: {e}"));
                    issues += 1;
                }
                Ok(config) => {
                    success("config file is valid TOML");

                    // --- Check 3: project directories ---
                    for (name, proj) in &config.projects {
                        let expanded = expand_home(&proj.cwd);
                        if Path::new(&expanded).is_dir() {
                            success(&format!("project '{name}': directory exists"));
                        } else {
                            error(&format!(
                                "project '{name}': directory not found: {}",
                                proj.cwd
                            ));
                            issues += 1;
                        }
                    }

                    // --- Check 4: duplicate session names ---
                    let mut seen: HashMap<String, Vec<String>> = HashMap::new();
                    for name in config.projects.keys() {
                        seen.entry(sanitize_session_name(name))
                            .or_default()
                            .push(name.clone());
                    }
                    for (sanitized, names) in &seen {
                        if names.len() > 1 {
                            let list = names.join(", ");
                            error(&format!(
                                "duplicate session name '{sanitized}': projects [{list}] collide"
                            ));
                            issues += 1;
                        }
                    }
                }
            },
        }
    }

    // --- Summary ---
    if issues == 0 {
        success("all checks passed");
    } else {
        error(&format!("{issues} issue(s) found"));
        std::process::exit(1);
    }

    Ok(())
}
