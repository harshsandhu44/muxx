use anyhow::Result;
use clap_complete::Shell;

/// Emit the shell registration for muxx's dynamic completion engine.
///
/// muxx uses clap_complete's dynamic (env-driven) completer, initialized in
/// `cli::run()` via `CompleteEnv`. The line printed here bootstraps that engine
/// for the given shell, so completions are computed live (sessions, tags, config
/// aliases) with inline descriptions — something the old static scripts could not do.
pub fn run(shell: Shell) -> Result<()> {
    match shell {
        Shell::Fish => println!("COMPLETE=fish muxx | source"),
        Shell::PowerShell => println!("COMPLETE=powershell muxx | Invoke-Expression"),
        // bash, zsh, elvish and anything else use process substitution.
        other => println!("source <(COMPLETE={other} muxx)"),
    }
    Ok(())
}
