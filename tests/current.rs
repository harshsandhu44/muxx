use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn current_errors_outside_tmux() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["current"])
        .env_remove("TMUX")
        .assert()
        .failure()
        .stderr(contains("not inside a tmux session"));
}

#[test]
fn current_alias_cur_errors_outside_tmux() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["cur"])
        .env_remove("TMUX")
        .assert()
        .failure()
        .stderr(contains("not inside a tmux session"));
}
