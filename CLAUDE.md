# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build                        # debug build
cargo build --release              # release build
cargo fmt                          # format code
cargo clippy -- -D warnings        # lint (warnings are errors)
cargo test                         # run all tests (requires tmux installed)
cargo test <test_name>             # run a single test
cargo run -- <subcommand>          # run the CLI
```

Integration tests require tmux. On macOS: `brew install tmux`.

## Architecture

muxx is a tmux session automation CLI. Single binary crate, no library crates.

**Layer flow:** `main.rs` → `cli.rs` → `commands/` → `core/`

- **`cli.rs`** — clap derive macros define `Cli` struct and `Commands` enum; dispatches to command handlers
- **`commands/`** — one file per subcommand (`connect`, `list`, `kill`, `current`, `completion`), each exports `pub fn run() -> Result<()>`
- **`core/`** — shared utilities:
  - `tmux.rs` — subprocess wrapper; `run()` captures stdout, `run_interactive()` inherits stdio (needed for attach/switch); public functions like `create_session()`, `attach_session()`, `switch_client()`
  - `config.rs` — `MuxxConfig` loaded from `~/.config/muxx/config.json` (or `MUXX_CONFIG_PATH` env var); `HashMap<String, ProjectConfig>` with optional `startup` command
  - `env.rs` — `is_inside_tmux()`, `expand_home()`, `resolve_dir()`
  - `session_name.rs` — sanitizes path-derived names (basename → lowercase, spaces/invalid chars → hyphens); respects `--name` override
  - `output.rs` — ANSI color helpers (`success`, `info`, `error`, `hint`)

**Testing:** unit tests live in-source on pure functions; integration tests in `tests/` use `assert_cmd` + `--no-attach` flag to avoid terminal hijacking.

**Releases:** automated via release-plz on push to main; uses Conventional Commits. semver_check is disabled (binary crate, no public API).

See `docs/architecture.md` for a detailed design doc including how to add a new command.
