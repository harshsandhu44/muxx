use assert_cmd::Command;
use predicates::str::contains;
use std::process::Stdio;

fn kill(session: &str) {
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .stderr(Stdio::null())
        .status();
}

fn isolated(notes_file: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("muxx").unwrap();
    cmd.env("MUXX_NOTES_PATH", notes_file);
    cmd
}

fn create_session(name: &str) {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", name])
        .assert()
        .success();
}

#[test]
fn note_set_and_get() {
    let session = "muxx-test-note-set-get";
    let notes = tempfile::NamedTempFile::new().unwrap();

    create_session(session);

    isolated(notes.path())
        .args(["note", session, "fixing the auth bug"])
        .assert()
        .success()
        .stdout(contains("note set"));

    isolated(notes.path())
        .args(["note", session])
        .assert()
        .success()
        .stdout(contains("fixing the auth bug"));

    kill(session);
}

#[test]
fn note_get_when_no_note_shows_hint() {
    let session = "muxx-test-note-no-note";
    let notes = tempfile::NamedTempFile::new().unwrap();

    create_session(session);

    isolated(notes.path())
        .args(["note", session])
        .assert()
        .success()
        .stdout(contains("no note"));

    kill(session);
}

#[test]
fn note_clear() {
    let session = "muxx-test-note-clear";
    let notes = tempfile::NamedTempFile::new().unwrap();

    create_session(session);

    isolated(notes.path())
        .args(["note", session, "some note"])
        .assert()
        .success();

    isolated(notes.path())
        .args(["note", session, "--clear"])
        .assert()
        .success()
        .stdout(contains("cleared"));

    isolated(notes.path())
        .args(["note", session])
        .assert()
        .success()
        .stdout(contains("no note"));

    kill(session);
}

#[test]
fn note_overwrites_existing() {
    let session = "muxx-test-note-overwrite";
    let notes = tempfile::NamedTempFile::new().unwrap();

    create_session(session);

    isolated(notes.path())
        .args(["note", session, "first note"])
        .assert()
        .success();

    isolated(notes.path())
        .args(["note", session, "second note"])
        .assert()
        .success();

    let output = isolated(notes.path())
        .args(["note", session])
        .output()
        .unwrap();

    kill(session);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("second note"),
        "expected overwritten note; got: {stdout}"
    );
    assert!(
        !stdout.contains("first note"),
        "old note should be gone; got: {stdout}"
    );
}

#[test]
fn note_clear_noop_when_no_note() {
    let session = "muxx-test-note-clear-noop";
    let notes = tempfile::NamedTempFile::new().unwrap();

    create_session(session);

    isolated(notes.path())
        .args(["note", session, "--clear"])
        .assert()
        .success();

    kill(session);
}
