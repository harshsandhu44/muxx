# Architecture

muxx is a single-binary Rust CLI. There is no library crate — all code lives under `src/` and is compiled into one executable.

## Layer overview

```
main.rs         — entry point, error handling
cli.rs          — argument parsing, command dispatch
commands/       — one file per subcommand
core/           — reusable utilities (config, env, tmux, etc.)
```

## Data flow

A typical `muxx connect -c ~/Code/myapp` call flows like this:

```
argv
  └─ cli::run()                   parse args via clap
       └─ commands::connect::run()
            ├─ core::config        load ~/.config/muxx/config.json (if present)
            ├─ core::env           expand ~, resolve absolute path
            ├─ core::session_name  derive session name from directory basename
            ├─ core::tmux          check if session exists (tmux ls)
            │    ├─ [new session]  tmux new-session -d -s <name> -c <dir>
            │    │                 optionally: tmux send-keys "<startup cmd>" Enter
            │    └─ [exists]       (skip creation)
            └─ core::tmux          attach-session or switch-client
```

## Entry point

`src/main.rs` declares three modules (`cli`, `commands`, `core`) and calls `cli::run()`. On error it prints to stderr and exits with code 1. Errors propagate via `anyhow::Result`.

## CLI layer (`src/cli.rs`)

Uses [clap](https://docs.rs/clap) with derive macros. Defines the `Cli` struct and `Commands` enum (one variant per subcommand). `run()` parses args and dispatches to the appropriate command module.

When no subcommand is given, it defaults to `connect` with no arguments (current directory behavior).

## Command modules (`src/commands/`)

Each file exposes a single `pub fn run()` function:

| File | What it does |
|---|---|
| `connect.rs` | Creates or reattaches to a session; resolves alias → directory → session name |
| `attach.rs` | Attaches or switches to an existing session; never creates |
| `list.rs` | Lists sessions as a table or `--json` |
| `kill.rs` | Kills a session; guards against killing the current one without `--force` |
| `current.rs` | Prints the current session name; errors if not in tmux |
| `completion.rs` | Emits a shell completion script via `clap_complete` with dynamic session-name values |

## Core layer (`src/core/`)

Utilities shared across command modules:

| File | Responsibility |
|---|---|
| `config.rs` | Loads `~/.config/muxx/config.json`; resolves project aliases to `ProjectConfig` |
| `env.rs` | `is_inside_tmux()`, home expansion, directory resolution |
| `tmux.rs` | Wraps tmux CLI calls; `run()` captures stdout, `run_interactive()` inherits stdio for attach/switch |
| `session_name.rs` | Sanitizes arbitrary strings into valid tmux session names (lowercase, hyphens) |
| `output.rs` | ANSI-colored print helpers (`success`, `info`, `error`, `hint`) |

## Pure vs shell-dependent

**Unit-testable without tmux** — these functions only do string manipulation or JSON parsing and are tested with in-source `#[cfg(test)]` modules:

- `core/config.rs` — JSON parsing and struct logic
- `core/env.rs` — path expansion
- `core/session_name.rs` — string sanitization

**Requires a running tmux server** — these are exercised by integration tests in `tests/`:

- `core/tmux.rs` — spawns real tmux subprocesses
- All command modules (they call `core::tmux` functions)

## Testing strategy

```
tests/
  connect.rs       — integration tests for muxx connect
  list.rs          — integration tests for muxx list
  kill.rs          — integration tests for muxx kill
  current.rs       — integration tests for muxx current
  completion.rs    — smoke tests for completion output
```

Integration tests use [`assert_cmd`](https://docs.rs/assert_cmd) to invoke the compiled binary, and [`tempfile`](https://docs.rs/tempfile) for isolated config directories. They create real tmux sessions and clean them up after each test using a unique, randomly suffixed session name.

The `--no-attach` flag is used in tests wherever attachment would be needed, to avoid terminal hijacking in CI.

CI installs tmux before running `cargo test`. On macOS: `brew install tmux`. On Linux: `apt install tmux`.

## Adding a new command

1. Add a variant to `Commands` in `src/cli.rs`
2. Create `src/commands/<name>.rs` with `pub fn run(...) -> anyhow::Result<()>`
3. Add the module declaration to `src/commands/mod.rs`
4. Dispatch from the match block in `cli::run()`
5. Add integration tests in `tests/<name>.rs`

Follow the pattern of an existing simple command (e.g. `current.rs`) to get the shape right before writing logic.
