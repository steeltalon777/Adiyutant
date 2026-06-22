# TZ: Phase 8.3 — Service Facade Split & Contract Tests

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-8.3-service-facade-split-and-contract-tests.md`

## Execution Strategy

- [x] 🟡 Sequential execution recommended
- **Reason:** Распил одного большого файла на модули требует согласованного порядка: сначала создать структуру каталога `service/`, затем перенести методы по модулям, затем извлечь тесты, затем проверить компиляцию. Параллельность здесь невозможна — все правки в одном логическом пространстве с жёсткими зависимостями импортов. Один агент, последовательно.

---

## Execution Checklist

- [x] 0. Context verified — Slice 1 и 2 завершены
- [x] 1. Create directory structure: `service/` with `mod.rs` + submodules
- [x] 2. Module: `service/today.rs` — get_today, get_startup_state, get_current_activity
- [x] 3. Module: `service/planning.rs` — plan items + waiting lifecycle
- [x] 4. Module: `service/checkpoints.rs` — checkpoint domain
- [x] 5. Module: `service/checklists.rs` — checklist domain
- [x] 6. Module: `service/journal.rs` — journal entries
- [x] 7. Module: `service/checkin.rs` — create_checkin
- [x] 8. Module: `service/basic_entities.rs` — habits, timers, reminders, alarms, context docs
- [x] 9. Module: `service/suggestions.rs` — get_suggestions
- [x] 10. Module: `service/helpers.rs` — shared helpers
- [x] 11. Old `service.rs` removed, `service/mod.rs` is sole module root
- [x] 12. Extract MockStore to `service/tests/mock_store.rs`
- [x] 13. Golden JSON contract examples in `docs/contracts/examples/`
- [x] 14. Contract validation tests: deserialize each golden JSON into DTO
- [x] 15. Update `docs/contracts/android-core-api.md` with examples reference
- [x] 16. Static checks: `fmt`, `check`, `clippy`
- [x] 17. Full test suite: `cargo test --workspace`
- [x] 18. Closeout docs: TASKS.md, pipeline-state.json, ROADMAP.md
- [x] 19. Final acceptance review complete

---

## 0. Context Verification

```bash
cd /home/makc/AI_sandbox/ADIYUTANT
git branch                    # dev
git log --oneline -3          # Slice 1 и 2 должны быть смержены

cd AdiyutantCore
cargo test --workspace        # все тесты проходят
cargo clippy --workspace --all-targets -- -D warnings

# Проверить, что DTO из Slice 2 на месте:
grep -c "pub struct HabitDto\|pub struct TimerDto\|pub struct ReminderDto\|pub struct AlarmDto\|pub struct ContextDocumentDto\|pub struct CheckInDto\|pub struct SuggestionDto\|pub struct HabitEventDto" \
  adiyutant_core/src/dto.rs
# ожидается: 8 (или больше, если старые DTO тоже match'ат)
```

**Acceptance:** Slice 1 и 2 завершены, все тесты проходят, 8 новых DTO присутствуют в dto.rs.

---

## 1. Создание структуры каталога

**Критически важно:** нельзя создавать `service/mod.rs` рядом с существующим `service.rs` — Rust неоднозначно разрешает модуль. Правильный порядок:

```bash
cd /home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore

# Шаг 1: Создать каталог и ПЕРЕМЕСТИТЬ старый файл
mkdir -p adiyutant_core/src/service/tests
git mv adiyutant_core/src/service.rs adiyutant_core/src/service/mod.rs

# Шаг 2: Проверить, что старый файл исчез
test ! -f adiyutant_core/src/service.rs && echo "OK: old service.rs removed"
test -f adiyutant_core/src/service/mod.rs && echo "OK: mod.rs exists"

# Шаг 3: Создать пустые подмодули
touch adiyutant_core/src/service/today.rs
touch adiyutant_core/src/service/planning.rs
touch adiyutant_core/src/service/checkpoints.rs
touch adiyutant_core/src/service/checklists.rs
touch adiyutant_core/src/service/journal.rs
touch adiyutant_core/src/service/checkin.rs
touch adiyutant_core/src/service/basic_entities.rs
touch adiyutant_core/src/service/suggestions.rs
touch adiyutant_core/src/service/helpers.rs
touch adiyutant_core/src/service/tests/mod.rs
touch adiyutant_core/src/service/tests/mock_store.rs

