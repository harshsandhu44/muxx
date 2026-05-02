use assert_cmd::Command;
use predicates::str::contains;
use std::process::Stdio;

fn tmux(args: &[&str]) -> bool {
    std::process::Command::new("tmux")
        .args(args)
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn kill(session: &str) {
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .stderr(Stdio::null())
        .status();
}

fn isolated_tags() -> tempfile::NamedTempFile {
    tempfile::NamedTempFile::new().unwrap()
}

fn isolated_notes() -> tempfile::NamedTempFile {
    tempfile::NamedTempFile::new().unwrap()
}

#[test]
fn rename_nonexistent_session() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["rename", "muxx-no-such-session-xyz", "muxx-new-name-xyz"])
        .assert()
        .failure()
        .stderr(contains("session not found"));
}

#[test]
fn rename_to_existing_name_fails() {
    let a = "muxx-test-rename-conflict-a";
    let b = "muxx-test-rename-conflict-b";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", a])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", b])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["rename", a, b])
        .assert()
        .failure()
        .stderr(contains("session already exists"));

    kill(a);
    kill(b);
}

#[test]
fn rename_success() {
    let old = "muxx-test-rename-old";
    let new = "muxx-test-rename-new";

    kill(old);
    kill(new);

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", old])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["rename", old, new])
        .assert()
        .success()
        .stdout(contains("renamed"));

    let new_exists = tmux(&["has-session", "-t", new]);
    let old_gone = !tmux(&["has-session", "-t", old]);

    kill(new);

    assert!(new_exists, "renamed session '{new}' should exist");
    assert!(old_gone, "old session '{old}' should no longer exist");
}

#[test]
fn rename_alias_rn() {
    let old = "muxx-test-rn-old";
    let new = "muxx-test-rn-new";

    kill(old);
    kill(new);

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", old])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["rn", old, new])
        .assert()
        .success()
        .stdout(contains("renamed"));

    kill(new);
}

#[test]
fn rename_sanitizes_new_name() {
    let old = "muxx-test-sanitize-src";
    let new_raw = "My New Session";
    let new_sanitized = "my-new-session";

    kill(old);
    kill(new_sanitized);

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", old])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["rename", old, new_raw])
        .assert()
        .success()
        .stdout(contains(new_sanitized));

    kill(new_sanitized);
}

#[test]
fn rename_migrates_tags() {
    let old = "muxx-test-rename-tags-old";
    let new = "muxx-test-rename-tags-new";
    let tags_file = isolated_tags();

    kill(old);
    kill(new);

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", old])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", old, "work", "rust"])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["rename", old, new])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls", new])
        .output()
        .unwrap();

    kill(new);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("work"),
        "tag 'work' should migrate to new name; got: {stdout}"
    );
    assert!(
        stdout.contains("rust"),
        "tag 'rust' should migrate to new name; got: {stdout}"
    );
}

#[test]
fn rename_migrates_notes() {
    let old = "muxx-test-rename-notes-old";
    let new = "muxx-test-rename-notes-new";
    let notes_file = isolated_notes();

    kill(old);
    kill(new);

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", old])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_NOTES_PATH", notes_file.path())
        .args(["note", old, "in progress: auth refactor"])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_NOTES_PATH", notes_file.path())
        .args(["rename", old, new])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_NOTES_PATH", notes_file.path())
        .args(["note", new])
        .output()
        .unwrap();

    kill(new);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("in progress: auth refactor"),
        "note should migrate to new session name; got: {stdout}"
    );
}
