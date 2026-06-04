# TZ: Phase 1 — Rust Core Workspace

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-1-rust-core-workspace.md`

## Execution Strategy

- [ ] 🟡 Sequential execution recommended
- **Reason:** Workspace → `adiyutant_core` → `adiyutant_store` → `adiyutant_cli` — естественный порядок зависимостей. `adiyutant_store` зависит от `adiyutant_core`, `adiyutant_cli` — от обоих. Workspace-зависимости должны быть объявлены до любого крейта. Размер работы мал (≈10 файлов), параллелизация не даст выигрыша и создаст риск конфликтов при разрешении path-зависимостей.

## Execution Checklist

- [ ] 0. Context verified
- [ ] 1. Stage 1: Workspace root `Cargo.toml` created
- [ ] 2. Stage 2: `adiyutant_core` crate with error model and base primitives
- [ ] 3. Stage 3: `adiyutant_store` crate skeleton
- [ ] 4. Stage 4: `adiyutant_cli` crate skeleton
- [ ] 5. Static checks: `cargo fmt`, `cargo check` — весь workspace
- [ ] 6. Unit tests: `cargo test` — весь workspace
- [ ] 7. Documentation updated: `AdiyutantCore/README.md`
- [ ] 8. Integration: `cargo check --workspace` после всех крейтов
- [ ] 9. Final acceptance review

## Check Rules

- Architect создаёт checklist и acceptance criteria.
- Executor agent отмечает пункты только после реализации и верификации.
- Если проверка не прошла — пункт остаётся незаполненным с причиной в отчёте.

---

## 1. Source Documents

Работа основана на следующих документах проекта:

| Документ | Роль |
|---|---|
| `ROADMAP.md` — Phase 1 | Источник scope |
| `SPECIFICATION.md` — §4.1, §18 день 1 | Состав AdiyutantCore |
| `ARCHITECTURE.md` — Core Boundary | Что Core owns |
| `docs/core-boundary.md` | Границы Core |
| `GLOSSARY.md` | Термины домена |
| `AGENTS.md` | Правила репозитория |
| `AdiyutantCore/README.md` | Текущее состояние (planned) |

---

## 2. Goal

Получить рабочий Rust workspace (`AdiyutantCore/`) без доменной сложности: три крейта с полной компиляцией, пройденными тестами, моделью ошибок и базовыми примитивами.

---

## 3. Scope

### In scope

- Workspace `Cargo.toml` в `AdiyutantCore/` с `[workspace]` секцией
- Workspace-level зависимости: `chrono`, `serde`, `uuid`, `thiserror`
- `adiyutant_core` — библиотечный крейт с модулями: `id`, `error`, `datetime`
- `adiyutant_store` — библиотечный крейт, зависит от `adiyutant_core`, trait `Store` (скелет)
- `adiyutant_cli` — бинарный крейт, зависит от `adiyutant_core`, `adiyutant_store`, `clap`, skeleton `main.rs`
- Обновление `AdiyutantCore/README.md` со статусом `Current`

### Out of scope

- Доменные модели (DailyLog, Habit и т.д.) — Фаза 2
- SQLite, миграции, схемы — Фаза 4
- Реальная реализация `Store` trait — Фаза 4
- CLI-команды бизнес-логики — Фаза 5
- LLM, Agent Gateway, sync, backend
- Изменения в `.gitignore` (уже содержит `target/`, `Cargo.lock`)

---

## 4. Implementation Stages

### Stage 1 — Workspace Root `Cargo.toml`

**Файл:** `AdiyutantCore/Cargo.toml`

Секция `[workspace]` с тремя members и workspace-level зависимостями.

```toml
[workspace]
resolver = "2"
members = [
    "adiyutant_core",
    "adiyutant_store",
    "adiyutant_cli",
]

