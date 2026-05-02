use assert_cmd::Command;
use predicates::str::contains;
use std::process::Stdio;

fn kill(session: &str) {
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .stderr(Stdio::null())
        .status();
}

fn isolated(tags: &std::path::Path, notes: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("muxx").unwrap();
    cmd.env("MUXX_TAGS_PATH", tags)
        .env("MUXX_NOTES_PATH", notes);
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
fn gc_noop_when_nothing_to_clean() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    isolated(tags.path(), notes.path())
        .args(["gc"])
        .assert()
        .success()
        .stdout(contains("nothing to clean up"));
}

#[test]
fn gc_removes_tags_for_dead_sessions() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();
    let session = "muxx-test-gc-dead-tags";

    create_session(session);

    isolated(tags.path(), notes.path())
        .args(["tag", "add", session, "work"])
        .assert()
        .success();

    kill(session);

    isolated(tags.path(), notes.path())
        .args(["gc"])
        .assert()
        .success()
        .stdout(contains(session));

    // Confirm tag is now gone.
    isolated(tags.path(), notes.path())
        .args(["tag", "ls", session])
        .assert()
        .success()
        .stdout(contains("no tags"));
}

#[test]
fn gc_removes_notes_for_dead_sessions() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();
    let session = "muxx-test-gc-dead-notes";

    create_session(session);

    isolated(tags.path(), notes.path())
        .args(["note", session, "some work in progress"])
        .assert()
        .success();

    kill(session);

    isolated(tags.path(), notes.path())
        .args(["gc"])
        .assert()
        .success()
        .stdout(contains(session));

    // Confirm note is now gone.
    isolated(tags.path(), notes.path())
        .args(["note", session])
        .assert()
        .success()
        .stdout(contains("no note"));
}

#[test]
fn gc_keeps_live_sessions_intact() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();
    let live = "muxx-test-gc-live";
    let dead = "muxx-test-gc-dead-keep";

    create_session(live);
    create_session(dead);

    isolated(tags.path(), notes.path())
        .args(["tag", "add", live, "keep"])
        .assert()
        .success();

    isolated(tags.path(), notes.path())
        .args(["tag", "add", dead, "remove"])
        .assert()
        .success();

    kill(dead);

    isolated(tags.path(), notes.path())
        .args(["gc"])
        .assert()
        .success();

    // Live session's tags should be untouched.
    isolated(tags.path(), notes.path())
        .args(["tag", "ls", live])
        .assert()
        .success()
        .stdout(contains("keep"));

    kill(live);
}

#[test]
fn gc_removes_both_tags_and_notes_for_dead_session() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();
    let session = "muxx-test-gc-both";

    create_session(session);

    isolated(tags.path(), notes.path())
        .args(["tag", "add", session, "work"])
        .assert()
        .success();

    isolated(tags.path(), notes.path())
        .args(["note", session, "in progress"])
        .assert()
        .success();

    kill(session);

    let output = isolated(tags.path(), notes.path())
        .args(["gc"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(output.status.success());
    // gc should report the session twice — once for tags, once for note.
    let count = stdout.matches(session).count();
    assert!(
        count >= 2,
        "expected session mentioned at least twice (tags + note); got: {stdout}"
    );
}
