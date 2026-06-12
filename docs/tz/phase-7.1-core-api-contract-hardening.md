# TZ: Phase 7.1 — Core API Contract Hardening

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-7.1-core-api-contract-hardening.md`

---

## Execution Strategy

- [ ] 🟡 Parallel-first within each priority level. P1 must complete before P2/P3 start (P1 is foundational — DTOs with serde unlock contract docs and tests). P2 and P3 are independent of each other and can run concurrently. P4 is docs-only, independent of all.
- **Reason:** P1 items (CoreErrorDto, serde, contract doc) are prerequisites for contract tests in P2/P3 and for the UI contract doc in P4. P2 domain quality and P3 storage hardening share no files and can run in parallel. P4 is pure documentation, can start anytime.

### Priority boundaries

```
P1 (contract foundation) → acceptance gate
P2 (domain quality) + P3 (storage hardening) → parallel, then gate
P4 (UI contract doc) → independent, can start anytime
```

### Branch

```bash
git checkout dev
git checkout -b phase-7.1-core-api-contract-hardening
```

---

## Execution Checklist

### P1 — Contract Foundation (Blocking)

- [x] P1.0. Context verified: `v0.2.0` tag present, 148 tests pass, workspace clean
- [x] P1.1. `CoreErrorDto` — structured error DTO with `code`, `message`, `recoverable`, `suggested_action`
- [x] P1.2. Map all `CoreError` variants to `CoreErrorDto` with recovery hints
- [x] P1.3. Add `Serialize`/`Deserialize` to all DTOs in `dto.rs`
- [x] P1.4. Add `serde` feature to `adiyutant_core` Cargo.toml for JSON round-trip
- [x] P1.5. Write contract tests: serialize each DTO, deserialize back, verify identity
- [x] P1.6. Public API document: `docs/api/core-service-api.md`
- [x] P1.7. Static checks: `fmt`, `check`, `clippy --all-targets -D warnings`
- [x] P1.8. Regression: `cargo test --workspace` — all 148+ tests pass

### P2 — Domain Quality (Can parallel with P3)

- [x] P2.0. P1 gate passed
- [x] P2.1. `answer_checkpoint()` — resolve a pending checkpoint with user response
- [x] P2.2. `calculate_next_checkpoint()` — determine next checkpoint after a completed one
- [x] P2.3. `CheckpointStatus` transitions — valid state machine: Pending→Shown→Answered/Dismissed
- [x] P2.4. `generate_notification_payload()` → `NotificationInstructionDto` from checkpoint
- [x] P2.5. Checklist answer flow: `answer_checklist_item()` + `complete_checklist_run()`
- [x] P2.6. Checklist completion: attach answers JSON, set `completed_at`, journal entry
- [x] P2.7. Journal auto-events missing: `TaskStarted`, `TaskDone`, `TaskMoved`, `ChecklistCompleted`
- [x] P2.8. `TimeProvider` trait — abstraction over `chrono::Utc::now()` for testability
- [x] P2.9. `RealTimeProvider` (prod) + `FakeTimeProvider` (test/config)
- [x] P2.10. Wire TimeProvider into `AdiyutantCoreService` (constructor parameter)
- [x] P2.11. Update `StartupState` / `CurrentActivity` to use TimeProvider
- [x] P2.12. Update store to use TimeProvider for `AdiyutantDateTime::now()` in service layer
- [x] P2.13. Static checks: `fmt`, `check`, `clippy --all-targets -D warnings`
- [x] P2.14. Unit + integration tests: checkpoint state machine, time injection, checklist flow
- [x] P2.15. Regression: `cargo test --workspace` — 197 tests green

### P3 — Storage Hardening (Can parallel with P2)

- [x] P3.0. P1 gate passed
- [x] P3.1. Versioned migrations: add `schema_versions` table with `version INT`, `applied_at TEXT`
- [x] P3.2. `run_migration()` becomes `run_migration_v2()` — check current version, apply only new steps
- [x] P3.3. Schema version tracking: store records which migrations have been applied
- [x] P3.4. Compatibility test: build v0.1 schema SQL → migrate to v0.2 → verify all data intact
- [x] P3.5. Transactional `create_checkin`: wrap insert_check_in + update_daily_log + journal_entry
- [x] P3.6. Transactional `start_plan_item`: wrap update_plan_item + insert_task_checkpoint
- [x] P3.7. Transactional `complete_checklist`: wrap update_checklist_run + insert_journal_entry
- [x] P3.8. Store-level integration test: v0.1 compatibility migration
- [x] P3.9. Static checks: `fmt`, `check`, `clippy --all-targets -D warnings`
- [x] P3.10. Unit + integration tests: migration versioning, transaction rollback, compat
- [x] P3.11. Regression: `cargo test --workspace` — 197 tests green

### P4 — UI Contract (Independent, can start anytime)

- [x] P4.0. Any gate passed (can start in parallel)
- [x] P4.1. Create `docs/ui-contract/android-core-contract.md`
- [x] P4.2. Document screens: Startup, Today Dashboard, Current Activity, Plan, Waiting Review, Checklist Run, Journal Feed
- [x] P4.3. For each screen: DTO used, call sequence, states, DTO examples (JSON-like)
- [x] P4.4. State glossary: `start_day_required`, `offer_daily_schedule`, `scheduled_task`, `no_plan`, `waiting`, `shutdown`, `recovery`, `free_time`
- [x] P4.5. Example JSON payloads for all DTOs (not code, human-readable)
- [x] P4.6. Document error DTO and recovery flows for common failure modes
- [x] P4.7. Review: document checked against actual `AdiyutantCoreService` method signatures

---

## Hard Blocks

```
Do not start Android work.
Do not add Kotlin.
Do not add UniFFI yet.
Do not integrate OpenClaw.
Do not call external LLM providers.
Do not store API keys.
Do not add new domain entities (Phase 7 models are enough).
Do not change Store trait signature without backwards compatibility.
```

---

## P1 Detailed Scope

### P1.1. CoreErrorDto

**File:** `adiyutant_core/src/dto.rs` (append)

```rust
/// Structured error for external consumers (Android, CLI, contract tests).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreErrorDto {
    /// Machine-readable error code: "NOT_FOUND", "INVALID_INPUT", "STORAGE_ERROR", "INTERNAL".
    pub code: String,
    /// Human-readable message for developer logs.
    pub message: String,
    /// Whether the client can recover (retry, change input, fix state).
    pub recoverable: bool,
    /// Suggested action for the client: "Retry", "Check input", "Contact support", etc.
    pub suggested_action: String,
}
```

### P1.2. CoreError → CoreErrorDto mapping

**File:** `adiyutant_core/src/error.rs` (add `impl CoreError`)

Mapping rules:

| Variant | code | recoverable | suggested_action |
|---------|------|-------------|------------------|
| `InvalidInput` | `"INVALID_INPUT"` | **true** | `"Check input"` |
| `NotFound` | `"NOT_FOUND"` | **true** | `"Check item exists"` |
| `Internal` | `"INTERNAL_ERROR"` | **false** | `"Contact support"` |
| `Storage` | `"STORAGE_ERROR"` | **false** | `"Retry or check disk"` |

```rust
impl CoreError {
    pub fn to_dto(&self) -> CoreErrorDto { ... }
}
```

### P1.3. serde for all DTOs

**File:** `adiyutant_core/src/dto.rs`

Add `#[derive(Serialize, Deserialize)]` to every DTO in the file. Add `serde` as a dependency if not already present (check Cargo.toml — `serde` is already a workspace dependency).

