# TZ: Phase 2 — Domain Model

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-2-domain-model.md`

## Execution Strategy

- [ ] 🟢 Parallel execution recommended (Stage 2)
- **Reason:** Модели слабо связаны: связи только через `Id<T>` (typed UUID), без прямых ссылок Rust. Три группы моделей могут разрабатываться параллельно независимыми агентами. Stage 1 (подготовка структуры) — последовательный, Stage 3 (интеграция) — последовательный.

## Execution Checklist

- [ ] 0. Context verified
- [ ] 1. Stage 1: Workspace update + `model/` module structure
- [ ] 2. Stage 2a: `daily_log` + `check_in` — агент A
- [ ] 3. Stage 2b: `habit` + `habit_event` + `context_document` — агент B
- [ ] 4. Stage 2c: `reminder` + `alarm` + `timer` + `plan` + `action_proposal` — агент C
- [ ] 5. Stage 3: Integration — `lib.rs`, `cargo check`, тесты каждой модели, `cargo test --workspace`
- [ ] 6. `cargo fmt --all -- --check`
- [ ] 7. `cargo clippy --workspace -- -D warnings` (опционально)
- [ ] 8. Documentation: `AdiyutantCore/README.md` обновлён
- [ ] 9. Final acceptance review

## Check Rules

- Architect создаёт checklist и acceptance criteria.
- Executor agent отмечает пункты только после реализации и верификации.
- Если проверка не прошла — пункт остаётся незаполненным с причиной в отчёте.

---

## 1. Source Documents

| Документ | Роль |
|---|---|
| `ROADMAP.md` — Phase 2 | Источник scope |
| `SPECIFICATION.md` — §5.1, §14.1–14.10 | Поля доменных моделей |
| `ARCHITECTURE.md` — Core Boundary | Что Core owns |
| `docs/core-boundary.md` | Границы Core |
| `GLOSSARY.md` | Термины домена |
| `AGENTS.md` | Правила репозитория |

---

## 2. Goal

Определить полную доменную модель MVP в `adiyutant_core::model` без SQLite, без UI, без LLM. Все 10 доменных сущностей с типами, валидацией, `Serialize`/`Deserialize` и unit-тестами.

---

## 3. Scope

### In scope

- 10 доменных моделей в `adiyutant_core/src/model/`:
  - `DailyLog` — дневной журнал
  - `CheckIn` — чек-ин (утро/день/вечер/закрытие)
  - `Habit` — привычка
  - `HabitEvent` — событие привычки
  - `ReminderDefinition` — напоминание
  - `AlarmDefinition` — будильник
  - `TimerDefinition` — таймер
  - `ContextDocument` — контекстный документ
  - `Plan` — план (агрегат намерений)
  - `ActionProposal` — предложение действия
- `serde_json` как workspace-зависимость (для `structured_data`, `payload_json`)
- Тесты для каждой модели: конструкторы, поля, serialize/deserialize round-trip

### Out of scope

- SQLite, миграции, репозитории — Фаза 4
- `TodayState` агрегатор — Фаза 3
- `LocalRuleAgentGateway` — Фаза 3
- CLI-команды бизнес-логики — Фаза 5
- Валидация бизнес-правил (пересечения, конфликты) — Фаза 3
- LLM, Agent Gateway, backend, sync
- Изменения в `adiyutant_store` или `adiyutant_cli`

---

## 4. Implementation Stages

### Stage 1 — Workspace Update + Module Structure

**Исполнитель:** один агент или orchestrator.

#### 1a. Добавить `serde_json` в workspace

**Файл:** `AdiyutantCore/Cargo.toml` — добавить в `[workspace.dependencies]`:

```toml
serde_json = "1"
```

**Файл:** `AdiyutantCore/adiyutant_core/Cargo.toml` — добавить в `[dependencies]`:

```toml
serde_json = { workspace = true }
```

#### 1b. Создать `model/mod.rs`

**Файл:** `AdiyutantCore/adiyutant_core/src/model/mod.rs`

Объявляет все модули-заглушки (файлы будут созданы агентами на Stage 2):

```rust
pub mod daily_log;
pub mod check_in;
pub mod habit;
pub mod habit_event;
pub mod reminder;
pub mod alarm;
pub mod timer;
pub mod context_document;
pub mod plan;
pub mod action_proposal;

