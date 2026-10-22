# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| latest release | ✅ |
| older releases | ❌ (upgrade) |

## Reporting a vulnerability

**Do not open a public issue for security reports.**

Use GitHub's private vulnerability reporting on this repository
(Security → Report a vulnerability), or email contact@codecora.dev with
`[dibibest security]` in the subject.

Please include: affected version/commit, reproduction steps, impact assessment.
You can expect a first response within 7 days.

## Scope notes

- dibibest parses **local text input**; it performs no network I/O in `dibibest-core`.
- The MCP server reads stdio only. Treat schema text from untrusted sources as
  hostile input — parsers are the attack surface (fuzzing tracked in M4).
