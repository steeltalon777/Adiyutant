# AdiyutantCore

Status: Current — Rust workspace active.

AdiyutantCore is the Rust local-first domain core for Adiyutant.

## Crates

- `adiyutant_core` — base types: `Id<T>`, `CoreError`, `AdiyutantDateTime`
- `adiyutant_store` — storage abstraction (`Store` trait)
- `adiyutant_cli` — CLI entrypoint (`adiyutant` binary)

## Quick Start

```bash
cargo check --workspace
cargo test --workspace
cargo run -p adiyutant_cli -- status
```

## Dependencies

- chrono, serde, uuid, thiserror (workspace)
- clap (cli crate)

## Next Phase

Phase 2 — Domain Model: DailyLog, CheckIn, Habit, and other domain entities.