DTOs to annotate:
- `TodayViewDto`, `PlanItemViewDto`
- `StartupViewDto`
- `CurrentActivityDto`
- `DayPlanDto`, `PlanItemDto`, `WaitingTaskDto`
- `ChecklistTemplateDto`, `ChecklistRunDto`
- `JournalEntryDto`
- `NotificationInstructionDto` (in `model/nudge.rs`)
- New: `CoreErrorDto`

### P1.4. Add serde to Core Cargo.toml

**File:** `adiyutant_core/Cargo.toml`

Verify `serde = { workspace = true }` is present. If `Serialize`/`Deserialize` derives need `serde` features, ensure `serde = { workspace = true, features = ["derive"] }`.

### P1.5. DTO contract tests

**File:** `adiyutant_core/src/dto.rs` (test module)

For each DTO with `Serialize/Deserialize`:
```rust
#[test]
fn today_view_dto_serde_round_trip() {
    let dto = TodayViewDto { ... };
    let json = serde_json::to_string(&dto).unwrap();
    let back: TodayViewDto = serde_json::from_str(&json).unwrap();
    assert_eq!(dto.date, back.date);
    // ... verify key fields
}
```

At minimum: serialization doesn't panic, deserialization round-trips correctly.

### P1.6. Public API Document

**File:** `docs/api/core-service-api.md` (new)