# Шаг 4: cargo check — должен скомпилироваться (mod.rs содержит весь старый код)
cargo check
```

**`lib.rs`** — менять не нужно. `pub mod service;` автоматически найдёт `service/mod.rs`.

### Порядок извлечения модулей

После шага 3 `mod.rs` содержит полный старый код (~2089 строк). Теперь последовательно:

1. Добавить `mod today;` в начало `mod.rs` и перенести методы из `mod.rs` → `today.rs`
2. `cargo check` (после каждого модуля)
3. Повторить для planning, checkpoints, checklists, journal, checkin, routines, suggestions, helpers
4. Извлечь MockStore в `tests/mock_store.rs`
5. `cargo test --workspace`

**Никогда не оставлять `service.rs` и `service/mod.rs` одновременно.**

**Acceptance:** Старый `service.rs` удалён, `service/mod.rs` существует и содержит только `struct` + конструкторы + объявления подмодулей. `cargo check` компилируется после каждого шага.

---

## 2-10. Перенос методов по модулям

**Правила переноса:**
- Каждый модуль содержит только методы `impl AdiyutantCoreService`
- `mod.rs` содержит: определение `struct AdiyutantCoreService`, `new()`, `with_time()`, и объявления `mod` для подмодулей. **Никаких `pub use` для методов не требуется** — методы из `impl AdiyutantCoreService` в подмодулях автоматически видны внешним потребителям через тип.
- Каждый подмодуль начинается с `use super::AdiyutantCoreService;` и реализует `impl AdiyutantCoreService { ... }`
- Private-методы остаются в том же модуле, что и использующие их public-методы
- `#[cfg(test)]` секции из подмодулей удаляются — все тесты переезжают в отдельные файлы

### 2. `service/today.rs` — строки 53-201

**Методы:** `build_today_state()` (private), `get_today()`, `get_startup_state()`, `get_current_activity()`

**Импорты:** `TodayState`, `TodayStateBuilder`, `LocalRuleAgentGateway`, `StartupState`, `CurrentActivity`, `dto`, все используемые DTO.

### 3. `service/planning.rs` — строки 202-406

**Методы:** `get_or_create_today_log()` (private), `add_plan_item()`, `list_plan_items()`, `start_plan_item()`, `done_plan_item()`, `move_to_waiting()`, `list_waiting_tasks()`, `review_waiting()`

**Импорты:** `PlanItem`, `PlanItemStatus`, `WaitingDecision`, `EisenhowerQuadrant`, `TaskCheckpoint`, `CheckpointKind`, `JournalEntry`, `JournalEntryType`, `dto`, `plan_item_to_dto` (из helpers), `parse_item_id` (из helpers).

### 4. `service/checkpoints.rs` — строки 408-535

**Методы:** `answer_checkpoint()`, `dismiss_checkpoint()`, `calculate_next_checkpoint()` (private), `get_pending_checkpoint_notification()`

**Импорты:** `TaskCheckpoint`, `CheckpointStatus`, `CheckpointResponse`, `CheckpointKind`, `JournalEntry`, `JournalEntryType`, `PlanItem`, `NotificationInstructionDto`, `parse_id`, `plan_item_to_dto`.

### 5. `service/checklists.rs` — строки 538-735

**Методы:** `list_checklist_templates()`, `seed_default_checklist()` (private), `run_checklist()`, `answer_checklist_item()`, `complete_checklist_run()`

**Импорты:** `ChecklistTemplate`, `ChecklistItem`, `ChecklistItemKind`, `ChecklistRun`, `ChecklistAnswer`, `JournalEntry`, `JournalEntryType`, `dto`, `parse_id`.

### 6. `service/journal.rs` — строки 737-776

**Методы:** `get_journal()`, `add_journal_entry()`

**Импорты:** `JournalEntry`, `JournalEntryType`, `JournalEntryDto`, `get_or_create_today_log` (from planning — нужен `pub(crate)`).

### 7. `service/checkin.rs` — строки 777-838

**Методы:** `create_checkin()`

**Импорты:** `CheckIn`, `CheckInType`, `CheckInDto`, `JournalEntry`, `JournalEntryType`, `get_or_create_today_log`.

### 8. `service/basic_entities.rs` — строки 838-1027

**Методы:** `add_habit()`, `list_habits()`, `mark_habit_done()`, `skip_habit()`, `add_timer()`, `list_timers()`, `add_reminder()`, `list_reminders()`, `add_alarm()`, `list_alarms()`, `add_context_document()`, `list_context_documents()`

