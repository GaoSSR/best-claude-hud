# Security Policy

## Supported Versions

Security fixes are handled on the latest released version of `best-claude-hud`.

## Reporting a Vulnerability

Please report security issues privately through GitHub security advisories if available. If GitHub advisories are unavailable, contact the repository owner directly and avoid opening a public issue with exploit details.

## Scope

Relevant security areas include:

- unsafe handling of Claude Code statusLine JSON input
- writing files outside the documented configuration directory
- npm packaging or binary resolution issues
- credential handling for optional Claude usage APIs
- patcher behavior that modifies Claude Code `cli.js`

`best-claude-hud` should not print, persist, or transmit secrets from Claude Code settings or credentials.
