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
- [Tags](#tags)
- [Notes](#notes)
- [Config](#config)
- [Export & Import](#export--import)
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

---

## Planned for April/May 2026 Release

### High impact

- **Zoxide integration** — jump to frecency-ranked directories via `muxx`; resolves partial names through zoxide before falling back to config aliases
- **JSON output for more commands** — `tag ls`, `note`, and `status` lack `--json`; would make muxx fully scriptable without parsing plain text
- **Session collision detection at creation time** — `doctor` detects name collisions retroactively; `connect` should fail early with a clear error instead of silently reusing the wrong session

### Medium impact

- **Bulk kill/tag** — accept multiple session args or a `--all-tagged <tag>` flag so you can kill or tag a set of sessions in one command
- **Tag suggestions on `tag add`** — the existing tag store is loaded but never surfaced as suggestions; fzf over known tags when adding would reduce re-typing
- **Additional `list` filters** — `--unattached`, `--older-than <duration>`, `--windows <n>` for more targeted GC workflows beyond tag filtering
- **Auto-create config directories** — if `~/.config/muxx/` or `~/.local/share/muxx/` are missing, show a clear message or create them rather than surfacing an obscure I/O error
- **`config add-alias` / `config remove-alias`** — add/remove project aliases from the CLI without opening `$EDITOR`; useful for scripts and dotfile bootstrapping

### Low impact / internal

- **Consolidate repeated tmux queries** — some commands call `list_sessions()`, `get_session_paths()`, and `get_panes_per_session()` separately in the same flow; batching reduces subprocess overhead
- **Homebrew tap** — distribute a prebuilt bottle so users don't need Rust installed

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
muxx version
```

---

## Quick start

```sh
# Connect to the current directory (create session if needed, attach if exists)
cd ~/Code/myapp
muxx

# Create a new session from a directory (shorthand for connect --cwd)
muxx new ~/Code/myapp
muxx new ~/Code/myapp --name work --cmd "npm run dev"

# Connect to any directory (long form)
muxx connect -c ~/Code/myapp

# Re-attach to the last used session
muxx last

# Interactively pick a session with fzf (requires fzf in PATH)
muxx pick

# List all sessions (table view with windows, panes, last seen, CWD, startup, tags, notes)
muxx list

# Tag a session interactively — fzf multi-select over all known tags
muxx tag edit myapp

# Add tags explicitly
muxx tag add myapp work python

# Filter the session list or picker by tag
muxx list --tag work
muxx pick --tag work

# Attach a note to a session
muxx note myapp "fixing the auth middleware"

# Print compact status for the current session (name + tags + note)
muxx status

# Clean up stale tags/notes for sessions that no longer exist
muxx gc

# Rename a session
muxx rename myapp work

# Kill a session
muxx kill myapp

# Print the current session name (useful in scripts)
muxx current

# Show or edit the config file
muxx config show
muxx config edit

# Export and restore tags and notes
muxx export > backup.toml
muxx import backup.toml

# Print version
muxx version
muxx version --verbose   # includes OS and arch
```

---

## Commands

| Command                                                                        | Alias | Description                                                               |
| ------------------------------------------------------------------------------ | ----- | ------------------------------------------------------------------------- |
| `muxx`                                                                         |       | Connect to a session in the current directory                             |
| `muxx connect [session] [-c <dir>] [--name <n>] [--no-attach] [--cmd "<cmd>"]` | `c`   | Attach to an existing session or create one from a directory              |
| `muxx new <path> [--name <n>] [--cmd "<cmd>"] [--no-attach]`                   | `n`   | Create a session from a directory path (shorthand for `connect --cwd`)    |
| `muxx attach <name>`                                                           | `a`   | Attach or switch to an existing session by name (never creates)           |
| `muxx last`                                                                    | `l`   | Re-attach to the last used session                                        |
| `muxx pick [--tag <tag>]...`                                                   | `p`   | Interactively pick a session using fzf; tags shown and searchable         |
| `muxx list [--json] [--tag <tag>]...`                                          | `ls`  | List sessions with windows, panes, last seen, CWD, startup, tags, notes   |
| `muxx note <session> [text] [--clear]`                                         |       | Get or set a short note on a session                                      |
| `muxx status`                                                                  |       | Print current session name, tags, and note (for shell prompt integration) |
| `muxx tag <subcommand>`                                                        | `t`   | Add, remove, delete, or list tags on sessions                             |
| `muxx kill <name> [--force]`                                                   | `k`   | Kill a session by name                                                    |
| `muxx rename <from> <to>`                                                      | `rn`  | Rename an existing session (tags and notes are migrated automatically)    |
| `muxx gc`                                                                      |       | Remove tags and notes for sessions that no longer exist in tmux           |
| `muxx current`                                                                 | `cur` | Print the current session name                                            |
| `muxx doctor`                                                                  | `doc` | Validate environment and config; report any issues                        |
| `muxx config <show\|edit\|path>`                                               |       | Inspect or edit the config file                                           |
| `muxx export [path]`                                                           |       | Export tags and notes to a TOML file (stdout if no path given)            |
| `muxx import <path> [--merge]`                                                 |       | Import tags and notes from a TOML file                                    |
| `muxx version [--verbose]`                                                     |       | Print version; `--verbose` adds OS and architecture                       |
| `muxx completion <bash\|zsh\|fish>`                                            |       | Print shell completion script                                             |

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

# Re-attach to the last used session
muxx last
muxx l                    # alias

# Create a new session from a path (shorthand for connect --cwd)
muxx new ~/Code/myapp
muxx n ~/Code/myapp       # alias
muxx new ~/Code/myapp --name work --cmd "npm run dev"

# Create a session without attaching (useful in scripts)
muxx new ~/Code/myapp --no-attach

# Pick a session interactively with fzf
muxx pick
muxx p                    # alias

# Pick only sessions tagged "work"
muxx pick --tag work

# List sessions as JSON (includes name, windows, attached, created, last_attached, tags, note)
muxx list --json

# List only sessions tagged "work"
muxx list --tag work

# Force-kill the current session
muxx kill mysession --force

# Rename a session (tags and notes are migrated automatically)
muxx rename old-name new-name
muxx rn old-name new-name   # alias

# Print version
muxx version
muxx version --verbose    # also prints OS and arch (useful for bug reports)
```

---

## Tags

Sessions can have any number of free-form tags (e.g. `work`, `python`, `personal`). Tags persist by session name — they survive kills and recreations, and are migrated automatically on rename.

Tags are stored in `~/.config/muxx/tags.toml` (overridable via `MUXX_TAGS_PATH`).

### Managing tags

```sh
# Interactive toggle — fzf multi-select over all known tags
# Currently applied tags appear first, marked with *
muxx tag edit myapp
muxx t e myapp              # alias

# Add tags — opens fzf picker when no tags given
muxx tag add myapp work python
muxx tag add myapp          # interactive: pick from known tags not already applied

# Remove tags — opens fzf picker when no tags given
muxx tag rm myapp python
muxx tag rm myapp           # interactive: pick from session's current tags

# Remove all tags from a session
muxx tag clear myapp

# Delete a tag globally — removes it from every session that has it
muxx tag delete work
muxx tag del work           # alias
muxx tag delete             # interactive: fzf picker over all known tags

# List tags
muxx tag ls                 # all tagged sessions
muxx tag ls myapp           # one session
muxx tag list myapp         # alias for ls
```

### Filtering by tag

`--tag` uses AND semantics — a session must have **all** listed tags to appear.

```sh
muxx list --tag work
muxx list --tag work --tag python   # sessions that have BOTH tags
muxx pick --tag work                # fzf picker pre-filtered to "work" sessions
```

### Tags in the fzf picker

`muxx pick` shows tags alongside session names in fzf, so you can fuzzy-search across both:

```
> my-project        python, work
  personal-site     personal
  scratch
```

---

## Notes

Each session can carry a short free-form note — a reminder of what you were working on when you return.

Notes are stored in `~/.config/muxx/notes.toml` (overridable via `MUXX_NOTES_PATH`). They persist across kills and recreations, and are migrated automatically on rename.

```sh
# Set a note
muxx note myapp "fixing the auth middleware"

# Read it back
muxx note myapp

# Clear it
muxx note myapp --clear
```

Notes appear in `muxx list` (NOTE column) and `muxx status`.

### `muxx status` — prompt integration

`muxx status` prints a compact one-liner for the current session:

```
myapp [rust,work] — fixing the auth middleware
```

Tags and note are omitted when not set. Designed for shell prompts — add it to your PS1 or starship config:

```sh
# ~/.zshrc (example)
RPROMPT='$(muxx status 2>/dev/null)'
```

### Garbage collection

Over time, tag and note entries can accumulate for sessions you killed long ago. Run `muxx gc` to clean them up:

```sh
muxx gc
# ✓ removed tags for dead session: old-project
# ✓ removed note for dead session: scratch-2
```

---

## Config

Optional. Place at `~/.config/muxx/config.toml`.

Defines named project aliases so you can run `muxx connect <name>` without typing the full path.

```toml
[projects.myapp]
cwd = "~/Code/myapp"

[projects.api]
cwd = "~/Code/api"
startup = "cargo run"

[projects.frontend]
cwd = "~/Code/frontend"
startup = "npm run dev"
```

See [`examples/config.toml`](examples/config.toml) for a fuller example.

**How it works:**

- If the session name matches a project key, its `cwd` is used as the session directory.
- `startup` is a shell command sent to the session's first pane on **first creation only**. Re-connecting to an existing session will not re-run it.
- `--cmd` on the command line takes precedence over `startup` in the config.
- The config path can be overridden with the `MUXX_CONFIG_PATH` environment variable.

### Managing the config file

```sh
# Print the config file path and its current contents
muxx config show

# Open the config file in $EDITOR (creates it if it doesn't exist)
muxx config edit

# Print just the path (for scripting)
muxx config path
cat "$(muxx config path)"
```

---

## Export & Import

Tags and notes can be backed up and restored as a TOML file. Useful before wiping sessions, migrating machines, or sharing a session setup.

```sh
# Export to stdout (pipe-friendly)
muxx export

# Export to a file
muxx export backup.toml

# Import (replaces all existing tags and notes)
muxx import backup.toml

# Import and merge with existing data (deduplicates tags)
muxx import backup.toml --merge
```

The export format is plain TOML with two top-level tables:

```toml
[tags]
myapp = ["python", "work"]
api = ["rust", "work"]

[notes]
myapp = "fixing the auth middleware"
```

Both tables are optional — an export file with only `[tags]` or only `[notes]` is valid for `muxx import`.

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
muxx validates `~/.config/muxx/config.toml` on load. Check for malformed TOML (e.g. missing quotes, bad key syntax). Run `muxx doctor` to see the exact parse error.

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
