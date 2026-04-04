use assert_cmd::Command;
use predicates::str::contains;

/// Connect to the current directory with --no-attach.
/// This creates (or reuses) a session without attaching — safe to run in CI.
#[test]
fn connect_no_attach_current_dir() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "connect --no-attach should succeed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("created") || stdout.contains("reused"),
        "expected 'created' or 'reused' in output, got: {stdout}"
    );
}

#[test]
fn connect_no_attach_with_name_override() {
    let session = "muxx-test-named-session";

    // Connect with a name override and --no-attach
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    // Verify session exists
    let result = std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .status();

    let session_exists = result.map(|s| s.success()).unwrap_or(false);

    // Clean up regardless of assertion
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(
        session_exists,
        "session '{session}' should exist after connect --no-attach"
    );
}

#[test]
fn connect_reuses_existing_session() {
    let session = "muxx-test-reuse-session";

    // Create first
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success()
        .stdout(contains("created"));

    // Connect again — should reuse
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success()
        .stdout(contains("reused"));

    // Clean up
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}

#[test]
fn connect_errors_on_nonexistent_directory() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--cwd", "/tmp/muxx-nonexistent-dir-xyz"])
        .assert()
        .failure()
        .stderr(contains("does not exist"));
}

#[test]
fn connect_alias_c_works() {
    let session = "muxx-test-alias-c";
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["c", "--no-attach", "--name", session])
        .assert()
        .success();
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}
