use assert_cmd::Command;

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
    std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", session])
        .status()
        .unwrap();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        // "fuzz-target" is a substring of "muxx-fuzz-target-abc"
        .args(["attach", "fuzz-target"])
        .env_remove("TMUX")
        .output()
        .unwrap();

    std::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status()
        .ok();

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
