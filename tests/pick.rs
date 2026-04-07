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
