use anyhow::{bail, Result};

use crate::cli::ConfigAction;
use crate::core::{config::config_path, output::hint};

pub fn run(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => show(),
        ConfigAction::Edit => edit(),
        ConfigAction::Path => path(),
    }
}

fn show() -> Result<()> {
    let path = config_path();
    hint(&format!("config: {}", path.display()));
    match std::fs::read_to_string(&path) {
        Ok(raw) => print!("{raw}"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            hint("config file does not exist yet");
        }
        Err(e) => bail!("failed to read config: {e}"),
    }
    Ok(())
}

fn edit() -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Create an empty file so editors don't complain about a missing path.
    if !path.exists() {
        std::fs::write(&path, "")?;
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new(&editor).arg(&path).status()?;
    if !status.success() {
        bail!("editor exited with non-zero status");
    }
    Ok(())
}

fn path() -> Result<()> {
    println!("{}", config_path().display());
    Ok(())
}
