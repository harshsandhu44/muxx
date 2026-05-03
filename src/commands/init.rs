use std::io::{self, Write};

use anyhow::{bail, Result};

use crate::core::{
    config::{config_path, load_config, save_config, ProjectConfig},
    output::{hint, info, success, warn},
    session_name::sanitize_session_name,
    tags::{load_tags, save_tags},
};

pub fn run(
    name_flag: Option<&str>,
    startup_flag: Option<&str>,
    tags_flag: &[String],
    no_create: bool,
    no_attach: bool,
    force: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cwd_str = cwd.to_string_lossy().into_owned();
    let default_name = sanitize_session_name(&cwd_str);

    if default_name.is_empty() {
        bail!("cannot derive a project name from the current directory");
    }

    let name = if let Some(n) = name_flag {
        sanitize_session_name(n)
    } else {
        let name_input = prompt("Project name", Some(&default_name))?;
        sanitize_session_name(if name_input.is_empty() {
            &default_name
        } else {
            &name_input
        })
    };
    if name.is_empty() {
        bail!("project name sanitizes to an empty string");
    }

    let startup = if let Some(s) = startup_flag {
        Some(s.to_string())
    } else {
        let startup_raw = prompt("Startup command (optional)", None)?;
        if startup_raw.is_empty() {
            None
        } else {
            Some(startup_raw)
        }
    };

    let tags: Vec<String> = if !tags_flag.is_empty() {
        tags_flag
            .iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect()
    } else {
        let tags_raw = prompt("Tags (comma-separated, optional)", None)?;
        tags_raw
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect()
    };

    let create_now = if no_create {
        false
    } else {
        prompt_bool("Create session now?", true)?
    };

    let mut config = load_config();
    if config.projects.contains_key(&name) && !force {
        warn(&format!(
            "project \"{name}\" already exists in config — overwriting"
        ));
    }

    config.projects.insert(
        name.clone(),
        ProjectConfig {
            cwd: cwd_str,
            startup,
        },
    );
    save_config(&config)?;
    success(&format!("config written: {}", config_path().display()));

    if !tags.is_empty() {
        let mut store = load_tags();
        store.add_tags(&name, &tags);
        save_tags(&store)?;
        info(&format!("tags: {}", tags.join(", ")));
    }

    if create_now {
        crate::commands::connect::run(Some(&name), None, None, no_attach, None, false)?;
    } else {
        hint(&format!("run `muxx {name}` to start your session"));
    }

    Ok(())
}

fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(d) => print!("{label} [{d}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn prompt_bool(label: &str, default: bool) -> Result<bool> {
    let indicator = if default { "[Y/n]" } else { "[y/N]" };
    print!("{label} {indicator}: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let t = buf.trim().to_lowercase();
    if t.is_empty() {
        return Ok(default);
    }
    Ok(t == "y" || t == "yes")
}
