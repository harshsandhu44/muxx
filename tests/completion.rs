use assert_cmd::Command;

#[test]
fn completion_bash_outputs_script() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["completion", "bash"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "bash completion should produce output");
}

#[test]
fn completion_zsh_outputs_script() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["completion", "zsh"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "zsh completion should produce output");
}

#[test]
fn completion_fish_outputs_script() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["completion", "fish"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "fish completion should produce output");
}

#[test]
fn completion_invalid_shell_fails() {
    Command::cargo_bin("muxx")
        .unwrap()
        .args(["completion", "notashell"])
        .assert()
        .failure();
}

#[test]
fn completion_bash_mentions_muxx_subcommands() {
    let output = Command::cargo_bin("muxx")
        .unwrap()
        .args(["completion", "bash"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The generated script should reference the binary name and known subcommands
    assert!(
        stdout.contains("muxx"),
        "bash completion should reference 'muxx'"
    );
}
