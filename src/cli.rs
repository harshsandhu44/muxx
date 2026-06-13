use std::ffi::OsStr;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use clap_complete::Shell;

use crate::commands;
use crate::core::{config::load_config, output, tags::load_tags, tmux::list_sessions};

/// Live tmux sessions, each described with window count and attach state.
fn complete_sessions(_prefix: &OsStr) -> Vec<CompletionCandidate> {
    list_sessions()
        .into_iter()
        .map(|s| {
            let state = if s.attached { "attached" } else { "detached" };
            CompletionCandidate::new(s.name).help(Some(format!("{}w · {state}", s.windows).into()))
        })
        .collect()
}

/// `connect` targets: live sessions plus configured project aliases.
fn complete_connect_targets(_prefix: &OsStr) -> Vec<CompletionCandidate> {
    let sessions = list_sessions();
    let live: std::collections::HashSet<String> = sessions.iter().map(|s| s.name.clone()).collect();

    let mut candidates: Vec<CompletionCandidate> = sessions
        .into_iter()
        .map(|s| {
            let state = if s.attached { "attached" } else { "detached" };
            CompletionCandidate::new(s.name)
                .help(Some(format!("session · {}w · {state}", s.windows).into()))
        })
        .collect();

    for (name, proj) in &load_config().projects {
        // Avoid duplicating a live session that shares the alias name.
        if live.contains(name) {
            continue;
        }
        candidates.push(
            CompletionCandidate::new(name)
                .help(Some(format!("config alias → {}", proj.cwd).into())),
        );
    }
    candidates
}

/// Every tag known across all sessions.
fn complete_all_tags(_prefix: &OsStr) -> Vec<CompletionCandidate> {
    load_tags()
        .all_known_tags()
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

/// Context-aware: only the tags the session named earlier on the line currently has.
/// Falls back to all known tags if the session cannot be determined.
fn complete_session_tags(_prefix: &OsStr) -> Vec<CompletionCandidate> {
    let tags = load_tags();
    match tag_session_from_cmdline() {
        Some(session) => {
            let owned = tags.get_tags(&session);
            if owned.is_empty() {
                return Vec::new();
            }
            owned.into_iter().map(CompletionCandidate::new).collect()
        }
        None => tags
            .all_known_tags()
            .into_iter()
            .map(CompletionCandidate::new)
            .collect(),
    }
}

/// Scan the completion command line for `tag <action> <session>` and return the session.
/// Used to scope tag completion (e.g. `tag rm`) to a session's own tags.
fn tag_session_from_cmdline() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let tag_pos = args.iter().position(|a| a == "tag" || a == "t")?;
    let mut rest = args[tag_pos + 1..].iter().filter(|a| !a.starts_with('-'));
    let _action = rest.next()?; // rm / clear / edit / ...
    rest.next().cloned() // session positional
}

