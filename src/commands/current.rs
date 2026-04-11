use anyhow::{bail, Result};

use crate::core::{env::is_inside_tmux, tmux::current_session};

pub fn run() -> Result<()> {
    if !is_inside_tmux() {
        bail!("not inside a tmux session");
    }

    match current_session() {
        Some(name) => {
            println!("{name}");
            Ok(())
        }
        None => bail!("could not determine current session"),
    }
}
