use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn status_errors_outside_tmux() {
    Command::cargo_bin("muxx")
        .unwrap()
        .env_remove("TMUX")
        .args(["status"])
        .assert()
        .failure()
        .stderr(contains("not inside a tmux session"));
}
