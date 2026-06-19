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

## Phase 6 — Stabilization [x] Done

**Completed: 2026-06-08**

Goal: Bring local Core to MVP 0.1 quality.

- [x] Test coverage pass
- [x] Documentation sync with implemented Core
- [x] MVP acceptance scenario
- [x] MVP 0.1 release marker

## Phase 7 — Core Service Facade & Android Readiness [x] Done

**Completed: 2026-06-09**

Goal: Prepare Core for Android consumption — service facade, stable DTOs, planning domain, checklists, journal.

- [x] Store trait moved to `adiyutant_core` (architectural refactor)
- [x] `AdiyutantCoreService` facade with DTO boundary
- [x] `StartupState` / `CurrentActivity` with time-aware heuristics
- [x] Planning domain: DayPlan, PlanItem, EisenhowerQuadrant, TaskCheckpoint
- [x] Nudge model: NudgeSource, NotificationInstructionDto
- [x] Waiting task lifecycle (move, review, resume, archive)
- [x] Checklist domain: templates, runs, auto-seeding
- [x] Journal entry tracking (auto-journal on checkin)
- [x] 15 CLI commands (all through facade)
- [x] 148 tests (92 core + 46 store + 7 CLI unit + 3 CLI smoke)
- [x] ADR-0001: JSON embedded collections
- [x] Tag: v0.2.0-core-android-readiness

## Phase 8 — Core Hardening [ ] In Progress

Goal: Synchronize documentation with v0.3.0 reality, fix ignored Store errors, complete DTO contract, split service facade.

### Slice 1: Doc Sync & Error Fix [x] Done
- [x] Update TASKS.md, pipeline-state.json, AI_CONTEXT.md to v0.3.0
- [x] Replace all `let _ = self.store...` with composite transactions
- [x] Add `insert_plan_item_with_checkpoint` and `answer_checkpoint_composite` to Store trait

### Slice 2: DTO Contract Completion [x] Done
- [x] HabitDto, TimerDto, ReminderDto, AlarmDto, ContextDocumentDto, SuggestionDto
- [x] All facade methods return DTOs (no tuples)

### Slice 3: Service Facade Split & Contract Tests [ ] Planned
- [ ] Split service.rs into submodules
- [ ] Golden JSON contract snapshots
