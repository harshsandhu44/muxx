use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Result;

use crate::cli::TagAction;
use crate::core::{
    output::{error, hint, success},
    tags::{load_tags, save_tags},
};

/// Spawns fzf with multi-select and returns the chosen lines (stripped of whitespace).
/// Returns an empty vec if the user cancels (Escape / Ctrl-C).
fn fzf_multi_select(items: &[String], header: &str, prompt: &str) -> Result<Vec<String>> {
    let input = items.join("\n");

    let mut child = match Command::new("fzf")
        .args([
            "--multi",
            "--header",
            header,
            "--prompt",
            prompt,
            "--layout=reverse",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            error("fzf not found in PATH — install it to use interactive tag selection");
            std::process::exit(1);
        }
        Err(e) => return Err(e.into()),
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        // User cancelled
        return Ok(vec![]);
    }

    let selected = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(selected)
}

pub fn run(action: TagAction) -> Result<()> {
    match action {
        TagAction::Add { session, tags } => {
            let mut store = load_tags();

            let to_add = if tags.is_empty() {
                // Interactive: show known tags not already on this session.
                let current = store.get_tags(&session);
                let available: Vec<String> = store
                    .all_known_tags()
                    .into_iter()
                    .filter(|t| !current.contains(t))
                    .collect();

                if available.is_empty() {
                    hint(&format!(
                        "no other known tags — use: muxx tag add {session} <tag>"
                    ));
                    return Ok(());
                }

                let selected = fzf_multi_select(
                    &available,
                    "TAB: select  ·  Enter: add tags  ·  Esc: cancel",
                    "add tags> ",
                )?;

                if selected.is_empty() {
                    return Ok(());
                }
                selected
            } else {
                tags
            };

            store.add_tags(&session, &to_add);
            save_tags(&store)?;
            let current = store.get_tags(&session);
            success(&format!("tagged {}: {}", session, current.join(", ")));
        }

        TagAction::Rm { session, tags } => {
            let mut store = load_tags();

            let to_remove = if tags.is_empty() {
                // Interactive: show current tags to pick from.
                let current = store.get_tags(&session);

                if current.is_empty() {
                    hint(&format!("no tags on {session}"));
                    return Ok(());
                }

                let selected = fzf_multi_select(
                    &current,
                    "TAB: select  ·  Enter: remove tags  ·  Esc: cancel",
                    "remove tags> ",
                )?;

                if selected.is_empty() {
                    return Ok(());
                }
                selected
            } else {
                tags
            };

            store.remove_tags(&session, &to_remove);
            save_tags(&store)?;
            let remaining = store.get_tags(&session);
            if remaining.is_empty() {
                success(&format!("removed tags from {session}"));
            } else {
                success(&format!(
                    "tags remaining on {}: {}",
                    session,
                    remaining.join(", ")
                ));
            }
        }

        TagAction::Edit { session } => {
            let mut store = load_tags();
            let current = store.get_tags(&session);
            let all_known = store.all_known_tags();

            if all_known.is_empty() {
                hint(&format!(
                    "no tags known yet — use: muxx tag add {session} <tag>"
                ));
                return Ok(());
            }

            // Build display list: current tags first (marked with "* "),
            // then remaining known tags (marked with "  ").
            // The user selects the desired final set — whatever is selected = new tags.
            let mut items: Vec<String> = current
                .iter()
                .map(|t| format!("* {t}"))
                .chain(
                    all_known
                        .iter()
                        .filter(|t| !current.contains(t))
                        .map(|t| format!("  {t}")),
                )
                .collect();

            // Deduplicate in case current and all_known overlap unexpectedly.
            items.dedup();

            let selected = fzf_multi_select(
                &items,
                "TAB: toggle  ·  Enter: save  ·  Esc: cancel  ·  (* = currently tagged)",
                "tags> ",
            )?;

            if selected.is_empty() {
                // User cancelled — leave tags unchanged.
                return Ok(());
            }

            // Strip the "* " / "  " visual prefix to get plain tag names.
            let new_tags: Vec<String> = selected
                .iter()
                .map(|line| line.trim_start_matches(['*', ' ']).trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            store.clear_tags(&session);
            store.add_tags(&session, &new_tags);
            save_tags(&store)?;

            let saved = store.get_tags(&session);
            if saved.is_empty() {
                success(&format!("cleared all tags from {session}"));
            } else {
                success(&format!("tags for {}: {}", session, saved.join(", ")));
            }
        }

        TagAction::Delete { tag } => {
            let mut store = load_tags();

            let to_delete = match tag {
                Some(t) => t,
                None => {
                    // Interactive: pick from all known tags.
                    let all = store.all_known_tags();
                    if all.is_empty() {
                        hint("no tags");
                        return Ok(());
                    }
                    let selected = fzf_multi_select(
                        &all,
                        "TAB: select  ·  Enter: delete globally  ·  Esc: cancel",
                        "delete tag> ",
                    )?;
                    match selected.into_iter().next() {
                        Some(t) => t,
                        None => return Ok(()),
                    }
                }
            };

            let affected = store.delete_tag(&to_delete);
            if affected == 0 {
                hint(&format!("tag '{to_delete}' not found"));
            } else {
                save_tags(&store)?;
                success(&format!(
                    "deleted tag '{to_delete}' from {affected} session{}",
                    if affected == 1 { "" } else { "s" }
                ));
            }
        }

        TagAction::Clear { session } => {
            let mut store = load_tags();
            store.clear_tags(&session);
            save_tags(&store)?;
            success(&format!("cleared all tags from {session}"));
        }

        TagAction::Ls { session } => {
            let store = load_tags();
            match session {
                Some(name) => {
                    let tags = store.get_tags(&name);
                    if tags.is_empty() {
                        hint(&format!("no tags for {name}"));
                    } else {
                        println!("{}: {}", name, tags.join(", "));
                    }
                }
                None => {
                    if store.tags.is_empty() {
                        hint("no tags");
                        return Ok(());
                    }
                    let mut entries: Vec<_> = store.tags.iter().collect();
                    entries.sort_by_key(|(k, _)| k.as_str());
                    for (name, tags) in entries {
                        println!("{}: {}", name, tags.join(", "));
                    }
                }
            }
        }
    }
    Ok(())
}
