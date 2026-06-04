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

## Phase 1 — Rust Core Workspace

Goal: Get a working Rust workspace with no domain complexity.

- Create `AdiyutantCore/` with Cargo workspace
- Create crates: `adiyutant_core`, `adiyutant_store`, `adiyutant_cli`
- Add core dependencies (chrono, serde, uuid, thiserror)
- Create error model and base primitives

## Phase 2 — Domain Model

Goal: Define the domain model for MVP without SQLite or UI.

- DailyLog and CheckIn domain
- Habit and HabitEvent domain
- Time objects: reminders, alarms, timers
- ContextDocument domain
- Plan and ActionProposal domain

## Phase 3 — Local Rules and Today State

Goal: Get useful behavior without external AI.

- TodayState aggregator
- LocalRuleAgentGateway (no LLM)

## Phase 4 — SQLite Storage

Goal: Make local state persistent.

- SQLite schema bootstrap
- Repositories for all domain entities

## Phase 5 — CLI MVP

Goal: First working interface to the Core.

- CLI skeleton with clap
- Daily/checkin commands
- Habit commands
- Timer/reminder/alarm commands
- Context commands
- Local suggestions command

## Phase 6 — Stabilization

Goal: Bring local Core to MVP 0.1 quality.

- Test coverage pass
- Documentation sync with implemented Core
- MVP acceptance scenario
- MVP 0.1 release marker
