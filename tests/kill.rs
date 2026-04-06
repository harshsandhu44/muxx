use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::*;

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

#[test]
fn kill_with_force_flag_kills_session() {
    let session = "muxx-test-kill-force-session";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", "--force", session])
        .env_remove("TMUX")
        .assert()
        .success()
        .stdout(contains("killed"));

    let exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    assert!(!exists, "session '{session}' should be gone after kill --force");
}

#[test]
fn kill_output_contains_session_name() {
    let session = "muxx-test-kill-name-in-output";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", session])
        .env_remove("TMUX")
        .assert()
        .success()
        .stdout(contains(session));
}

#[test]
fn kill_errors_message_contains_session_name() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", "muxx-no-such-session-abc"])
        .assert()
        .failure()
        .stderr(contains("muxx-no-such-session-abc"));
}

#[test]
fn kill_nonexistent_does_not_output_to_stdout() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", "muxx-no-such-session-def"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty());
}
