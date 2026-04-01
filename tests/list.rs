use assert_cmd::Command;

/// When there are no sessions (or tmux is not running), list returns successfully
/// with either empty output or "no sessions". We can't guarantee tmux state in CI,
/// so we test that the command exits 0 (list never fails for "no sessions").
#[test]
fn list_exits_successfully() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["list"])
        .assert()
        .success();
}

#[test]
fn list_json_flag_outputs_valid_json() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["list", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must be parseable as JSON array
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output is not valid JSON");
    assert!(parsed.is_array(), "JSON output should be an array");
}

#[test]
fn list_alias_ls_works() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["ls"])
        .assert()
        .success();
}
