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

## Installation

```sh
npm install -g @harshsandhu44/muxx
```

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

## Releases

Releases are fully automated via [semantic-release](https://github.com/semantic-release/semantic-release) on every push to `main`.

### How it works

1. Every push to `main` runs typecheck, tests, and build.
2. If all pass, `semantic-release` analyzes commits since the last release.
3. If releasable commits exist, it bumps the version, publishes to npm, and creates a GitHub release.

### Commit convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/). Only certain commit types trigger a release:

| Commit type | Release type |
|---|---|
| `fix:` | patch (e.g. `0.1.0` → `0.1.1`) |
| `feat:` | minor (e.g. `0.1.0` → `0.2.0`) |
| `BREAKING CHANGE:` in footer | major (e.g. `0.1.0` → `1.0.0`) |

Types like `chore:`, `docs:`, `test:`, `refactor:` do not trigger a release.

### Required GitHub secrets

| Secret | Description |
|---|---|
| `NPM_TOKEN` | npm access token with publish rights (create at npmjs.com → Access Tokens) |
| `GITHUB_TOKEN` | Automatically provided by GitHub Actions — no setup needed |

## Smoke test (pre-publish verification)

```sh
# 1. Build and pack (produces muxx-<version>.tgz)
npm run build
npm pack

# 2. Install the tarball globally
npm install -g ./muxx-*.tgz

# 3. Verify the binary runs
muxx --help
muxx list

# 4. Uninstall when done
npm uninstall -g muxx
```