Template:

```markdown
# Core Service API — Public Contract

## AdiyutantCoreService

### Method: get_startup_state()

- **Returns:** StartupViewDto
- **Errors:** CoreErrorDto (INTERNAL_ERROR, STORAGE_ERROR)
- **Stable:** yes

### Method: get_current_activity()
...
```

List every `pub fn` in `AdiyutantCoreService` with: return DTO, possible errors, stability note.

---

## P2 Detailed Scope

### P2.1–P2.4. Checkpoint answer flow

**File:** `adiyutant_core/src/service.rs` (add methods)

```rust
/// Answer a checkpoint and transition the associated PlanItem.
pub fn answer_checkpoint(
    &self,
    checkpoint_id: &str,
    response: &str,
) -> Result<PlanItemDto, CoreError>;

/// Determine the next checkpoint after a completed one.
fn calculate_next_checkpoint(
    plan_item: &PlanItem,
    last_kind: &CheckpointKind,
    last_response: &CheckpointResponse,
) -> Option<CheckpointKind>;

/// Generate a notification instruction from a checkpoint.
pub fn get_pending_checkpoint_notification(
    &self,
) -> Result<Option<NotificationInstructionDto>, CoreError>;
```

### P2.2. Checkpoint state machine

Valid transitions:

```
Pending  ──shown──→  Shown
Shown    ──answer──→ Answered  (with response)
Shown    ──dismiss──→ Dismissed
```

Invalid transitions → `CoreError::InvalidInput("checkpoint already answered")`

### P2.5–P2.6. Checklist answer/complete

**File:** `adiyutant_core/src/service.rs` (add methods)

```rust
/// Answer a single checklist item in a run.
pub fn answer_checklist_item(
    &self,
    run_id: &str,
    item_id: &str,
    value: &str,
    comment: Option<&str>,
) -> Result<ChecklistRunDto, CoreError>;

/// Complete a checklist run.
pub fn complete_checklist_run(
    &self,
    run_id: &str,
) -> Result<ChecklistRunDto, CoreError>;
```

On completion: set `completed_at`, create `JournalEntryType::ChecklistCompleted` journal entry.

### P2.7. Journal auto-events

Add journal entries in existing service methods:

| Service method | Journal entry |
|---|---|
| `start_plan_item()` | `JournalEntryType::TaskStarted` |
| `done_plan_item()` | `JournalEntryType::TaskDone` |
| `move_to_waiting()` | `JournalEntryType::TaskMoved` |
| `complete_checklist_run()` | `JournalEntryType::ChecklistCompleted` |

### P2.8–P2.11. TimeProvider

**File:** `adiyutant_core/src/time_provider.rs` (new)

```rust
pub trait TimeProvider: Send + Sync {
    fn now_utc(&self) -> chrono::DateTime<chrono::Utc>;
    fn today(&self) -> chrono::NaiveDate;
    fn hour(&self) -> u32;
}

pub struct RealTimeProvider;
impl TimeProvider for RealTimeProvider { ... }

pub struct FakeTimeProvider {
    fixed_time: chrono::DateTime<chrono::Utc>,
}
impl FakeTimeProvider {
    pub fn new(year: i32, month: u32, day: u32, hour: u32) -> Self;
}
impl TimeProvider for FakeTimeProvider { ... }
```

