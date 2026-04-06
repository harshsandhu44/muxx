use assert_cmd::Command;
use predicates::str::contains;

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

#[test]
fn list_shows_created_session() {
    let session = "muxx-test-list-visible-session";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(output.status.success());
    assert!(
        stdout.contains(session),
        "expected '{session}' in list output, got: {stdout}"
    );
}

#[test]
fn list_json_contains_expected_fields() {
    let session = "muxx-test-list-json-fields-session";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["list", "--json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    assert!(output.status.success());
    let arr: serde_json::Value = serde_json::from_str(&stdout).expect("not valid JSON");
    let arr = arr.as_array().expect("should be array");
    let entry = arr
        .iter()
        .find(|s| s["name"].as_str() == Some(session))
        .expect("session should appear in JSON output");

    assert!(
        entry["windows"].is_number(),
        "windows field should be a number"
    );
    assert!(
        entry["attached"].is_boolean(),
        "attached field should be a boolean"
    );
    assert!(
        entry["created"].is_number(),
        "created field should be a number"
    );
}

#[test]
fn list_json_array_even_when_no_sessions() {
    // Even with no tmux server running, --json must output a valid empty array.
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["list", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn list_without_json_does_not_crash_with_sessions() {
    let session = "muxx-test-list-text-output";

    Command::cargo_bin("muxx")
        .unwrap()
        .args(["connect", "--no-attach", "--name", session])
        .assert()
        .success();

    let result = Command::cargo_bin("muxx")
        .unwrap()
        .args(["list"])
        .assert()
        .success();

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();

    // The text output should contain the session name
    result.stdout(contains(session));
}
