# TZ: Phase 7 — Core Service Facade & Android Readiness

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-7-core-android-readiness.md`

---

## Execution Strategy

- [x] 🟡 Sequential execution in 3 waves with control gates
- **Reason:** Core domain models (day-plan, checklists, nudges) are interdependent. Service facade (Wave A) must stabilise before planning domain (Wave B) can depend on it. Journal and DTOs (Wave C) aggregate the work of both previous waves. Each wave ends with a full static+test gate and a manual audit before proceeding to the next wave.

### Wave boundaries

```
Wave A (TASK-0007–0008) → tests → audit → commit
Wave B (TASK-0009–0011) → tests → audit → commit
Wave C (TASK-0012–0015) → full acceptance → commit → tag
```

### Branch

```bash
git checkout dev
git checkout -b phase-7-core-android-readiness
```

---

## Execution Checklist

### Wave A — Service Layer

- [x] A0. Context verified: baseline tests pass, branch created
- [x] A1. `AdiyutantCoreService` struct — use-case/application layer facade
- [x] A2. `StartupState` + `StartupIntent` — what to show on launch
- [x] A3. `CurrentActivity` + `ExpectedActivityKind` — what's happening now
- [x] A4. Primary DTO/output models: `TodayViewDto`, `StartupViewDto`, `CurrentActivityDto`
- [x] A5. Reuse existing `Store` trait; no direct SQL from facade
- [x] A6. Static checks: `fmt`, `check`, `clippy --all-targets -D warnings`
- [x] A7. Unit + integration tests: startup state logic, current-activity resolution
- [x] A8. Regression: `cargo test --workspace` — all existing 103+ tests pass
- [x] A9. Manual CLI smoke: `startup`, `today`, `current` on fresh DB
- [x] A10. Wave A audit — gate check before Wave B

### Wave B — Planning Domain

- [x] B0. Wave A gate passed
- [x] B1. `DayPlan` struct — daily plan with items
- [x] B2. `PlanItem` — individual task with status, quadrant, times, source
- [x] B3. `EisenhowerQuadrant`, `PlanItemStatus`, `PlanningMode`
- [x] B4. `WaitingRepository` — lifecycle state on PlanItem, not a separate entity
- [x] B5. `TaskCheckpoint` + `CheckpointKind` + `CheckpointStatus` + `CheckpointResponse`
- [x] B6. Nudge model: `NudgeSource`, `NotificationInstructionDto`
- [x] B7. SQLite migrations — new tables: `plan_items`, `task_checkpoints`
- [x] B8. `Store` trait extension — planning persistence
- [x] B9. `SqliteStore` implementation — CRUD for new entities
- [x] B10. CLI commands: `plan`, `plan add-item`, `plan list`, `plan start-item`, `plan done-item`, `plan move-to-waiting`
- [x] B11. Static checks: `fmt`, `check`, `clippy --all-targets -D warnings`
- [x] B12. Unit + integration tests: planning domain and checkpoint resolution
- [x] B13. Regression: `cargo test --workspace` — 100+ tests green
- [x] B14. Wave B audit — gate check before Wave C

### Wave C — Journal & UI Boundary

- [x] C0. Wave B gate passed
- [x] C1. `ChecklistTemplate` + `ChecklistItem` — questionnaire template
- [x] C2. `ChecklistRun` + `ChecklistAnswer` — run instance and answers
- [x] C3. `JournalEntry` — daily activity log
- [x] C4. Complete DTO boundary: ensure all public outputs are DTOs, not raw domain structs
- [x] C5. `LifeCoreProfile` — minimal model through `ContextDocument` type extension
- [x] C6. SQLite migrations — new tables: `checklist_templates`, `checklist_items`, `checklist_runs`, `checklist_answers`, `journal_entries`  
  *(Note: `checklist_items` and `checklist_answers` stored as JSON columns `items_json` / `answers_json` within parent tables. ADR-0001.)*
- [x] C7. `Store` trait extension — checklist and journal persistence
- [x] C8. `SqliteStore` implementation — CRUD for new entities
- [x] C9. CLI commands: `checklist list`, `checklist run`, `journal`, `waiting list`, `waiting review`
- [x] C10. Static checks: `fmt`, `check`, `clippy --all-targets -D warnings`
- [x] C11. Unit + integration tests: checklists, journal, DTO stability
- [x] C12. Full acceptance scenario (§15) via CLI
- [x] C13. Documentation updated: `ARCHITECTURE.md`, `README.md`, `ROADMAP.md`
- [x] C14. Final audit — all 3 waves verified
- [x] C15. Commit + tag `v0.2.0-core-android-readiness`

---

## Hard Blocks

```
Do not start Android work.
Do not add Kotlin.
Do not add UniFFI yet.
Do not integrate OpenClaw.
Do not call external LLM providers.
Do not store API keys.
Do not duplicate domain logic in CLI.
Do not make UI-specific decisions inside Store.
```

## Layer Discipline

```
Store      — persistence (SQLite DDL, row mapping, CRUD).
Core service — application / use-case layer (facade for UI).
Domain models — internal structs with validation and rules.
DTOs       — stable, flat output models for external clients (Android / CLI views).
```

CLI and Android are both consumers of the public facade, never of raw Store or domain internals.

---

## Wave A Detailed Scope

### A1. `AdiyutantCoreService`

**File:** `adiyutant_core/src/service.rs` (new)

```rust
pub struct AdiyutantCoreService {
    store: Box<dyn Store<Error = CoreError>>,
}