**Импорты:** `Habit`, `HabitDto`, `HabitEvent`, `HabitEventStatus`, `HabitEventLevel`, `HabitEventDto`, `TimerDefinition`, `TimerDto`, `ReminderDefinition`, `ReminderDto`, `AlarmDefinition`, `AlarmDto`, `ContextDocument`, `ContextDocumentType`, `ContextDocumentDto`, `parse_id`.

### 9. `service/suggestions.rs` — строки 1027-1052

**Методы:** `get_suggestions()`

**Импорты:** `LocalRuleAgentGateway`, `SuggestionDto`.

### 10. `service/helpers.rs` — строки 1054-1086

**Функции:** `plan_item_to_dto()`, `parse_item_id()`, `parse_id()` (все pub(crate))

### `service/mod.rs`

```rust
mod today;
mod planning;
mod checkpoints;
mod checklists;
mod journal;
mod checkin;
mod routines;
mod suggestions;
mod helpers;

#[cfg(test)]
mod tests;

use crate::store::Store;
use crate::error::CoreError;
use crate::time_provider::{RealTimeProvider, TimeProvider};

/// Application-layer facade / use-case boundary.
pub struct AdiyutantCoreService {
    store: Box<dyn Store<Error = CoreError>>,
    time: Box<dyn TimeProvider>,
}

impl AdiyutantCoreService {
    pub fn new(store: Box<dyn Store<Error = CoreError>>) -> Self {
        Self {
            store,
            time: Box::new(RealTimeProvider),
        }
    }

    pub fn with_time(
        store: Box<dyn Store<Error = CoreError>>,
        time: Box<dyn TimeProvider>,
    ) -> Self {
        Self { store, time }
    }
}
```

> **Важно:** `struct AdiyutantCoreService` имеет два приватных поля `store` и `time`. Все подмодули с `impl AdiyutantCoreService` будут иметь к ним доступ, потому что они в том же крейте. Это корректно.

**Acceptance для пунктов 2-10:**
- Каждый подмодуль компилируется независимо
- Все публичные методы доступны через `use adiyutant_core::service::AdiyutantCoreService;` без изменений
- `service.rs` удалён (заменён на `service/mod.rs`)
- Сигнатуры методов не изменились
- `cargo check -p adiyutant_core` без ошибок

---

## 11. Извлечение MockStore

**Файл:** `adiyutant_core/src/service/tests/mock_store.rs`

Извлечь ВЕСЬ блок `impl Store for MockStore` и структуру `MockStore` из `service.rs` (строки 1108-1680+).

**Файл:** `adiyutant_core/src/service/tests/mod.rs`

```rust
mod mock_store;

use super::*;
// Импортировать MockStore для использования в тестах:
use mock_store::MockStore;
```

> **Примечание:** Тесты, которые были раньше в `#[cfg(test)] mod tests` внутри `service.rs`, теперь должны жить в отдельных тестовых файлах или в `service/tests/mod.rs`. Если тесты были специфичны для конкретного домена (planning, checkpoints, etc.), их можно оставить как `#[cfg(test)] mod tests` внутри соответствующих подмодулей — но проще перенести всё в `service/tests/`.

**Acceptance:** `cargo test -p adiyutant_core` проходит все тесты. MockStore реализует все методы Store trait (включая composite из Slice 1).

---

## 12. Обновление `lib.rs`

**Было:**
```rust
pub mod service;
```

**Стало:** Без изменений. Rust автоматически находит `service/mod.rs`, когда `service.rs` удалён.

Но нужно убедиться, что публичный API крейта не изменился:
```rust
// Проверить, что старые импорты всё ещё работают:
use adiyutant_core::service::AdiyutantCoreService;  // должно компилироваться
```

**Acceptance:** `cargo check -p adiyutant_cli` — CLI компилируется без изменений (CLI импортирует `adiyutant_core::service::AdiyutantCoreService`).

---

## 13. Golden JSON Contract Examples

**Каталог:** `docs/contracts/examples/`

Создать 7 JSON-файлов — эталонные примеры для каждого Android-facing экрана. Не нужно вызывать реальный код; это hand-written примеры, соответствующие DTO-структурам.

### 13a. `startup-state.json`

```json
{
  "intent": "start_day_required",
  "reason": "No morning check-in yet. Start your day by checking in."
}
```

### 13b. `today-dashboard.json`

