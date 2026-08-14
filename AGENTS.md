# AGENTS

Rules for AI coding agents working in this repository.

## Repository Status

**MVP 0.1 in progress.** Core Rust workspace with 3 crates: `adiyutant_core` (domain models, aggregator, local rules), `adiyutant_store` (SQLite persistence), `adiyutant_cli` (clap CLI with 9 commands). 100 tests pass.

## Primary Priority

Complete the current task without exceeding its scope. Do not add unrequested features.

## Allowed Changes

- Rust source code in `AdiyutantCore/` workspace
- Android Kotlin/Compose source code in `AdiyutantAndroid/`, **scoped to Phase 9** (see `ROADMAP.md`):
  - Gradle project bootstrap and build files
  - Compose theme / Adiyutant UI Kit tokens
  - Navigation shell with 5 tabs and stub screens
  - `CorePort` interface and `FakeCorePort` implementation
  - `CorePort`-based feature code
- Documentation files (markdown)
- Directory structure
- `.gitignore`
- `docs/adr/` files when explicitly requested

Out of scope until a later phase (recorded in `ROADMAP.md`):
- Full screen implementations from the design prototypes (design system still being finalized)
- Extending Core DTOs to match every design element
- Android ↔ Rust Core integration (see ADR-0003; UI works against `FakeCorePort` meanwhile)

## Forbidden Changes

- Do not add LLM provider calls to Core.
- Do not store API keys in client projects.
- Do not duplicate Core domain logic in UI projects.
- If a document conflicts with an ADR, ADR wins.
- If code conflicts with docs, document the drift in ADR.

## Current Tech Stack

- **Language:** Rust (edition 2024)
- **Build:** Cargo workspace in `AdiyutantCore/`
- **Storage:** SQLite via `rusqlite` (bundled)
- **CLI:** `clap` 4 with derive macros
- **Testing:** `cargo test` (in-memory SQLite for store tests)

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
