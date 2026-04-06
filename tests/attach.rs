use assert_cmd::Command;
use std::process::Stdio;

fn kill_session(name: &str) {
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", name])
        .stderr(Stdio::null())
        .status();
}

fn new_session(name: &str) {
    kill_session(name);
    let _ = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", name])
        .stderr(Stdio::null())
        .status();
}

// --- error cases ---

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

// --- fuzzy matching ---

#[test]
fn fuzzy_match_resolves_single_candidate() {
    // Create a uniquely named session so fuzzy matching has exactly one target.
    let session = "muxx-fuzz-target-abc";
    new_session(session);

    let output = Command::cargo_bin("muxx")
        .unwrap()
        // "fuzz-target" is a substring of "muxx-fuzz-target-abc"
        .args(["attach", "fuzz-target"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    kill_session(session);

    // The process will fail because there's no real terminal to attach to,
    // but it should NOT print "does not exist" — it should match and attempt to attach.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("does not exist"),
        "expected fuzzy match, got: {stderr}"
    );
}

#[test]
fn fuzzy_no_match_shows_does_not_exist() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["attach", "zzz-no-such-session-zzz"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
}

// --- attach - (last session) ---

#[test]
fn attach_dash_without_prior_session_fails() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["attach", "-"])
        .env_remove("TMUX")
        // Use a temp HOME so there's no leftover last_session state file.
        .env("HOME", std::env::temp_dir())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no last session"),
        "expected 'no last session', got: {stderr}"
    );
}

// --- ambiguous fuzzy matching ---

#[test]
fn fuzzy_ambiguous_match_reports_error_and_candidates() {
    let session_a = "muxx-ambig-test-session-aaa";
    let session_b = "muxx-ambig-test-session-bbb";

    // Create two sessions that share the prefix "muxx-ambig-test-session"
    new_session(session_a);
    new_session(session_b);

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["attach", "muxx-ambig-test-session"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    kill_session(session_a);
    kill_session(session_b);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ambiguous"),
        "expected 'ambiguous' in stderr, got: {stderr}"
    );
}

// --- error message quality ---

#[test]
fn attach_nonexistent_stderr_contains_session_name() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["attach", "muxx-specific-nonexistent-name"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("muxx-specific-nonexistent-name"),
        "error message should include the session name, got: {stderr}"
    );
}

#[test]
fn attach_nonexistent_hints_list_command() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["attach", "muxx-no-such-session-hint-test"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The hint ("muxx list") is written to stdout via hint()
    assert!(
        stdout.contains("list"),
        "expected a hint mentioning 'list', got stdout: {stdout}"
    );
}
