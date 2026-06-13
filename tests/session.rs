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

#[test]
fn session_ls_smoke() {
    // `session ls` is the canonical form of the top-level `ls` shortcut.
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["session", "ls"])
        .assert()
        .success();
}

#[test]
fn session_new_and_kill() {
    let name = "muxx-test-session-new-kill";
    let dir = tempfile::tempdir().unwrap();

    kill(name);

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "session",
            "new",
            dir.path().to_str().unwrap(),
            "--name",
            name,
            "--no-attach",
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    assert!(
        tmux(&["has-session", "-t", name]),
        "`session new` should create the session"
    );

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["session", "kill", name])
        .assert()
        .success()
        .stdout(contains("killed"));

    assert!(
        !tmux(&["has-session", "-t", name]),
        "`session kill` should remove the session"
    );
}

#[test]
fn session_rename_via_group() {
    let old = "muxx-test-session-rename-old";
    let new = "muxx-test-session-rename-new";

    kill(old);
    kill(new);

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", old])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["session", "rename", old, new])
        .assert()
        .success()
        .stdout(contains("renamed"));

    let renamed = tmux(&["has-session", "-t", new]);
    kill(new);
    assert!(renamed, "`session rename` should rename the session");
}

#[test]
fn hidden_alias_new_still_works() {
    // The old flat `new` spelling is hidden from help but must keep working.
    let name = "muxx-test-hidden-new-alias";
    let dir = tempfile::tempdir().unwrap();

    kill(name);

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--name",
            name,
            "--no-attach",
        ])
        .assert()
        .success();

    let created = tmux(&["has-session", "-t", name]);
    kill(name);
    assert!(created, "hidden `new` alias should still create a session");
}

#[test]
fn session_note_roundtrip() {
    let name = "muxx-test-session-note";
    let notes_file = tempfile::NamedTempFile::new().unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_NOTES_PATH", notes_file.path())
        .args(["session", "note", name, "wip: refactor"])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_NOTES_PATH", notes_file.path())
        .args(["session", "note", name])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("wip: refactor"),
        "`session note` should round-trip; got: {stdout}"
    );
}

#[test]
fn no_color_flag_suppresses_ansi() {
    // With --no-color, success output must contain no ANSI escape bytes.
    let name = "muxx-test-no-color";
    let dir = tempfile::tempdir().unwrap();

    kill(name);

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "--no-color",
            "session",
            "new",
            dir.path().to_str().unwrap(),
            "--name",
            name,
            "--no-attach",
        ])
        .output()
        .unwrap();

    kill(name);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('\u{1b}'),
        "--no-color output should have no ANSI escapes; got: {stdout:?}"
    );
}
