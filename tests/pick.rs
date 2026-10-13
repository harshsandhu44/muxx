use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn pick_help_shows_command() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["pick", "--help"])
        .assert()
        .success()
        .stdout(contains("fzf"));
}

#[test]
fn pick_alias_p_help_recognized() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["p", "--help"])
        .assert()
        .success();
}

/// `pick` against an isolated tmux server and a temp config holding one alias.
/// fzf runs non-interactively via `--filter`, selecting lines matching `query`.
fn pick_with_alias(query: &str, extra: &[&str]) -> (assert_cmd::assert::Assert, tempfile::TempDir) {
    let tmux_dir = tempfile::TempDir::new().unwrap();
    let project = tempfile::TempDir::new().unwrap();
    let config = tmux_dir.path().join("config.toml");
    std::fs::write(
        &config,
        format!(
            "[projects.muxx-pick-alias]\ncwd = \"{}\"\n",
            project.path().display()
        ),
    )
    .unwrap();

    let assert = Command::cargo_bin("muxx")
        .unwrap()
        .arg("pick")
        .args(extra)
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .env("MUXX_CONFIG_PATH", &config)
        .env("MUXX_TAGS_PATH", tmux_dir.path().join("tags.toml"))
        .env("FZF_DEFAULT_OPTS", format!("--filter={query}"))
        .assert();

    let _ = std::process::Command::new("tmux")
        .arg("kill-server")
        .env_remove("TMUX")
        .env("TMUX_TMPDIR", tmux_dir.path())
        .status();
    (assert, tmux_dir)
}

#[test]
fn pick_creates_session_from_idle_config_alias() {
    let (assert, _dir) = pick_with_alias("muxx-pick-alias", &[]);
    assert
        .success()
        .stdout(contains("created: muxx-pick-alias"));
}

#[test]
fn pick_tag_filter_hides_idle_aliases() {
    let (assert, _dir) = pick_with_alias("muxx-pick-alias", &["--tag", "work"]);
    assert.success().stdout(contains("no sessions"));
}
