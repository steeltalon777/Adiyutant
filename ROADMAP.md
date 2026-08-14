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

### Slice 3: Service Facade Split & Contract Tests [x] Done
- [x] Split service.rs into 10 submodules
- [x] 7 golden JSON contract examples with deserialization tests

**Phase 8 complete.** Core hardened for Android consumption.

## Phase 9 — Android Shell Bootstrap [x] In Progress → Slice 1–5 Done

Goal: Create a buildable `AdiyutantAndroid/` Gradle project with a Compose theme, a 5-tab navigation shell, stub screens, and a `CorePort`/`FakeCorePort` boundary. UI and Rust integration evolve independently (ADR-0002, ADR-0003). The design system and full screen implementations come later.

### Slice 1: Docs, Policy & ADR [x] Done
- [x] Allow Android Kotlin code in `AdiyutantAndroid/` in `AGENTS.md` (Phase 9 scope)
- [x] ADR-0002: Android bootstrap stack (Kotlin, Compose, Material 3 as infrastructure, Navigation Compose, ViewModel, CorePort/FakeCorePort)
- [x] ADR-0003: Android ↔ Rust Core integration spike-first (UniFFI preferred candidate, gated on spike)
- [x] Decompose `TASK-0019` into sub-tasks in `TASKS.md`

### Slice 2: Gradle Project Bootstrap [x] Done
- [x] Root `build.gradle.kts`, `settings.gradle.kts`, `gradle.properties`, Gradle wrapper
- [x] `app/` module with Compose + Material 3 + Navigation Compose dependencies
- [x] `AndroidManifest.xml`, `MainActivity`, `Application` class
- [x] App builds successfully (`./gradlew assembleDebug`)

### Slice 3: Theme / Adiyutant UI Kit Tokens [x] Done
- [x] `ui/theme/Color.kt`, `Type.kt`, `Shape.kt`, `AdiyutantTheme.kt` from `docs/design/today-screen.html` tokens
- [x] Dark theme only; light theme deferred until the design system finalizes

### Slice 4: Navigation Shell + Stub Screens [x] Done
- [x] Navigation Compose graph with 5 destinations: Сегодня / План / Проекты / Время / Настройки
- [x] Bottom navigation bar matching the tab design
- [x] Stub screen composables for all 5 tabs

### Slice 5: CorePort + FakeCorePort [x] Done
- [x] `CorePort` interface (repository-style boundary, suspend functions returning UI models)
- [x] `FakeCorePort` with sample data; wired via manual DI / ServiceLocator
- [x] No Rust dependency; real integration deferred (ADR-0003)

### Slice 6: Verification [x] Done
- [x] `./gradlew build` green
- [x] Emulator smoke test: app launches, 5 tabs navigate, theme renders
- [x] Documentation sync: `AGENTS.md`, `TASKS.md`, `AI_CONTEXT.md`

**Phase 9 scope guard:** full screen implementations, Core DTO extensions, and Android ↔ Rust integration are out of scope here (recorded in `AGENTS.md`).
