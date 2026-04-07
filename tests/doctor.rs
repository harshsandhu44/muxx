use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn doctor_exits_successfully_with_tmux() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["doctor"])
        .assert()
        .success()
        .stdout(contains("tmux is installed"));
}

#[test]
fn doctor_alias_doc_works() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["doc"])
        .assert()
        .success();
}

#[test]
fn doctor_no_config_file_exits_zero() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["doctor"])
        .env(
            "MUXX_CONFIG_PATH",
            "/tmp/muxx-nonexistent-config-doctor.json",
        )
        .assert()
        .success();
}

#[test]
fn doctor_invalid_json_exits_nonzero() {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, "not valid json").unwrap();
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["doctor"])
        .env("MUXX_CONFIG_PATH", f.path())
        .assert()
        .failure()
        .stderr(contains("invalid JSON"));
}

#[test]
fn doctor_missing_project_dir_exits_nonzero() {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(
        f,
        r#"{{"projects":{{"phantom":{{"cwd":"/tmp/muxx-doctor-nonexistent-dir-xyz"}}}}}}"#
    )
    .unwrap();
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["doctor"])
        .env("MUXX_CONFIG_PATH", f.path())
        .assert()
        .failure()
        .stderr(contains("directory not found"));
}
