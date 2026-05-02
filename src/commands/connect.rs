use anyhow::{bail, Result};

use crate::core::{
    config::{load_config, resolve_project},
    env::{is_inside_tmux, resolve_dir},
    output::{info, success},
    session_name::resolve_session_name,
    state,
    tmux::{attach_session, create_session, has_session, has_tmux, send_keys, switch_client},
};

pub fn run(
    session: Option<&str>,
    cwd: Option<&str>,
    name_override: Option<&str>,
    no_attach: bool,
    cmd_flag: Option<&str>,
) -> Result<()> {
    if !has_tmux() {
        bail!("tmux not found in PATH");
    }

    let config = load_config();

    // --cwd flag: dir-based flow (old positional behavior)
    if cwd.is_some() {
        return run_dir_based(cwd, name_override, no_attach, cmd_flag);
    }

    // Session name or config alias provided
    if let Some(target) = session {
        let project = resolve_project(&config, target);

        if let Some(proj) = project {
            // Config alias: resolve the project's directory
            let startup = cmd_flag.or(proj.startup.as_deref());
            let effective_name = name_override.or(Some(target));
            return run_dir_based(Some(proj.cwd.as_str()), effective_name, no_attach, startup);
        }

        // Existing tmux session by name
        if has_session(target) {
            info(&format!("reused: {target}"));
            return do_attach(target, no_attach);
        }

        bail!("session not found: {target}");
    }

    // No args: fall back to current directory
    run_dir_based(None, name_override, no_attach, cmd_flag)
}

fn run_dir_based(
    dir_target: Option<&str>,
    name_override: Option<&str>,
    no_attach: bool,
    startup_cmd: Option<&str>,
) -> Result<()> {
    let dir = resolve_dir(dir_target)?;
    let dir_str = dir.to_string_lossy();
    let session_name = resolve_session_name(&dir_str, name_override);

    let existed = has_session(&session_name);

    if !existed {
        if !create_session(&session_name, &dir_str) {
            bail!("failed to create session: {session_name}");
        }
        success(&format!("created: {session_name}"));
        if let Some(cmd) = startup_cmd {
            send_keys(&session_name, cmd);
        }
    } else {
        info(&format!("reused: {session_name}"));
    }

    do_attach(&session_name, no_attach)
}

fn do_attach(session_name: &str, no_attach: bool) -> Result<()> {
    if no_attach {
        return Ok(());
    }

    state::save_last_session(session_name);

    if is_inside_tmux() {
        if !switch_client(session_name) {
            bail!("failed to switch to session: {session_name}");
        }
    } else if !attach_session(session_name) {
        bail!("failed to attach to session: {session_name}");
    }

    Ok(())
}
