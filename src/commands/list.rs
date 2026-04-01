use anyhow::Result;

use crate::core::{
    output::{error, hint},
    tmux::{has_tmux, list_sessions},
};

pub fn run(json: bool) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    let sessions = list_sessions();

    if json {
        println!("{}", serde_json::to_string_pretty(&sessions)?);
        return Ok(());
    }

    if sessions.is_empty() {
        hint("no sessions");
        return Ok(());
    }

    let name_width = sessions.iter().map(|s| s.name.len()).max().unwrap_or(0);

    for s in &sessions {
        let name = format!("{:<width$}", s.name, width = name_width);
        let wins = format!("{:>2}", s.windows);
        let state = if s.attached {
            "\x1b[32mattached\x1b[0m"
        } else {
            "\x1b[2mdetached\x1b[0m"
        };
        println!("{name}  {wins}  {state}");
    }

    Ok(())
}
