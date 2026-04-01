# muxx

Minimal tmux session automation CLI.

## What it is

A focused CLI for managing tmux sessions from the terminal. No TUI, no plugins, no telemetry — just a clean interface over `tmux` commands.

## Requirements

- Rust stable (for building from source)
- tmux installed on the system

## Installation

```sh
cargo install muxx
```

## Commands

| Command | Alias | Description |
|---|---|---|
| `muxx` | | Connect to a session in the current directory |
| `muxx connect [dir] [--name <n>] [--no-attach] [--cmd "<cmd>"]` | `c` | Connect to or create a tmux session |
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

# List sessions
muxx list
muxx list --json

# Kill a session
muxx kill myapp

# Print current session name
muxx current
```

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
