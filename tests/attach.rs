use assert_cmd::Command;

#[test]
fn attach_nonexistent_session_fails() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["attach", "muxx-test-nonexistent-xyz"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
}

#[test]
fn attach_alias_works() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["a", "muxx-test-nonexistent-xyz"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
}
