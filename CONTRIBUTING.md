# Contributing to muxx

muxx is a small, focused project. Contributions that fit within its scope are welcome.

If you're unsure whether your idea fits, open an issue before writing code — it saves everyone time.

## Contents

- [Getting started](#getting-started)
- [Development workflow](#development-workflow)
- [Project structure](#project-structure)
- [Making changes](#making-changes)
- [Commit convention](#commit-convention)
- [Opening a pull request](#opening-a-pull-request)
- [What's in scope](#whats-in-scope)
- [Good first issues](#good-first-issues)

---

## Getting started

**Requirements:** Rust stable, tmux.

```sh
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install tmux (macOS)
brew install tmux

# Clone and build
git clone https://github.com/harshsandhu44/muxx
cd muxx
cargo build
```

Verify everything works:

```sh
cargo test
cargo clippy -- -D warnings
```

---

## Development workflow

```sh
cargo fmt                        # format code (required before pushing)
cargo clippy -- -D warnings      # lint (CI enforces zero warnings)
cargo test                       # run all tests (requires tmux in PATH)
cargo build --release            # build release binary
```

Run a specific test by name:

```sh
cargo test test_session_name     # unit test
cargo test connect_creates       # integration test
```

Run the CLI from source:

```sh
cargo run -- list
cargo run -- connect --help
```

---

## Project structure

A quick orientation — see [docs/architecture.md](docs/architecture.md) for the full design doc.

```
src/
  main.rs           entry point
  cli.rs            argument parsing (clap), command dispatch
  commands/         one file per subcommand
  core/             shared utilities (config, tmux, env, etc.)
tests/              integration tests (use assert_cmd, require tmux)
examples/           shell integration snippets and sample config
docs/               design documentation
```

---

## Making changes

- **Keep PRs focused.** One concern per PR. If you want to fix two unrelated things, open two PRs.
- **Add or update tests** for any behavior changes.
- **Run `cargo fmt` and `cargo clippy -- -D warnings`** before pushing — CI will reject failures.
- **Update docs** if you change a command's behavior, flags, or config format.
- **Match the existing style** — prefer clarity over cleverness.

### Adding a new command

1. Add a variant to `Commands` in `src/cli.rs`
2. Create `src/commands/<name>.rs` with `pub fn run(...) -> anyhow::Result<()>`
3. Add the module to `src/commands/mod.rs`
4. Dispatch from the match block in `cli::run()`
5. Add integration tests in `tests/`

See [docs/architecture.md](docs/architecture.md#adding-a-new-command) for the full walkthrough.

---

## Commit convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/) because release-plz uses commit messages to determine version bumps and generate the changelog automatically.

| Type | When to use | Release effect |
|------|-------------|----------------|
| `fix:` | bug fixes | patch (`1.2.2` → `1.2.3`) |
| `feat:` | new behavior visible to users | minor (`1.2.2` → `1.3.0`) |
| `docs:` | documentation only | none |
| `chore:` | maintenance, tooling, deps | none |
| `test:` | adding or improving tests | none |
| `refactor:` | restructuring without behavior change | none |

`BREAKING CHANGE:` in the commit footer triggers a major release (`1.2.2` → `2.0.0`).

**Examples:**

```
fix: handle missing tmux server gracefully
feat: add --json flag to current command
docs: clarify startup command behavior in README
refactor(session_name): extract sanitize logic into helper
```

---

## Opening a pull request

1. Fork the repo and create a branch from `main`.
2. Make your changes with tests and docs.
3. Run the full check suite locally:
   ```sh
   cargo fmt && cargo clippy -- -D warnings && cargo test
   ```
4. Open a PR against `main` using the pull request template.
5. A maintainer will review and leave feedback or merge.

**What to expect:**
- Feedback within a few days for small changes.
- Larger or design-heavy changes may take longer and require discussion.
- CI must pass before merging.

---

## What's in scope

- Bug fixes
- Improved error messages or user-facing output
- Shell completion improvements
- Config parsing improvements and better error output
- Test coverage gaps
- Documentation fixes and examples

## What's out of scope

muxx is intentionally minimal. These types of changes are generally not a good fit:

- TUI or interactive session picker
- zoxide or directory-jumping integration
- Pane and window layout orchestration
- Plugin or extension system
- Telemetry or analytics
- New subcommands that duplicate existing tmux functionality

---

## Good first issues

Issues labeled [`good first issue`](https://github.com/harshsandhu44/muxx/labels/good%20first%20issue) are a good starting point if you're new to the codebase. They're scoped to a single file or function and don't require deep knowledge of the overall design.

If nothing labeled appeals to you, look at [`help wanted`](https://github.com/harshsandhu44/muxx/labels/help%20wanted) for higher-impact but still well-defined tasks.
