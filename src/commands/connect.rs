use std::collections::HashMap;
use std::io::IsTerminal;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};

use crate::core::{
    config::{load_config, resolve_project},
    env::{is_inside_tmux, resolve_dir},
    fuzzy,
    output::{hint, info, success, warn},
    resurrect,
    session_name::{resolve_session_name, sanitize_session_name},
    tmux::{
        attach_session, create_session, get_session_paths, has_session, has_tmux, list_sessions,
        send_keys, switch_client, TmuxSession,
    },
};

pub fn run(
    session: Option<&str>,
    cwd: Option<&str>,
    name_override: Option<&str>,
    no_attach: bool,
    cmd_flag: Option<&str>,
    force: bool,
) -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    let config = load_config();

    // --cwd flag: dir-based flow (old positional behavior)
    if cwd.is_some() {
        return run_dir_based(cwd, name_override, no_attach, cmd_flag, force);
    }

    // Session name or config alias provided
    if let Some(target) = session {
        let project = resolve_project(&config, target);

        if let Some(proj) = project {
            // Config alias: resolve the project's directory
            let startup = cmd_flag.or(proj.startup.as_deref());
            let effective_name = name_override.or(Some(target));
            return run_dir_based(
                Some(proj.cwd.as_str()),
                effective_name,
                no_attach,
                startup,
                force,
            );
        }

        // Existing tmux session by name (exact, then fuzzy)
        if let Some(name) = resolve_live_session(target)? {
            info(&format!("reused: {name}"));
            return do_attach(&name, no_attach);
        }

        // Last resort: a directory zoxide knows
        if let Some(dir) = zoxide_query(target) {
            info(&format!("zoxide: {target} → {dir}"));
            return run_dir_based(Some(&dir), name_override, no_attach, cmd_flag, force);
        }

        hint("run 'muxx ls' to see active sessions");
        bail!("session not found: {target}");
    }

    // No args: fall back to current directory
    run_dir_based(None, name_override, no_attach, cmd_flag, force)
}

/// Config aliases with no live session of the same name, as sorted `(alias, cwd)`.
/// Shared by `connect` completion and `pick`.
pub fn idle_aliases(live: &[TmuxSession]) -> Vec<(String, String)> {
    let mut aliases: Vec<(String, String)> = load_config()
        .projects
        .into_iter()
        .filter(|(name, _)| !live.iter().any(|s| &s.name == name))
        .map(|(name, proj)| (name, proj.cwd))
        .collect();
    aliases.sort();
    aliases
}

/// Resolve `query` to a live session: exact name first, then a unique fuzzy match.
/// `None` when nothing matches; ambiguous matches are an error.
fn resolve_live_session(query: &str) -> Result<Option<String>> {
    if has_session(query) {
        return Ok(Some(query.to_string()));
    }

    let sessions = list_sessions();
    let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
    let matches = fuzzy::find_matches(query, &names);

    match matches.as_slice() {
        [] => Ok(None),
        [only] => {
            info(&format!("matched session '{only}'"));
            Ok(Some(only.to_string()))
        }
        _ => {
            for m in &matches {
                hint(&format!("  {m}"));
            }
            bail!("ambiguous session name '{query}', did you mean one of the above?");
        }
    }
}

/// Best-ranked zoxide match for `query`; None if zoxide is missing or knows nothing.
fn zoxide_query(query: &str) -> Option<String> {
    let out = Command::new("zoxide")
        .args(["query", "--", query])
        .output()
        .ok()?;
    let dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !dir.is_empty()).then_some(dir)
}

