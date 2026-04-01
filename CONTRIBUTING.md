# Contributing to muxx

muxx is a small, focused project. Contributions are welcome when they fit within that scope.

## Setup

Requires Rust stable and tmux.

```sh
git clone https://github.com/harshsandhu44/muxx
cd muxx
cargo build
```

## Dev workflow

```sh
cargo fmt                        # format code
cargo clippy -- -D warnings      # lint (CI enforces zero warnings)
cargo test                       # run all tests
cargo build --release            # build release binary
```

Tests require tmux to be installed and available in PATH.

## Making changes

- Keep PRs focused: one concern per PR
- Add or update tests for any behavior changes
- Run `cargo fmt` and `cargo clippy -- -D warnings` before pushing — CI will reject failures
- Keep the code style consistent with what's already there

## Commit convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/) because release-plz uses commit messages to determine version bumps and generate the changelog.

| Type | When to use |
|------|-------------|
| `fix:` | bug fixes (triggers patch release) |
| `feat:` | new behavior (triggers minor release) |
| `docs:` | documentation only |
| `chore:` | maintenance, tooling, deps |
| `test:` | adding or improving tests |
| `refactor:` | code restructuring without behavior change |

`BREAKING CHANGE:` in the commit footer triggers a major release.

`chore:`, `docs:`, `test:`, and `refactor:` do not trigger a release.

## What's welcome

- Bug fixes
- Improved error messages or user-facing output
- Shell completion improvements
- Config parsing improvements
- Test coverage gaps
- Documentation fixes

## What's likely out of scope

muxx is intentionally minimal. The following types of changes are generally not a good fit:

- TUI or interactive session picker
- zoxide or directory-jumping integration
- Pane and window orchestration
- Plugin or extension system
- Telemetry or analytics
- New subcommands that duplicate existing tmux functionality

If you're unsure whether an idea fits, open an issue before writing code.
