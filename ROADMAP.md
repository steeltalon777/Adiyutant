# ROADMAP

## MVP Goal

MVP 0.1 — local `AdiyutantCore` on Rust + SQLite + CLI.

The core must prove that the organizer works without backend, sync, Android/Web/Desktop UI, external LLM, OpenClaw, or provider API keys.

## Phase 0 — Repository Foundation [x] Done

**Completed: 2026-06-04**

Goal: Create a repository that is immediately understandable to humans and AI agents.

- [x] Create bootstrap documentation structure
- [x] Define project boundaries
- [x] Mark all code as not yet implemented

## Phase 1 — Rust Core Workspace [x] Done

**Completed: 2026-06-08**

Goal: Get a working Rust workspace with no domain complexity.

- [x] Create `AdiyutantCore/` with Cargo workspace
- [x] Create crates: `adiyutant_core`, `adiyutant_store`, `adiyutant_cli`
- [x] Add core dependencies (chrono, serde, uuid, thiserror)
- [x] Create error model and base primitives

## Phase 2 — Domain Model [x] Done

**Completed: 2026-06-08**

Goal: Define the domain model for MVP without SQLite or UI.

- [x] DailyLog and CheckIn domain
- [x] Habit and HabitEvent domain
- [x] Time objects: reminders, alarms, timers
- [x] ContextDocument domain
- [x] Plan and ActionProposal domain

## Phase 3 — Local Rules and Today State [x] Done

**Completed: 2026-06-08**

Goal: Get useful behavior without external AI.

- [x] TodayState aggregator
- [x] LocalRuleAgentGateway (no LLM)

## Phase 4 — SQLite Storage [x] Done

**Completed: 2026-06-08**

Goal: Make local state persistent.

- [x] SQLite schema bootstrap
- [x] Repositories for all domain entities

## Phase 5 — CLI MVP [x] Done

**Completed: 2026-06-08**

Goal: First working interface to the Core.

- [x] CLI skeleton with clap
- [x] Daily/checkin commands
- [x] Habit commands
- [x] Timer/reminder/alarm commands
- [x] Context commands
- [x] Local suggestions command

## Phase 6 — Stabilization

Goal: Bring local Core to MVP 0.1 quality.

- Test coverage pass
- Documentation sync with implemented Core
- MVP acceptance scenario
- MVP 0.1 release marker