fn run_dir_based(
    dir_target: Option<&str>,
    name_override: Option<&str>,
    no_attach: bool,
    startup_cmd: Option<&str>,
    force: bool,
) -> Result<()> {
    let dir = resolve_dir(dir_target)?;
    let dir_str = dir.to_string_lossy();
    let paths = get_session_paths();

    let session_name = match name_override.filter(|n| !n.trim().is_empty()) {
        // Explicit name (--name or config alias): use it as-is; a session by that
        // name in another directory is a hard error unless --force.
        Some(name) => {
            let name = resolve_session_name(&dir_str, Some(name));
            if name.is_empty() {
                bail!("cannot derive a valid session name from \"{dir_str}\"");
            }
            if let Some(existing_path) = paths.get(&name) {
                if !same_dir(existing_path, &dir) && !force {
                    bail!(
                        "session name collision: {name}\n\n  \
                         Existing session path:  {existing_path}\n  \
                         Requested path:         {dir_str}\n\n\
                         Rename the new session:\n  \
                         muxx --cwd {dir_str} --name <new-name>\n\n\
                         Or attach to the existing session anyway:\n  \
                         muxx --cwd {dir_str} --force"
                    );
                }
            }
            name
        }
        // Derived name: reuse whatever session already lives in this directory
        // (even if renamed), else the first free name from the directory.
        None => match session_at_dir(&dir, &paths) {
            Some(name) => name,
            None => free_name(&dir, &paths)?,
        },
    };

    let mut created = false;
    if paths.contains_key(&session_name) {
        info(&format!("reused: {session_name}"));
    } else if create_session(&session_name, &dir_str) {
        created = true;
        success(&format!("created: {session_name}"));
        if let Some(cmd) = startup_cmd {
            if !send_keys(&session_name, cmd) {
                warn(&format!("startup command may not have been sent: {cmd}"));
            }
        }
    } else if has_session(&session_name) {
        // Another process created it concurrently — reuse.
        info(&format!("reused: {session_name}"));
    } else {
        bail!("failed to create session: {session_name}");
    }

    // Refresh resurrect's snapshot before attaching (attach hijacks the terminal).
    if created {
        resurrect::save_snapshot();
    }

    do_attach(&session_name, no_attach)
}

fn same_dir(session_path: &str, dir: &Path) -> bool {
    Path::new(session_path)
        .canonicalize()
        .is_ok_and(|p| p == dir)
}

/// The live session rooted at `dir`. If several are, prefer the one named after
/// the directory, then the most recently attached.
fn session_at_dir(dir: &Path, paths: &HashMap<String, String>) -> Option<String> {
    let at_dir: Vec<&String> = paths
        .iter()
        .filter(|(_, p)| same_dir(p, dir))
        .map(|(name, _)| name)
        .collect();

    match at_dir.as_slice() {
        [] => None,
        [only] => Some(only.to_string()),
        _ => {
            let derived = sanitize_session_name(&dir.to_string_lossy());
            if at_dir.iter().any(|n| **n == derived) {
                return Some(derived);
            }
            list_sessions()
                .into_iter()
                .filter(|s| at_dir.contains(&&s.name))
                .max_by_key(|s| s.last_attached)
                .map(|s| s.name)
        }
    }
}

/// Names to try for a new session in `dir`: `api`, then `oss-api` (parent-basename).
fn name_candidates(dir: &Path) -> Vec<String> {
    let base = sanitize_session_name(&dir.to_string_lossy());
    if base.is_empty() {
        return Vec::new();
    }
    let mut names = vec![base.clone()];
    if let Some(parent) = dir.parent().and_then(|p| p.file_name()) {
        let parent = sanitize_session_name(&parent.to_string_lossy());
        if !parent.is_empty() {
            names.push(format!("{parent}-{base}"));
        }
    }
    names
}

fn free_name(dir: &Path, paths: &HashMap<String, String>) -> Result<String> {
    let candidates = name_candidates(dir);
    if candidates.is_empty() {
        bail!(
            "cannot derive a valid session name from \"{}\"",
            dir.display()
        );
    }
    match candidates.iter().find(|n| !paths.contains_key(*n)) {
        Some(name) => Ok(name.clone()),
        None => bail!(
            "session name collision: {} already used by other directories\n\n  \
             Pick a name:\n  \
             muxx --cwd {} --name <new-name>",
            candidates.join(", "),
            dir.display()
        ),
    }
}

/// Hand the terminal to `session_name`: switch-client inside tmux, otherwise
/// exec into `tmux attach`. Without a terminal (launchers, scripts) the session
/// is left detached and a hint is printed instead of failing.
pub fn do_attach(session_name: &str, no_attach: bool) -> Result<()> {
    if no_attach {
        return Ok(());
    }

    if is_inside_tmux() {
        if !switch_client(session_name) {
            bail!("failed to switch to session: {session_name}");
        }
        return Ok(());
    }

    if !std::io::stdin().is_terminal() {
        hint(&format!(
            "not a terminal — attach with 'muxx {session_name}'"
        ));
        return Ok(());
    }

    let err = attach_session(session_name);
    bail!("failed to attach to session {session_name}: {err}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_candidates_basename_then_parent_basename() {
        assert_eq!(
            name_candidates(Path::new("/home/me/oss/api")),
            vec!["api", "oss-api"]
        );
    }

    #[test]
    fn name_candidates_sanitizes_parent() {
        assert_eq!(
            name_candidates(Path::new("/x/My Projects/Web.App")),
            vec!["web-app", "my-projects-web-app"]
        );
    }

    #[test]
    fn name_candidates_top_level_dir_has_no_parent_fallback() {
        assert_eq!(name_candidates(Path::new("/api")), vec!["api"]);
    }
}