// Re-exports for convenience
pub use daily_log::DailyLog;
pub use check_in::CheckIn;
pub use habit::Habit;
pub use habit_event::HabitEvent;
pub use reminder::ReminderDefinition;
pub use alarm::AlarmDefinition;
pub use timer::TimerDefinition;
pub use context_document::ContextDocument;
pub use plan::Plan;
pub use action_proposal::ActionProposal;
```

#### 1c. Обновить `lib.rs`

**Файл:** `AdiyutantCore/adiyutant_core/src/lib.rs`

Добавить `pub mod model;`:

```rust
pub mod datetime;
pub mod error;
pub mod id;
pub mod model;
```

#### 1d. Проверка Stage 1

```bash
cargo check -p adiyutant_core
# Должен упасть с «file not found» для каждого module — это ожидаемо.
# Пустые файлы создавать не нужно — агенты Stage 2 их создадут.
```

**Acceptance criteria:**
- [ ] `serde_json` доступен в workspace и в `adiyutant_core`
- [ ] `model/mod.rs` содержит `pub mod` и re-exports для всех 10 моделей
- [ ] `lib.rs` содержит `pub mod model;`
- [ ] Коммит Stage 1 (или ветка) готов к параллельной работе

---

### Stage 2 — Parallel Model Creation

**Исполнители:** 3 агента параллельно (A, B, C).

**Важно:** каждый агент работает в своём наборе файлов — конфликтов быть не должно. `model/mod.rs` уже создан на Stage 1 со всеми объявлениями.

**Общие требования ко всем моделям:**
- Каждый файл содержит ровно одну доменную структуру + связанные enum
- `derive(Debug, Clone, Serialize, Deserialize)` на всех структурах
- `derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)` на всех enum
- `#[serde(rename_all = "snake_case")]` на enum для читаемых JSON-ключей
- Конструктор `::new(...)` с обязательными полями
- Базовый builder-метод `::builder()` не обязателен, но приветствуется
- `#[cfg(test)] mod tests` в каждом файле с минимум 3 тестами:
  1. Создание через `new()` — все поля доступны
  2. Serialize → JSON → Deserialize → round-trip равенство
  3. Два экземпляра с разными ID не равны

---

#### Stage 2a — Agent A: `daily_log` + `check_in`

**Файлы:**
- `AdiyutantCore/adiyutant_core/src/model/daily_log.rs`
- `AdiyutantCore/adiyutant_core/src/model/check_in.rs`

##### daily_log.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

/// Marker type for DailyLog ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DailyLogId;
// Примечание: DailyLogId — это unit-структура-маркер, не сам ID.
// Id<DailyLogId> — типизированный идентификатор.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLog {
    pub id: Id<Self>,
    pub date: chrono::NaiveDate,
    pub mode: Option<String>,
    pub sleep_score: Option<u8>,
    pub energy: Option<u8>,
    pub mood: Option<u8>,
    pub raw_notes: Option<String>,
    pub ai_summary: Option<String>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}
```

Особенности:
- `id: Id<Self>` — self-referential type parameter (работает в Rust: `Id<DailyLog>`)
- `mode` — свободная строка: `"recovery"`, `"normal"`, `"focus"`, etc.
- `sleep_score: Option<u8>` — 0..100
- `energy: Option<u8>` — 1..10
- `mood: Option<u8>` — 1..10
- `date: chrono::NaiveDate` — только дата, без времени

Конструктор:
```rust
impl DailyLog {
    pub fn new(date: chrono::NaiveDate) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            date,
            mode: None,
            sleep_score: None,
            energy: None,
            mood: None,
            raw_notes: None,
            ai_summary: None,
            created_at: now,
            updated_at: now,
        }
    }
}
```

Тесты:
1. `new()` создаёт запись с заданной датой
2. Все `Option` поля по умолчанию `None`
3. Два `DailyLog` с одной датой имеют разные `id`
4. Serialize/deserialize round-trip

##### check_in.rs

Enum для типа чек-ина и сама модель:

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::daily_log::DailyLog;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckInType {
    Morning,
    Day,
    Evening,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckIn {
    pub id: Id<Self>,
    pub daily_log_id: Id<DailyLog>,
    pub check_in_type: CheckInType,
    pub raw_input: String,
    pub structured_data: Option<serde_json::Value>,
    pub agent_response: Option<String>,
    pub created_at: AdiyutantDateTime,
}
```

