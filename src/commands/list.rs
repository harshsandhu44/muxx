use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::Serialize;

use crate::core::{
    config::load_config,
    output::{error, hint},
    tags::load_tags,
    tmux::{get_panes_per_session, get_session_paths, has_tmux, list_sessions, TmuxSession},
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

#[derive(Serialize)]
struct SessionWithTags<'a> {
    #[serde(flatten)]
    session: &'a TmuxSession,
    tags: Vec<String>,
}

pub fn run(json: bool, filter_tags: &[String]) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    let mut sessions = list_sessions();
    let tags_store = load_tags();

    // Apply tag filter: keep sessions that have ALL of the requested tags.
    if !filter_tags.is_empty() {
        let normalized: Vec<String> = filter_tags
            .iter()
            .map(|t| t.trim().to_lowercase())
            .collect();
        sessions.retain(|s| {
            let session_tags = tags_store.get_tags(&s.name);
            normalized.iter().all(|ft| session_tags.contains(ft))
        });
    }

    if json {
        let with_tags: Vec<SessionWithTags> = sessions
            .iter()
            .map(|s| SessionWithTags {
                session: s,
                tags: tags_store.get_tags(&s.name),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&with_tags)?);
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
        tags: String,
    }

    let rows: Vec<Row> = sessions
        .iter()
        .map(|s| {
            let proj = config.projects.get(&s.name);
            let tag_list = tags_store.get_tags(&s.name);
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
                tags: if tag_list.is_empty() {
                    "-".to_string()
                } else {
                    tag_list.join(", ")
                },
            }
        })
        .collect();

    // Column widths — at least as wide as the header label.
    let name_w = rows.iter().map(|r| r.name.len()).max().unwrap_or(0).max(4);
    let wins_w = rows.iter().map(|r| r.wins.len()).max().unwrap_or(0).max(4);
    let panes_w = rows.iter().map(|r| r.panes.len()).max().unwrap_or(0).max(5);
    let age_w = rows.iter().map(|r| r.age.len()).max().unwrap_or(0).max(9);
    let cwd_w = rows.iter().map(|r| r.cwd.len()).max().unwrap_or(0).max(3);
    let tags_w = rows.iter().map(|r| r.tags.len()).max().unwrap_or(0).max(4);
    // "attached" / "detached" are always 8 chars — header "STATE" is 5.
    let state_w = 8_usize;
    // "STARTUP" header is 7 chars; symbol value is 1 char — pad explicitly.
    let startup_w = 7_usize;

    // Header
    println!(
        "\x1b[2m{:<name_w$}  {:<wins_w$}  {:<panes_w$}  {:<age_w$}  {:<state_w$}  {:<cwd_w$}  {:<startup_w$}  TAGS\x1b[0m",
        "NAME", "WINS", "PANES", "LAST SEEN", "STATE", "CWD", "STARTUP",
    );
    // Separator
    let total = name_w
        + 2
        + wins_w
        + 2
        + panes_w
        + 2
        + age_w
        + 2
        + state_w
        + 2
        + cwd_w
        + 2
        + startup_w
        + 2
        + tags_w;
    println!("\x1b[2m{}\x1b[0m", "─".repeat(total));

    // Rows
    for r in &rows {
        let state_plain = if r.attached { "attached" } else { "detached" };
        let state_colored = if r.attached {
            format!("\x1b[32m{state_plain}\x1b[0m")
        } else {
            format!("\x1b[2m{state_plain}\x1b[0m")
        };
        let startup_sym = if r.startup {
            "\x1b[32m✓\x1b[0m"
        } else {
            "\x1b[2m-\x1b[0m"
        };
        // state_colored and startup_sym have invisible ANSI bytes; pad manually using plain widths.
        let state_pad = " ".repeat(state_w.saturating_sub(state_plain.len()));
        // startup symbol is always 1 visible char; pad to startup_w.
        let startup_pad = " ".repeat(startup_w.saturating_sub(1));
        println!(
            "{:<name_w$}  {:<wins_w$}  {:<panes_w$}  {:<age_w$}  {state_colored}{state_pad}  {:<cwd_w$}  {startup_sym}{startup_pad}  {tags}",
            r.name, r.wins, r.panes, r.age, r.cwd, tags = r.tags,
        );
    }

    Ok(())
}
