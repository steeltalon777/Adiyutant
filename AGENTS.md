# AGENTS

Rules for AI coding agents working in this repository.

## Repository Status

This repository is in **bootstrap state**. No application code exists.

## Primary Priority

Complete the current task without exceeding its scope. Do not add unrequested features.

## Allowed Changes

- Documentation files (markdown)
- Directory structure
- `.gitkeep` files in empty directories
- `.gitignore`
- `docs/adr/` files when explicitly requested

## Forbidden Changes

- Do not invent implemented architecture.
- Do not add source code during TASK-0000.
- Do not add backend during TASK-0000.
- Do not add LLM provider calls.
- Do not store API keys in client projects.
- Do not duplicate future Core domain logic in UI projects.
- If a document conflicts with an ADR, ADR wins.
- If code conflicts with docs in future, document the drift.

## Documentation Rules

- Use `Current`, `Planned`, `Unknown` status markers where appropriate.
- Do not claim features as implemented if they are not in code.
- Do not use marketing language ("production-ready", "enterprise-grade", "scalable") unless confirmed by code.

## Core Boundary Rules

- Core stores meaning. Platforms execute OS-specific behavior.
- Core must not call LLM providers directly.
- Mobile clients must not store LLM provider API keys.

## Security Rules

- Never commit secrets, API keys, or tokens.
- Never log sensitive data.
- Never hardcode credentials.

## Verification

After any change to Core code (when it exists), run:
```bash
cargo fmt
cargo check
cargo test
```
