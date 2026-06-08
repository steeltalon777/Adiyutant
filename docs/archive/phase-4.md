# TZ: Phase 4 — SQLite Storage

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-4.md`

## Execution Strategy

- [ ] 🟡 Sequential execution recommended
- **Reason:** Схема SQLite — общий артефакт, от которого зависят все репозитории. Трейт `Store` определён в одном файле (`adiyutant_store/src/lib.rs`) и его имплементация `SqliteStore` разделяет одно `rusqlite::Connection`. Все 10 сущностей вносят изменения в один `impl Store for SqliteStore`-блок или соседние модули того же крейта. При этом `adiyutant_core` (CoreError) и `adiyutant_store` (трейт + имплементация) изменяются в разных крейтах, что исключает конфликты на уровне workspace, но внутри `adiyutant_store` нужна последовательная сборка.

## Execution Checklist

- [x] 0. Context verified
- [x] 1. Dependencies added: `rusqlite` into workspace and `adiyutant_store`
- [x] 2. `CoreError` extended with `Storage` variant in `adiyutant_core`
- [x] 3. `Store` trait expanded with CRUD methods for all 10 domain entities
- [x] 4. Schema bootstrap implemented (`CREATE TABLE IF NOT EXISTS` for all tables)
- [x] 5. `SqliteStore` struct + `impl Store for SqliteStore` with all repository methods
- [x] 6. Static checks: `cargo fmt`, `cargo check --workspace`
- [x] 7. Unit tests: row mapping, JSON serialization helpers
- [x] 8. Integration tests: real SQLite (in-memory), CRUD for every entity
- [x] 9. Regression: `cargo test --workspace` — тесты Фаз 1-3 проходят
- [x] 10. Documentation updated (ROADMAP Phase 4 marked done)
- [x] 11. Final acceptance review

## Check Rules

- Architect создаёт checklist и acceptance criteria.
- Executor agent отмечает пункты только после реализации и верификации.
- Если проверка не прошла — пункт остаётся незаполненным с причиной.
- QA verifier проверяет финальную приёмку только после review evidence.

---

## 1. Source Documents

| Документ | Роль |
|---|---|
| `ROADMAP.md` — Phase 4 | Источник scope |
| `ARCHITECTURE.md` — Core Boundary | Core owns local SQLite storage |
| `SPECIFICATION.md` (при наличии) | Поведение доменных сущностей |
| `AGENTS.md` | Правила репозитория |
| `docs/core-boundary.md` | Границы Core |
| `AdiyutantCore/adiyutant_store/src/lib.rs` | Текущий `Store` trait |
| `AdiyutantCore/adiyutant_core/src/error.rs` | Текущий `CoreError` |
| `AdiyutantCore/adiyutant_core/src/model/*.rs` | Доменные модели (10 сущностей) |

---

## 2. Goal

Сделать локальное состояние персистентным через SQLite:

- **SQLite schema bootstrap** — создание всех таблиц при первом запуске
- **Repositories for all domain entities** — полный CRUD для 10 доменных сущностей через `Store` trait

`adiyutant_store` получает реальную реализацию `SqliteStore` взамен (или в дополнение к) `NoopStore`. `adiyutant_core` получает вариант `CoreError::Storage` для передачи storage-ошибок.

---

## 3. Scope

### In scope

| # | Что | Где |
|---|---|---|
| 1 | Зависимость `rusqlite` (bundled) | Workspace `Cargo.toml` + `adiyutant_store/Cargo.toml` |
| 2 | `CoreError::Storage(String)` | `adiyutant_core/src/error.rs` |
| 3 | Расширение `Store` trait: `migrate()`, CRUD-методы для 10 сущностей | `adiyutant_store/src/lib.rs` |
| 4 | SQLite schema: `CREATE TABLE IF NOT EXISTS` для всех таблиц | `adiyutant_store/src/lib.rs` (или `schema.rs`) |
| 5 | `SqliteStore` struct + `impl Store for SqliteStore` | `adiyutant_store/src/lib.rs` |
| 6 | Mapping row → domain struct для каждой сущности | В имплементации |
| 7 | Integration tests: in-memory SQLite, CRUD для каждой сущности | `adiyutant_store/src/lib.rs` (модуль `#[cfg(test)]`) |

### Out of scope

- ❌ Миграции версий (только `CREATE TABLE IF NOT EXISTS` для MVP)
- ❌ Индексы (кроме PRIMARY KEY по `id`)
- ❌ Внешние ключи (FK объявляются, но `PRAGMA foreign_keys` для MVP опционально)
- ❌ Транзакции и batch-операции
- ❌ Connection pool / многопоточный доступ
- ❌ `adiyutant_cli` — CLI не меняется в этой фазе
- ❌ Изменения в доменных моделях `adiyutant_core/src/model/*`
- ❌ LLM, Agent Gateway, backend, sync
- ❌ `TodayState` или `LocalRuleAgentGateway` (не требуют изменений)

---

## 4. Implementation Stages

### Stage 0 — Предварительные условия

Перед началом убедиться:
- [ ] `cargo check --workspace` проходит (текущий код Фаз 1-3)
- [ ] `cargo test --workspace` — все существующие тесты зелёные
- [ ] Workspace `Cargo.toml` (`AdiyutantCore/Cargo.toml`) готов к добавлению workspace-зависимости

---

### Stage 1 — Зависимости и CoreError

**Файлы:**
- `AdiyutantCore/Cargo.toml` — добавить workspace-зависимость `rusqlite`
- `AdiyutantCore/adiyutant_store/Cargo.toml` — добавить `rusqlite` с features
- `AdiyutantCore/adiyutant_core/src/error.rs` — добавить `CoreError::Storage`

**Шаги:**

1. В workspace `Cargo.toml`:
```toml
[workspace.dependencies]
# ... существующие ...
rusqlite = { version = "0.32", features = ["bundled"] }
```

2. В `adiyutant_store/Cargo.toml`:
```toml
[dependencies]
adiyutant_core = { path = "../adiyutant_core" }
rusqlite = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
```

3. В `adiyutant_core/src/error.rs` добавить вариант:
```rust
#[derive(Debug, Error)]
pub enum CoreError {
    // ... существующие ...
    #[error("storage error: {0}")]
    Storage(String),
}
```

**Acceptance criteria:**
- [ ] `cargo check` в `adiyutant_core` и `adiyutant_store` проходит
- [ ] `CoreError::Storage("msg".into())` компилируется
- [ ] `rusqlite` с `bundled` резолвится без системного `libsqlite3`

---

### Stage 2 — Расширение `Store` trait

**Файл:** `AdiyutantCore/adiyutant_store/src/lib.rs`

Расширить `pub trait Store` методами для всех 10 сущностей.

**Сигнатуры методов (по сущностям):**

```rust
use adiyutant_core::error::CoreError;
use adiyutant_core::id::Id;
use adiyutant_core::model::*;
use chrono::NaiveDate;

pub trait Store {
    type Error;

    // Bootstrap
    fn migrate(&self) -> Result<(), Self::Error>;
    fn health_check(&self) -> Result<(), Self::Error>;

    // ── DailyLog ──
    fn insert_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error>;
    fn get_daily_log(&self, id: Id<DailyLog>) -> Result<Option<DailyLog>, Self::Error>;
    fn get_daily_log_by_date(&self, date: NaiveDate) -> Result<Option<DailyLog>, Self::Error>;
    fn list_daily_logs(&self) -> Result<Vec<DailyLog>, Self::Error>;
    fn update_daily_log(&self, log: &DailyLog) -> Result<(), Self::Error>;
    fn delete_daily_log(&self, id: Id<DailyLog>) -> Result<(), Self::Error>;

    // ── CheckIn ──
    fn insert_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error>;
    fn get_check_in(&self, id: Id<CheckIn>) -> Result<Option<CheckIn>, Self::Error>;
    fn list_check_ins_by_log(&self, daily_log_id: Id<DailyLog>) -> Result<Vec<CheckIn>, Self::Error>;
    fn update_check_in(&self, ci: &CheckIn) -> Result<(), Self::Error>;
    fn delete_check_in(&self, id: Id<CheckIn>) -> Result<(), Self::Error>;

    // ── Habit ──
    fn insert_habit(&self, h: &Habit) -> Result<(), Self::Error>;
    fn get_habit(&self, id: Id<Habit>) -> Result<Option<Habit>, Self::Error>;
    fn list_habits(&self) -> Result<Vec<Habit>, Self::Error>;
    fn update_habit(&self, h: &Habit) -> Result<(), Self::Error>;
    fn delete_habit(&self, id: Id<Habit>) -> Result<(), Self::Error>;

    // ── HabitEvent ──
    fn insert_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error>;
    fn get_habit_event(&self, id: Id<HabitEvent>) -> Result<Option<HabitEvent>, Self::Error>;
    fn list_habit_events_by_habit(&self, habit_id: Id<Habit>) -> Result<Vec<HabitEvent>, Self::Error>;
    fn update_habit_event(&self, he: &HabitEvent) -> Result<(), Self::Error>;
    fn delete_habit_event(&self, id: Id<HabitEvent>) -> Result<(), Self::Error>;

    // ── ReminderDefinition ──
    fn insert_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error>;
    fn get_reminder(&self, id: Id<ReminderDefinition>) -> Result<Option<ReminderDefinition>, Self::Error>;
    fn list_reminders(&self) -> Result<Vec<ReminderDefinition>, Self::Error>;
    fn update_reminder(&self, r: &ReminderDefinition) -> Result<(), Self::Error>;
    fn delete_reminder(&self, id: Id<ReminderDefinition>) -> Result<(), Self::Error>;

    // ── AlarmDefinition ──
    fn insert_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error>;
    fn get_alarm(&self, id: Id<AlarmDefinition>) -> Result<Option<AlarmDefinition>, Self::Error>;
    fn list_alarms(&self) -> Result<Vec<AlarmDefinition>, Self::Error>;
    fn update_alarm(&self, a: &AlarmDefinition) -> Result<(), Self::Error>;
    fn delete_alarm(&self, id: Id<AlarmDefinition>) -> Result<(), Self::Error>;

    // ── TimerDefinition ──
    fn insert_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error>;
    fn get_timer(&self, id: Id<TimerDefinition>) -> Result<Option<TimerDefinition>, Self::Error>;
    fn list_timers(&self) -> Result<Vec<TimerDefinition>, Self::Error>;
    fn update_timer(&self, t: &TimerDefinition) -> Result<(), Self::Error>;
    fn delete_timer(&self, id: Id<TimerDefinition>) -> Result<(), Self::Error>;

    // ── ContextDocument ──
    fn insert_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error>;
    fn get_context_document(&self, id: Id<ContextDocument>) -> Result<Option<ContextDocument>, Self::Error>;
    fn list_context_documents(&self) -> Result<Vec<ContextDocument>, Self::Error>;
    fn update_context_document(&self, cd: &ContextDocument) -> Result<(), Self::Error>;
    fn delete_context_document(&self, id: Id<ContextDocument>) -> Result<(), Self::Error>;

    // ── Plan ──
    fn insert_plan(&self, p: &Plan) -> Result<(), Self::Error>;
    fn get_plan(&self, id: Id<Plan>) -> Result<Option<Plan>, Self::Error>;
    fn get_plan_by_daily_log(&self, daily_log_id: Id<DailyLog>) -> Result<Option<Plan>, Self::Error>;
    fn update_plan(&self, p: &Plan) -> Result<(), Self::Error>;
    fn delete_plan(&self, id: Id<Plan>) -> Result<(), Self::Error>;

    // ── ActionProposal ──
    fn insert_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error>;
    fn get_action_proposal(&self, id: Id<ActionProposal>) -> Result<Option<ActionProposal>, Self::Error>;
    fn list_action_proposals(&self) -> Result<Vec<ActionProposal>, Self::Error>;
    fn list_action_proposals_by_status(&self, status: ProposalStatus) -> Result<Vec<ActionProposal>, Self::Error>;
    fn update_action_proposal(&self, ap: &ActionProposal) -> Result<(), Self::Error>;
    fn delete_action_proposal(&self, id: Id<ActionProposal>) -> Result<(), Self::Error>;
}
```

**Acceptance criteria:**
- [ ] Трейт компилируется (без имплементации — достаточно `cargo check`)
- [ ] Все методы имеют осмысленные сигнатуры (принимают и возвращают domain-типы, не сырой SQL)
- [ ] Ассоциированный тип `Error` остаётся (позволяет разным реализациям выбирать свой тип ошибки)

---

### Stage 3 — Schema Bootstrap

**Файл:** `AdiyutantCore/adiyutant_store/src/lib.rs` (или отдельный `schema.rs`)

Создать функцию `run_migration(conn: &rusqlite::Connection) -> Result<(), CoreError>` которая выполняет `CREATE TABLE IF NOT EXISTS` для всех 10 сущностей.

**Конвенции колонок:**
- `id` — `TEXT PRIMARY KEY` (UUID v4 в строковом формате)
- Все foreign key — `TEXT REFERENCES table(id)`
- Опциональные поля — без `NOT NULL`
- Даты/время — `TEXT` (ISO 8601)
- Булевы — `INTEGER NOT NULL DEFAULT 1` (0/1)
- Числовые — `INTEGER`
- JSON-поля (Plan.items, ActionProposal.payload_json, CheckIn.structured_data) — `TEXT`

**Схема (10 таблиц):**

```sql
CREATE TABLE IF NOT EXISTS daily_logs (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    mode TEXT,
    sleep_score INTEGER,
    energy INTEGER,
    mood INTEGER,
    raw_notes TEXT,
    ai_summary TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS check_ins (
    id TEXT PRIMARY KEY,
    daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
    check_in_type TEXT NOT NULL,
    raw_input TEXT NOT NULL,
    structured_data TEXT,
    agent_response TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS habits (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    frequency TEXT,
    target TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS habit_events (
    id TEXT PRIMARY KEY,
    habit_id TEXT NOT NULL REFERENCES habits(id),
    date_time TEXT NOT NULL,
    status TEXT NOT NULL,
    level TEXT NOT NULL,
    comment TEXT
);

CREATE TABLE IF NOT EXISTS reminders (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    message TEXT,
    schedule_rule TEXT NOT NULL,
    next_fire_at TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS alarms (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    time TEXT NOT NULL,
    repeat_rule TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    platform_binding_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS timers (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    mode TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS context_documents (
    id TEXT PRIMARY KEY,
    doc_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY,
    daily_log_id TEXT NOT NULL REFERENCES daily_logs(id),
    title TEXT NOT NULL,
    items_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS action_proposals (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    proposal_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    applied_at TEXT
);
```

**Acceptance criteria:**
- [ ] `run_migration(&conn)` выполняется без ошибок на свежей in-memory БД
- [ ] Повторный вызов идемпотентен (не ломается на существующих таблицах)
- [ ] После миграции `SELECT name FROM sqlite_master WHERE type='table'` возвращает 10 таблиц

---

### Stage 4 — SqliteStore и репозитории

**Файл:** `AdiyutantCore/adiyutant_store/src/lib.rs`

**Структура:**
```rust
pub struct SqliteStore {
    conn: rusqlite::Connection,
}

impl SqliteStore {
    pub fn new(path: &str) -> Result<Self, CoreError> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { conn })
    }

    pub fn new_in_memory() -> Result<Self, CoreError> {
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { conn })
    }
}

impl Store for SqliteStore {
    type Error = CoreError;

    fn migrate(&self) -> Result<(), Self::Error> { /* ... */ }
    fn health_check(&self) -> Result<(), Self::Error> { /* ... */ }
    // ... все CRUD-методы ...
}
```

**Mapping-правила:**

| Rust-тип | SQLite-тип | Сериализация |
|---|---|---|
| `Id<T>` | `TEXT` | `id.value().to_string()` / `Uuid::parse_str(s)` |
| `AdiyutantDateTime` | `TEXT` | `dt.inner().to_rfc3339()` / парсинг обратно |
| `NaiveDate` | `TEXT` | `date.to_string()` / `NaiveDate::parse_from_str(s, "%Y-%m-%d")` |
| `NaiveTime` | `TEXT` | `time.to_string()` / `NaiveTime::parse_from_str(s, "%H:%M:%S")` |
| `bool` | `INTEGER` | `if b { 1 } else { 0 }` |
| `Option<T>` | `T NULL` | `None` → SQL NULL |
| `Vec<PlanItem>` | `TEXT` | `serde_json::to_string(&items)` / `serde_json::from_str(s)` |
| `serde_json::Value` | `TEXT` | `value.to_string()` / `serde_json::from_str(s)` |
| enum (CheckInType, etc.) | `TEXT` | snake_case через `serde_json::to_value` / `from_value` |
| `u8`, `u32`, `u64` | `INTEGER` | `as i64` |

**Обработка ошибок:**
Все ошибки `rusqlite` мапятся в `CoreError::Storage(msg)`. Ошибки `serde_json` мапятся в `CoreError::Internal(msg)`. Ошибки парсинга UUID / дат мапятся в `CoreError::InvalidInput(msg)`.

**Для `NoopStore`:**
- Добавить реализации-заглушки для всех новых методов трейта (возвращать `Ok(())`, `Ok(None)`, `Ok(vec![])`), чтобы сохранить обратную совместимость.

**Acceptance criteria:**
- [ ] `SqliteStore::new_in_memory()` создаёт подключение без ошибок
- [ ] `migrate()` создаёт все 10 таблиц
- [ ] `health_check()` возвращает `Ok(())` (выполняет `SELECT 1`)
- [ ] Каждый метод `insert_*` вставляет одну строку и возвращает `Ok(())`
- [ ] Каждый метод `get_*` возвращает `Some(entity)` по валидному id и `None` по несуществующему
- [ ] Каждый `list_*` возвращает вектор (пустой для свежей БД)
- [ ] Каждый `update_*` изменяет существующую запись
- [ ] Каждый `delete_*` удаляет запись
- [ ] `NoopStore` реализует все методы трейта (заглушки)
- [ ] `cargo check --workspace` — 0 errors

---

### Stage 5 — Интеграционные тесты

**Файл:** `AdiyutantCore/adiyutant_store/src/lib.rs` (модуль `#[cfg(test)]`)