[workspace.dependencies]
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "2"
```

**Acceptance criteria:**
- [ ] Файл существует и содержит валидный TOML
- [ ] Секция `[workspace]` объявляет все три крейта
- [ ] `[workspace.dependencies]` содержит `chrono`, `serde`, `uuid`, `thiserror`
- [ ] Версии зависимостей совместимы с Rust edition 2024

**Удалить:** `AdiyutantCore/.gitkeep` — больше не нужен, директория не пустая.

---

### Stage 2 — `adiyutant_core` Crate

#### 2a. Crate manifest

**Файл:** `AdiyutantCore/adiyutant_core/Cargo.toml`

```toml
[package]
name = "adiyutant_core"
version = "0.1.0"
edition = "2024"

[dependencies]
chrono = { workspace = true }
serde = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
```

#### 2b. Library root

**Файл:** `AdiyutantCore/adiyutant_core/src/lib.rs`

Экспортирует три публичных модуля: `id`, `error`, `datetime`.

```rust
pub mod id;
pub mod error;
pub mod datetime;
```

#### 2c. Error model

**Файл:** `AdiyutantCore/adiyutant_core/src/error.rs`

Базовый тип ошибок через `thiserror::Error`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
```

**Поля перечисления:** `InvalidInput`, `NotFound`, `Internal` — минимальный набор для начала. Расширение — в следующих фазах.

#### 2d. Id type

**Файл:** `AdiyutantCore/adiyutant_core/src/id.rs`

Типизированный идентификатор:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id<T> {
    value: Uuid,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new() -> Self {
        Self {
            value: Uuid::new_v4(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn value(&self) -> Uuid {
        self.value
    }
}
```

#### 2e. DateTime type

**Файл:** `AdiyutantCore/adiyutant_core/src/datetime.rs`

Обёртка над `chrono::DateTime<Utc>`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AdiyutantDateTime(DateTime<Utc>);

impl AdiyutantDateTime {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn inner(&self) -> DateTime<Utc> {
        self.0
    }
}
```

#### 2f. Unit tests

Добавить модуль `#[cfg(test)]` в `lib.rs` или отдельный `tests/`:

- Тест создания `Id<T>`: два последовательных `Id::new()` не равны
- Тест `CoreResult<T>`: `Ok` и `Err` корректно конструируются
- Тест `AdiyutantDateTime::now()`: возвращает non-zero значение
- Тест `CoreError::display()`: проверка форматирования

**Acceptance criteria:**
- [ ] Все 4 файла крейта созданы
- [ ] `cargo check -p adiyutant_core` проходит без ошибок
- [ ] `cargo test -p adiyutant_core` проходит, все тесты зелёные
- [ ] Публичный API: `Id<T>`, `CoreError`, `CoreResult<T>`, `AdiyutantDateTime`

---

### Stage 3 — `adiyutant_store` Crate

#### 3a. Crate manifest

**Файл:** `AdiyutantCore/adiyutant_store/Cargo.toml`

```toml
[package]
name = "adiyutant_store"
version = "0.1.0"
edition = "2024"

[dependencies]
adiyutant_core = { path = "../adiyutant_core" }
```

#### 3b. Library root

**Файл:** `AdiyutantCore/adiyutant_store/src/lib.rs`

Trait `Store` — скелет с ассоциированным типом ошибки:

```rust
use adiyutant_core::error::CoreError;

pub trait Store {
    type Error;

    fn health_check(&self) -> Result<(), Self::Error>;
}

pub struct NoopStore;

impl Store for NoopStore {
    type Error = CoreError;

    fn health_check(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_store_health_check_ok() {
        let store = NoopStore;
        assert!(store.health_check().is_ok());
    }
}
```

**Acceptance criteria:**
- [ ] `Cargo.toml` и `src/lib.rs` созданы
- [ ] `cargo check -p adiyutant_store` проходит
- [ ] `cargo test -p adiyutant_store` проходит, тест зелёный
- [ ] Крейт корректно ссылается на `adiyutant_core` через path

---

### Stage 4 — `adiyutant_cli` Crate

#### 4a. Crate manifest

**Файл:** `AdiyutantCore/adiyutant_cli/Cargo.toml`

```toml
[package]
name = "adiyutant_cli"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "adiyutant"
path = "src/main.rs"

[dependencies]
adiyutant_core = { path = "../adiyutant_core" }
adiyutant_store = { path = "../adiyutant_store" }
clap = { version = "4", features = ["derive"] }
```

#### 4b. Binary entrypoint

**Файл:** `AdiyutantCore/adiyutant_cli/src/main.rs`

Skeleton с clap:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "adiyutant")]
#[command(about = "Adiyutant — local-first organizer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Show current status
    Status,
    /// Health check
    Health,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status) => {
            println!("Adiyutant Core v0.1.0 — status: ok");
        }
        Some(Commands::Health) => {
            let store = adiyutant_store::NoopStore;
            match store.health_check() {
                Ok(()) => println!("health: ok"),
                Err(e) => eprintln!("health: {e}"),
            }
        }
        None => {
            println!("Adiyutant CLI v0.1.0. Use --help for commands.");
        }
    }
}
```

#### 4c. Smoke test (manual)

После сборки:

```bash
cargo run -p adiyutant_cli -- status
cargo run -p adiyutant_cli -- health
cargo run -p adiyutant_cli -- --help
```

**Acceptance criteria:**
- [ ] `Cargo.toml` и `src/main.rs` созданы
- [ ] `cargo check -p adiyutant_cli` проходит
- [ ] `cargo run -p adiyutant_cli -- status` выводит `Adiyutant Core v0.1.0 — status: ok`
- [ ] `cargo run -p adiyutant_cli -- health` выводит `health: ok`
- [ ] `cargo run -p adiyutant_cli -- --help` показывает usage

---

### Stage 5 — Workspace-wide Verification

**Команды:**

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings  # если clippy установлен
```

