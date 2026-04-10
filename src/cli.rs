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
        /// Filter sessions by tag (repeatable: --tag work --tag rust)
        #[arg(long = "tag", action = clap::ArgAction::Append)]
        tags: Vec<String>,
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

    /// Rename an existing tmux session
    #[command(alias = "rn")]
    Rename {
        /// Current session name
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        from: String,
        /// New session name
        to: String,
    },

    /// Interactively pick a session using fzf
    #[command(alias = "p")]
    Pick {
        /// Select without attaching (for testing)
        #[arg(long = "no-attach")]
        no_attach: bool,
        /// Only show sessions matching all given tags
        #[arg(long = "tag", action = clap::ArgAction::Append)]
        tags: Vec<String>,
    },

    /// Add, remove, or list tags on sessions
    #[command(alias = "t")]
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// Print the current session name
    #[command(alias = "cur")]
    Current,

    /// Validate environment and configuration
    #[command(alias = "doc")]
    Doctor,

    /// Print shell completion script
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum TagAction {
    /// Add tags to a session; opens fzf picker when no tags are given
    Add {
        /// Session name to tag
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
        /// Tags to add — omit to pick interactively with fzf
        #[arg(num_args = 0..)]
        tags: Vec<String>,
    },

    /// Remove tags from a session; opens fzf picker when no tags are given
    Rm {
        /// Session name
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
        /// Tags to remove — omit to pick interactively with fzf
        #[arg(num_args = 0..)]
        tags: Vec<String>,
    },

    /// Delete a tag from every session that has it; opens fzf picker when no tag given
    #[command(alias = "del")]
    Delete {
        /// Tag to delete globally (omit to pick interactively with fzf)
        tag: Option<String>,
    },

    /// Interactively toggle tags on a session (fzf multi-select)
    #[command(alias = "e")]
    Edit {
        /// Session name
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
    },

    /// Remove all tags from a session
    Clear {
        /// Session name
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
    },

    /// List tags for a session, or all sessions if no name given
    #[command(alias = "list")]
    Ls {
        /// Session name (omit to list all tagged sessions)
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: Option<String>,
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
        Some(Commands::List { json, tags }) => commands::list::run(json, &tags),
        Some(Commands::Kill { name, force }) => commands::kill::run(&name, force),
        Some(Commands::Rename { from, to }) => commands::rename::run(&from, &to),
        Some(Commands::Pick { no_attach, tags }) => commands::pick::run(no_attach, &tags),
        Some(Commands::Tag { action }) => commands::tag::run(action),
        Some(Commands::Current) => commands::current::run(),
        Some(Commands::Doctor) => commands::doctor::run(),
        Some(Commands::Completion { shell }) => {
            commands::completion::run(shell, &mut Cli::command())
        }
    }
}
