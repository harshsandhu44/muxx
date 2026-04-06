use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;

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
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            "/tmp/muxx-nonexistent-dir-xyz",
        ])
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

#[test]
fn connect_cwd_flag_creates_session_in_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-cwd-flag-session";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            dir.path().to_str().unwrap(),
            "--name",
            session,
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    let exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(exists, "session '{session}' should exist after connect --cwd");
}

#[test]
fn connect_derives_session_name_from_basename() {
    // Create a dir with a known name so the basename-derived session name is predictable.
    let base = std::env::temp_dir().join("muxx-basename-derive-test");
    std::fs::create_dir_all(&base).unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            base.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("muxx-basename-derive-test"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", "muxx-basename-derive-test"])
        .status();

    let _ = std::fs::remove_dir(&base);
}

#[test]
fn connect_with_config_alias() {
    let project_dir = tempfile::TempDir::new().unwrap();
    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    write!(
        config_file,
        r#"{{"projects":{{"my-proj":{{"cwd":"{}"}}}}}}"#,
        project_dir.path().to_str().unwrap()
    )
    .unwrap();

    let session = "muxx-test-config-alias-session";

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_CONFIG_PATH", config_file.path())
        .args(["connect", "--no-attach", "--name", session, "my-proj"])
        .assert()
        .success()
        .stdout(contains("created"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}

#[test]
fn connect_with_config_alias_startup_cmd() {
    let project_dir = tempfile::TempDir::new().unwrap();
    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    write!(
        config_file,
        r#"{{"projects":{{"startupproj":{{"cwd":"{}","startup":"echo hello"}}}}}}"#,
        project_dir.path().to_str().unwrap()
    )
    .unwrap();

    let session = "muxx-test-startup-cmd-session";

    // The --cmd flag should override the config's startup command; either way,
    // the session must be created successfully.
    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_CONFIG_PATH", config_file.path())
        .args([
            "connect",
            "--no-attach",
            "--name",
            session,
            "startupproj",
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}

#[test]
fn connect_unknown_session_name_fails() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "muxx-this-session-does-not-exist-xyz"])
        .assert()
        .failure()
        .stderr(contains("session not found"));
}

#[test]
fn connect_cmd_flag_does_not_prevent_session_creation() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-cmd-flag-session";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            dir.path().to_str().unwrap(),
            "--name",
            session,
            "--cmd",
            "echo from-cmd-flag",
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}