impl AdiyutantCoreService {
    pub fn new(store: Box<dyn Store<Error = CoreError>>) -> Self;
    pub fn get_startup_state(&self) -> Result<StartupViewDto, CoreError>;
    pub fn get_today(&self) -> Result<TodayViewDto, CoreError>;
    pub fn get_current_activity(&self) -> Result<CurrentActivityDto, CoreError>;
}
```

Public methods correspond to the checklist in §1:
- `open_or_create` = `SqliteStore::new` + `migrate` (already exists, just wired)
- `get_today()`, `get_startup_state()`, `get_current_activity()`
- CRUD methods delegated through the facade (`create_checkin`, `add_habit`, `mark_habit_done`, etc.)
- `get_suggestions()` → delegates to `LocalRuleAgentGateway`

### A2. `StartupState` + `StartupIntent`

**File:** `adiyutant_core/src/startup.rs` (new)

```rust
pub enum StartupIntent {
    StartDayRequired,
    OfferDailySchedule,
    ShowTodayDashboard,
    SuggestEveningShutdown,
    NoAction,
}

pub struct StartupState {
    pub intent: StartupIntent,
    pub reason: String,
}

impl StartupState {
    pub fn determine(today_state: &TodayState) -> Self;
}
```

Logic:
1. No morning check-in → `StartDayRequired`
2. Morning check-in exists, no plan → `OfferDailySchedule`
3. Evening / late hour, no shutdown → `SuggestEveningShutdown`
4. Fallback → `ShowTodayDashboard`

### A3. `CurrentActivity` + `ExpectedActivityKind`

**File:** `adiyutant_core/src/current_activity.rs` (new)

```rust
pub enum ExpectedActivityKind {
    NoPlan,
    StartDay,
    ScheduledTask,
    Break,
    FreeTime,
    Meal,
    Recovery,
    Shutdown,
    SleepWindow,
}

pub struct CurrentActivity {
    pub kind: ExpectedActivityKind,
    pub active_task: Option<String>,
    pub active_block: Option<String>,
    pub next_checkpoint: Option<chrono::NaiveTime>,
    pub recommended_prompt: String,
}

