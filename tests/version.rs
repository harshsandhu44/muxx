use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn version_prints_version_number() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["version"])
        .assert()
        .success()
        .stdout(contains("muxx "));
}

#[test]
fn version_output_matches_cargo_version() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["version"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = format!("muxx {}", env!("CARGO_PKG_VERSION"));
    assert!(
        stdout.trim() == expected,
        "expected '{expected}', got '{}'",
        stdout.trim()
    );
}

#[test]
fn version_verbose_includes_os_and_arch() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["version", "--verbose"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("os:"), "verbose output should include os");
    assert!(
        stdout.contains("arch:"),
        "verbose output should include arch"
    );
}

#[test]
fn version_verbose_includes_version_number() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["version", "--verbose"])
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}