```json
{
  "date": "2026-06-19",
  "has_daily_log": true,
  "mode": "focus",
  "sleep_score": "7",
  "energy": "6",
  "mood": "8",
  "check_in_count": 2,
  "has_morning_check_in": true,
  "has_evening_check_in": false,
  "habit_count": 3,
  "habit_events_done": 1,
  "has_plan": true,
  "plan_title": "Plan for 2026-06-19",
  "plan_item_count": 3,
  "plan_items": [
    {
      "description": "Write API docs",
      "status": "Started",
      "estimated_minutes": "",
      "notes": "Phase 8.3"
    },
    {
      "description": "Review PR",
      "status": "Planned",
      "estimated_minutes": "30",
      "notes": ""
    }
  ],
  "suggestion_count": 2
}
```

### 13c. `current-activity.json`

```json
{
  "date": "2026-06-19",
  "activity_kind": "scheduled_task",
  "active_task": "Write API docs",
  "active_block": "",
  "next_checkpoint_at": "14:00",
  "recommended_prompt": "Working on: Write API docs"
}
```

### 13d. `plan-item-lifecycle.json`

```json
{
  "lifecycle": [
    {
      "stage": "created",
      "plan_item": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "title": "Write API docs",
        "description": "Phase 8.3 contract examples",
        "quadrant": "important-urgent",
        "planned_start": "09:00",
        "planned_end": "11:00",
        "status": "Planned",
        "priority": 1,
        "source": "Manual",
        "created_at": "2026-06-19T08:00:00Z",
        "updated_at": "2026-06-19T08:00:00Z"
      }
    },
    {
      "stage": "started",
      "plan_item": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "title": "Write API docs",
        "description": "Phase 8.3 contract examples",
        "quadrant": "important-urgent",
        "planned_start": "09:00",
        "planned_end": "11:00",
        "status": "Started",
        "priority": 1,
        "source": "Manual",
        "created_at": "2026-06-19T08:00:00Z",
        "updated_at": "2026-06-19T09:00:00Z"
      }
    },
    {
      "stage": "done",
      "plan_item": {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "title": "Write API docs",
        "description": "Phase 8.3 contract examples",
        "quadrant": "important-urgent",
        "planned_start": "09:00",
        "planned_end": "11:00",
        "status": "Done",
        "priority": 1,
        "source": "Manual",
        "created_at": "2026-06-19T08:00:00Z",
        "updated_at": "2026-06-19T11:00:00Z"
      }
    }
  ]
}
```

### 13e. `waiting-review.json`

```json
{
  "waiting_tasks": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440002",
      "title": "Wait for design feedback",
      "waiting_since": "2026-06-12T10:00:00Z",
      "review_due": "2026-06-19"
    }
  ]
}
```

### 13f. `checklist-run.json`

```json
{
  "template": {
    "id": "770e8400-e29b-41d4-a716-446655440003",
    "title": "Morning Check-in",
    "category": "morning",
    "item_count": 4,
    "is_active": true
  },
  "run": {
    "id": "880e8400-e29b-41d4-a716-446655440004",
    "template_title": "Morning Check-in",
    "started_at": "2026-06-19T07:00:00Z",
    "completed_at": "2026-06-19T07:05:00Z",
    "answer_count": 4
  }
}
```

### 13g. `journal-feed.json`

```json
{
  "entries": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440005",
      "entry_type": "CheckInCreated",
      "summary": "morning check-in: Feeling good today",
      "timestamp": "2026-06-19T07:00:00Z"
    },
    {
      "id": "990e8400-e29b-41d4-a716-446655440006",
      "entry_type": "TaskStarted",
      "summary": "Started: Write API docs",
      "timestamp": "2026-06-19T09:00:00Z"
    },
    {
      "id": "990e8400-e29b-41d4-a716-446655440007",
      "entry_type": "Note",
      "summary": "Remember to call dentist",
      "timestamp": "2026-06-19T12:30:00Z"
    }
  ]
}
```

### 13h. `README.md` для examples

```markdown
# Contract Examples

Golden JSON snapshots for Android-Core API contract.
Each file represents a stable DTO shape that Android clients can rely on.

## Stability Guarantee

- Field additions are backward-compatible.
- Field renames are breaking changes.
- New optional fields use empty string `""` or sensible defaults.

## Files

| File | DTO | Screen |
|------|-----|--------|
| startup-state.json | StartupViewDto | Startup |
| today-dashboard.json | TodayViewDto | Today Dashboard |
| current-activity.json | CurrentActivityDto | Current Activity |
| plan-item-lifecycle.json | PlanItemDto × 3 stages | Plan Screen |
| waiting-review.json | Vec\<WaitingTaskDto\> | Waiting Review |
| checklist-run.json | ChecklistTemplateDto + ChecklistRunDto | Checklist Run |
| journal-feed.json | Vec\<JournalEntryDto\> | Journal |
```

