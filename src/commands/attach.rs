use anyhow::Result;

use crate::commands::{connect, last};

/// Hidden back-compat alias: `attach <name>` is `connect <name>`, `attach -` is `last`.
pub fn run(session: &str) -> Result<()> {
    if session == "-" {
        return last::run();
    }
    connect::run(Some(session), None, None, false, None, false)
}
