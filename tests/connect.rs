use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;
use tempfile::TempDir;

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

    assert!(
        exists,
        "session '{session}' should exist after connect --cwd"
    );
}

#[test]
fn connect_derives_session_name_from_basename() {
    // Create a dir with a known name so the basename-derived session name is predictable.
    let base = std::env::temp_dir().join("muxx-basename-derive-test");
    std::fs::create_dir_all(&base).unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--cwd", base.to_str().unwrap()])
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
        "[projects.my-proj]\ncwd = \"{}\"\n",
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
        "[projects.startupproj]\ncwd = \"{}\"\nstartup = \"echo hello\"\n",
        project_dir.path().to_str().unwrap()
    )
    .unwrap();

    let session = "muxx-test-startup-cmd-session";

    // The --cmd flag should override the config's startup command; either way,
    // the session must be created successfully.
    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_CONFIG_PATH", config_file.path())
        .args(["connect", "--no-attach", "--name", session, "startupproj"])
        .assert()
        .success()
        .stdout(contains("created"));

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
}

#[test]
fn connect_config_alias_uses_alias_as_session_name() {
    // The project dir has a different basename than the alias; without the fix
    // the session would be named after the dir basename, not the alias.
    let project_dir = std::env::temp_dir().join("muxx-alias-cwd-dir");
    std::fs::create_dir_all(&project_dir).unwrap();

    let alias = "my-alias-proj";

    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    write!(
        config_file,
        "[projects.{}]\ncwd = \"{}\"\n",
        alias,
        project_dir.to_str().unwrap()
    )
    .unwrap();

    // Connect via alias with no --name override
    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_CONFIG_PATH", config_file.path())
        .args(["connect", "--no-attach", alias])
        .assert()
        .success()
        .stdout(contains(alias));

    // Session named after alias should exist, not after the dir basename
    let alias_exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", alias])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let dir_session_exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", "muxx-alias-cwd-dir"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", alias])
        .status();
    let _ = std::fs::remove_dir(&project_dir);

    assert!(alias_exists, "session '{alias}' should exist");
    assert!(
        !dir_session_exists,
        "session named after cwd basename should NOT exist"
    );
}

#[test]
fn connect_unknown_session_name_fails() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "muxx-this-session-does-not-exist-xyz",
        ])
        .assert()
        .failure()
        .stderr(contains("session not found"));
}

#[test]
fn connect_errors_on_path_collision() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let session = "muxx-test-path-collision";

    // Create session pointing at dir1
    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            dir1.path().to_str().unwrap(),
            "--name",
            session,
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    // Connect with same name but different dir — must fail with a clear error
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            dir2.path().to_str().unwrap(),
            "--name",
            session,
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(!output.status.success(), "should fail on path collision");
    assert!(
        stderr.contains("session name collision"),
        "should report collision; stderr: {stderr}"
    );
    assert!(
        stderr.contains("--force"),
        "should suggest --force; stderr: {stderr}"
    );
}

#[test]
fn connect_force_bypasses_path_collision() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let session = "muxx-test-force-collision";

    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            dir1.path().to_str().unwrap(),
            "--name",
            session,
        ])
        .assert()
        .success()
        .stdout(contains("created"));

    // --force should allow attaching to the existing session
    Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--force",
            "--cwd",
            dir2.path().to_str().unwrap(),
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

#[test]
fn connect_no_warning_when_path_matches() {
    let dir = TempDir::new().unwrap();
    let session = "muxx-test-path-match";

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

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args([
            "connect",
            "--no-attach",
            "--cwd",
            dir.path().to_str().unwrap(),
            "--name",
            session,
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(output.status.success());
    assert!(
        !stderr.contains("exists but its path is"),
        "should not warn when path matches; stderr: {stderr}"
    );
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

#[test]
fn connect_fuzzy_matches_unique_session() {
    let session = "muxx-test-connect-fuzzy-zq";
    let _ = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", session])
        .status();

    let assert = Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "connect-fuzzy-zq"])
        .assert();

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", &format!("={session}")])
        .status();

    assert
        .success()
        .stdout(contains(format!("matched session '{session}'")));
}

#[test]
fn connect_ambiguous_fuzzy_match_fails() {
    let a = "muxx-test-ambig-zq-one";
    let b = "muxx-test-ambig-zq-two";
    for s in [a, b] {
        let _ = std::process::Command::new("tmux")
            .args(["new-session", "-d", "-s", s])
            .status();
    }

    let assert = Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "ambig-zq"])
        .assert();

    for s in [a, b] {
        let _ = std::process::Command::new("tmux")
            .args(["kill-session", "-t", &format!("={s}")])
            .status();
    }

    assert.failure().stderr(contains("ambiguous"));
}

#[test]
fn connect_without_terminal_creates_but_does_not_attach() {
    // assert_cmd gives the child a non-TTY stdin, like a launcher or script.
    let tmux_dir = TempDir::new().unwrap();
    let assert = Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--name", "muxx-test-no-tty"])
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .assert();

    let exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", "=muxx-test-no-tty"])
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let _ = std::process::Command::new("tmux")
        .args(["kill-server"])
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .status();

    assert
        .success()
        .stdout(contains("created: muxx-test-no-tty"))
        .stdout(contains("not a terminal"));
    assert!(exists, "session should be created even without a terminal");
}

