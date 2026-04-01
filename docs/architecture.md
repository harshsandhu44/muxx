# Architecture

muxx is a single-binary Rust CLI. There is no library crate — all code lives under `src/` and is compiled into one executable.

## Layer overview

```
main.rs         — entry point, error handling
cli.rs          — argument parsing, command dispatch
commands/       — one file per subcommand
core/           — reusable utilities (config, env, tmux, etc.)
```

## Entry point

`src/main.rs` declares three modules (`cli`, `commands`, `core`) and calls `cli::run()`. On error it prints to stderr and exits with code 1.

## CLI layer (`src/cli.rs`)

Uses [clap](https://docs.rs/clap) with derive macros. Defines the `Cli` struct and `Commands` enum (one variant per subcommand). `run()` parses args and dispatches to the appropriate command module. When no subcommand is given, it defaults to `connect` with no arguments (current directory).

## Command modules (`src/commands/`)

Each file exposes a single `run()` function:

| File | What it does |
|---|---|
| `connect.rs` | Creates or reattaches to a session; resolves alias → directory → session name |
| `list.rs` | Lists sessions as a table or `--json` |
| `kill.rs` | Kills a session; guards against killing the current one without `--force` |
| `current.rs` | Prints the current session name; errors if not in tmux |
| `completion.rs` | Emits a shell completion script via `clap_complete` |

## Core layer (`src/core/`)

Utilities shared across command modules:

| File | Responsibility |
|---|---|
| `config.rs` | Loads `~/.config/muxx/config.json`; resolves project aliases |
| `env.rs` | `is_inside_tmux()`, home expansion, directory resolution |
| `tmux.rs` | Wraps tmux CLI calls; `run()` captures output, `run_interactive()` inherits stdio for attach/switch |
| `session_name.rs` | Sanitizes arbitrary strings into valid tmux session names |
| `output.rs` | ANSI-colored print helpers (`success`, `info`, `error`, `hint`) |

## Pure vs shell-dependent

**Unit-testable without tmux** (`cargo test` only):
- `core/config.rs` — pure JSON parsing and struct logic
- `core/env.rs` — path expansion (mocked via `tempfile` in tests)
- `core/session_name.rs` — string transformation

**Requires tmux** (integration tests in `tests/`):
- `core/tmux.rs` — spawns tmux subprocesses
- All command modules (they call `core::tmux` functions)

Integration tests use `assert_cmd` and create/clean up real tmux sessions. CI installs tmux before running `cargo test`.

## Adding a new command

1. Add a variant to `Commands` in `src/cli.rs`
2. Create `src/commands/<name>.rs` with a `pub fn run(...) -> anyhow::Result<()>`
3. Add the module to `src/commands/mod.rs`
4. Dispatch from `cli::run()` in the match block