Для каждой сущности — минимум один интеграционный тест с реальной in-memory SQLite БД:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use adiyutant_core::model::*;
    use adiyutant_core::id::Id;

    fn setup() -> SqliteStore {
        let store = SqliteStore::new_in_memory().unwrap();
        store.migrate().unwrap();
        store
    }

    #[test]
    fn insert_and_get_daily_log() {
        let store = setup();
        let log = DailyLog::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap());
        store.insert_daily_log(&log).unwrap();
        let retrieved = store.get_daily_log(log.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, log.id);
    }

    // ... тесты для всех сущностей ...
}
```

**Минимальный набор тестов (по одному на сущность, всего 10+):**

| # | Тест | Сущность |
|---|---|---|
| 1 | `insert_and_get_daily_log` | DailyLog |
| 2 | `get_daily_log_by_date` | DailyLog |
| 3 | `insert_and_get_check_in` | CheckIn |
| 4 | `list_check_ins_by_log` | CheckIn |
| 5 | `insert_and_get_habit` | Habit |
| 6 | `insert_and_get_habit_event` | HabitEvent |
| 7 | `insert_and_get_reminder` | ReminderDefinition |
| 8 | `insert_and_get_alarm` | AlarmDefinition |
| 9 | `insert_and_get_timer` | TimerDefinition |
| 10 | `insert_and_get_context_document` | ContextDocument |
| 11 | `insert_and_get_plan_with_items` | Plan (с JSON-сериализацией items) |
| 12 | `insert_and_get_action_proposal` | ActionProposal |
| 13 | `update_daily_log` | DailyLog (update) |
| 14 | `delete_daily_log` | DailyLog (delete) |
| 15 | `get_nonexistent_returns_none` | Любая сущность |
| 16 | `noop_store_methods_dont_panic` | NoopStore |

**Acceptance criteria:**
- [ ] `cargo test -p adiyutant_store` — все 16+ тестов зелёные
- [ ] Каждый тест использует реальную in-memory SQLite (не замоканную)
- [ ] `NoopStore` тест проверяет, что заглушки не паникуют

---

### Stage 6 — Регрессия и статические проверки

**Команды:**
```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings  # если clippy установлен
```

**Acceptance criteria:**
- [ ] `cargo fmt` — без расхождений
- [ ] `cargo check --workspace` — 0 errors, 0 warnings (кроме допустимых)
- [ ] `cargo test --workspace` — все тесты Фаз 1-4 зелёные
- [ ] `adiyutant_cli` компилируется (зависит от `adiyutant_store`)

---

## 5. Required Test Levels

| Level | Name | Applicable? | Details |
|---|---|---|---|
| 1 | Static checks | ✅ | `cargo fmt --check`, `cargo check --workspace`, `cargo clippy` |
| 2 | Unit tests | ✅ | Mapping-функции (row → struct), JSON-сериализация PlanItem, парсинг UUID/дат |
| 3 | Component tests | ❌ | Нет UI-компонентов |
| 4 | Integration tests | ✅ | **Real SQLite in-memory** — CRUD для каждой сущности |
| 5 | Stand smoke tests | ✅ | `SqliteStore::new(":memory:")` + `migrate()` + insert/get в одном тесте |
| 6 | UI automation | ❌ | CLI, не GUI |
| 7 | User scenarios | ❌ | CLI-команды — Фаза 5 |
| 8 | Regression pack | ✅ | `cargo test --workspace` — тесты Фаз 1-3 проходят без изменений |
| 9 | Acceptance review | ✅ | Финальная сверка по чеклисту |

**Уровни 4 и 5 объединены:** реальная SQLite in-memory в тестах покрывает и интеграцию, и smoke (нет отдельного процесса БД — SQLite встроенная).

---

## 6. Test Stand

### SQLite Database

| Параметр | Значение |
|---|---|
| **Тип** | SQLite 3.x (встроенная через `rusqlite` с feature `bundled`) |
| **Режим тестов** | In-memory (`:memory:`) — создаётся заново для каждого теста |
| **Миграция** | `run_migration()` вызывается в `setup()` каждого теста |
| **Жизненный цикл** | БД создаётся в `setup()`, автоматически уничтожается при дропе `Connection` |

### Seed Data

Не требуется. Все тесты создают собственные данные через доменные конструкторы (`DailyLog::new()`, `Habit::new()`, etc.).

### Environment

| Переменная | Назначение |
|---|---|
| *не требуется* | SQLite bundled — внешних зависимостей нет |

### Health Check

```bash
cargo test -p adiyutant_store -- --nocapture
```

### Reset / Cleanup

```bash
cargo clean -p adiyutant_store
cargo test -p adiyutant_store
```

---

## 7. Files

### Изменить (4 файла)

| # | File | Change |
|---|---|---|
| 1 | `AdiyutantCore/Cargo.toml` | Добавить `rusqlite` в `[workspace.dependencies]` |
| 2 | `AdiyutantCore/adiyutant_store/Cargo.toml` | Добавить `rusqlite`, `serde_json`, `chrono` |
| 3 | `AdiyutantCore/adiyutant_core/src/error.rs` | Добавить `CoreError::Storage(String)` |
| 4 | `AdiyutantCore/adiyutant_store/src/lib.rs` | Расширить `Store` trait, добавить `SqliteStore`, `run_migration`, тесты. `NoopStore` получает заглушки для новых методов. |

### Не изменять

- `AdiyutantCore/adiyutant_core/src/model/*` — все 10 доменных моделей
- `AdiyutantCore/adiyutant_core/src/today_state.rs`
- `AdiyutantCore/adiyutant_core/src/local_rule_gateway.rs`
- `AdiyutantCore/adiyutant_core/src/id.rs`
- `AdiyutantCore/adiyutant_core/src/datetime.rs`
- `AdiyutantCore/adiyutant_cli/` — CLI не трогаем (Фаза 5)
- Корневые документы проекта (кроме ROADMAP.md — опционально обновить статус)

---

## 8. Acceptance Criteria (Final)

- [ ] `cargo fmt --all -- --check` — без расхождений
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test --workspace` — все тесты зелёные, включая:
  - Тесты Фаз 1-3 (регрессия)
  - 16+ интеграционных тестов с real SQLite in-memory
- [ ] `cargo clippy --workspace -- -D warnings` — без ошибок (опционально, если установлен)
- [ ] `Store` trait содержит методы для всех 10 доменных сущностей
- [ ] `SqliteStore` реализует `Store` с `type Error = CoreError`
- [ ] `migrate()` создаёт 10 таблиц, идемпотентен
- [ ] `health_check()` возвращает `Ok(())`
- [ ] `NoopStore` реализует все методы трейта (заглушки) и не паникует
- [ ] `adiyutant_cli` компилируется без изменений
- [ ] Все ошибки мапятся в `CoreError` (нет голых `rusqlite::Error` в сигнатурах)
- [ ] Нет SQL-инъекций: все значения передаются через параметры `rusqlite`, не через форматирование строк
- [ ] `Plan.items` и `ActionProposal.payload_json` корректно сериализуются/десериализуются через JSON

---

## 9. Risks

| Risk | Mitigation |
|---|---|
| `rusqlite` не компилируется без системного `libsqlite3-dev` | Используем feature `bundled` — SQLite компилируется из исходников |
| Большой объём trait-методов (~50 сигнатур) | Группировать методы по сущностям с комментариями. Каждый метод — 3-10 строк. |
| `Plan.items` — вложенный Vec, сложная (де)сериализация | Хранить как JSON TEXT. Использовать `serde_json` для конверсии. |
| `NoopStore` — нужно реализовать все методы-заглушки | Простые однострочники: `Ok(())`, `Ok(None)`, `Ok(vec![])` |
| `serde_json::Value` и `Option<serde_json::Value>` в CheckIn/ActionProposal | Конвертировать через `serde_json::to_string` / `from_str`. NULL в SQL — это Rust `None`. |
| UUID парсинг может падать на битых данных | Обрабатывать `uuid::Error` → `CoreError::InvalidInput` |
| Миграция на существующей БД может конфликтовать | `CREATE TABLE IF NOT EXISTS` идемпотентна. ALTER TABLE для MVP не требуется. |

---

## 10. Out of Scope Reminder

- ❌ Версионирование схемы / миграции (только bootstrap)
- ❌ Индексы, оптимизация запросов
- ❌ Транзакции, batch-вставки
- ❌ Connection pool, многопоточный доступ
- ❌ `adiyutant_cli` — CLI-команды (Фаза 5)
- ❌ `TodayState` через SQLite (остаётся in-memory builder)
- ❌ `LocalRuleAgentGateway` через SQLite (остаётся без изменений)
- ❌ LLM, Agent Gateway, backend, sync
- ❌ Изменения в доменных моделях (не добавлять поля, не менять типы)

---

## 11. Evidence Table (to be filled by executor)

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static checks | `cargo fmt --all -- --check` | | |
| Static checks | `cargo check --workspace` | | |
| Unit tests | `cargo test -p adiyutant_store` | | |
| Integration tests (SQLite) | `cargo test -p adiyutant_store -- --nocapture` | | |
| Regression | `cargo test --workspace` | | |
| NoopStore | `cargo test -p adiyutant_store -- noop` | | |
| CLI compiles | `cargo check -p adiyutant_cli` | | |
| Clippy (optional) | `cargo clippy --workspace -- -D warnings` | | |
