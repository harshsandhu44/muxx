use clap::Command;
use clap_complete::{generate, Shell};

use anyhow::Result;

pub fn run(shell: Shell, cmd: &mut Command) -> Result<()> {
    generate(shell, cmd, "muxx", &mut std::io::stdout());
    Ok(())
}