**Acceptance:** 7 JSON-файлов + README.md в `docs/contracts/examples/`. JSON валиден (`python3 -m json.tool` для каждого).

### 13i. Contract Validation Tests (обязательно)

Golden JSON — не просто «красивые примеры», а реальный контракт. Каждый файл должен проходить десериализацию в соответствующий DTO.

**Файл:** `adiyutant_core/src/dto.rs` — добавить в `#[cfg(test)] mod tests`:

```rust
#[test]
fn contract_example_startup_state_deserializable() {
    let json = include_str!("../../../../docs/contracts/examples/startup-state.json");
    let dto: StartupViewDto = serde_json::from_str(json)
        .expect("startup-state.json must deserialize into StartupViewDto");
    assert!(!dto.intent.is_empty());
    assert!(!dto.reason.is_empty());
}

#[test]
fn contract_example_today_dashboard_deserializable() {
    let json = include_str!("../../../../docs/contracts/examples/today-dashboard.json");
    let dto: TodayViewDto = serde_json::from_str(json)
        .expect("today-dashboard.json must deserialize into TodayViewDto");
    assert!(!dto.date.is_empty());
}

#[test]
fn contract_example_current_activity_deserializable() {
    let json = include_str!("../../../../docs/contracts/examples/current-activity.json");
    let dto: CurrentActivityDto = serde_json::from_str(json)
        .expect("current-activity.json must deserialize into CurrentActivityDto");
    assert!(!dto.activity_kind.is_empty());
}

// Для plan-item-lifecycle.json — test-only wrapper:
#[derive(Debug, Deserialize)]
struct PlanItemLifecycleWrapper {
    lifecycle: Vec<LifecycleStage>,
}
#[derive(Debug, Deserialize)]
struct LifecycleStage {
    stage: String,
    plan_item: PlanItemDto,
}

#[test]
fn contract_example_plan_item_lifecycle_deserializable() {
    let json = include_str!("../../../../docs/contracts/examples/plan-item-lifecycle.json");
    let wrapper: PlanItemLifecycleWrapper = serde_json::from_str(json)
        .expect("plan-item-lifecycle.json must deserialize");
    assert_eq!(wrapper.lifecycle.len(), 3);
    assert_eq!(wrapper.lifecycle[0].stage, "created");
    assert_eq!(wrapper.lifecycle[1].plan_item.status, "Started");
    assert_eq!(wrapper.lifecycle[2].plan_item.status, "Done");
}

// Для waiting-review.json:
#[derive(Debug, Deserialize)]
struct WaitingReviewWrapper { waiting_tasks: Vec<WaitingTaskDto> }

#[test]
fn contract_example_waiting_review_deserializable() {
    let json = include_str!("../../../../docs/contracts/examples/waiting-review.json");
    let wrapper: WaitingReviewWrapper = serde_json::from_str(json)
        .expect("waiting-review.json must deserialize");
    assert!(!wrapper.waiting_tasks.is_empty());
}

// Для checklist-run.json:
#[derive(Debug, Deserialize)]
struct ChecklistRunWrapper {
    template: ChecklistTemplateDto,
    run: ChecklistRunDto,
}

#[test]
fn contract_example_checklist_run_deserializable() {
    let json = include_str!("../../../../docs/contracts/examples/checklist-run.json");
    let wrapper: ChecklistRunWrapper = serde_json::from_str(json)
        .expect("checklist-run.json must deserialize");
    assert_eq!(wrapper.template.category, "morning");
    assert_eq!(wrapper.run.answer_count, 4);
}

// Для journal-feed.json:
#[derive(Debug, Deserialize)]
struct JournalFeedWrapper { entries: Vec<JournalEntryDto> }

#[test]
fn contract_example_journal_feed_deserializable() {
    let json = include_str!("../../../../docs/contracts/examples/journal-feed.json");
    let wrapper: JournalFeedWrapper = serde_json::from_str(json)
        .expect("journal-feed.json must deserialize");
    assert_eq!(wrapper.entries.len(), 3);
    assert_eq!(wrapper.entries[0].entry_type, "CheckInCreated");
}
```