impl CurrentActivity {
    pub fn determine(today_state: &TodayState) -> Self;
}
```

### A4. Primary DTOs

**File:** `adiyutant_core/src/dto.rs` (new)

```rust
pub struct TodayViewDto { /* flat strings, no domain-type nesting */ }
pub struct StartupViewDto { /* intent + reason */ }
pub struct CurrentActivityDto { /* flat view for central screen */ }
```

Rule: no `Id<T>`, no `serde_json::Value`, no chrono inner types in DTOs. All dates/times are ISO-8601 strings.

### A5. Reuse Store via trait

Facade owns `Box<dyn Store<Error = CoreError>>`. CLI creates `SqliteStore`, wraps in facade, then calls facade methods. No direct store access from CLI after this wave.

---

## Wave B Detailed Scope

### B1–B3. DayPlan & PlanItem

**File:** `adiyutant_core/src/model/day_plan.rs` (new)

```rust
pub struct DayPlan {
    pub id: Id<DayPlan>,
    pub date: chrono::NaiveDate,
    pub daily_log_id: Id<DailyLog>,
    pub items: Vec<PlanItem>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

pub struct PlanItem {
    pub id: Id<PlanItem>,
    pub title: String,
    pub description: Option<String>,
    pub quadrant: EisenhowerQuadrant,
    pub planned_start: Option<chrono::NaiveTime>,
    pub planned_end: Option<chrono::NaiveTime>,
    pub status: PlanItemStatus,
    pub priority: u8,
    pub source: PlanningMode,
    pub waiting_review_at: Option<chrono::NaiveDate>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}

pub enum EisenhowerQuadrant {
    ImportantUrgent,
    ImportantNotUrgent,
    NotImportantUrgent,
    NotImportantNotUrgent,
}

pub enum PlanItemStatus {
    Planned,
    Started,
    Done,
    Skipped,
    Moved,
    Waiting,
    Archived,
    Cancelled,
}

pub enum PlanningMode {
    Manual,
    AiGenerated,
    Template,
    CarryOver,
    LocalRule,
}
```

### B4. Waiting — lifecycle state, not separate repository

`WaitingRepository` is NOT a new entity. It's behaviour on `PlanItem`.  
When status is `Waiting`, `waiting_review_at` is set.  
Methods:
- `move_to_waiting(item_id, next_review)`
- `review_waiting(decision: WaitingDecision)` — keep/start/delete/archive
- `get_waiting_tasks_due_for_review(date)`

### B5. TaskCheckpoint

**File:** `adiyutant_core/src/model/task_checkpoint.rs` (new)

```rust
pub struct TaskCheckpoint {
    pub id: Id<TaskCheckpoint>,
    pub plan_item_id: Id<PlanItem>,
    pub kind: CheckpointKind,
    pub status: CheckpointStatus,
    pub response: Option<CheckpointResponse>,
    pub created_at: AdiyutantDateTime,
    pub answered_at: Option<AdiyutantDateTime>,
}

pub enum CheckpointKind { StartCheck, ProgressCheck, FinishCheck, RescheduleCheck, RelevanceReview, ShutdownPrompt }
pub enum CheckpointStatus { Pending, Shown, Answered, Dismissed }
pub enum CheckpointResponse { Started, Done, Snooze, DoingOther, Move, Skip, Cancel, KeepWaiting, Delete }
```

### B7–B9. SQLite + Store

New tables: `plan_items`, `task_checkpoints`.  
`DayPlan` uses existing `plans` table with extended columns OR `day_plans` as new table.  
Decision: keep existing `plans` table unchanged (backward compat) and add new `plan_items` + `task_checkpoints` tables. `DayPlan` is a query over `plan_items` grouped by date.

### B10. CLI commands

```
adiyutant plan add-item "Make Android shell" --quadrant important-urgent
adiyutant plan list
adiyutant plan start-item <id>
adiyutant plan done-item <id>
adiyutant plan move-to-waiting <id>
adiyutant waiting list
adiyutant waiting review <id> --decision keep
```

---

## Wave C Detailed Scope

### C1–C2. Checklist domain

**Files:** `core/src/model/checklist_template.rs`, `core/src/model/checklist_run.rs`

```rust
pub struct ChecklistTemplate {
    pub id: Id<ChecklistTemplate>,
    pub title: String,
    pub category: String,  // morning, day, evening, shutdown, recovery
    pub items: Vec<ChecklistItem>,
    pub version: u32,
    pub is_active: bool,
}

pub struct ChecklistItem {
    pub id: Id<ChecklistItem>,
    pub question: String,
    pub kind: ChecklistItemKind,
    pub options: Vec<String>,  // for Choice/Checkbox
    pub order: u32,
}

pub enum ChecklistItemKind {
    Checkbox, Choice, Scale, Text, OptionalComment, HabitEvent, TaskLink, TimerStart,
}

pub struct ChecklistRun {
    pub id: Id<ChecklistRun>,
    pub template_id: Id<ChecklistTemplate>,
    pub daily_log_id: Id<DailyLog>,
    pub started_at: AdiyutantDateTime,
    pub completed_at: Option<AdiyutantDateTime>,
    pub answers: Vec<ChecklistAnswer>,
}

pub struct ChecklistAnswer {
    pub id: Id<ChecklistAnswer>,
    pub checklist_item_id: Id<ChecklistItem>,
    pub value: String,
    pub comment: Option<String>,
    pub answered_at: AdiyutantDateTime,
}
```

### C3. JournalEntry

**File:** `adiyutant_core/src/model/journal_entry.rs` (new)

```rust
pub struct JournalEntry {
    pub id: Id<JournalEntry>,
    pub daily_log_id: Id<DailyLog>,
    pub timestamp: AdiyutantDateTime,
    pub entry_type: JournalEntryType,
    pub summary: String,
}

pub enum JournalEntryType {
    CheckInCreated, ChecklistCompleted, TaskStarted, TaskDone,
    TaskMoved, NudgeAnswered, Note, Shutdown,
}
```

### C4. DTO completeness

Ensure `Dto` suffix for every public output:  
`TodayViewDto`, `StartupViewDto`, `CurrentActivityDto`, `DayPlanDto`, `PlanItemDto`, `ChecklistRunDto`, `WaitingTaskDto`, `JournalEntryDto`, `NotificationInstructionDto`.

### C5. LifeCoreProfile

Minimal: extend `ContextDocumentType` with `LifeCore`, `Goals`, `Rules`, `Routine`, `RecoveryProtocol`, `Tone`, `PlanningPreferences`. No new table, just type tags.

### C9. CLI commands

```
adiyutant checklist list
adiyutant checklist run morning
adiyutant journal
adiyutant waiting list
adiyutant waiting review <id> --decision start
adiyutant startup
adiyutant current
```

---

## Required Test Levels (per wave)

| Level | Wave A | Wave B | Wave C |
|---|---|---|---|
| Static (fmt, check, clippy all-targets) | ✅ | ✅ | ✅ |
| Unit (domain logic, state machines) | ✅ | ✅ | ✅ |
| Integration (in-memory SQLite, new tables) | ✅ | ✅ | ✅ |
| Regression (all existing tests) | ✅ | ✅ | ✅ |
| CLI smoke (manual) | ✅ | ✅ | ✅ |
| Full acceptance scenario | — | — | ✅ |

---

## Test Stand

Same as MVP 0.1: `ADIYUTANT_DB_PATH` with temp file-based SQLite.  
In-memory DB for integration tests via `SqliteStore::new_in_memory()`.

---

## Acceptance Criteria

### Per-wave gates

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — 0 errors
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
- `cargo test --workspace` — all tests pass
- Manual CLI smoke for new commands

### Final acceptance (Wave C)

- Full acceptance scenario (§15) passes:
  1. Startup → `StartDayRequired`
  2. Morning check-in
  3. Plan day with 4-quadrant items
  4. Current activity shows task
  5. Checkpoint asks progress
  6. User answers — checkpoint resolved
  7. Unfinished task → Waiting
  8. Evening checklist + shutdown + journal
- `v0.2.0-core-android-readiness` tag on final commit

---

## Files

### Wave A (new)

| File | Purpose |
|---|---|
| `adiyutant_core/src/service.rs` | `AdiyutantCoreService` facade |
| `adiyutant_core/src/startup.rs` | `StartupState`, `StartupIntent` |
| `adiyutant_core/src/current_activity.rs` | `CurrentActivity`, `ExpectedActivityKind` |
| `adiyutant_core/src/dto.rs` | Public DTO layer |

### Wave A (modified)

| File | Change |
|---|---|
| `adiyutant_core/src/lib.rs` | Export new modules |
| `adiyutant_cli/src/common.rs` | Wire facade instead of raw store |
| `adiyutant_cli/src/commands/daily.rs` | Use facade for `today`, `checkin` |
| `adiyutant_cli/src/commands/suggest.rs` | Use facade for `suggest` |

### Wave B (new)

| File | Purpose |
|---|---|
| `adiyutant_core/src/model/day_plan.rs` | `DayPlan`, `PlanItem`, enums |
| `adiyutant_core/src/model/task_checkpoint.rs` | `TaskCheckpoint`, checkpoint enums |
| `adiyutant_cli/src/commands/plan.rs` | CLI commands for planning |

### Wave B (modified)

| File | Change |
|---|---|
| `adiyutant_core/src/model/mod.rs` | Export new model modules |
| `adiyutant_store/src/lib.rs` | New tables, CRUD, migrations |
| `adiyutant_core/src/service.rs` | Add planning methods |
| `adiyutant_core/src/dto.rs` | Add planning DTOs |
| `adiyutant_cli/src/main.rs` | Wire new commands |
| `adiyutant_cli/src/commands/mod.rs` | Register plan module |

### Wave C (new)

| File | Purpose |
|---|---|
| `adiyutant_core/src/model/checklist_template.rs` | `ChecklistTemplate`, `ChecklistItem` |
| `adiyutant_core/src/model/checklist_run.rs` | `ChecklistRun`, `ChecklistAnswer` |
| `adiyutant_core/src/model/journal_entry.rs` | `JournalEntry`, `JournalEntryType` |
| `adiyutant_cli/src/commands/checklist.rs` | CLI for checklists |
| `adiyutant_cli/src/commands/journal.rs` | CLI for journal |
| `adiyutant_cli/src/commands/waiting.rs` | CLI for waiting review |

### Wave C (modified)

| File | Change |
|---|---|
| `adiyutant_core/src/model/mod.rs` | Export new modules |
| `adiyutant_store/src/lib.rs` | New tables, CRUD |
| `adiyutant_core/src/service.rs` | Add checklist, journal, waiting methods |
| `adiyutant_core/src/dto.rs` | Add checklist, journal DTOs |
| `adiyutant_cli/src/main.rs` | Wire new commands |
| `adiyutant_cli/src/commands/mod.rs` | Register new modules |

---

## Risks

| Risk | Mitigation |
|---|---|
| Large scope, many files | Strict wave boundaries, one wave at a time |
| DTO vs domain confusion | `Dto` suffix convention, separate `dto.rs` module |
| CLI duplicates service logic | CLI calls facade only, no raw Store access after Wave A |
| Existing `plans` table conflict | Keep `plans` table unchanged; new `plan_items` table with FK to `plans.id` |
| Idempotent migrations break old DB | `CREATE TABLE IF NOT EXISTS` for all new tables; test on MVP 0.1 DB |

---

## Out of Scope

- ❌ Android UI, Kotlin, Compose
- ❌ UniFFI / C ABI
- ❌ OpenClaw, LLM providers, API keys
- ❌ Sync backend
- ❌ Calendar integration
- ❌ Monthly calendar views
- ❌ Charts / analytics
- ❌ Push notifications
- ❌ TUI / GUI
- ❌ Changes to existing domain models (no field additions to MVP 0.1 entities)

---

## Evidence Table

To be filled by executor after each wave.

| Wave | Check | Command | Result |
|---|---|---|---|---|
| A | fmt | `cargo fmt --all -- --check` | ✅ |
| A | check | `cargo check --workspace` | ✅ |
| A | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| A | tests | `cargo test --workspace` | ✅ (114 → 122 → 128) |
| A | CLI smoke | `adiyutant startup`, `adiyutant current`, `adiyutant today` | ✅ |
| B | fmt | `cargo fmt --all -- --check` | ✅ |
| B | check | `cargo check --workspace` | ✅ |
| B | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| B | tests | `cargo test --workspace` | ✅ (128 tests) |
| B | CLI smoke | plan commands, checkpoint flow, waiting | ✅ |
| C | fmt | `cargo fmt --all -- --check` | ✅ |
| C | check | `cargo check --workspace` | ✅ |
| C | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| C | tests | `cargo test --workspace` | ✅ (128 tests) |
| C | acceptance | Full §15 scenario | ✅ |
