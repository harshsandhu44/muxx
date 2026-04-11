use std::io::IsTerminal;

pub fn success(msg: &str) {
    if std::io::stdout().is_terminal() {
        println!("\x1b[32m✓\x1b[0m {msg}");
    } else {
        println!("✓ {msg}");
    }
}

pub fn info(msg: &str) {
    if std::io::stdout().is_terminal() {
        println!("\x1b[36m→\x1b[0m {msg}");
    } else {
        println!("→ {msg}");
    }
}

pub fn error(msg: &str) {
    if std::io::stderr().is_terminal() {
        eprintln!("\x1b[31m✗\x1b[0m {msg}");
    } else {
        eprintln!("✗ {msg}");
    }
}

pub fn hint(msg: &str) {
    if std::io::stdout().is_terminal() {
        println!("\x1b[2m  {msg}\x1b[0m");
    } else {
        println!("  {msg}");
    }
}