> **Важно:** Wrapper-структуры (`PlanItemLifecycleWrapper`, etc.) — test-only, в `#[cfg(test)]` модуле. Они не экспортируются.

### 13j. Проверка соответствия полей

Каждый golden JSON обязан использовать **те же имена полей и форматы значений**, что и реальные DTO:
- Статусы: `"Planned"`, `"Started"`, `"Done"`, `"Waiting"`, `"Archived"` (PascalCase — как в `PlanItemStatus::as_str()`)
- Даты: `"YYYY-MM-DD"` (ISO-8601 date)
- Timestamps: `"YYYY-MM-DDTHH:MM:SSZ"` (ISO-8601 с Z)
- Времена: `"HH:MM"` (24-hour)
- Числовые: строки `"7"`, не числа `7`
- Пустые optional: `""`, не `null`

Executor должен сверить каждый пример с актуальным `dto.rs` перед коммитом.

**Acceptance для 13a-13j:** 7 JSON примеров + 7 десериализационных тестов + проверка соответствия полей. Все тесты проходят.

---

## 14. Обновление Contract Documentation

**Файл:** `/home/makc/AI_sandbox/ADIYUTANT/docs/contracts/android-core-api.md`

Добавить после существующего содержания:

```markdown
## Contract Examples

Golden JSON examples for each screen are available in `docs/contracts/examples/`.
These represent stable DTO shapes guaranteed by the Core API.

| Example file | Screen |
|---|---|
| `startup-state.json` | Startup Screen |
| `today-dashboard.json` | Today Dashboard |
| `current-activity.json` | Current Activity |
| `plan-item-lifecycle.json` | Plan Screen (create → start → done) |
| `waiting-review.json` | Waiting Review |
| `checklist-run.json` | Checklist Run |
| `journal-feed.json` | Journal Feed |
```

**Acceptance:** `android-core-api.md` содержит ссылки на все 7 examples.

---

## 15. Static Checks

```bash
cd /home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

**Acceptance:** Без ошибок и warnings.

---

## 16. Full Test Suite

```bash
cd /home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore
cargo test --workspace
```

**Acceptance:** Все тесты проходят. Количество тестов не уменьшилось. Ожидается ~196+.

---

## 17. Closeout Docs

После завершения Slice 3 синхронизировать состояние проекта.

### 17a. `TASKS.md`

```markdown
## Current

None (Phase 8 hardening complete)

## Next

- [ ] TASK-0019 — Phase 9: Android/Kotlin Project Bootstrap

## Done

... (все предыдущие)
- [x] TASK-0016 — Phase 8.1: Doc Sync & Error Fix
- [x] TASK-0017 — Phase 8.2: DTO Contract Completion
- [x] TASK-0018 — Phase 8.3: Service Facade Split & Contract Tests
```

### 17b. `pipeline-state.json`

Добавить Phase 8.3:
```json
"Phase 8.3 — Service Facade Split & Contract Tests": {
  "index": 11,
  "status": "done",
  "tz": "docs/tz/phase-8.3-service-facade-split-and-contract-tests.md",
  "notes": "Completed. service.rs split into 10 submodules. 7 golden JSON contract examples. 196+ tests. Tag candidate: v0.4.0."
}
```

`current_phase`: `null` (hardening завершён)
`last_updated`: текущая дата/время UTC

### 17c. `ROADMAP.md`

```markdown
### Slice 1: Doc Sync & Error Fix [x] Done
### Slice 2: DTO Contract Completion [x] Done
### Slice 3: Service Facade Split & Contract Tests [x] Done

**Phase 8 complete.** Core hardened for Android consumption.
```

### 17d. `AI_CONTEXT.md`

```markdown
## Repository Status

v0.4.0-ready. Core hardened: service facade split into 10 modules, DTO contract complete (20+ DTOs), 196+ tests, golden JSON contract examples.

## Implemented

