# AdiyutantCore

Status: Current — Rust workspace active.

AdiyutantCore is the Rust local-first domain core for Adiyutant.

## Crates

- `adiyutant_core` — base types and domain model:
  - `Id<T>`, `CoreError`, `AdiyutantDateTime`
  - `DailyLog`, `CheckIn`, `Habit`, `HabitEvent`
  - `ReminderDefinition`, `AlarmDefinition`, `TimerDefinition`
  - `ContextDocument`, `Plan`, `ActionProposal`
- `adiyutant_store` — storage abstraction (`Store` trait)
- `adiyutant_cli` — CLI entrypoint (`adiyutant` binary)

## Quick Start

```bash
cargo check --workspace
cargo test --workspace
cargo run -p adiyutant_cli -- status
```

## Dependencies

- chrono, serde, serde_json, uuid, thiserror (workspace)
- clap (cli crate)

## Next Phase

Phase 3 — Local Rules and Today State: aggregator + local rule engine.
