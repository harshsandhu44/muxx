# muxx

[![crates.io](https://img.shields.io/crates/v/muxx.svg)](https://crates.io/crates/muxx)
[![CI](https://github.com/harshsandhu44/muxx/actions/workflows/release.yml/badge.svg)](https://github.com/harshsandhu44/muxx/actions/workflows/release.yml)

Minimal tmux session automation CLI.

## What it is

A focused CLI for managing tmux sessions from the terminal. No TUI, no plugins, no telemetry — just a clean interface over `tmux` commands.

## Design principles

- **Minimal by default.** muxx does one thing: manage tmux sessions.
- **No surprises.** Every action maps directly to a tmux operation.
- **Config is optional.** Works without any config file; config only adds named aliases.
- **Composable.** Plain text and JSON output so it fits into scripts and other tools.

## Non-goals

muxx does not aim to provide:

- A TUI or interactive session picker
- Pane and window orchestration
- zoxide or directory-jumping integration
- A plugin or extension system
- Telemetry or usage analytics
- A feature-complete replacement for sesh or tmuxinator

## Requirements

- Rust stable (for building from source)
- tmux installed on the system

## Installation

```sh
cargo install muxx
```

### Distribution

`cargo install` is the only first-class install method right now. Pre-built binaries are attached to each [GitHub release](https://github.com/harshsandhu44/muxx/releases) but there is no install script or Homebrew tap yet. A Homebrew tap is planned for a future release.

## Commands

| Command | Alias | Description |
|---|---|---|
| `muxx` | | Connect to a session in the current directory |
| `muxx connect [dir] [--name <n>] [--no-attach] [--cmd "<cmd>"]` | `c` | Connect to or create a tmux session by directory |
| `muxx attach <name>` | `a` | Attach or switch to an existing session by name |
| `muxx list [--json]` | `ls` | List all tmux sessions |
| `muxx kill <name> [--force]` | `k` | Kill a session by name |
| `muxx current` | `cur` | Print the current session name |
| `muxx completion <bash\|zsh\|fish>` | | Print shell completion script |

## Examples

```sh
# Connect to session in current directory (creates if it doesn't exist)
muxx

# Connect to a specific directory
muxx connect ~/Code/myapp

# Connect using a config alias
muxx connect myapp

# Create a session without attaching
muxx connect --no-attach ~/Code/myapp

# Override the session name
muxx connect --name work ~/Code/myapp

# Run a command when the session is first created
muxx connect --cmd "npm run dev" ~/Code/myapp

# Attach to an existing session by name (never creates a session)
muxx attach work
muxx a work               # alias

# List sessions
muxx list
muxx list --json

# Kill a session
muxx kill myapp

# Print current session name
muxx current
```

### `connect` vs `attach`

| | `connect` | `attach` |
|---|---|---|
| Input | directory path or config alias | tmux session name |
| Creates session? | yes (if not exists) | no |
| Runs startup command? | yes (if configured) | no |
| Use when | starting work in a project | returning to a named session |

## Config

Optional config file at `~/.config/muxx/config.json`. Defines named project aliases so you can run `muxx connect <name>` without typing the full path.

```json
{
  "projects": {
    "myapp": { "cwd": "~/Code/myapp" },
    "api": { "cwd": "~/Code/api", "startup": "cargo run" }
  }
}
```

See [`examples/config.json`](examples/config.json) for a fuller example.

- If the target matches a project key, its `cwd` is used as the session directory.
- `startup` is a shell command sent to the session's first pane on first creation only. Re-connecting to an existing session will not re-run it.
- `--cmd` on the command line takes precedence over `startup` in the config.

## Shell Completions

muxx generates completion scripts via `clap_complete`.

### bash

Add to `~/.bashrc`:

```sh
eval "$(muxx completion bash)"
```

### zsh

Add to `~/.zshrc` (after `compinit`):

```sh
eval "$(muxx completion zsh)"
```

Or write to a file in your `$fpath` for faster startup (run once):

```sh
muxx completion zsh > "${fpath[1]}/_muxx"
```

### fish

Run once to install:

```sh
muxx completion fish > ~/.config/fish/completions/muxx.fish
```

## Shell integration

muxx is designed to fit naturally into shell workflows. All output is plain text or `--json`, so it composes with standard tools.

The [`examples/`](examples/) directory has ready-to-use snippets:

- [`zsh-integration.zsh`](examples/zsh-integration.zsh) — a short `mx` wrapper function and an optional `mxp` interactive picker (requires `fzf`)
- [`tmux-status.conf`](examples/tmux-status.conf) — how to show the current session name in the tmux status bar

## Troubleshooting

**`tmux: command not found`**
muxx requires tmux to be installed and available in `PATH`. Install it via your system package manager (e.g. `brew install tmux`, `apt install tmux`).

**Shell completions not working**
Make sure the `eval "$(muxx completion <shell>)"` line is in your shell rc file and that your shell was restarted (or the file was sourced). For zsh, the eval must come after `compinit`.

**Config parse error on startup**
muxx validates `~/.config/muxx/config.json` on load. Check for trailing commas, unquoted strings, or invalid JSON. Run `cat ~/.config/muxx/config.json | python3 -m json.tool` to validate.

**Behavior differs inside vs outside tmux**
When run inside an existing tmux session, muxx uses `switch-client` to move to the target session. When run outside tmux, it uses `attach-session`. Use `--no-attach` to create a session without switching to it at all.

**Startup command did not run / ran unexpectedly**
`startup` in config and `--cmd` on the CLI are only sent to the session's first pane on *new session creation*. Re-connecting to an existing session will not re-run it. To force a re-run, kill the session first (`muxx kill <name>`) and reconnect.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, workflow, and commit conventions. For a quick orientation to the codebase, see [docs/architecture.md](docs/architecture.md).

## Development

```sh
# Build and run
cargo run -- list
cargo run -- --help

# Run tests
cargo test

# Lint and format
cargo fmt
cargo clippy -- -D warnings

# Build release binary
cargo build --release
./target/release/muxx --help
```

## Releases

Releases are fully automated via [release-plz](https://release-plz.dev) on every push to `main`.

### How it works

1. Every push to `main` runs format check, clippy, tests, and a release build.
2. If all pass, release-plz analyzes commits since the last tag.
3. If releasable commits exist, it:
   - Opens a release PR that bumps the version in `Cargo.toml` and generates a changelog
   - When that PR is merged, publishes the crate to [crates.io](https://crates.io) and creates a GitHub release

### Commit convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

| Commit type | Release type |
|---|---|
| `fix:` | patch (e.g. `1.2.2` → `1.2.3`) |
| `feat:` | minor (e.g. `1.2.2` → `1.3.0`) |
| `BREAKING CHANGE:` in footer | major (e.g. `1.2.2` → `2.0.0`) |

Types like `chore:`, `docs:`, `test:`, `refactor:` do not trigger a release.

### Required secrets

| Secret | Description |
|---|---|
| `CARGO_REGISTRY_TOKEN` | crates.io API token — create at crates.io → Account Settings → API Tokens |
| `GITHUB_TOKEN` | Automatically provided by GitHub Actions — no setup needed |
