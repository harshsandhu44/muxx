use anyhow::Result;

use crate::commands::connect;

/// Shorthand for `connect --cwd <path>`.
pub fn run(path: &str, name: Option<&str>, cmd: Option<&str>, no_attach: bool) -> Result<()> {
    connect::run(None, Some(path), name, no_attach, cmd)
}