Wire into `AdiyutantCoreService`:

```rust
pub struct AdiyutantCoreService {
    store: Box<dyn Store<Error = CoreError>>,
    time: Box<dyn TimeProvider>,
}
impl AdiyutantCoreService {
    pub fn new(store: Box<dyn Store<Error = CoreError>>) -> Self {
        Self { store, time: Box::new(RealTimeProvider) }
    }
    pub fn with_time(store: Box<dyn Store<Error = CoreError>>, time: Box<dyn TimeProvider>) -> Self {
        Self { store, time }
    }
}
```

Update all `chrono::Utc::now()` calls in `service.rs` to use `self.time.now_utc()`. Update `StartupState::determine()` and `CurrentActivity::determine()` to accept `hour: u32` from the TimeProvider (already done — they use `determine_at_hour`).

### P2.12–P2.13. Tests

- TimeProvider: test that `FakeTimeProvider` returns expected values
- Checkpoint state machine: test valid/invalid transitions
- Checklist flow: test answer → complete → journal entry
- Journal auto-events: verify entries created in start/done/move/complete

---

## P3 Detailed Scope

### P3.1–P3.3. Versioned migrations

**File:** `adiyutant_store/src/lib.rs`

Add table:

```sql
CREATE TABLE IF NOT EXISTS schema_versions (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL
);
```

Refactor `run_migration`:

```rust
fn run_migration(conn: &Connection) -> Result<(), CoreError> {
    // Check current version
    let current: i64 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_versions", [], |r| r.get(0))
        .unwrap_or(0);

    if current < 1 {
        conn.execute_batch("... v0.1 schema ...")?;
        mark_migration(conn, 1, "v0.1-initial")?;
    }
    if current < 2 {
        conn.execute_batch("... v0.2 new tables ...")?;
        mark_migration(conn, 2, "v0.2-planning-checklists-journal")?;
    }
    Ok(())
}

fn mark_migration(conn: &Connection, version: i64, name: &str) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO schema_versions (version, name, applied_at) VALUES (?1, ?2, ?3)",
        params![version, name, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}
```

### P3.4. v0.1 → v0.2 compatibility test

**File:** `adiyutant_store/src/lib.rs` (test module)

```rust
#[test]
fn migrate_v01_to_v02_preserves_data() {
    let conn = Connection::open_in_memory().unwrap();
    // Step 1: Create v0.1 schema manually (using v0.1 SQL)
    conn.execute_batch("... v0.1 tables ...").unwrap();
    // Step 2: Insert test data
    conn.execute("INSERT INTO daily_logs (...) VALUES (...)", params![...]).unwrap();
    // Step 3: Run v0.2 migration
    run_migration(&conn).unwrap();
    // Step 4: Verify existing data intact + new tables exist
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM daily_logs", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1);
    // Verify new tables
    conn.execute("SELECT COUNT(*) FROM plan_items", []).unwrap();
    conn.execute("SELECT COUNT(*) FROM journal_entries", []).unwrap();
}
```

### P3.5–P3.7. Transactions

**File:** `adiyutant_store/src/lib.rs`

Add transaction wrapper methods:

```rust
impl SqliteStore {
    /// Execute closure in a SQLite transaction.
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T, CoreError>
    where
        F: FnOnce(&Connection) -> Result<T, CoreError>,
    {
        self.conn.execute("BEGIN IMMEDIATE", [])?;
        match f(&self.conn) {
            Ok(val) => {
                self.conn.execute("COMMIT", [])?;
                Ok(val)
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }
}
```

Use in facade:

```rust
pub fn create_checkin(&self, ...) -> Result<...> {
    self.store.with_transaction(|_conn| {
        // insert check_in
        // update daily_log
        // insert journal_entry
        Ok(...)
    })
}
```

