use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn config_path_prints_a_path() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["config", "path"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().ends_with("config.toml"),
        "config path should end with config.toml, got: {stdout}"
    );
}

#[test]
fn config_path_respects_muxx_config_path_env() {
    let tmp = tempfile::NamedTempFile::new().unwrap();

    let output = Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_CONFIG_PATH", tmp.path())
        .args(["config", "path"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim() == tmp.path().to_str().unwrap(),
        "config path should match MUXX_CONFIG_PATH, got: {stdout}"
    );
}

#[test]
fn config_show_reports_missing_file() {
    // Use a path that definitely does not exist
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let missing = tmp_dir.path().join("no-such-config.toml");

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_CONFIG_PATH", &missing)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(contains("does not exist"));
}

#[test]
fn config_show_prints_file_contents() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    write!(tmp, "[projects.myapp]\ncwd = \"/tmp/myapp\"\n").unwrap();

    Command::cargo_bin("muxx")
        .unwrap()
        .env("MUXX_CONFIG_PATH", tmp.path())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(contains("myapp"));
}