- AdiyutantCore/adiyutant_core/src/service/ — 10-module facade (mod.rs, today.rs, planning.rs, checkpoints.rs, checklists.rs, journal.rs, checkin.rs, basic_entities.rs, suggestions.rs, helpers.rs)
- docs/contracts/examples/ — 7 golden JSON contract examples
```

**Acceptance:** Все 4 closeout-файла обновлены.

---

## 18. Tag Candidate

После успешного завершения всех трёх слайсов — кандидат на тег `v0.4.0-core-hardened`:

```bash
git tag -a v0.4.0-core-hardened -m "Phase 8: Core Android Readiness Hardening complete
- Slice 1: Doc sync + ignored Store error fixes with composite transactions
- Slice 2: DTO contract completion (8 new DTOs, 14 methods)
- Slice 3: Service facade split (10 modules) + 7 golden JSON contract examples
196+ tests passing"
```

**Acceptance:** Тег создан (опционально — по решению release manager).

---

## Out of Scope

- Изменение публичного API `AdiyutantCoreService`
- Изменение сигнатур методов
- Изменение DTO
- Изменение доменной логики
- Новые фичи
- Android/Kotlin код
- `CurrentActivity` привязка к `planned_start/planned_end` — остаётся hour-based эвристикой (можно улучшить в будущем)
- Автоматическая генерация JSON из тестов (golden файлы — ручные)

---

## Test Strategy

| Уровень | Применимость | Что проверяется |
|---------|-------------|-----------------|
| Static checks | ✅ | `cargo fmt`, `cargo check`, `cargo clippy` |
| Unit tests | ✅ | MockStore тесты — проверяют, что распил не сломал логику |
| Component tests | ✅ | Все тесты `adiyutant_core` — неизменны |
| Integration tests | ✅ | `adiyutant_store` тесты — неизменны |
| Stand smoke tests | ❌ | CLI smoke не требуется — API не изменился |
| UI automation | ❌ | Нет UI |
| User scenarios | ❌ | Не применимо |
| Regression pack | ✅ | `cargo test --workspace` — все существующие тесты |
| Contract validation | ✅ | `python3 -m json.tool` для каждого из 7 golden-файлов |

---

## Stand Requirements

Не требуется. Изменения — чистый рефакторинг структуры файлов.

---

## Check Rules

- Architect создаёт checklist и acceptance criteria ✅
- Executor агент отмечает implementation и test items только после запуска проверок
- QA verifier проверяет final acceptance после ревью evidence

---

## Evidence Table

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Context verified | `cargo test --workspace`, DTO count | pass/fail | 196+ tests, 8 DTO |
| Directory created | `ls adiyutant_core/src/service/` | pass/fail | 11+ .rs файлов |
| Old service.rs removed | `test ! -f adiyutant_core/src/service.rs` | pass/fail | файла нет |
| mod.rs exists | `test -f adiyutant_core/src/service/mod.rs` | pass/fail | mod.rs существует |
| mod.rs compiles | `cargo check -p adiyutant_core` | pass/fail | без ошибок |
| today.rs | `grep "fn get_today" adiyutant_core/src/service/today.rs` | pass/fail | найдено |
| planning.rs | `grep "fn add_plan_item" adiyutant_core/src/service/planning.rs` | pass/fail | найдено |
| checkpoints.rs | `grep "fn answer_checkpoint" adiyutant_core/src/service/checkpoints.rs` | pass/fail | найдено |
| checklists.rs | `grep "fn run_checklist" adiyutant_core/src/service/checklists.rs` | pass/fail | найдено |
| journal.rs | `grep "fn get_journal" adiyutant_core/src/service/journal.rs` | pass/fail | найдено |
| checkin.rs | `grep "fn create_checkin" adiyutant_core/src/service/checkin.rs` | pass/fail | найдено |
| basic_entities.rs | `grep "fn add_habit\|fn add_timer\|fn add_reminder\|fn add_alarm" adiyutant_core/src/service/basic_entities.rs` | pass/fail | 4 метода |
| suggestions.rs | `grep "fn get_suggestions" adiyutant_core/src/service/suggestions.rs` | pass/fail | найдено |
| helpers.rs | `grep "fn parse_item_id\|fn parse_id" adiyutant_core/src/service/helpers.rs` | pass/fail | 2 функции |
| MockStore extracted | `grep "impl Store for MockStore" adiyutant_core/src/service/tests/mock_store.rs` | pass/fail | найдено |
| CLI compiles | `cargo check -p adiyutant_cli` | pass/fail | без изменений |
| 7 golden JSON | `ls docs/contracts/examples/*.json | wc -l` | pass/fail | 7 файлов |
| JSON valid | `for f in docs/contracts/examples/*.json; do python3 -m json.tool "$f" > /dev/null || echo "INVALID: $f"; done` | pass/fail | все валидны |
| Contract DTO tests | `cargo test -p adiyutant_core -- contract_example` | pass/fail | 7 десериализационных тестов |
| Public API preserved | `grep -Rh "pub fn " adiyutant_core/src/service/ \| wc -l` | pass/fail | совпадает с до-распила |
| Contract doc updated | `grep "examples/" docs/contracts/android-core-api.md` | pass/fail | найдено |
| Static checks | `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && git diff --check` | pass/fail | чисто |
| Tests | `cargo test --workspace` | pass/fail | 196+ passed |
| Closeout docs | `grep "TASK-0018.*Done" TASKS.md && grep "Phase 8.3" pipeline-state.json && grep "Slice 3.*Done" ROADMAP.md` | pass/fail | все найдены |

---

## Additional Acceptance Amendments

1. **Публичный API не меняется.** `use adiyutant_core::service::AdiyutantCoreService` должен работать без изменений. CLI не требует правок.

2. **Каждый подмодуль использует `impl AdiyutantCoreService`.** Private-поля `store` и `time` доступны через `self.store` / `self.time` в рамках крейта.

3. **Private-методы (`build_today_state`, `get_or_create_today_log`, `seed_default_checklist`, `calculate_next_checkpoint`) должны быть доступны из других подмодулей, если используются перекрёстно:**
   - `build_today_state` → `pub(crate)` (используется в today, suggestions)
   - `get_or_create_today_log` → `pub(crate)` (используется в planning, journal, checkin, checklists)
   - `seed_default_checklist` → оставить private (используется только в checklists)
   - `calculate_next_checkpoint` → оставить private (используется только в checkpoints)

4. **`plan_item_to_dto()`, `parse_item_id()`, `parse_id()`** в `helpers.rs` — `pub(crate)`.

5. **MockStore должен реализовывать ВСЕ методы Store trait**, включая 7 composite из Slice 1:
   ```bash
   grep -c "fn insert_plan_item_with_checkpoint\|fn answer_checkpoint_composite" \
     adiyutant_core/src/service/tests/mock_store.rs
   # ожидается: 2
   ```

6. **`service.rs` должен быть УДАЛЁН** после успешной компиляции `service/mod.rs`:
   ```bash
   test ! -f adiyutant_core/src/service.rs && echo "OK: old removed"
   test -f adiyutant_core/src/service/mod.rs && echo "OK: mod.rs exists"
   ```

7. **Публичный API не изменился.** После распила проверить, что все публичные методы на месте:
   ```bash
   grep -Rh "pub fn " AdiyutantCore/adiyutant_core/src/service/ | sort > /tmp/after_split.txt
   wc -l /tmp/after_split.txt
   ```
   Количество публичных методов должно совпадать с до-распила. Reviewer сверяет список с чеклистом модулей.

7. **Финальная проверка:**
   ```bash
   cd /home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore
   cargo check --workspace          # все крейты компилируются
   cargo test --workspace           # все тесты проходят

   cd /home/makc/AI_sandbox/ADIYUTANT
   python3 -m json.tool pipeline-state.json > /dev/null
   git diff --check
   ls docs/contracts/examples/*.json | wc -l  # 7
   test ! -f AdiyutantCore/adiyutant_core/src/service.rs && echo "old service.rs removed"
   ```

---

## Architecture Review

**Date:** 2026-06-19
**Reviewer:** Architect (self-review)

### Verdict

✅ **Approved.** Чисто механический рефакторинг — ноль изменений поведения, только структура файлов.

### 🔴 Blockers

Нет.

### 🟡 Warnings

**W1. Перекрёстные private-методы.** `build_today_state()` используется в `today.rs` и `suggestions.rs`. `get_or_create_today_log()` — в `planning.rs`, `journal.rs`, `checkin.rs`, `checklists.rs`. Оба должны стать `pub(crate)`. Если агент забудет — `cargo check` поймает, но лучше явно перечислить в ТЗ.

**W2. Неявные импорты.** При переносе кода в подмодули все `use`-импорты нужно добавлять заново. Компилятор подскажет, но процесс может занять несколько итераций. Рекомендация: переносить по одному модулю и проверять `cargo check` после каждого.

**W3. `service.rs` vs `service/mod.rs`.** Если `service.rs` не удалён, Rust может спутать модуль. Обязательно удалить старый файл после компиляции `mod.rs`.

### 🔵 Notes

**N1.** `CurrentActivity` привязка к `planned_start/planned_end` — осознанно отложена. Текущая hour-based эвристика достаточна для первого Android-релиза.

**N2.** Tag `v0.4.0-core-hardened` — кандидат. Решение о тегировании принимает release manager после приёмочного тестирования.
