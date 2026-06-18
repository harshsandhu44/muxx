use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// Build a temp dir containing an executable stub `save.sh` that records its
/// invocation by creating an `invoked` marker file next to itself.
///
/// Returns the temp dir (kept alive by the caller), the script path to pass via
/// `MUXX_RESURRECT_SAVE_SCRIPT`, and the marker path to assert on.
fn stub_save_script() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    let script = dir.path().join("save.sh");
    let marker = dir.path().join("invoked");

    let mut f = std::fs::File::create(&script).unwrap();
    // Record the invocation (and its args) so tests can assert it ran.
    writeln!(f, "#!/bin/sh\necho \"$@\" > \"$(dirname \"$0\")/invoked\"").unwrap();
    drop(f);

    let mut perms = std::fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).unwrap();

    (dir, script, marker)
}

fn kill_tmux(session: &str) {
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .stderr(std::process::Stdio::null())
        .status();
}

/// Regression test for issue #60: after `muxx kill`, muxx must refresh
/// tmux-resurrect's snapshot so the killed session does not come back.
#[test]
fn kill_triggers_resurrect_save() {
    let session = "muxx-test-resurrect-kill";
    let (_dir, script, marker) = stub_save_script();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", session])
        .env_remove("TMUX")
        .env("MUXX_RESURRECT_SAVE_SCRIPT", &script)
        .assert()
        .success();

    assert!(
        marker.exists(),
        "resurrect save script should have been invoked after kill"
    );
}

#[test]
fn new_triggers_resurrect_save() {
    let session = "muxx-test-resurrect-new";
    let dir = tempfile::TempDir::new().unwrap();
    let (_stub, script, marker) = stub_save_script();

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--no-attach",
            "--name",
            session,
        ])
        .env("MUXX_RESURRECT_SAVE_SCRIPT", &script)
        .assert()
        .success();

    kill_tmux(session);

    assert!(
        marker.exists(),
        "resurrect save script should have been invoked after creating a session"
    );
}

#[test]
fn rename_triggers_resurrect_save() {
    let from = "muxx-test-resurrect-rename-from";
    let to = "muxx-test-resurrect-rename-to";
    let (_stub, script, marker) = stub_save_script();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", from])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["session", "rename", from, to])
        .env("MUXX_RESURRECT_SAVE_SCRIPT", &script)
        .assert()
        .success();

    kill_tmux(from);
    kill_tmux(to);

    assert!(
        marker.exists(),
        "resurrect save script should have been invoked after rename"
    );
}

#[test]
fn gc_triggers_resurrect_save() {
    let (_stub, script, marker) = stub_save_script();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["gc"])
        .env("MUXX_RESURRECT_SAVE_SCRIPT", &script)
        .assert()
        .success();

    assert!(
        marker.exists(),
        "resurrect save script should have been invoked after gc"
    );
}

#[test]
fn disabled_config_skips_resurrect_save() {
    let session = "muxx-test-resurrect-disabled";
    let (_stub, script, marker) = stub_save_script();

    // Config that disables the integration.
    let cfg = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(cfg.path(), "[resurrect]\nenabled = false\n").unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", session])
        .env_remove("TMUX")
        .env("MUXX_RESURRECT_SAVE_SCRIPT", &script)
        .env("MUXX_CONFIG_PATH", cfg.path())
        .assert()
        .success();

    assert!(
        !marker.exists(),
        "resurrect save must be skipped when disabled in config"
    );
}

#[test]
fn kill_succeeds_when_resurrect_not_installed() {
    let session = "muxx-test-resurrect-absent";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    // Point at a path that does not exist, and an empty config dir so no real
    // plugin is probed into. The kill must still succeed with no error output.
    let empty_cfg = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(empty_cfg.path(), "").unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["kill", session])
        .env_remove("TMUX")
        .env("MUXX_RESURRECT_SAVE_SCRIPT", "/nonexistent/muxx/save.sh")
        .env("MUXX_CONFIG_PATH", empty_cfg.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("killed"));
}

#[test]
fn reused_session_does_not_trigger_resurrect_save() {
    let dir = tempfile::TempDir::new().unwrap();
    let session = "muxx-test-resurrect-reused";
    let (_stub, script, marker) = stub_save_script();

    // First create (this would trigger a save, but we use a separate stub below).
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
        .success();

    // Reuse the existing session — must NOT trigger a save.
    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "new",
            dir.path().to_str().unwrap(),
            "--no-attach",
            "--name",
            session,
        ])
        .env("MUXX_RESURRECT_SAVE_SCRIPT", &script)
        .assert()
        .success()
        .stdout(predicate::str::contains("reused"));

    kill_tmux(session);

    assert!(
        !marker.exists(),
        "reusing an existing session must not trigger a resurrect save"
    );
}
