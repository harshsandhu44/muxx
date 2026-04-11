use anyhow::Result;

pub fn run(verbose: bool) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    if verbose {
        println!(
            "muxx {}\nos:   {}\narch: {}",
            version,
            std::env::consts::OS,
            std::env::consts::ARCH,
        );
    } else {
        println!("muxx {version}");
    }
    Ok(())
}