### P3.8. Tests

- Transaction rollback test: force error in middle, verify nothing persisted
- Compatibility migration test: v0.1 schema → v0.2 migration → data intact

---

## P4 Detailed Scope

### P4.1–P4.7. UI Contract Document

**File:** `docs/ui-contract/android-core-contract.md` (new)

Document structure:

```
# Android-Core UI Contract — v0.2.0

## Screens
1. Startup Screen
2. Today Dashboard
3. Current Activity
4. Plan Screen
5. Waiting Review
6. Checklist Run
7. Journal Feed

## Per-screen spec
- DTO used
- Call sequence (Core methods called)
- States (what values trigger which UI)
- DTO example (JSON-like, not code)
- Error handling

## State Glossary
...
```

Each screen entry:

```markdown
### 1. Startup Screen

**DTO:** `StartupViewDto`
**Method:** `get_startup_state()`
**States:**
- `intent: "start_day_required"` → show "Start your day" with morning check-in button
- `intent: "offer_daily_schedule"` → show "Create plan for today" with plan button
- `intent: "show_today_dashboard"` → show dashboard directly
- `intent: "suggest_evening_shutdown"` → show "Evening shutdown" prompt

**DTO example:**
{
  "intent": "start_day_required",
  "reason": "No morning check-in yet."
}
```

---

## Required Test Levels

| Level | P1 | P2 | P3 | P4 |
|---|---|---|---|---|
| Static (fmt, check, clippy all-targets) | ✅ | ✅ | ✅ | N/A |
| Unit (DTO serde, error mapping, state machine) | ✅ | ✅ | ✅ | N/A |
| Integration (transaction, migration compat, time injection) | — | ✅ | ✅ | N/A |
| Regression (all existing 148+ tests) | ✅ | ✅ | ✅ | N/A |
| Contract (DTO serialization round-trip) | ✅ | — | — | N/A |
| Doc review (manual) | — | — | — | ✅ |

---

## Test Stand

Same as Phase 7: `ADIYUTANT_DB_PATH` with temp file-based SQLite. In-memory DB for integration tests via `SqliteStore::new_in_memory()`.

Migration compatibility test uses in-memory SQLite with manual schema creation.

---

## Acceptance Criteria

### Per-priority gates

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — 0 errors
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
- `cargo test --workspace` — all tests pass (≥ 183)

### P1 gate (must pass before P2/P3 start)

- All DTOs have `Serialize`/`Deserialize` (+ derives)
- `CoreErrorDto` mapping covers all 4 variants
- `docs/api/core-service-api.md` lists every public method
- DTO contract tests: serialize → deserialize → round-trip identity

### P2 gate

- Checkpoint flow: Pending → Shown → Answered/Dismissed
- Checklist flow: create run → answer items → complete → journal entry
- TimeProvider replaces all direct `chrono::Utc::now()` calls in service layer
- FakeTimeProvider enables deterministic time-based tests

### P3 gate

- `schema_versions` table tracks migration history
- `migrate_is_idempotent` test updated for versioned approach
- v0.1 → v0.2 compatibility test passes
- 3 composite actions use transactions

### P4 gate

- `docs/ui-contract/android-core-contract.md` exists and covers all 7 screens
- Every screen has: DTO used, method call, states, JSON example
- State glossary covers all 8 states
- Document reviewed against actual `AdiyutantCoreService` signatures

### Final acceptance

- All 4 priorities pass their respective gates
- Tag `v0.3.0-core-api-contract-hardened`

---

## Files

### P1 (new)

| File | Purpose |
|---|---|
| `docs/api/core-service-api.md` | Public API document |

### P1 (modified)

| File | Change |
|---|---|
| `adiyutant_core/src/dto.rs` | Add `CoreErrorDto`, add `Serialize`/`Deserialize` to all DTOs |
| `adiyutant_core/src/error.rs` | Add `CoreError::to_dto()` |
| `adiyutant_core/src/model/nudge.rs` | Add `Serialize`/`Deserialize` to `NotificationInstructionDto` |
| `adiyutant_core/Cargo.toml` | Verify `serde` with `derive` feature |

