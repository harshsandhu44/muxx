use assert_cmd::Command;
use predicates::str::contains;

/// `muxx last` delegates to `attach -`.
/// Outside tmux with no state file it should fail with a clear error.
/// Inside tmux it calls switch-to-last which is hard to test in CI,
/// so we focus on the error path and alias coverage.

fn empty_state_file() -> tempfile::NamedTempFile {
    // Create an empty temp file — state.rs reads from MUXX_STATE_PATH if set,
    // so pointing it here isolates tests from real user state.
    tempfile::NamedTempFile::new().unwrap()
}

#[test]
fn last_errors_outside_tmux_with_no_prior_session() {
    let state = empty_state_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["last"])
        .env_remove("TMUX")
        .env("MUXX_STATE_PATH", state.path())
        .assert()
        .failure()
        .stderr(contains("no last session"));
}

#[test]
fn last_alias_l_errors_outside_tmux_with_no_prior_session() {
    let state = empty_state_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["l"])
        .env_remove("TMUX")
        .env("MUXX_STATE_PATH", state.path())
        .assert()
        .failure()
        .stderr(contains("no last session"));
}
