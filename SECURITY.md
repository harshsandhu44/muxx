# Security Policy

## Supported versions

Only the latest released version of muxx receives security fixes.

| Version | Supported |
|---------|-----------|
| Latest  | Yes       |
| Older   | No        |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security issue, report it privately via GitHub's [security advisories](https://github.com/harshsandhu44/muxx/security/advisories/new) feature. This lets us assess and fix the issue before it becomes public.

Include in your report:
- A description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested fix, if you have one

You can expect an acknowledgment within 72 hours and a fix or resolution plan within 14 days, depending on severity.

## Scope

muxx is a local CLI tool that wraps tmux. It does not:

- Make network requests
- Store or transmit user data
- Run as a daemon or service
- Require elevated privileges

Most relevant attack surfaces involve shell command injection through config values or session names. If you find a path where untrusted input reaches a shell or subprocess unsanitized, that's worth reporting.