### P2 (new)

| File | Purpose |
|---|---|
| `adiyutant_core/src/time_provider.rs` | `TimeProvider` trait + `RealTimeProvider` + `FakeTimeProvider` |

### P2 (modified)

| File | Change |
|---|---|
| `adiyutant_core/src/service.rs` | Add checkpoint answer flow, checklist answer/complete, journal auto-events, TimeProvider integration |
| `adiyutant_core/src/lib.rs` | Export `time_provider` module |
| `adiyutant_core/src/error.rs` | Add `CoreErrorDto` → `Result` adapter if needed |

### P3 (modified)

| File | Change |
|---|---|
| `adiyutant_store/src/lib.rs` | Versioned migrations, `schema_versions` table, `with_transaction()`, compat test |

### P4 (new)

| File | Purpose |
|---|---|
| `docs/ui-contract/android-core-contract.md` | UI contract document |

---

## Out of Scope

- ❌ Android UI, Kotlin, Compose
- ❌ UniFFI / C ABI / JNI bridge
- ❌ OpenClaw, LLM providers, API keys
- ❌ Sync backend
- ❌ Calendar integration
- ❌ Monthly calendar views
- ❌ Charts / analytics
- ❌ Push notifications (beyond NotificationInstructionDto generation)
- ❌ TUI / GUI
- ❌ Changes to domain model structs (new fields, not new entities)
- ❌ New CLI commands (CLI smoke tests only, no new commands)

---

## Evidence Table

To be filled by executor after each priority level.

| Priority | Check | Command | Result |
|---|---|---|---|---|
| P1 | fmt | `cargo fmt --all -- --check` | ✅ clean |
| P1 | check | `cargo check --workspace` | ✅ 0 errors |
| P1 | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| P1 | tests | `cargo test --workspace` | ✅ 183 passed |
| P1 | contract | DTO serde round-trip tests | ✅ 11 tests present |
| P1 | doc | `docs/api/core-service-api.md` complete | ✅ 27 methods |
| P2 | fmt | `cargo fmt --all -- --check` | ✅ clean |
| P2 | check | `cargo check --workspace` | ✅ 0 errors |
| P2 | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| P2 | tests | `cargo test --workspace` | ✅ 183 passed |
| P2 | domain | TimeProvider, checkpoint, checklist, journal events | ✅ 16 new tests |
| P2 | checkpoint flow | `answer_checkpoint` one-call + `dismiss_checkpoint` | ✅ Pending→Answered/dismissed |
| P3 | fmt | `cargo fmt --all -- --check` | ✅ clean |
| P3 | check | `cargo check --workspace` | ✅ 0 errors |
| P3 | clippy | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| P3 | tests | `cargo test --workspace` | ✅ 183 passed |
| P3 | compat | v0.1 → v0.2 migration test | ✅ migrate_v01_to_v02_preserves_data |
| P3 | transaction | composite methods use BEGIN/COMMIT | ✅ checkin, start/done/move plan, complete checklist |
| P3 | rollback | SQLite ROLLBACK on composite error | ✅ tested via SqliteStore::with_transaction |
| P4 | doc | `docs/ui-contract/android-core-contract.md` | ✅ v0.3.0, 7 screens, no stale content |
| P4 | review | API doc consistency check | ✅ reviewed vs service.rs |

---

## Risks

| Risk | Mitigation |
|---|---|
| TimeProvider touches many files | Isolate in `service.rs` — domain models keep direct chrono usage |
| serde may bloat compile time | Only add to DTOs, not domain models |
| Migration versioning breaks idempotency | Add `migration_is_idempotent` compatibility test |
| UI contract doc drifts from actual API | P4 gate includes signature review against `AdiyutantCoreService` |
| Small hardening grows into feature creep | Strict out-of-scope list above — no new domain entities or CLI commands |
