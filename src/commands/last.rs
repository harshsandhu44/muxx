use anyhow::Result;

use crate::commands::attach;

/// Re-attach to the last recorded session.
/// Delegates to `attach -` which handles both inside-tmux (switch) and
/// outside-tmux (attach) cases and falls back to the state file.
pub fn run() -> Result<()> {
    attach::run("-")
}
