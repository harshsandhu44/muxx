use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use clap_complete::Shell;

use crate::commands;

fn complete_sessions(_prefix: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Ok(out) = std::process::Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
    else {
        return vec![];
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(CompletionCandidate::new)
        .collect()
}

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
    /// Attach to or switch to an existing session by name
    #[command(alias = "a")]
    Attach {
        /// Name of the tmux session to attach to
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
    },

    /// Connect to or create a session (default when no subcommand given)
    #[command(alias = "c")]
    Connect {
        /// Existing session name or config alias to connect to
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: Option<String>,
        /// Create a new session from this directory path
        #[arg(short = 'c', long = "cwd", value_hint = clap::ValueHint::DirPath)]
        cwd: Option<String>,
        /// Override the session name (only applies with --cwd)
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
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
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
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    match cli.command {
        None => commands::connect::run(None, None, None, false, None),
        Some(Commands::Attach { session }) => commands::attach::run(&session),
        Some(Commands::Connect {
            session,
            cwd,
            name,
            no_attach,
            cmd,
        }) => commands::connect::run(
            session.as_deref(),
            cwd.as_deref(),
            name.as_deref(),
            no_attach,
            cmd.as_deref(),
        ),
        Some(Commands::List { json }) => commands::list::run(json),
        Some(Commands::Kill { name, force }) => commands::kill::run(&name, force),
        Some(Commands::Current) => commands::current::run(),
        Some(Commands::Completion { shell }) => {
            commands::completion::run(shell, &mut Cli::command())
        }
    }
}
