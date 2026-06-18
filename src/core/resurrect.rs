use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::{
    config::{load_config, MuxxConfig},
    env::expand_home,
    output::warn,
};

/// Best-effort: refresh tmux-resurrect's snapshot from current live tmux state.
///
/// muxx mutates tmux (kill/new/rename/gc) but resurrect's saved snapshot is only
/// rewritten on its own timer (via tmux-continuum) or a manual save. Without this,
/// a killed session lingers in the snapshot and gets restored at the next restore
/// event (see issue #60). Re-running resurrect's `save.sh` reconciles the snapshot
/// immediately.
///
/// No-op (silent) when the integration is disabled in config or resurrect isn't
/// installed. Non-fatal: failures only emit a warning — the caller's operation has
/// already succeeded.
pub fn save_snapshot() {
    let config = load_config();
    if !config.resurrect.enabled {
        return;
    }

    let Some(script) = resolve_save_script_with(&config) else {
        return;
    };

    // `quiet` suppresses resurrect's own "saved" message; capture output so it
    // doesn't leak into muxx's stdout/stderr.
    match Command::new(&script).arg("quiet").output() {
        Ok(out) if out.status.success() => {}
        Ok(out) => warn(&format!(
            "tmux-resurrect save exited with status {} — snapshot may be stale",
            out.status.code().unwrap_or(-1)
        )),
        Err(e) => warn(&format!("could not run tmux-resurrect save script: {e}")),
    }
}

/// Resolve the resurrect `save.sh` path, or `None` if not found.
/// Loads config itself; used by `muxx doctor`.
pub(crate) fn resolve_save_script() -> Option<PathBuf> {
    resolve_save_script_with(&load_config())
}

fn resolve_save_script_with(config: &MuxxConfig) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. Explicit env override (highest priority; also the integration-test seam).
    if let Ok(p) = std::env::var("MUXX_RESURRECT_SAVE_SCRIPT") {
        if !p.is_empty() {
            candidates.push(PathBuf::from(expand_home(&p)));
        }
    }
    // 2. Config override for non-standard installs.
    if let Some(s) = config.resurrect.save_script.as_deref() {
        candidates.push(PathBuf::from(expand_home(s)));
    }
    // 3. Probed default install locations.
    candidates.extend(probe_default_paths());

    first_existing(&candidates, |p| p.is_file())
}

/// Default tmux-resurrect install locations for `save.sh`.
fn probe_default_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let suffix = "tmux-resurrect/scripts/save.sh";

    if let Ok(tpm) = std::env::var("TMUX_PLUGIN_MANAGER_PATH") {
        if !tpm.is_empty() {
            paths.push(PathBuf::from(tpm).join(suffix));
        }
    }
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".tmux/plugins").join(suffix));
    }
    let xdg_config = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")));
    if let Some(cfg) = xdg_config {
        paths.push(cfg.join("tmux/plugins").join(suffix));
    }
    paths
}

/// First candidate that satisfies `exists`, preserving priority order.
fn first_existing(candidates: &[PathBuf], exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    candidates.iter().find(|p| exists(p)).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_existing_returns_highest_priority_present() {
        let env = PathBuf::from("/env/save.sh");
        let cfg = PathBuf::from("/cfg/save.sh");
        let probe = PathBuf::from("/probe/save.sh");
        let candidates = vec![env.clone(), cfg.clone(), probe.clone()];

        // env present -> env wins
        assert_eq!(
            first_existing(&candidates, |p| p == env || p == cfg),
            Some(env.clone())
        );
        // env absent, config present -> config wins
        assert_eq!(
            first_existing(&candidates, |p| p == cfg || p == probe),
            Some(cfg)
        );
        // only probe present -> probe wins
        assert_eq!(first_existing(&candidates, |p| p == probe), Some(probe));
    }

    #[test]
    fn first_existing_returns_none_when_nothing_present() {
        let candidates = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        assert_eq!(first_existing(&candidates, |_| false), None);
    }

    #[test]
    fn first_existing_empty_candidates_is_none() {
        assert_eq!(first_existing(&[], |_| true), None);
    }
}
