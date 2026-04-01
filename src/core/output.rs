pub fn success(msg: &str) {
    println!("\x1b[32m✓\x1b[0m {msg}");
}

pub fn info(msg: &str) {
    println!("\x1b[36m→\x1b[0m {msg}");
}

pub fn error(msg: &str) {
    eprintln!("\x1b[31m✗\x1b[0m {msg}");
}

pub fn hint(msg: &str) {
    println!("\x1b[2m  {msg}\x1b[0m");
}
