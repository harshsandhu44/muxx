use assert_cmd::Command;
use predicates::str::contains;

/// Run export and import against isolated tag/note stores so tests don't
/// touch the real user data in ~/.config/muxx/.
fn isolated_env(tags_file: &std::path::Path, notes_file: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("muxx").unwrap();
    cmd.env("MUXX_TAGS_PATH", tags_file)
        .env("MUXX_NOTES_PATH", notes_file);
    cmd
}

/// Write a tags + notes TOML file with known content for import tests.
fn write_export_toml(path: &std::path::Path) {
    std::fs::write(
        path,
        "[tags]\nmyapp = [\"rust\", \"work\"]\n\n[notes]\nmyapp = \"fixing auth\"\n",
    )
    .unwrap();
}

#[test]
fn export_stdout_is_valid_toml() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    let output = isolated_env(tags.path(), notes.path())
        .args(["export"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Empty stores still produce valid TOML (two empty tables).
    toml::from_str::<toml::Value>(&stdout).expect("export output should be valid TOML");
}

#[test]
fn export_to_file_creates_file() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();
    let out_dir = tempfile::TempDir::new().unwrap();
    let out_path = out_dir.path().join("backup.toml");

    isolated_env(tags.path(), notes.path())
        .args(["export", out_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("exported to"));

    assert!(out_path.exists(), "export file should be created");
}

#[test]
fn export_includes_tags_and_notes() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    // Seed data via import so we have something to export.
    let seed = tempfile::NamedTempFile::new().unwrap();
    write_export_toml(seed.path());

    isolated_env(tags.path(), notes.path())
        .args(["import", seed.path().to_str().unwrap()])
        .assert()
        .success();

    let output = isolated_env(tags.path(), notes.path())
        .args(["export"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("myapp"),
        "export should contain session name"
    );
    assert!(stdout.contains("rust"), "export should contain tag");
    assert!(stdout.contains("fixing auth"), "export should contain note");
}

#[test]
fn import_replaces_existing_data() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    let import_file = tempfile::NamedTempFile::new().unwrap();
    write_export_toml(import_file.path());

    isolated_env(tags.path(), notes.path())
        .args(["import", import_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("imported"));

    // Confirm data is present by exporting and checking content.
    let output = isolated_env(tags.path(), notes.path())
        .args(["export"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("myapp"));
    assert!(stdout.contains("rust"));
    assert!(stdout.contains("fixing auth"));
}

#[test]
fn import_merge_keeps_existing_data() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    // Seed initial data.
    let seed = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        seed.path(),
        "[tags]\nother = [\"personal\"]\n\n[notes]\nother = \"existing note\"\n",
    )
    .unwrap();
    isolated_env(tags.path(), notes.path())
        .args(["import", seed.path().to_str().unwrap()])
        .assert()
        .success();

    // Merge in a second batch.
    let batch = tempfile::NamedTempFile::new().unwrap();
    write_export_toml(batch.path());
    isolated_env(tags.path(), notes.path())
        .args(["import", batch.path().to_str().unwrap(), "--merge"])
        .assert()
        .success()
        .stdout(contains("merged"));

    let output = isolated_env(tags.path(), notes.path())
        .args(["export"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("other"),
        "original session should survive merge"
    );
    assert!(stdout.contains("myapp"), "merged session should appear");
}

#[test]
fn import_errors_on_missing_file() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    isolated_env(tags.path(), notes.path())
        .args(["import", "/tmp/muxx-nonexistent-import-xyz.toml"])
        .assert()
        .failure()
        .stderr(contains("failed to read"));
}

#[test]
fn import_errors_on_invalid_toml() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    let bad = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(bad.path(), "this is not [ valid toml !!!").unwrap();

    isolated_env(tags.path(), notes.path())
        .args(["import", bad.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(contains("invalid TOML"));
}

#[test]
fn export_import_round_trip() {
    let tags = tempfile::NamedTempFile::new().unwrap();
    let notes = tempfile::NamedTempFile::new().unwrap();

    // Seed data.
    let seed = tempfile::NamedTempFile::new().unwrap();
    write_export_toml(seed.path());
    isolated_env(tags.path(), notes.path())
        .args(["import", seed.path().to_str().unwrap()])
        .assert()
        .success();

    // Export to a file.
    let out_dir = tempfile::TempDir::new().unwrap();
    let backup = out_dir.path().join("backup.toml");
    isolated_env(tags.path(), notes.path())
        .args(["export", backup.to_str().unwrap()])
        .assert()
        .success();

    // Reset stores and import from the backup.
    let tags2 = tempfile::NamedTempFile::new().unwrap();
    let notes2 = tempfile::NamedTempFile::new().unwrap();
    isolated_env(tags2.path(), notes2.path())
        .args(["import", backup.to_str().unwrap()])
        .assert()
        .success();

    // Data should survive the round-trip.
    let output = isolated_env(tags2.path(), notes2.path())
        .args(["export"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("myapp"));
    assert!(stdout.contains("rust"));
    assert!(stdout.contains("fixing auth"));
}
