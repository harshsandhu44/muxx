use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::core::{
    config::load_config,
    output::{error, hint},
    tmux::{get_panes_per_session, has_tmux, list_sessions},
};

fn format_age(ts: u64) -> String {
    if ts == 0 {
        return "never".to_string();
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs = now.saturating_sub(ts);
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

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

    let panes_map = get_panes_per_session();
    let config = load_config();

    let name_width = sessions.iter().map(|s| s.name.len()).max().unwrap_or(0);

    for s in &sessions {
        let name = format!("{:<width$}", s.name, width = name_width);
        let wins = format!("{:>2}w", s.windows);
        let panes = format!("{:>2}p", panes_map.get(&s.name).copied().unwrap_or(0));
        let age = format_age(s.last_attached);
        let state = if s.attached {
            "\x1b[32mattached\x1b[0m"
        } else {
            "\x1b[2mdetached\x1b[0m"
        };
        let proj = config.projects.get(&s.name);
        let cwd = proj.map(|p| p.cwd.as_str()).unwrap_or("-");
        let startup = if proj.and_then(|p| p.startup.as_ref()).is_some() {
            "  startup \x1b[32m✓\x1b[0m"
        } else {
            ""
        };
        println!(
            "{name}  {wins}  {panes}  {:<10}  {state}  {cwd}{startup}",
            age
        );
    }

    Ok(())
}
