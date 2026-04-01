use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn kill_errors_on_nonexistent_session() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", "muxx-no-such-session-xyz"])
        .assert()
        .failure()
        .stderr(contains("session not found"));
}

#[test]
fn kill_alias_k_errors_on_nonexistent_session() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["k", "muxx-no-such-session-xyz"])
        .assert()
        .failure()
        .stderr(contains("session not found"));
}

#[test]
fn kill_removes_existing_session() {
    let session = "muxx-test-kill-session";

    // Create session first
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    // Kill it
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", session])
        .env_remove("TMUX") // ensure we're not seen as "inside tmux" with this session
        .assert()
        .success()
        .stdout(contains("killed"));

    // Verify it's gone
    let exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    assert!(!exists, "session '{session}' should be gone after kill");
}