#[derive(Parser)]
#[command(
    name = "muxx",
    about = "Minimal tmux session manager",
    long_about = "Minimal tmux session manager.\n\n\
        Run `muxx` with no arguments to create or attach a session for the current\n\
        directory. Pass a name to attach to an existing session or a configured\n\
        project alias.\n\n\
        Examples:\n  \
          muxx                       connect to a session for the current directory\n  \
          muxx web                   attach to the session (or alias) named 'web'\n  \
          muxx --cwd ~/code/api      create a session from a directory\n  \
          muxx session ls            list sessions\n  \
          muxx session kill web      kill the 'web' session\n  \
          muxx tag add web rust      tag a session",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Disable colored output (also honors the NO_COLOR env var)
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Connect to or create a session (default when no subcommand given)
    #[command(alias = "c")]
    Connect {
        /// Existing session name or config alias to connect to
        #[arg(add = ArgValueCompleter::new(complete_connect_targets))]
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
        /// Attach to existing session even if its path differs from the requested path
        #[arg(long)]
        force: bool,
    },

    /// List all tmux sessions (shortcut for `session ls`)
    #[command(alias = "ls")]
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Filter sessions by tag (repeatable: --tag work --tag rust)
        #[arg(long = "tag", action = clap::ArgAction::Append, add = ArgValueCompleter::new(complete_all_tags))]
        tags: Vec<String>,
    },

    /// Kill a session by name (shortcut for `session kill`)
    #[command(alias = "k")]
    Kill {
        /// Session name to kill
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        name: String,
        /// Kill even if it is the current session
        #[arg(long)]
        force: bool,
    },

    /// Interactively pick a session using fzf (shortcut for `session pick`)
    #[command(alias = "p")]
    Pick {
        /// Select without attaching (for testing)
        #[arg(long = "no-attach")]
        no_attach: bool,
        /// Only show sessions matching all given tags
        #[arg(long = "tag", action = clap::ArgAction::Append, add = ArgValueCompleter::new(complete_all_tags))]
        tags: Vec<String>,
    },

    /// Re-attach to the last used session (shortcut for `session last`)
    #[command(alias = "l")]
    Last,

    /// Manage tmux sessions (ls, kill, rename, pick, last, new, note)
    #[command(alias = "s")]
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// Add, remove, or list tags on sessions
    #[command(alias = "t")]
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// Print the current session name (for shell prompt integration)
    #[command(alias = "cur")]
    Current,

    /// Print current session name, tags, and note (for shell prompt integration)
    Status,

    /// Validate environment and configuration
    #[command(alias = "doc")]
    Doctor,

    /// Remove tags and notes for sessions that no longer exist in tmux
    Gc,

    /// Register the current project and optionally create a session
    Init {
        /// Project name (default: sanitized directory name)
        #[arg(long)]
        name: Option<String>,
        /// Startup command to run when the session is created
        #[arg(long)]
        startup: Option<String>,
        /// Tags to assign; repeatable: --tag work --tag rust
        #[arg(long = "tag", add = ArgValueCompleter::new(complete_all_tags))]
        tags: Vec<String>,
        /// Register without creating a session
        #[arg(long)]
        no_create: bool,
        /// Create the session without attaching to it
        #[arg(long)]
        no_attach: bool,
        /// Overwrite an existing project config without warning
        #[arg(long)]
        force: bool,
    },

    /// Manage the muxx configuration file
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Print shell completion setup (add to your shell rc)
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Print version information
    Version {
        /// Show OS and architecture details
        #[arg(long)]
        verbose: bool,
    },

    // ----- Hidden back-compat aliases (old flat spellings) -----
    /// Attach to or switch to an existing session by name
    #[command(alias = "a", hide = true)]
    Attach {
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
    },

    /// Rename an existing tmux session
    #[command(alias = "rn", hide = true)]
    Rename {
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        from: String,
        to: String,
    },

    /// Get or set a short note on a session
    #[command(hide = true)]
    Note {
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
        text: Option<String>,
        #[arg(long)]
        clear: bool,
    },

    /// Create a new session from a directory path
    #[command(alias = "n", hide = true)]
    New {
        #[arg(value_hint = clap::ValueHint::DirPath)]
        path: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        cmd: Option<String>,
        #[arg(long = "no-attach")]
        no_attach: bool,
        #[arg(long)]
        force: bool,
    },

    /// Export tags and notes to a TOML file
    #[command(hide = true)]
    Export { path: Option<String> },

    /// Import tags and notes from a TOML file
    #[command(hide = true)]
    Import {
        path: String,
        #[arg(long)]
        merge: bool,
    },
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// List all tmux sessions
    #[command(alias = "ls")]
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Filter sessions by tag (repeatable: --tag work --tag rust)
        #[arg(long = "tag", action = clap::ArgAction::Append, add = ArgValueCompleter::new(complete_all_tags))]
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
        #[arg(long = "tag", action = clap::ArgAction::Append, add = ArgValueCompleter::new(complete_all_tags))]
        tags: Vec<String>,
    },

    /// Re-attach to the last used session
    #[command(alias = "l")]
    Last,

    /// Create a new session from a directory path
    #[command(alias = "n")]
    New {
        /// Directory path for the new session
        #[arg(value_hint = clap::ValueHint::DirPath)]
        path: String,
        /// Override the session name
        #[arg(long)]
        name: Option<String>,
        /// Shell command to send on session creation
        #[arg(long)]
        cmd: Option<String>,
        /// Create the session without attaching to it
        #[arg(long = "no-attach")]
        no_attach: bool,
        /// Attach to existing session even if its path differs from the requested path
        #[arg(long)]
        force: bool,
    },

    /// Get or set a short note on a session
    Note {
        /// Session name
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
        /// Note text to set (omit to print the current note)
        text: Option<String>,
        /// Clear the note
        #[arg(long)]
        clear: bool,
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
        #[arg(num_args = 0.., add = ArgValueCompleter::new(complete_all_tags))]
        tags: Vec<String>,
    },

    /// Remove tags from a session; opens fzf picker when no tags are given
    Rm {
        /// Session name
        #[arg(add = ArgValueCompleter::new(complete_sessions))]
        session: String,
        /// Tags to remove — omit to pick interactively with fzf
        #[arg(num_args = 0.., add = ArgValueCompleter::new(complete_session_tags))]
        tags: Vec<String>,
    },

    /// Delete a tag from every session that has it; opens fzf picker when no tag given
    #[command(alias = "del")]
    Delete {
        /// Tag to delete globally (omit to pick interactively with fzf)
        #[arg(add = ArgValueCompleter::new(complete_all_tags))]
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

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print the config file path and its contents
    Show,
    /// Open the config file in $EDITOR (falls back to vi)
    Edit,
    /// Print the config file path only (for scripting)
    Path,
    /// Export tags and notes to a TOML file
    Export {
        /// Output file path (omit to print to stdout)
        path: Option<String>,
    },
    /// Import tags and notes from a TOML file
    Import {
        /// Input file path
        path: String,
        /// Merge with existing data instead of replacing
        #[arg(long)]
        merge: bool,
    },
}