/// Run muxx against an isolated tmux server rooted at `tmux_dir`.
fn muxx_isolated(tmux_dir: &TempDir, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(args)
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .assert()
}

fn tmux_isolated(tmux_dir: &TempDir, args: &[&str]) {
    let _ = std::process::Command::new("tmux")
        .args(args)
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .status();
}

#[test]
fn connect_reuses_renamed_session_for_same_directory() {
    let tmux_dir = TempDir::new().unwrap();
    let root = TempDir::new().unwrap();
    let dir = root.path().join("pathfirst");
    std::fs::create_dir(&dir).unwrap();
    let dir = dir.to_str().unwrap();

    muxx_isolated(&tmux_dir, &["connect", "--no-attach", "--cwd", dir])
        .success()
        .stdout(contains("created: pathfirst"));
    tmux_isolated(
        &tmux_dir,
        &["rename-session", "-t", "=pathfirst", "renamed"],
    );

    let assert = muxx_isolated(&tmux_dir, &["connect", "--no-attach", "--cwd", dir]);
    tmux_isolated(&tmux_dir, &["kill-server"]);

    assert.success().stdout(contains("reused: renamed"));
}

#[test]
fn connect_same_basename_falls_back_to_parent_basename() {
    let tmux_dir = TempDir::new().unwrap();
    let root = TempDir::new().unwrap();
    let work = root.path().join("work/api");
    let oss = root.path().join("oss/api");
    std::fs::create_dir_all(&work).unwrap();
    std::fs::create_dir_all(&oss).unwrap();
    let (work, oss) = (work.to_str().unwrap(), oss.to_str().unwrap());

    let first = muxx_isolated(&tmux_dir, &["connect", "--no-attach", "--cwd", work]);
    let second = muxx_isolated(&tmux_dir, &["connect", "--no-attach", "--cwd", oss]);
    let again = muxx_isolated(&tmux_dir, &["connect", "--no-attach", "--cwd", oss]);
    tmux_isolated(&tmux_dir, &["kill-server"]);

    first.success().stdout(contains("created: api"));
    second.success().stdout(contains("created: oss-api"));
    again.success().stdout(contains("reused: oss-api"));
}

/// Put a fake `zoxide` that runs `body` first on PATH; returns (bin dir, PATH value).
#[cfg(unix)]
fn fake_zoxide(body: &str) -> (TempDir, String) {
    use std::os::unix::fs::PermissionsExt;
    let bin = TempDir::new().unwrap();
    let script = bin.path().join("zoxide");
    std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    (bin, path)
}

#[cfg(unix)]
#[test]
fn connect_falls_back_to_zoxide() {
    let parent = TempDir::new().unwrap();
    let dir = parent.path().join("muxx-test-zoxide-dir");
    std::fs::create_dir(&dir).unwrap();
    let dir = dir.canonicalize().unwrap();
    let (_bin, path) = fake_zoxide(&format!("echo '{}'", dir.display()));
    let session = "muxx-test-zoxide-dir";

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "muxx-zoxide-query-xyz"])
        .env("PATH", &path)
        .output()
        .unwrap();

    let session_path = std::process::Command::new("tmux")
        .args(["display-message", "-p", "-t", session, "#{session_path}"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("created: muxx-test-zoxide-dir"));
    assert_eq!(session_path, dir.to_string_lossy());
}

#[cfg(unix)]
#[test]
fn connect_zoxide_no_match_still_fails() {
    let (_bin, path) = fake_zoxide("exit 1");

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "muxx-zoxide-nomatch-xyz"])
        .env("PATH", &path)
        .assert()
        .failure()
        .stderr(contains("session not found"));
}

/// A tmux server started by muxx must not inherit the project dir as its cwd:
/// if that dir is later deleted, everything the server spawns fails getcwd.
#[test]
fn connect_starts_tmux_server_outside_project_dir() {
    let tmux_dir = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let project_path = std::fs::canonicalize(project.path()).unwrap();
    let session = "muxx-test-server-cwd";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .current_dir(&project_path)
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .assert()
        .success();

    let tmux = |args: &[&str]| {
        std::process::Command::new("tmux")
            .args(args)
            .env_remove("TMUX")
            .env("TMUX_TMPDIR", tmux_dir.path())
            .output()
            .unwrap()
    };
    let pid = String::from_utf8(tmux(&["display", "-p", "-t", session, "#{pid}"]).stdout).unwrap();
    let lsof = std::process::Command::new("lsof")
        .args(["-a", "-p", pid.trim(), "-d", "cwd", "-Fn"])
        .output()
        .unwrap();
    tmux(&["kill-server"]);

    let lsof = String::from_utf8_lossy(&lsof.stdout);
    let server_cwd = lsof.lines().find_map(|l| l.strip_prefix('n')).unwrap_or("");
    assert!(
        !server_cwd.is_empty(),
        "could not read tmux server cwd: {lsof}"
    );
    assert_ne!(std::path::Path::new(server_cwd), project_path);
}
