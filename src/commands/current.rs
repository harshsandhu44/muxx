use anyhow::Result;

use crate::core::{env::is_inside_tmux, output::error, tmux::current_session};

pub fn run() -> Result<()> {
    if !is_inside_tmux() {
        error("not inside a tmux session");
        std::process::exit(1);
    }

    match current_session() {
        Some(name) => {
            println!("{name}");
            Ok(())
        }
        None => {
            error("could not determine current session");
            std::process::exit(1);
        }
    }
}
