mod cli;
mod commands;
mod core;

fn main() {
    if let Err(e) = cli::run() {
        core::output::error(&e.to_string());
        std::process::exit(1);
    }
}
