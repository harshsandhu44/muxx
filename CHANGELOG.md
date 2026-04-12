# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.9.1](https://github.com/harshsandhu44/muxx/compare/v1.9.0...v1.9.1) - 2026-04-12

### Other

- add integration tests for v1.9.0 commands
- add planned roadmap section

## [1.9.0](https://github.com/harshsandhu44/muxx/compare/v1.8.3...v1.9.0) - 2026-04-11

### Added

- *(commands)* add version, last, new, config, export, import

### Fixed

- *(import)* formatting error in code

### Other

- document version, last, new, config, export, import commands

## [1.8.3](https://github.com/harshsandhu44/muxx/compare/v1.8.2...v1.8.3) - 2026-04-11

### Other

- *(commands)* replace process::exit with bail!, use gc() APIs, write! in status
- *(core)* gc() returns removed names, HashSet lookups, TagsStore::gc(), let-else in tmux
- *(output)* suppress ANSI codes when stdout/stderr is not a TTY

## [1.8.2](https://github.com/harshsandhu44/muxx/compare/v1.8.1...v1.8.2) - 2026-04-11

### Other

- *(deps)* bump clap_complete from 4.6.0 to 4.6.1 ([#32](https://github.com/harshsandhu44/muxx/pull/32))
- add open source contribution infrastructure

## [1.8.1](https://github.com/harshsandhu44/muxx/compare/v1.8.0...v1.8.1) - 2026-04-10

### Fixed

- update toml version

## [1.8.0](https://github.com/harshsandhu44/muxx/compare/v1.7.0...v1.8.0) - 2026-04-10

### Added

- add status command for shell prompt integration
- add gc command to clean up orphaned tags and notes
- add note command for session annotations
- add session notes store
- *(tag)* add delete subcommand to remove a tag from all sessions

### Fixed

- use ~/.config path and migrate stores from JSON to TOML

### Other

- update for note, gc, status commands and TOML config format
- document tag delete subcommand

## [1.7.0](https://github.com/harshsandhu44/muxx/compare/v1.6.0...v1.7.0) - 2026-04-10

### Added

- add session tags with fzf-powered interactive editing ([#26](https://github.com/harshsandhu44/muxx/pull/26))

## [1.6.0](https://github.com/harshsandhu44/muxx/compare/v1.5.1...v1.6.0) - 2026-04-07

### Added

- add doctor subcommand for environment and config validation ([#25](https://github.com/harshsandhu44/muxx/pull/25))
- add pick subcommand for interactive session selection via fzf ([#24](https://github.com/harshsandhu44/muxx/pull/24))
- richer muxx list output with table layout and session metadata ([#23](https://github.com/harshsandhu44/muxx/pull/23))
- add rename subcommand ([#21](https://github.com/harshsandhu44/muxx/pull/21))

## [1.5.1](https://github.com/harshsandhu44/muxx/compare/v1.5.0...v1.5.1) - 2026-04-06

### Other

- add planned section
- expand examples and fix completion commands in README
- update dirs package
- add extensive unit and integration tests across all modules
- enhance tests in the code
- overhaul project documentation for open source contributors

## [1.5.0](https://github.com/harshsandhu44/muxx/compare/v1.4.0...v1.5.0) - 2026-04-04

### Added

- *(completion)* dynamic session name completion via clap_complete ArgValueCompleter

### Other

- update connect command usage for session-name and --cwd flag

## [1.4.0](https://github.com/harshsandhu44/muxx/compare/v1.3.0...v1.4.0) - 2026-04-04

### Added

- *(connect)* attach to session by name, use --cwd for dir-based creation

### Fixed

- *(config)* eliminate env var mutation in tests to fix parallel test race

### Other

- apply rustfmt to connect test
- *(connect)* update nonexistent-dir test to use --cwd flag
- *(connect)* replace dir positional with session name and --cwd flag

## [1.3.0](https://github.com/harshsandhu44/muxx/compare/v1.2.3...v1.3.0) - 2026-04-01

### Added

- add attach command with fuzzy matching and last-session shorthand ([#5](https://github.com/harshsandhu44/muxx/pull/5))

### Other

- add CLAUDE.md, architecture doc, examples, and README improvements ([#4](https://github.com/harshsandhu44/muxx/pull/4))
- explain why semver_check is disabled in release-plz
- add badges, design principles, non-goals, troubleshooting, contributing
- add CODE_OF_CONDUCT, CONTRIBUTING, and GitHub templates

## [1.2.3](https://github.com/harshsandhu44/muxx/compare/v1.2.2...v1.2.3) - 2026-04-01

### Fixed

- add DirPath value hint to connect dir arg for shell completion