**Acceptance criteria:**
- [ ] `cargo fmt` — без изменений (все файлы уже отформатированы)
- [ ] `cargo check --workspace` — без ошибок
- [ ] `cargo test --workspace` — все тесты зелёные
- [ ] `cargo clippy` — без warning (опционально, только если clippy доступен)

---

### Stage 6 — Documentation Update

**Файл:** `AdiyutantCore/README.md`

Заменить текущее содержимое на:

```markdown
# AdiyutantCore

Status: Current — Rust workspace active.

AdiyutantCore is the Rust local-first domain core for Adiyutant.

## Crates

- `adiyutant_core` — base types: `Id<T>`, `CoreError`, `AdiyutantDateTime`
- `adiyutant_store` — storage abstraction (`Store` trait)
- `adiyutant_cli` — CLI entrypoint (`adiyutant` binary)

## Quick Start

```bash
cargo check --workspace
cargo test --workspace
cargo run -p adiyutant_cli -- status
```

## Dependencies

- chrono, serde, uuid, thiserror (workspace)
- clap (cli crate)

## Next Phase

Phase 2 — Domain Model: DailyLog, CheckIn, Habit, and other domain entities.
```

**Acceptance criteria:**
- [ ] README отражает текущее состояние
- [ ] Статус изменён с `Planned` на `Current`
- [ ] Приведены актуальные команды

---

## 5. Required Test Levels

| Level | Name | Applicable? | Details |
|---|---|---|---|
| 1 | Static checks | ✅ | `cargo fmt --check`, `cargo check --workspace` |
| 2 | Unit tests | ✅ | `cargo test --workspace` |
| 3 | Component tests | ❌ | Нет фреймворка — чистые функции |
| 4 | Integration tests | ❌ | Нет real dependencies (SQLite позже) |
| 5 | Stand smoke tests | ✅ | `cargo run` smoke — ручной запуск CLI |
| 6 | UI automation | ❌ | CLI, не GUI |
| 7 | User scenarios | ❌ | Нет бизнес-логики |
| 8 | Regression pack | ❌ | Первый код, нечего регрессить |
| 9 | Acceptance review | ✅ | Финальная сверка по чеклисту |

---

## 6. Test Stand

Не требуется. Все проверки — локальный `cargo` без внешних зависимостей.