Конструктор:
```rust
impl CheckIn {
    pub fn new(daily_log_id: Id<DailyLog>, check_in_type: CheckInType, raw_input: String) -> Self {
        Self {
            id: Id::new(),
            daily_log_id,
            check_in_type,
            raw_input,
            structured_data: None,
            agent_response: None,
            created_at: AdiyutantDateTime::now(),
        }
    }
}
```

Тесты:
1. `new()` создаёт чек-ин с переданными параметрами
2. `CheckInType` serialize → `"morning"`, `"shutdown"` и т.д.
3. Два чек-ина с одним `daily_log_id` имеют разные `id`
4. Serialize/deserialize round-trip с `structured_data = Some(json!({"sleep_hours": 6}))`

---

#### Stage 2b — Agent B: `habit` + `habit_event` + `context_document`

**Файлы:**
- `AdiyutantCore/adiyutant_core/src/model/habit.rs`
- `AdiyutantCore/adiyutant_core/src/model/habit_event.rs`
- `AdiyutantCore/adiyutant_core/src/model/context_document.rs`

##### habit.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: Id<Self>,
    pub name: String,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub target: Option<String>,
    pub is_active: bool,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}
```

Конструктор:
```rust
impl Habit {
    pub fn new(name: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            name,
            description: None,
            frequency: None,
            target: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
```

Тесты (≥3): создание, активность по умолчанию, serde round-trip.

##### habit_event.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::habit::Habit;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitEventStatus {
    Done,
    Skipped,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitEventLevel {
    Min,
    Light,
    Base,
    Full,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitEvent {
    pub id: Id<Self>,
    pub habit_id: Id<Habit>,
    pub date_time: AdiyutantDateTime,
    pub status: HabitEventStatus,
    pub level: HabitEventLevel,
    pub comment: Option<String>,
}
```

Конструктор:
```rust
impl HabitEvent {
    pub fn new(
        habit_id: Id<Habit>,
        status: HabitEventStatus,
        level: HabitEventLevel,
    ) -> Self {
        Self {
            id: Id::new(),
            habit_id,
            date_time: AdiyutantDateTime::now(),
            status,
            level,
            comment: None,
        }
    }
}
```

Тесты (≥3): создание, статусы serialize корректно, serde round-trip.

##### context_document.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextDocumentType {
    Core,
    Goals,
    Rules,
    Routine,
    Health,
    Work,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDocument {
    pub id: Id<Self>,
    pub doc_type: ContextDocumentType,
    pub title: String,
    pub content_markdown: String,
    pub version: u32,
    pub is_active: bool,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}
```

Конструктор:
```rust
impl ContextDocument {
    pub fn new(doc_type: ContextDocumentType, title: String, content_markdown: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            doc_type,
            title,
            content_markdown,
            version: 1,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
```

Тесты (≥3): создание, версия = 1 по умолчанию, serde round-trip с markdown-контентом.

---

#### Stage 2c — Agent C: `reminder` + `alarm` + `timer` + `plan` + `action_proposal`

**Файлы:**
- `AdiyutantCore/adiyutant_core/src/model/reminder.rs`
- `AdiyutantCore/adiyutant_core/src/model/alarm.rs`
- `AdiyutantCore/adiyutant_core/src/model/timer.rs`
- `AdiyutantCore/adiyutant_core/src/model/plan.rs`
- `AdiyutantCore/adiyutant_core/src/model/action_proposal.rs`

##### reminder.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderDefinition {
    pub id: Id<Self>,
    pub title: String,
    pub message: Option<String>,
    pub schedule_rule: String,
    pub next_fire_at: Option<AdiyutantDateTime>,
    pub enabled: bool,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}
```

Конструктор:
```rust
impl ReminderDefinition {
    pub fn new(title: String, schedule_rule: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            title,
            message: None,
            schedule_rule,
            next_fire_at: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}
```

Примечание: `schedule_rule` — строка (позже Cron или свой DSL). В Фазе 2 — свободный формат.

##### alarm.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmDefinition {
    pub id: Id<Self>,
    pub title: String,
    pub time: chrono::NaiveTime,
    pub repeat_rule: Option<String>,
    pub enabled: bool,
    pub platform_binding_id: Option<String>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}
```

Конструктор:
```rust
impl AlarmDefinition {
    pub fn new(title: String, time: chrono::NaiveTime) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            title,
            time,
            repeat_rule: None,
            enabled: true,
            platform_binding_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}
```

##### timer.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Focus,
    Rest,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerDefinition {
    pub id: Id<Self>,
    pub title: String,
    pub duration_seconds: u64,
    pub mode: TimerMode,
    pub created_at: AdiyutantDateTime,
}
```

Конструктор:
```rust
impl TimerDefinition {
    pub fn new(title: String, duration_seconds: u64, mode: TimerMode) -> Self {
        Self {
            id: Id::new(),
            title,
            duration_seconds,
            mode,
            created_at: AdiyutantDateTime::now(),
        }
    }
}
```

##### plan.rs

**Архитектурное решение:** Plan не детализирован в SPECIFICATION.md. Принимается минимальная модель — Plan как контейнер намерений на день, связанный с DailyLog.

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use crate::model::daily_log::DailyLog;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanItemStatus {
    Pending,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub description: String,
    pub status: PlanItemStatus,
    pub estimated_minutes: Option<u32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Id<Self>,
    pub daily_log_id: Id<DailyLog>,
    pub title: String,
    pub items: Vec<PlanItem>,
    pub created_at: AdiyutantDateTime,
    pub updated_at: AdiyutantDateTime,
}
```

Конструктор:
```rust
impl Plan {
    pub fn new(daily_log_id: Id<DailyLog>, title: String) -> Self {
        let now = AdiyutantDateTime::now();
        Self {
            id: Id::new(),
            daily_log_id,
            title,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_item(&mut self, description: String) {
        self.items.push(PlanItem {
            description,
            status: PlanItemStatus::Pending,
            estimated_minutes: None,
            notes: None,
        });
        self.updated_at = AdiyutantDateTime::now();
    }
}
```

Тесты (≥4):
1. `new()` создаёт пустой план
2. `add_item()` добавляет элемент в статусе `Pending`
3. После `add_item` `updated_at` обновляется
4. Serde round-trip с одним элементом

##### action_proposal.rs

```rust
use crate::datetime::AdiyutantDateTime;
use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalSource {
    LocalRule,
    Agent,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProposal {
    pub id: Id<Self>,
    pub source: ProposalSource,
    pub proposal_type: String,
    pub payload_json: serde_json::Value,
    pub status: ProposalStatus,
    pub created_at: AdiyutantDateTime,
    pub applied_at: Option<AdiyutantDateTime>,
}
```

Конструктор:
```rust
impl ActionProposal {
    pub fn new(
        source: ProposalSource,
        proposal_type: String,
        payload_json: serde_json::Value,
    ) -> Self {
        Self {
            id: Id::new(),
            source,
            proposal_type,
            payload_json,
            status: ProposalStatus::Pending,
            created_at: AdiyutantDateTime::now(),
            applied_at: None,
        }
    }
}
```

Тесты (≥3):
1. `new()` создаёт предложение со статусом `Pending`
2. `applied_at` изначально `None`
3. Serde round-trip с JSON-полезной нагрузкой `json!({"action": "add_habit", "name": "speech"})`

---

### Stage 3 — Integration & Verification

**Исполнитель:** orchestrator (после завершения Stage 2a, 2b, 2c).

#### 3a. Проверка, что все файлы созданы

```bash
ls AdiyutantCore/adiyutant_core/src/model/
# Должно быть: mod.rs, daily_log.rs, check_in.rs, habit.rs, habit_event.rs,
#             reminder.rs, alarm.rs, timer.rs, context_document.rs,
#             plan.rs, action_proposal.rs
```

#### 3b. Компиляция

```bash
cargo check -p adiyutant_core
cargo check --workspace
```

#### 3c. Тесты

```bash
cargo test -p adiyutant_core
cargo test --workspace
```

Ожидается, что каждый файл модели содержит `#[cfg(test)] mod tests` со своими тестами.

#### 3d. Форматирование и clippy

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings  # опционально
```

#### 3e. Обновление README

**Файл:** `AdiyutantCore/README.md`

Добавить в описание `adiyutant_core` список моделей:

```markdown
- `adiyutant_core` — base types and domain model:
  - `Id<T>`, `CoreError`, `AdiyutantDateTime`
  - `DailyLog`, `CheckIn`, `Habit`, `HabitEvent`
  - `ReminderDefinition`, `AlarmDefinition`, `TimerDefinition`
  - `ContextDocument`, `Plan`, `ActionProposal`
```

Обновить секцию Next Phase:
```markdown
## Next Phase

Phase 3 — Local Rules and Today State: aggregator + local rule engine.
```

**Acceptance criteria:**
- [ ] `cargo check --workspace` — 0 errors, 0 warnings
- [ ] `cargo test --workspace` — все тесты пройдены (≥30 тестов: минимум 3 на модель)
- [ ] `cargo fmt --all -- --check` — без расхождений
- [ ] Все 10 моделей доступны через `adiyutant_core::model::*`
- [ ] README обновлён

---

## 5. Required Test Levels

| Level | Name | Applicable? | Details |
|---|---|---|---|
| 1 | Static checks | ✅ | `cargo fmt --check`, `cargo check --workspace`, `cargo clippy` |
| 2 | Unit tests | ✅ | Минимум 3 теста на модель: конструктор, поля, serde round-trip |
| 3 | Component tests | ❌ | Чистые структуры данных, нет компонентов |
| 4 | Integration tests | ❌ | Нет real dependencies (SQLite позже) |
| 5 | Stand smoke tests | ❌ | CLI не задействует модели (Фаза 5) |
| 6 | UI automation | ❌ | CLI, не GUI |
| 7 | User scenarios | ❌ | Нет бизнес-логики |
| 8 | Regression pack | ❌ | Второй слой кода, регрессия — тесты Фазы 1 должны проходить |
| 9 | Acceptance review | ✅ | Финальная сверка по чеклисту |

**Регрессия:** тесты Фазы 1 (error.rs, id.rs, datetime.rs) должны проходить без изменений.

---

## 6. Test Stand

Не требуется. Все проверки — локальный `cargo` без внешних зависимостей.

**Требования к окружению:**
- Rust toolchain stable (edition 2024)
- `cargo`, `rustc`, `rustfmt`
- `clippy` (опционально)

---

## 7. Files

### Изменить (4 файла)

| # | File | Change |
|---|---|---|
| 1 | `AdiyutantCore/Cargo.toml` | Добавить `serde_json` в workspace |
| 2 | `AdiyutantCore/adiyutant_core/Cargo.toml` | Добавить `serde_json` в deps |
| 3 | `AdiyutantCore/adiyutant_core/src/lib.rs` | Добавить `pub mod model;` |
| 4 | `AdiyutantCore/README.md` | Обновить описание крейта и Next Phase |

### Создать (11 файлов)

| # | File | Stage |
|---|---|---|
| 5 | `AdiyutantCore/adiyutant_core/src/model/mod.rs` | 1 |
| 6 | `AdiyutantCore/adiyutant_core/src/model/daily_log.rs` | 2a |
| 7 | `AdiyutantCore/adiyutant_core/src/model/check_in.rs` | 2a |
| 8 | `AdiyutantCore/adiyutant_core/src/model/habit.rs` | 2b |
| 9 | `AdiyutantCore/adiyutant_core/src/model/habit_event.rs` | 2b |
| 10 | `AdiyutantCore/adiyutant_core/src/model/context_document.rs` | 2b |
| 11 | `AdiyutantCore/adiyutant_core/src/model/reminder.rs` | 2c |
| 12 | `AdiyutantCore/adiyutant_core/src/model/alarm.rs` | 2c |
| 13 | `AdiyutantCore/adiyutant_core/src/model/timer.rs` | 2c |
| 14 | `AdiyutantCore/adiyutant_core/src/model/plan.rs` | 2c |
| 15 | `AdiyutantCore/adiyutant_core/src/model/action_proposal.rs` | 2c |

### Не изменять

- `adiyutant_store/` — не трогать в этой фазе
- `adiyutant_cli/` — не трогать в этой фазе
- Корневые документы проекта — не трогать

---

## 8. Dependency Graph

```
adiyutant_core
  ├── id.rs          Id<T>
  ├── error.rs       CoreError, CoreResult<T>
  ├── datetime.rs    AdiyutantDateTime
  └── model/
       ├── daily_log.rs       Id<DailyLog>, AdiyutantDateTime, chrono::NaiveDate
       ├── check_in.rs        Id<CheckIn>, Id<DailyLog>, AdiyutantDateTime, serde_json::Value
       ├── habit.rs           Id<Habit>, AdiyutantDateTime
       ├── habit_event.rs     Id<HabitEvent>, Id<Habit>, AdiyutantDateTime
       ├── reminder.rs        Id<ReminderDefinition>, AdiyutantDateTime
       ├── alarm.rs           Id<AlarmDefinition>, AdiyutantDateTime, chrono::NaiveTime
       ├── timer.rs           Id<TimerDefinition>, AdiyutantDateTime
       ├── context_document.rs Id<ContextDocument>, AdiyutantDateTime
       ├── plan.rs            Id<Plan>, Id<DailyLog>, AdiyutantDateTime
       └── action_proposal.rs Id<ActionProposal>, AdiyutantDateTime, serde_json::Value
```

---

## 9. Type Reference (from SPECIFICATION.md §14)

| Сущность | ID type | Поля с внешними типами |
|---|---|---|
| DailyLog | `Id<DailyLog>` | `chrono::NaiveDate`, `Option<u8>` |
| CheckIn | `Id<CheckIn>` | `Id<DailyLog>`, `Option<serde_json::Value>` |
| Habit | `Id<Habit>` | — |
| HabitEvent | `Id<HabitEvent>` | `Id<Habit>`, enum Status, enum Level |
| ReminderDefinition | `Id<ReminderDefinition>` | `Option<AdiyutantDateTime>` (next_fire_at) |
| AlarmDefinition | `Id<AlarmDefinition>` | `chrono::NaiveTime`, `Option<String>` |
| TimerDefinition | `Id<TimerDefinition>` | `u64`, enum Mode |
| ContextDocument | `Id<ContextDocument>` | enum Type, `u32` (version) |
| Plan | `Id<Plan>` | `Id<DailyLog>`, `Vec<PlanItem>` |
| ActionProposal | `Id<ActionProposal>` | enum Source, `serde_json::Value` |

---

## 10. Acceptance Criteria (Final)

- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test --workspace` — все тесты пройдены, включая тесты Фазы 1 (регрессия)
- [ ] `cargo fmt --all -- --check` — без расхождений
- [ ] 10 моделей в `adiyutant_core::model::*`, все публичны
- [ ] Каждая модель имеет `derive(Debug, Clone, Serialize, Deserialize)`
- [ ] `Id<T>` корректно типизирован для каждой модели
- [ ] Все `enum` варианты serialize в `snake_case`
- [ ] `AdiyutantCore/README.md` обновлён
- [ ] `adiyutant_store` и `adiyutant_cli` не изменены и компилируются
- [ ] Крейт не содержит SQLite, LLM, backend, sync-кода
- [ ] Ни одна модель не хранит API-ключи

---

## 11. Risks

| Risk | Mitigation |
|---|---|
| `Id<Self>` в `DailyLog` может не скомпилироваться | Альтернатива: маркерный тип `DailyLogId` + `Id<DailyLogId>` |
| `serde_json::Value` не реализует `PartialEq` для тестов | `PartialEq` реализован для `Value` в serde_json |
| Конфликт имён `Plan` со стандартной библиотекой | Нет конфликта в пользовательском крейте |
| `cargo fmt` изменяет файлы агентов | Допустимо; после форматирования повторить `--check` |

---

## 12. Out of Scope Reminder

- ❌ `TodayState` агрегатор — Фаза 3
- ❌ `LocalRuleAgentGateway` — Фаза 3
- ❌ SQLite, `rusqlite`, миграции — Фаза 4
- ❌ Репозитории, CRUD над моделями — Фаза 4
- ❌ CLI-команды бизнес-логики — Фаза 5
- ❌ Бизнес-валидация (пересечение таймеров, конфликты привычек) — Фаза 3
- ❌ `CalendarEvent` — не входит в Фазу 2 (ROADMAP упоминает «Time objects: reminders, alarms, timers», но не CalendarEvent как приоритет)
- ❌ Изменения `adiyutant_store` или `adiyutant_cli`
