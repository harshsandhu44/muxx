# muxx

Minimal tmux session automation CLI.

## Goal

A focused, dependency-light CLI for managing tmux sessions from the terminal. No TUI, no plugins, no telemetry — just a clean interface over `tmux` commands.

## MVP Scope

- List active tmux sessions
- Attach to a session by name or default to the most recent
- Kill a session by name
- Print the currently attached session

## Commands

| Command | Description |
|---|---|
| `muxx list` | List all tmux sessions |
| `muxx connect [target]` | Attach to a session (omit target to use most recent) |
| `muxx kill <name>` | Kill a named session |
| `muxx current` | Print the currently attached session name |

## Requirements

- Node.js 18+
- tmux installed on the system

## Development

```sh
npm install
npm run dev -- list        # run via tsx (no build needed)
npm run build              # compile to dist/
npm run typecheck          # type-check without emitting
npm run clean              # remove dist/
```

## Config

Optional config file at `~/.config/muxx/config.json`. Defines named projects so you can run `muxx connect <name>` without typing the full path.

```json
{
  "projects": {
    "vitaq": { "cwd": "~/Code/vitaq" },
    "muxx": { "cwd": "~/Code/personal/muxx" }
  }
}
```

If the target matches a project key, its `cwd` is used. Otherwise the target is treated as a directory path.

## Installation (from source)

```sh
npm run build
npm link
```
