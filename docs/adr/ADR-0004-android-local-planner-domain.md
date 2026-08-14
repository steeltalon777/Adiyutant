# ADR-0004: Android-local planner domain for Today + Plan v1

**Status:** Accepted
**Date:** 2026-08-14
**Driver:** TZ_ADIYUTANT_TODAY_PLAN_ANDROID_V1 (Today + Plan v1, Kotlin/Compose port)

## Context

The Kotlin port of the Сегодня + План screens is specified in
`docs/TZ_ADIYUTANT_TODAY_PLAN_ANDROID_V1.md`, which takes `AdiyutantAndroid/design/App.jsx`
as the behavioral source of truth. The prototype requires a domain model that the
Rust Core does not yet have:

```text
Task.taskId → ScheduledBlock.blockId → WorkSession
```

plus `CheckIn`, `QuickNote` and day/execution state (day phase, hero state machine,
active session with pause/resume, End-of-Day resolution).

`AdiyutantCore` currently exposes a different domain (DailyLog, Habit, Plan/PlanItem,
etc.) — there are no Task/ScheduledBlock/WorkSession aggregates in the facade, and
extending the Core DTOs to match the design is explicitly out of scope for this stage
(see `AGENTS.md`, `ROADMAP.md` Phase 9 scope guard).

ADR-0002 states: «UI consumes only CorePort» and «no domain logic lives in the UI».
`AdiyutantAndroid/design/MEMORY.md` (project-level, newer) explicitly instructs the
opposite for this milestone: «local/in-memory repository first», recommended modules
`domain`/`data`, native models `Task`, `ScheduledBlock`, `WorkSession`, `CheckIn`,
`QuickNote`, and «the first milestone for the Kotlin port: Today + Plan build
successfully with one shared state model».

## Decision

For Today + Plan v1, the planner domain is implemented **locally in the Android
project**, separated from the UI layer:

- `com.adiyutant.app.domain/` — pure Kotlin domain: models, `PlannerCore`
  (synchronous reducer `reduce(state, event): PlannerState`), `ScheduleValidator`,
  `TimeMath`, injectable `Clock`. No Android dependencies.
- `com.adiyutant.app.data/` — `PlannerRepository` interface + in-memory
  implementation with seed data.
- `com.adiyutant.app.ui/` — Compose screens and a thin `PlannerViewModel`
  (StateFlow host) that delegates all transitions to `PlannerCore`.

`CorePort` / `FakeCorePort` / `TodaySnapshot` / `CurrentActivity` remain untouched:
they are the seam for the Rust integration track (ADR-0003), which evolves
independently.

## Rationale

1. **The Core does not have this domain.** Waiting for the Core to grow the
   Task/ScheduledBlock/WorkSession aggregates would block the behavioral port
   indefinitely; the JSX prototype is the frozen specification for this milestone.
2. **The drift is bounded and reversible.** `PlannerRepository` is the future swap
   point: when the Rust Core gains the planner domain, the in-memory implementation
   is replaced by a Core-backed one behind the same interface, without UI changes.
3. **Domain logic stays out of the UI layer**, preserving the spirit of ADR-0002
   (UI shells consume, they do not own logic): `PlannerCore` is pure, synchronous
   and JVM-testable without Compose or an emulator.
4. **Persistence is not implemented** in the prototype and remains out of scope;
   in-memory state reset on process death is an accepted limitation for v1.

## Consequences

- This ADR records an explicit, accepted deviation from the letter of ADR-0002
  («UI consumes only CorePort»). ADR-0002 remains in force for the Core integration
  track.
- `ui/feature/today/TodayViewModel` (the CorePort-based stub) is superseded by
  `ui/planner/PlannerViewModel`.
- Migration path: extend the Rust facade with the planner domain, then implement
  `PlannerRepository` on top of `CorePort`; UI contracts (`PlannerState`,
  `PlannerEvent`) are expected to survive that swap.
- All domain invariants (single-entity rule, backlog projections excluding
  scheduled tasks, completion/session invariant, plan-vs-fact separation,
  lossless cancel) live in `PlannerCore` and are covered by JVM unit tests.

## Compliance

- Follows `AdiyutantAndroid/design/MEMORY.md` (implementation handoff section).
- Does not violate the Core boundary rule in spirit: no domain logic in the UI
  layer, no LLM provider calls, no API keys in the client.
- Does not modify the Rust Core or the existing `core/` Android package.
