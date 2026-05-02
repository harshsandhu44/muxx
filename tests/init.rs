use assert_cmd::Command;
use predicates::str::contains;

/// Return a `muxx` command with isolated config, tags, and notes stores.
fn isolated(config: &std::path::Path, tags: &std::path::Path) -> assert_cmd::Command {
    let mut cmd = Command::cargo_bin("muxx").unwrap();
    cmd.env("MUXX_CONFIG_PATH", config)
        .env("MUXX_TAGS_PATH", tags);
    cmd
}

/// Simulate `muxx init --no-attach` feeding the four prompt answers via stdin.
fn run_init(
    dir: &std::path::Path,
    config: &std::path::Path,
    tags: &std::path::Path,
    stdin: &str,
) -> std::process::Output {
    isolated(config, tags)
        .current_dir(dir)
        .args(["init", "--no-attach"])
        .write_stdin(stdin)
        .output()
        .unwrap()
}

// ── config write tests ──────────────────────────────────────────────────────

#[test]
fn init_writes_project_to_config() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let out = run_init(dir.path(), config.path(), tags.path(), "myapp\n\n\nn\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let raw = std::fs::read_to_string(config.path()).unwrap();
    assert!(
        raw.contains("[projects.myapp]"),
        "config should contain the project"
    );
    assert!(
        raw.contains(dir.path().to_str().unwrap()),
        "config should contain the cwd"
    );
}

#[test]
fn init_writes_startup_command() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let out = run_init(
        dir.path(),
        config.path(),
        tags.path(),
        "startupapp\nnpm run dev\n\nn\n",
    );
    assert!(out.status.success());

    let raw = std::fs::read_to_string(config.path()).unwrap();
    assert!(
        raw.contains("npm run dev"),
        "config should contain startup command"
    );
}

#[test]
fn init_omits_startup_when_empty() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let out = run_init(dir.path(), config.path(), tags.path(), "plainproj\n\n\nn\n");
    assert!(out.status.success());

    let raw = std::fs::read_to_string(config.path()).unwrap();
    assert!(
        !raw.contains("startup"),
        "config should not have startup key when input was empty; got:\n{raw}"
    );
}

#[test]
fn init_writes_cwd_as_current_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let out = run_init(dir.path(), config.path(), tags.path(), "cwdproj\n\n\nn\n");
    assert!(out.status.success());

    let raw = std::fs::read_to_string(config.path()).unwrap();
    // The path may be canonicalized; check that the temp dir path appears.
    let dir_str = dir.path().to_str().unwrap();
    assert!(
        raw.contains(dir_str),
        "config cwd should be the working directory; config:\n{raw}"
    );
}

#[test]
fn init_default_name_from_cwd_basename() {
    let base = std::env::temp_dir().join("muxx-init-default-name-test");
    std::fs::create_dir_all(&base).unwrap();

    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    // Empty first line → accept default derived from basename.
    let out = run_init(&base, config.path(), tags.path(), "\n\n\nn\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let raw = std::fs::read_to_string(config.path()).unwrap();
    assert!(
        raw.contains("muxx-init-default-name-test"),
        "config should use the directory basename as project name; config:\n{raw}"
    );

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn init_overwrites_existing_project() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    // First init
    let out = run_init(
        dir.path(),
        config.path(),
        tags.path(),
        "overwrite-me\nold cmd\n\nn\n",
    );
    assert!(out.status.success());

    // Second init — different startup command, same project name
    let out = run_init(
        dir.path(),
        config.path(),
        tags.path(),
        "overwrite-me\nnew cmd\n\nn\n",
    );
    assert!(out.status.success());

    let raw = std::fs::read_to_string(config.path()).unwrap();
    assert!(
        raw.contains("new cmd"),
        "config should reflect the new startup command"
    );
    assert!(
        !raw.contains("old cmd"),
        "old startup command should be gone after overwrite"
    );
}

#[test]
fn init_prints_config_written_message() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    isolated(config.path(), tags.path())
        .current_dir(dir.path())
        .args(["init", "--no-attach"])
        .write_stdin("printmsg\n\n\nn\n")
        .assert()
        .success()
        .stdout(contains("config written"));
}

// ── tags tests ──────────────────────────────────────────────────────────────

#[test]
fn init_writes_tags_to_store() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let out = run_init(
        dir.path(),
        config.path(),
        tags.path(),
        "taggedproj\n\nwork,nextjs\nn\n",
    );
    assert!(out.status.success());

    let raw = std::fs::read_to_string(tags.path()).unwrap();
    assert!(raw.contains("work"), "tags store should contain 'work'");
    assert!(raw.contains("nextjs"), "tags store should contain 'nextjs'");
}

#[test]
fn init_normalises_tags_to_lowercase() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let out = run_init(
        dir.path(),
        config.path(),
        tags.path(),
        "lcproj\n\nWORK,Rust\nn\n",
    );
    assert!(out.status.success());

    let raw = std::fs::read_to_string(tags.path()).unwrap();
    assert!(raw.contains("work"), "tags should be lowercased");
    assert!(raw.contains("rust"), "tags should be lowercased");
    assert!(!raw.contains("WORK"), "uppercase tag should not appear");
}

#[test]
fn init_no_tags_leaves_tags_file_unchanged() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    // Pre-seed the tags file with existing data.
    std::fs::write(tags.path(), "[tags]\nother = [\"personal\"]\n").unwrap();

    let out = run_init(dir.path(), config.path(), tags.path(), "notagproj\n\n\nn\n");
    assert!(out.status.success());

    let raw = std::fs::read_to_string(tags.path()).unwrap();
    assert!(
        raw.contains("other"),
        "pre-existing tags should be untouched when no tags entered"
    );
    assert!(
        !raw.contains("notagproj"),
        "project with no tags should not appear in tags store"
    );
}

#[test]
fn init_prints_tags_in_output() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    isolated(config.path(), tags.path())
        .current_dir(dir.path())
        .args(["init", "--no-attach"])
        .write_stdin("tagoutput\n\nrust,personal\nn\n")
        .assert()
        .success()
        .stdout(contains("rust"))
        .stdout(contains("personal"));
}

// ── session creation tests ──────────────────────────────────────────────────

#[test]
fn init_skips_session_when_answered_no() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let session = "muxx-init-test-no-session";

    let out = run_init(
        dir.path(),
        config.path(),
        tags.path(),
        &format!("{session}\n\n\nn\n"),
    );
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("muxx"),
        "should print hint to run muxx; stdout: {stdout}"
    );

    let session_exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    assert!(
        !session_exists,
        "no tmux session should be created when user answers 'n'"
    );
}

#[test]
fn init_creates_session_when_answered_yes() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    let session = "muxx-init-test-yes-session";

    let out = run_init(
        dir.path(),
        config.path(),
        tags.path(),
        &format!("{session}\n\n\ny\n"),
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let session_exists = std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(
        session_exists,
        "tmux session '{session}' should exist after init with 'y'"
    );
}

#[test]
fn init_prints_hint_when_session_not_created() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = tempfile::NamedTempFile::new().unwrap();
    let tags = tempfile::NamedTempFile::new().unwrap();

    isolated(config.path(), tags.path())
        .current_dir(dir.path())
        .args(["init", "--no-attach"])
        .write_stdin("hintproj\n\n\nn\n")
        .assert()
        .success()
        .stdout(contains("hintproj"));
}
