use anyhow::Result;

use crate::core::{
    config::{load_config, resolve_project},
    env::{is_inside_tmux, resolve_dir},
    output::{error, info, success},
    session_name::resolve_session_name,
    state,
    tmux::{attach_session, create_session, has_session, has_tmux, send_keys, switch_client},
};

pub fn run(
    target: Option<&str>,
    name_override: Option<&str>,
    no_attach: bool,
    cmd_flag: Option<&str>,
) -> Result<()> {
    if !has_tmux() {
        error("tmux not found in PATH");
        std::process::exit(1);
    }

    let config = load_config();
    let project = target.and_then(|t| resolve_project(&config, t));

    let cwd_target = project.map(|p| p.cwd.as_str()).or(target);
    let dir = match resolve_dir(cwd_target) {
        Ok(d) => d,
        Err(e) => {
            error(&e.to_string());
            std::process::exit(1);
        }
    };

    let dir_str = dir.to_string_lossy();
    let session_name = resolve_session_name(&dir_str, name_override);

    // --cmd takes precedence over config startup
    let startup_cmd = cmd_flag.or_else(|| project.and_then(|p| p.startup.as_deref()));

    let existed = has_session(&session_name);

    if !existed {
        if !create_session(&session_name, &dir_str) {
            error(&format!("failed to create session: {session_name}"));
            std::process::exit(1);
        }
        success(&format!("created: {session_name}"));
        if let Some(cmd) = startup_cmd {
            send_keys(&session_name, cmd);
        }
    } else {
        info(&format!("reused: {session_name}"));
    }

    if no_attach {
        return Ok(());
    }

    state::save_last_session(&session_name);

    if is_inside_tmux() {
        if !switch_client(&session_name) {
            error(&format!("failed to switch to session: {session_name}"));
            std::process::exit(1);
        }
    } else if !attach_session(&session_name) {
        error(&format!("failed to attach to session: {session_name}"));
        std::process::exit(1);
    }

    Ok(())
}
