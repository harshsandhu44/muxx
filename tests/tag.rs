use assert_cmd::Command;
use predicates::str::contains;
use std::process::Stdio;

fn kill(session: &str) {
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .stderr(Stdio::null())
        .status();
}

fn with_tags_file() -> tempfile::NamedTempFile {
    tempfile::NamedTempFile::new().unwrap()
}

#[test]
fn tag_add_and_ls_shows_tag() {
    let session = "muxx-test-tag-add-ls";
    let tags_file = with_tags_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", session, "work"])
        .assert()
        .success()
        .stdout(contains("work"));

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls", session])
        .assert()
        .success()
        .stdout(contains("work"));

    kill(session);
}

#[test]
fn tag_add_multiple_tags() {
    let session = "muxx-test-tag-multi";
    let tags_file = with_tags_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", session, "work", "python"])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls", session])
        .output()
        .unwrap();

    kill(session);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(stdout.contains("work"), "expected 'work' in: {stdout}");
    assert!(stdout.contains("python"), "expected 'python' in: {stdout}");
}

#[test]
fn tag_rm_removes_specific_tag() {
    let session = "muxx-test-tag-rm";
    let tags_file = with_tags_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", session, "work", "python"])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "rm", session, "python"])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls", session])
        .output()
        .unwrap();

    kill(session);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("work"),
        "expected 'work' to remain: {stdout}"
    );
    assert!(
        !stdout.contains("python"),
        "expected 'python' removed: {stdout}"
    );
}

#[test]
fn tag_clear_removes_all() {
    let session = "muxx-test-tag-clear";
    let tags_file = with_tags_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", session, "work", "python"])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "clear", session])
        .assert()
        .success()
        .stdout(contains("cleared"));

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls", session])
        .assert()
        .success()
        .stdout(contains("no tags"));

    kill(session);
}

#[test]
fn tag_ls_all_shows_multiple_sessions() {
    let a = "muxx-test-tag-ls-all-a";
    let b = "muxx-test-tag-ls-all-b";
    let tags_file = with_tags_file();

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
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", a, "work"])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", b, "personal"])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls"])
        .output()
        .unwrap();

    kill(a);
    kill(b);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(stdout.contains(a), "expected session a in output: {stdout}");
    assert!(stdout.contains(b), "expected session b in output: {stdout}");
    assert!(stdout.contains("work"), "expected 'work' tag: {stdout}");
    assert!(
        stdout.contains("personal"),
        "expected 'personal' tag: {stdout}"
    );
}

#[test]
fn tag_ls_unknown_session_shows_hint() {
    let tags_file = with_tags_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls", "muxx-no-such-session-xyz"])
        .assert()
        .success()
        .stdout(contains("no tags"));
}

#[test]
fn tag_alias_t_works() {
    let session = "muxx-test-tag-alias-t";
    let tags_file = with_tags_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["t", "add", session, "alias-test"])
        .assert()
        .success();

    kill(session);
}

#[test]
fn tag_add_normalises_case() {
    let session = "muxx-test-tag-normalise";
    let tags_file = with_tags_file();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "add", session, "WORK", "Python"])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_TAGS_PATH", tags_file.path())
        .args(["tag", "ls", session])
        .output()
        .unwrap();

    kill(session);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("work"),
        "tag should be lowercased: {stdout}"
    );
    assert!(
        stdout.contains("python"),
        "tag should be lowercased: {stdout}"
    );
    assert!(
        !stdout.contains("WORK"),
        "should not contain uppercase: {stdout}"
    );
}