**Требования к окружению:**
- Rust toolchain (stable, минимум 1.85 для edition 2024)
- `cargo`, `rustc`, `rustfmt`
- `clippy` (опционально)

Проверка окружения перед началом:

```bash
rustc --version
cargo --version
```

---

## 7. Files to Create / Modify

### Создать (8 файлов)

| # | File | Stage |
|---|---|---|
| 1 | `AdiyutantCore/Cargo.toml` | 1 |
| 2 | `AdiyutantCore/adiyutant_core/Cargo.toml` | 2 |
| 3 | `AdiyutantCore/adiyutant_core/src/lib.rs` | 2 |
| 4 | `AdiyutantCore/adiyutant_core/src/error.rs` | 2 |
| 5 | `AdiyutantCore/adiyutant_core/src/id.rs` | 2 |
| 6 | `AdiyutantCore/adiyutant_core/src/datetime.rs` | 2 |
| 7 | `AdiyutantCore/adiyutant_store/Cargo.toml` | 3 |
| 8 | `AdiyutantCore/adiyutant_store/src/lib.rs` | 3 |
| 9 | `AdiyutantCore/adiyutant_cli/Cargo.toml` | 4 |
| 10 | `AdiyutantCore/adiyutant_cli/src/main.rs` | 4 |

### Изменить (1 файл)

| # | File | Change |
|---|---|---|
| 1 | `AdiyutantCore/README.md` | Обновить статус на Current |

### Удалить (1 файл)

| # | File | Reason |
|---|---|---|
| 1 | `AdiyutantCore/.gitkeep` | Директория больше не пустая |

---

## 8. Dependency Graph

```
workspace (Cargo.toml)
  ├── chrono, serde, uuid, thiserror  [workspace.dependencies]
  │
  ├── adiyutant_core  ────────────────  chrono, serde, uuid, thiserror
  │   └── pub mod: id, error, datetime
  │
  ├── adiyutant_store  ───────────────  adiyutant_core (path)
  │   └── pub trait Store + NoopStore
  │
  └── adiyutant_cli  ─────────────────  adiyutant_core (path)
       └── main.rs                       adiyutant_store (path)
                                         clap 4
```

---

## 9. Acceptance Criteria (Final)

После завершения всех стадий:

- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test --workspace` — все тесты пройдены
- [ ] `cargo fmt --all -- --check` — без расхождений
- [ ] `cargo run -p adiyutant_cli -- status` → `Adiyutant Core v0.1.0 — status: ok`
- [ ] `cargo run -p adiyutant_cli -- health` → `health: ok`
- [ ] `AdiyutantCore/.gitkeep` удалён
- [ ] `AdiyutantCore/README.md` обновлён (Current)
- [ ] Крейты не содержат доменных моделей (DailyLog, Habit, etc.)
- [ ] Крейты не содержат SQLite, LLM, backend, sync-кода

---

## 10. Risks

| Risk | Mitigation |
|---|---|
| Rust edition 2024 требует nightly/нового stable | Проверить `rustup update stable` перед началом |
| `cargo clippy` не установлен | Опционально; если недоступен — пропустить с пометкой |
| `cargo fmt` изменяет файлы при первом прогоне | Это нормально; после форматирования повторить `--check` |
| Конфликт версий workspace-зависимостей | `cargo update` при необходимости |

---

## 11. Out of Scope Reminder

Не добавлять в эту фазу:

- ❌ `DailyLog`, `CheckIn`, `Habit`, `HabitEvent` — это Фаза 2
- ❌ `TodayState`, `LocalRuleAgentGateway` — это Фаза 3
- ❌ SQLite, `rusqlite`, миграции — это Фаза 4
- ❌ Реальные CLI-команды бизнес-логики — это Фаза 5
- ❌ `ActionProposal`, `ContextDocument`, `Plan` — это Фаза 2
- ❌ Любые вызовы LLM, API ключи, Agent Gateway
- ❌ Изменения в `.gitignore`, `AGENTS.md`, `SPECIFICATION.md`
