use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn new_creates_session_from_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-new-basic";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--no-attach",
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

    assert!(exists, "session '{session}' should exist after muxx new");
}

#[test]
fn new_alias_n_works() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-new-alias-n";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "n",
            dir.path().to_str().unwrap(),
            "--no-attach",
            "--name",
            session,
        ])
        .assert()
        .success();

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}

#[test]
fn new_derives_session_name_from_directory_basename() {
    let base = std::env::temp_dir().join("muxx-new-basename-test");
    std::fs::create_dir_all(&base).unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["new", base.to_str().unwrap(), "--no-attach"])
        .assert()
        .success()
        .stdout(contains("muxx-new-basename-test"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", "muxx-new-basename-test"])
        .status();

    let _ = std::fs::remove_dir(&base);
}

#[test]
fn new_with_name_override() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-new-name-override";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--no-attach",
            "--name",
            session,
        ])
        .assert()
        .success()
        .stdout(contains(session));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}

#[test]
fn new_with_cmd_flag_creates_session() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-new-cmd-flag";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--no-attach",
            "--name",
            session,
            "--cmd",
            "echo hello",
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}

#[test]
fn new_errors_on_nonexistent_directory() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["new", "/tmp/muxx-nonexistent-xyz", "--no-attach"])
        .assert()
        .failure()
        .stderr(contains("does not exist"));
}

#[test]
fn new_reuses_existing_session() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-new-reuse";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--no-attach",
            "--name",
            session,
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    // Running again should reuse the session
    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--no-attach",
            "--name",
            session,
        ])
        .assert()
        .success()
        .stdout(contains("reused"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}
