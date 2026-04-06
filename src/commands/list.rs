use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::core::{
    config::load_config,
    output::{error, hint},
    tmux::{get_panes_per_session, get_session_paths, has_tmux, list_sessions},
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
    let paths_map = get_session_paths();
    let config = load_config();

    // Pre-compute display values so we can measure widths before printing.
    struct Row {
        name: String,
        wins: String,
        panes: String,
        age: String,
        attached: bool,
        cwd: String,
        startup: bool,
    }

    let rows: Vec<Row> = sessions
        .iter()
        .map(|s| {
            let proj = config.projects.get(&s.name);
            Row {
                name: s.name.clone(),
                wins: format!("{}w", s.windows),
                panes: format!("{}p", panes_map.get(&s.name).copied().unwrap_or(0)),
                age: format_age(s.last_attached),
                attached: s.attached,
                cwd: proj
                    .map(|p| p.cwd.clone())
                    .or_else(|| paths_map.get(&s.name).cloned())
                    .unwrap_or_else(|| "-".to_string()),
                startup: proj.and_then(|p| p.startup.as_ref()).is_some(),
            }
        })
        .collect();

    // Column widths — at least as wide as the header label.
    let name_w = rows.iter().map(|r| r.name.len()).max().unwrap_or(0).max(4);
    let wins_w = rows.iter().map(|r| r.wins.len()).max().unwrap_or(0).max(4);
    let panes_w = rows.iter().map(|r| r.panes.len()).max().unwrap_or(0).max(5);
    let age_w = rows.iter().map(|r| r.age.len()).max().unwrap_or(0).max(9);
    let cwd_w = rows.iter().map(|r| r.cwd.len()).max().unwrap_or(0).max(3);
    // "attached" / "detached" are always 8 chars — header "STATE" is 5.
    let state_w = 8_usize;

    // Header
    println!(
        "\x1b[2m{:<name_w$}  {:<wins_w$}  {:<panes_w$}  {:<age_w$}  {:<state_w$}  {:<cwd_w$}  STARTUP\x1b[0m",
        "NAME", "WINS", "PANES", "LAST SEEN", "STATE", "CWD",
    );
    // Separator
    let total = name_w + 2 + wins_w + 2 + panes_w + 2 + age_w + 2 + state_w + 2 + cwd_w + 2 + 7;
    println!("\x1b[2m{}\x1b[0m", "─".repeat(total));

    // Rows
    for r in &rows {
        let state_plain = if r.attached { "attached" } else { "detached" };
        let state_colored = if r.attached {
            format!("\x1b[32m{state_plain}\x1b[0m")
        } else {
            format!("\x1b[2m{state_plain}\x1b[0m")
        };
        let startup = if r.startup {
            "\x1b[32m✓\x1b[0m"
        } else {
            "\x1b[2m-\x1b[0m"
        };
        // state_colored has invisible ANSI bytes; pad manually using the plain width.
        let state_pad = " ".repeat(state_w.saturating_sub(state_plain.len()));
        println!(
            "{:<name_w$}  {:<wins_w$}  {:<panes_w$}  {:<age_w$}  {state_colored}{state_pad}  {:<cwd_w$}  {startup}",
            r.name, r.wins, r.panes, r.age, r.cwd,
        );
    }

    Ok(())
}
