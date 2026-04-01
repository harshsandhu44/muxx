use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use crate::commands;

#[derive(Parser)]
#[command(
    name = "muxx",
    about = "Minimal tmux session manager",
    long_about = None,
    disable_help_subcommand = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Connect to or create a session (default when no subcommand given)
    #[command(alias = "c")]
    Connect {
        /// Directory or config alias to connect to (defaults to current directory)
        dir: Option<String>,
        /// Override the session name
        #[arg(long)]
        name: Option<String>,
        /// Create the session without attaching to it
        #[arg(long = "no-attach")]
        no_attach: bool,
        /// Shell command to send on new session creation only
        #[arg(long)]
        cmd: Option<String>,
    },

    /// List all tmux sessions
    #[command(alias = "ls")]
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Kill a session by name
    #[command(alias = "k")]
    Kill {
        /// Session name to kill
        name: String,
        /// Kill even if it is the current session
        #[arg(long)]
        force: bool,
    },

    /// Print the current session name
    #[command(alias = "cur")]
    Current,

    /// Print shell completion script
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => commands::connect::run(None, None, false, None),
        Some(Commands::Connect {
            dir,
            name,
            no_attach,
            cmd,
        }) => commands::connect::run(dir.as_deref(), name.as_deref(), no_attach, cmd.as_deref()),
        Some(Commands::List { json }) => commands::list::run(json),
        Some(Commands::Kill { name, force }) => commands::kill::run(&name, force),
        Some(Commands::Current) => commands::current::run(),
        Some(Commands::Completion { shell }) => {
            commands::completion::run(shell, &mut Cli::command())
        }
    }
}