fn dispatch_session(action: SessionAction) -> anyhow::Result<()> {
    match action {
        SessionAction::List { json, tags } => commands::list::run(json, &tags),
        SessionAction::Kill { name, force } => commands::kill::run(&name, force),
        SessionAction::Rename { from, to } => commands::rename::run(&from, &to),
        SessionAction::Pick { no_attach, tags } => commands::pick::run(no_attach, &tags),
        SessionAction::Last => commands::last::run(),
        SessionAction::New {
            path,
            name,
            cmd,
            no_attach,
            force,
        } => commands::new::run(&path, name.as_deref(), cmd.as_deref(), no_attach, force),
        SessionAction::Note {
            session,
            text,
            clear,
        } => commands::note::run(&session, text.as_deref(), clear),
    }
}

pub fn run() -> anyhow::Result<()> {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    output::set_no_color(cli.no_color);

    match cli.command {
        None => commands::connect::run(None, None, None, false, None, false),
        Some(Commands::Connect {
            session,
            cwd,
            name,
            no_attach,
            cmd,
            force,
        }) => commands::connect::run(
            session.as_deref(),
            cwd.as_deref(),
            name.as_deref(),
            no_attach,
            cmd.as_deref(),
            force,
        ),
        Some(Commands::List { json, tags }) => commands::list::run(json, &tags),
        Some(Commands::Kill { name, force }) => commands::kill::run(&name, force),
        Some(Commands::Pick { no_attach, tags }) => commands::pick::run(no_attach, &tags),
        Some(Commands::Last) => commands::last::run(),
        Some(Commands::Session { action }) => dispatch_session(action),
        Some(Commands::Tag { action }) => commands::tag::run(action),
        Some(Commands::Current) => commands::current::run(),
        Some(Commands::Status) => commands::status::run(),
        Some(Commands::Doctor) => commands::doctor::run(),
        Some(Commands::Gc) => commands::gc::run(),
        Some(Commands::Init {
            name,
            startup,
            tags,
            no_create,
            no_attach,
            force,
        }) => commands::init::run(
            name.as_deref(),
            startup.as_deref(),
            &tags,
            no_create,
            no_attach,
            force,
        ),
        Some(Commands::Config { action }) => commands::config::run(action),
        Some(Commands::Completion { shell }) => commands::completion::run(shell),
        Some(Commands::Version { verbose }) => commands::version::run(verbose),

        // Hidden back-compat aliases.
        Some(Commands::Attach { session }) => commands::attach::run(&session),
        Some(Commands::Rename { from, to }) => commands::rename::run(&from, &to),
        Some(Commands::Note {
            session,
            text,
            clear,
        }) => commands::note::run(&session, text.as_deref(), clear),
        Some(Commands::New {
            path,
            name,
            cmd,
            no_attach,
            force,
        }) => commands::new::run(&path, name.as_deref(), cmd.as_deref(), no_attach, force),
        Some(Commands::Export { path }) => commands::export::run(path.as_deref()),
        Some(Commands::Import { path, merge }) => commands::import::run(&path, merge),
    }
}
