use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

/// `muxx last` reads tmux's own `session_last_attached`, so tests point
/// TMUX_TMPDIR at an empty dir: a fresh tmux server with no sessions.
/// Inside tmux it calls switch-client -l which is hard to test in CI,
/// so we focus on the error path and alias coverage.

#[test]
fn last_errors_outside_tmux_with_no_prior_session() {
    let tmux_dir = TempDir::new().unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["last"])
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .assert()
        .failure()
        .stderr(contains("no last session"));
}

#[test]
fn last_alias_l_errors_outside_tmux_with_no_prior_session() {
    let tmux_dir = TempDir::new().unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["l"])
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .assert()
        .failure()
        .stderr(contains("no last session"));
}

#[test]
fn last_ignores_sessions_never_attached() {
    let tmux_dir = TempDir::new().unwrap();
    let tmux = |args: &[&str]| {
        std::process::Command::new("tmux")
            .args(args)
            .env_remove("TMUX")
            .env("TMUX_TMPDIR", tmux_dir.path())
            .status()
            .unwrap()
    };
    tmux(&["new-session", "-d", "-s", "never-attached"]);

    let assert = Command::cargo_bin("muxx")
        .unwrap()
        .args(["last"])
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .assert();

    tmux(&["kill-server"]);
    assert.failure().stderr(contains("no last session"));
}
