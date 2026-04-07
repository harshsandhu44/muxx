# muxx

[![crates.io](https://img.shields.io/crates/v/muxx.svg)](https://crates.io/crates/muxx)
[![Downloads](https://img.shields.io/crates/d/muxx.svg)](https://crates.io/crates/muxx)
[![CI](https://github.com/harshsandhu44/muxx/actions/workflows/release.yml/badge.svg)](https://github.com/harshsandhu44/muxx/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Minimal tmux session automation CLI.**

Jump into any project in one command. muxx creates a tmux session from a directory (or a named alias in your config), attaches to it if it's already running, and gets out of your way.

```sh
# Go to a project — creates the session if it doesn't exist, attaches if it does
muxx connect ~/Code/myapp

# Or just run `muxx` from inside the project directory
cd ~/Code/myapp && muxx
```

---

## Contents

- [Why muxx](#why-muxx)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Commands](#commands)
- [Config](#config)
- [Shell completions](#shell-completions)
- [Shell integration](#shell-integration)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)

---

## Why muxx

Most tmux session managers grow into full workspace orchestrators — TUIs, pane layouts, plugins, zoxide hooks. muxx doesn't. It does one thing: get you into a session, fast, from anywhere in your shell.

**Design principles:**

- **Minimal by default.** muxx manages sessions. Not windows, not panes, not your entire workspace.
- **No surprises.** Every action maps directly to a tmux operation you could run yourself.
- **Config is optional.** Works out of the box with no config file; config only adds named project aliases.
- **Composable.** Plain text and `--json` output so it fits into scripts and shell functions.

**Not trying to be:**

- A full TUI with pane/window layouts
- A replacement for [sesh](https://github.com/joshmedeski/sesh) or [tmuxinator](https://github.com/tmuxinator/tmuxinator)
- A pane/window orchestration tool

**Planned:**

- [zoxide](https://github.com/ajeetdsouza/zoxide) integration — jump to frecency-ranked directories directly via `muxx`

---

## Installation

### From crates.io (recommended)

```sh
cargo install muxx
```

Requires Rust stable. On macOS, get Rust via [rustup](https://rustup.rs).

### Pre-built binaries

Pre-built binaries for Linux and macOS are attached to each [GitHub release](https://github.com/harshsandhu44/muxx/releases). Download, make executable, and place in your `PATH`.

```sh
# Example for macOS arm64
curl -L https://github.com/harshsandhu44/muxx/releases/latest/download/muxx-aarch64-apple-darwin \
  -o /usr/local/bin/muxx && chmod +x /usr/local/bin/muxx
```

> A Homebrew tap is planned for a future release.

### From source

```sh
git clone https://github.com/harshsandhu44/muxx
cd muxx
cargo install --path .
```

### Verify

```sh
muxx --version
```

---

## Quick start

```sh
# Connect to the current directory (create session if needed, attach if exists)
cd ~/Code/myapp
muxx

# Connect to any directory
muxx connect -c ~/Code/myapp

# Give the session a custom name
muxx connect -c ~/Code/myapp --name work

# Run a command when the session is first created
muxx connect -c ~/Code/myapp --cmd "npm run dev"

# Interactively pick a session with fzf (requires fzf in PATH)
muxx pick

# List all sessions (table view with windows, panes, last seen, CWD, startup)
muxx list

# Rename a session
muxx rename myapp work

# Kill a session
muxx kill myapp

# Print the current session name (useful in scripts)
muxx current
```

---

## Commands

| Command                                                                        | Alias | Description                                                     |
| ------------------------------------------------------------------------------ | ----- | --------------------------------------------------------------- |
| `muxx`                                                                         |       | Connect to a session in the current directory                   |
| `muxx connect [session] [-c <dir>] [--name <n>] [--no-attach] [--cmd "<cmd>"]` | `c`   | Attach to an existing session or create one from a directory    |
| `muxx attach <name>`                                                           | `a`   | Attach or switch to an existing session by name (never creates) |
| `muxx pick`                                                                    | `p`   | Interactively pick a session to attach to using fzf             |
| `muxx list [--json]`                                                           | `ls`  | List sessions with windows, panes, last seen, CWD, and startup  |
| `muxx kill <name> [--force]`                                                   | `k`   | Kill a session by name                                          |
| `muxx rename <from> <to>`                                                      | `rn`  | Rename an existing session                                      |
| `muxx current`                                                                 | `cur` | Print the current session name                                  |
| `muxx doctor`                                                                  | `doc` | Validate environment and config; report any issues              |
| `muxx completion <bash\|zsh\|fish>`                                            |       | Print shell completion script                                   |

### `connect` vs `attach`

|                       | `connect <name>`             | `connect -c <dir>`           | `attach <name>`              |
| --------------------- | ---------------------------- | ---------------------------- | ---------------------------- |
| Input                 | session name or config alias | directory path               | tmux session name            |
| Creates session?      | only if config alias         | yes (if not exists)          | **no**                       |
| Runs startup command? | yes (if config alias)        | yes (if configured)          | no                           |
| Use when              | switching to a known alias   | starting work in a directory | returning to a named session |

### Full examples

```sh
# Connect to a config alias (resolves to its configured cwd + startup command)
muxx connect myapp
muxx c myapp              # alias

# Attach to a running session by name (errors if it doesn't exist)
muxx attach work
muxx a work               # alias

# Pick a session interactively with fzf
muxx pick
muxx p                    # alias

# Create a session without attaching (useful in scripts)
muxx connect -c ~/Code/myapp --no-attach

# List sessions as JSON (includes name, windows, attached, created, last_attached)
muxx list --json

# Force-kill the current session
muxx kill mysession --force

# Rename a session
muxx rename old-name new-name
muxx rn old-name new-name   # alias
```

---

## Config

Optional. Place at `~/.config/muxx/config.json`.

Defines named project aliases so you can run `muxx connect <name>` without typing the full path.

```json
{
  "projects": {
    "myapp": { "cwd": "~/Code/myapp" },
    "api": { "cwd": "~/Code/api", "startup": "cargo run" },
    "frontend": { "cwd": "~/Code/frontend", "startup": "npm run dev" }
  }
}
```

See [`examples/config.json`](examples/config.json) for a fuller example.

**How it works:**

- If the session name matches a project key, its `cwd` is used as the session directory.
- `startup` is a shell command sent to the session's first pane on **first creation only**. Re-connecting to an existing session will not re-run it.
- `--cmd` on the command line takes precedence over `startup` in the config.
- The config path can be overridden with the `MUXX_CONFIG_PATH` environment variable.

---

## Shell completions

muxx supports dynamic completions — session names are completed live from the running tmux server.

### zsh

Add to `~/.zshrc` (after `compinit`):

```sh
eval "$(muxx completion zsh)"
```

### bash

Add to `~/.bashrc`:

```sh
eval "$(muxx completion bash)"
```

### fish

Run once to install:

```sh
muxx completion fish > ~/.config/fish/completions/muxx.fish
```

---

## Shell integration

The [`examples/`](examples/) directory has ready-to-use shell snippets:

- [`zsh-integration.zsh`](examples/zsh-integration.zsh) — `mx` (connect), `mxp` (fzf session picker), `mxk` (fzf session killer), completion setup
- [`bash-integration.bash`](examples/bash-integration.bash) — same helpers for bash
- [`fish-integration.fish`](examples/fish-integration.fish) — fish-native `mx` and `mxk` helpers
- [`tmux-status.conf`](examples/tmux-status.conf) — show the session name in the status bar; keybinds for in-tmux session switching

All muxx output is plain text or `--json`, so it composes naturally with `fzf`, `jq`, and other tools.

---

## Troubleshooting

Run `muxx doctor` first — it checks tmux availability, config validity, project directories, and session name collisions in one pass.

**`tmux: command not found`**
muxx requires tmux in `PATH`. Install via your package manager (`brew install tmux`, `apt install tmux`, etc.).

**Shell completions not working**
Make sure the `source` / `eval` line is in your shell rc file and your shell was restarted (or the file was sourced). For zsh, the line must come after `compinit`.

**Config parse error on startup**
muxx validates `~/.config/muxx/config.json` on load. Check for trailing commas or invalid JSON. Run `muxx doctor` to see the exact parse error.

**Behavior differs inside vs outside tmux**
Inside an existing session, muxx uses `switch-client`. Outside tmux, it uses `attach-session`. Use `--no-attach` to create a session without switching.

**Startup command did not run / ran unexpectedly**
`startup` and `--cmd` only fire on **new session creation**. To force a re-run, kill the session first (`muxx kill <name>`) and reconnect.

---

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, workflow, and commit conventions.

For a codebase overview, see [docs/architecture.md](docs/architecture.md).

To report a security issue, see [SECURITY.md](SECURITY.md).

---

## License

[MIT](LICENSE)
