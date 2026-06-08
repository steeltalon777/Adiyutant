# TZ: Phase 5 — CLI MVP

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-5.md`

## Execution Strategy

- [ ] 🟡 Sequential execution recommended
- **Reason:** `main.rs` — общий файл-диспетчер, через который проходят все subcommand-ы. Subcommand-модули (`commands/*.rs`) могут разрабатываться параллельно в Stage 2, но `main.rs` должен быть согласован с каждым модулем. Кроме того, `common.rs` (контекст с `Store`) — общая зависимость для всех команд. Рекомендуется: Stage 1 (фундамент) → Stage 2 (параллельные subcommand-ы) → Stage 3 (интеграция в `main.rs`) → Stage 4 (тесты и регрессия).

## Execution Checklist

- [x] 0. Context verified
- [x] 1. Stage 1 — Foundation: `common.rs` (Store context), DB path, `main.rs` skeleton
- [x] 2. Stage 2a — Daily/checkin commands
- [x] 3. Stage 2b — Habit commands
- [x] 4. Stage 2c — Timer/reminder/alarm commands
- [x] 5. Stage 2d — Context commands
- [x] 6. Stage 2e — Local suggestions command
- [x] 7. Stage 3 — Integration: wire all subcommands in `main.rs`
- [x] 8. Static checks: `cargo fmt --check`, `cargo check --workspace`
- [x] 9. Unit tests: command argument parsing, CLI-specific helpers
- [ ] 10. Integration tests: CLI binary with real SQLite (in-memory)
- [ ] 11. Stand smoke tests: `adiyutant today`, `adiyutant checkin morning`, etc.
- [x] 12. Regression: `cargo test --workspace` — тесты Фаз 1-4 зелёные
- [x] 13. Documentation updated (ROADMAP Phase 5 marked done)
- [ ] 14. Final acceptance review

## Check Rules

- Architect создаёт checklist и acceptance criteria.
- Executor agent отмечает пункты только после реализации и верификации.
- Если проверка не прошла — пункт остаётся незаполненным с причиной.
- QA verifier проверяет финальную приёмку только после review evidence.

---

## 1. Source Documents

| Документ | Роль |
|---|---|
| `ROADMAP.md` — Phase 5 | Источник scope |
| `SPECIFICATION.md` — §10 (MVP 0.1), примеры CLI-команд | Поведение CLI |
| `ARCHITECTURE.md` — Core Boundary | Core owns CLI |
| `docs/core-boundary.md` | Границы Core |
| `AGENTS.md` | Правила репозитория |
| `AdiyutantCore/adiyutant_cli/src/main.rs` | Текущий CLI scaffold |
| `AdiyutantCore/adiyutant_cli/Cargo.toml` | Зависимости CLI (clap уже добавлен) |
| `AdiyutantCore/adiyutant_core/src/today_state.rs` | TodayState + Builder |
| `AdiyutantCore/adiyutant_core/src/local_rule_gateway.rs` | LocalRuleAgentGateway |
| `AdiyutantCore/adiyutant_store/src/lib.rs` | Store trait + SqliteStore + NoopStore |

---

## 2. Goal

Создать первый рабочий интерфейс к Core — CLI, который позволяет пользователю взаимодействовать с органайзером через терминал. CLI должен:

- Иметь полноценный clap-скелет с подкомандами для всех групп сущностей
- Работать с реальной SQLite-базой через `SqliteStore`
- Позволять создавать и просматривать DailyLog / CheckIn
- Управлять привычками и отмечать события привычек
- Создавать и просматривать таймеры, напоминания, будильники
- Управлять контекстными документами
- Показывать `TodayState` и локальные предложения от `LocalRuleAgentGateway`
- Не требовать интернета, LLM, сервера или API-ключей

---

## 3. Scope

### In scope

| # | Что | Где |
|---|---|---|
| 1 | `src/common.rs` — контекст (инициализация `SqliteStore`, путь к БД) | `adiyutant_cli/src/common.rs` |
| 2 | `src/commands/` — модуль с подкомандами | `adiyutant_cli/src/commands/` |
| 3 | `main.rs` — clap-диспетчер с подкомандами | `adiyutant_cli/src/main.rs` |
| 4 | Daily/checkin команды: `today`, `log`, `checkin` | `adiyutant_cli/src/commands/daily.rs` |
| 5 | Habit команды: `habit add`, `habit list`, `habit done`, `habit skip` | `adiyutant_cli/src/commands/habit.rs` |
| 6 | Timer/reminder/alarm команды: `timer start`, `reminder add`, `alarm add`, `alarm list` | `adiyutant_cli/src/commands/time.rs` |
| 7 | Context команды: `context add`, `context list`, `context show` | `adiyutant_cli/src/commands/context.rs` |
| 8 | Suggestions команда: `suggest` | `adiyutant_cli/src/commands/suggest.rs` |
| 9 | Интеграционные тесты CLI с in-memory SQLite | `adiyutant_cli/src/` (тесты) |

### Out of scope

- ❌ TUI / curses-интерфейс (только однострочные команды)
- ❌ Интерактивный режим (REPL)
- ❌ JSON-вывод (MVP — human-readable текст)
- ❌ Изменения в `adiyutant_core` или `adiyutant_store`
- ❌ Новые доменные модели или поля
- ❌ Platform alarms / notifications (Core хранит определения, платформа исполняет)
- ❌ LLM, Agent Gateway (кроме LocalRule), backend, sync
- ❌ Конфигурационные файлы (только env var для пути БД)
- ❌ `Plan` и `ActionProposal` CRUD через CLI (только просмотр через `today` и `suggest`)
- ❌ CalendarEvent команды (не входит в MVP 0.1)

---

## 4. Implementation Stages

### Stage 0 — Предварительные условия

Перед началом убедиться:
- [ ] `cargo check --workspace` проходит (текущий код Фаз 1-4)
- [ ] `cargo test --workspace` — все существующие тесты зелёные
- [ ] `cargo run -p adiyutant_cli -- status` работает и выводит `Adiyutant Core v0.1.0 — status: ok`

---

### Stage 1 — Foundation: `common.rs` и обновление `main.rs`

**Создать:**
- `AdiyutantCore/adiyutant_cli/src/common.rs`
- `AdiyutantCore/adiyutant_cli/src/commands/mod.rs`

**Изменить:**
- `AdiyutantCore/adiyutant_cli/src/main.rs`

#### `common.rs` — контекст CLI

```rust
use adiyutant_store::{SqliteStore, Store};
use std::path::PathBuf;

/// Путь к SQLite-файлу: из env ADIYUTANT_DB_PATH или ~/.adiyutant/adiyutant.db
pub fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("ADIYUTANT_DB_PATH") {
        return PathBuf::from(p);
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".adiyutant").join("adiyutant.db")
}

/// Инициализировать SqliteStore, создать директорию и запустить миграцию
pub fn init_store() -> Result<SqliteStore, adiyutant_core::error::CoreError> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| adiyutant_core::error::CoreError::Storage(e.to_string()))?;
    }
    let store = SqliteStore::new(path.to_str().unwrap_or("adiyutant.db"))?;
    store.migrate()?;
    Ok(store)
}
```

**Требования:**
- Зависимость `dirs` добавить в `adiyutant_cli/Cargo.toml` для `home_dir()`
- Если `ADIYUTANT_DB_PATH` не задан, использовать `~/.adiyutant/adiyutant.db`
- Директория `~/.adiyutant/` создаётся автоматически при первом запуске
- `migrate()` вызывается при каждом запуске (идемпотентен)

#### `main.rs` — обновлённый скелет

```rust
mod commands;

use clap::{Parser, Subcommand};
use commands::*;

#[derive(Parser)]
#[command(name = "adiyutant")]
#[command(about = "Adiyutant — local-first organizer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show today's state
    Today,
    /// Show daily log for today
    Log,
    /// Create a check-in
    Checkin(CheckinArgs),
    /// Manage habits
    Habit {
        #[command(subcommand)]
        cmd: HabitCmd,
    },
    /// Timers
    Timer {
        #[command(subcommand)]
        cmd: TimerCmd,
    },
    /// Reminders
    Reminder {
        #[command(subcommand)]
        cmd: ReminderCmd,
    },
    /// Alarms
    Alarm {
        #[command(subcommand)]
        cmd: AlarmCmd,
    },
    /// Context documents
    Context {
        #[command(subcommand)]
        cmd: ContextCmd,
    },
    /// Show local suggestions
    Suggest,
}

fn main() {
    let cli = Cli::parse();
    // Инициализация store — будет общей для всех команд
    // ... match cli.command { ... }
}
```

**Acceptance criteria:**
- [ ] `cargo check -p adiyutant_cli` — без ошибок
- [ ] `cargo run -p adiyutant_cli -- --help` выводит список всех подкоманд
- [ ] `cargo run -p adiyutant_cli -- today` не падает (даже без реализованного вывода — должен вывести placeholder)
- [ ] `dirs` добавлен в `Cargo.toml` CLI
- [ ] Путь к БД корректно резолвится из `ADIYUTANT_DB_PATH` или default

---

### Stage 2a — Daily / Check-in команды

**Файл:** `AdiyutantCore/adiyutant_cli/src/commands/daily.rs`

#### Команда `today`

Показывает состояние текущего дня.

**Алгоритм:**
1. Получить сегодняшнюю дату
2. Найти `DailyLog` по дате через `store.get_daily_log_by_date(today)`
3. Если нет — вывести "No daily log for today. Use `adiyutant checkin morning` to start."
4. Если есть — собрать `TodayState`:
   - `store.list_check_ins_by_log(log.id)` → check_ins
   - `store.list_habits()` → habits
   - `store.list_habit_events_by_habit()` для каждой привычки (с фильтрацией по сегодня) → habit_events
   - `store.get_plan_by_daily_log(log.id)` → plan (если есть)
   - `store.list_alarms()`, `store.list_reminders()`, `store.list_timers()` → alarms, reminders, timers
   - `store.list_context_documents()` → context_documents
5. Собрать `TodayState` через `TodayStateBuilder`
6. Вывести человекочитаемое резюме

**Формат вывода:**
```
── Today: 2026-06-08 ──
Mode: none
Sleep: - | Energy: - | Mood: -
Check-ins: 0 (morning: no, evening: no)
Habits: 2 tracked, 0 done today
Plan: no plan
Alarms: 1 | Reminders: 2 | Timers: 1
Context documents: 3
```

#### Команда `log`

Показывает raw DailyLog для сегодня (или указанной даты).

```
$ adiyutant log
Date: 2026-06-08
Mode: none
Sleep: - | Energy: - | Mood: -
Notes: <none>
```

#### Команда `checkin`

Создаёт check-in.

```rust
#[derive(clap::Args)]
pub struct CheckinArgs {
    /// Check-in type: morning, day, evening, shutdown
    #[arg(value_enum)]
    pub checkin_type: CheckinTypeArg,

    /// Free-form text
    pub text: String,
}

#[derive(clap::ValueEnum, Clone)]
pub enum CheckinTypeArg {
    Morning,
    Day,
    Evening,
    Shutdown,
}
```

**Алгоритм:**
1. Получить/создать DailyLog на сегодня
2. Создать `CheckIn::new(log.id, type, text)`
3. `store.insert_check_in(&ci)`
4. Вывести подтверждение: `✅ Morning check-in recorded.`

**Acceptance criteria:**
- [ ] `adiyutant today` — выводит состояние (даже если пустое)
- [ ] `adiyutant checkin morning "спал 6 часов, энергия 5"` — создаёт DailyLog + CheckIn
- [ ] `adiyutant today` после check-in показывает morning check-in
- [ ] `adiyutant log` — показывает daily log
- [ ] `adiyutant checkin shutdown "закончил день"` — работает
- [ ] Повторный `adiyutant checkin morning "..."` создаёт второй morning check-in (MVP не запрещает)

---

### Stage 2b — Habit команды

**Файл:** `AdiyutantCore/adiyutant_cli/src/commands/habit.rs`

#### `habit add <name>`

```bash
$ adiyutant habit add speech
✅ Habit "speech" created.
```

Алгоритм:
1. `Habit::new(name)`
2. `store.insert_habit(&h)`
3. Вывести подтверждение

#### `habit list`

```bash
$ adiyutant habit list
📋 Habits (2):
  • speech — active
  • exercise — active
```

Алгоритм:
1. `store.list_habits()`
2. Вывести список

#### `habit done <name> [--level <level>]`

```bash
$ adiyutant habit done speech --level min
✅ speech done (min)
```

Алгоритм:
1. `store.list_habits()` → найти по имени
2. Если не найдена — ошибка: `Habit "xyz" not found.`
3. `HabitEvent::new(habit.id, Done, level)`
4. `store.insert_habit_event(&he)`
5. Вывести подтверждение

**Level enum:**
```rust
#[derive(clap::ValueEnum, Clone)]
pub enum LevelArg {
    Min,
    Light,
    Base,
    Full,
}
```

#### `habit skip <name>`

Аналогично `done`, но `status = Skipped`.

**Acceptance criteria:**
- [ ] `adiyutant habit add "read"` — создаёт привычку
- [ ] `adiyutant habit list` — показывает список
- [ ] `adiyutant habit done speech --level base` — отмечает событие
- [ ] `adiyutant habit skip speech` — отмечает пропуск
- [ ] `adiyutant habit done nonexistent` — выводит ошибку
- [ ] `adiyutant today` после добавления привычки и события показывает их

---

### Stage 2c — Timer / Reminder / Alarm команды

**Файл:** `AdiyutantCore/adiyutant_cli/src/commands/time.rs`

#### Timer

```bash
$ adiyutant timer start "focus" --minutes 25
⏱️ Timer "focus" started: 25 minutes (focus)
```

```rust
#[derive(Subcommand)]
pub enum TimerCmd {
    /// Start a timer
    Start {
        name: String,
        #[arg(short, long, default_value = "25")]
        minutes: u64,
        #[arg(short, long, default_value = "focus")]
        mode: String,
    },
}
```

Mode mapping: `"focus"` → `TimerMode::Focus`, `"rest"` → `TimerMode::Rest`, иначе → `TimerMode::Custom`.

**Примечание:** CLI не запускает реальный таймер (это задача платформы). CLI только сохраняет определение и выводит сообщение.

#### Reminder

```bash
$ adiyutant reminder add "вечерний speech" --at "21:00"
🔔 Reminder "вечерний speech" added (rule: 21:00)
```

```rust
#[derive(Subcommand)]
pub enum ReminderCmd {
    /// Add a reminder
    Add {
        title: String,
        #[arg(short, long)]
        at: String,
    },
    /// List reminders
    List,
}
```

Schedule rule хранится как строка (например, `"at 21:00"` или cron-выражение).

#### Alarm

```bash
$ adiyutant alarm add "wake up" --at "07:00"
⏰ Alarm "wake up" set for 07:00:00
```

```rust
#[derive(Subcommand)]
pub enum AlarmCmd {
    /// Add an alarm
    Add {
        title: String,
        #[arg(short, long)]
        at: String,  // формат HH:MM
    },
    /// List alarms
    List,
}
```

Алгоритм для `alarm add`:
1. Парсить `--at` как `NaiveTime::parse_from_str(s, "%H:%M")`
2. `AlarmDefinition::new(title, time)`
3. `store.insert_alarm(&a)`
4. Вывести подтверждение

**Acceptance criteria:**
- [ ] `adiyutant timer start "pomodoro" --minutes 25` — сохраняет таймер
- [ ] `adiyutant reminder add "drink water" --at "every 2h"` — сохраняет напоминание
- [ ] `adiyutant reminder list` — показывает список напоминаний
- [ ] `adiyutant alarm add "wake up" --at "07:00"` — сохраняет будильник
- [ ] `adiyutant alarm list` — показывает список будильников
- [ ] Все сущности видны в `adiyutant today`

---

### Stage 2d — Context команды

**Файл:** `AdiyutantCore/adiyutant_cli/src/commands/context.rs`

#### `context add <type> <title> <content>`

```bash
$ adiyutant context add core "My Core" "# Values\n\nBe kind."
📄 Context document "My Core" added (type: core, v1)
```

```rust
#[derive(clap::ValueEnum, Clone)]
pub enum ContextTypeArg {
    Core,
    Goals,
    Rules,
    Routine,
    Health,
    Work,
    Custom,
}

#[derive(Subcommand)]
pub enum ContextCmd {
    /// Add a context document
    Add {
        #[arg(value_enum)]
        doc_type: ContextTypeArg,
        title: String,
        content: String,
    },
    /// List context documents
    List,
    /// Show a context document by title (first match)
    Show {
        title: String,
    },
}
```

#### `context list`

```bash
$ adiyutant context list
📚 Context documents (2):
  • [core] My Core (v1)
  • [goals] 2026 Goals (v1)
```

#### `context show <title>`

```bash
$ adiyutant context show "My Core"
── My Core (core, v1) ──
# Values

Be kind.
```

**Acceptance criteria:**
- [ ] `adiyutant context add core "My Values" "Be kind."` — создаёт документ
- [ ] `adiyutant context list` — показывает список
- [ ] `adiyutant context show "My Values"` — выводит содержимое
- [ ] `adiyutant today` показывает количество context documents

---

### Stage 2e — Local suggestions command

**Файл:** `AdiyutantCore/adiyutant_cli/src/commands/suggest.rs`

#### `suggest`

```bash
$ adiyutant suggest
💡 Suggestions based on today's state:
  • morning_checkin: No morning check-in yet today
    → Do a morning check-in to start the day
  • create_plan: No plan for today yet
    → Create a plan to organize the day
```

**Алгоритм:**
1. Собрать `TodayState` (переиспользовать логику из `today`)
2. Вызвать `LocalRuleAgentGateway::evaluate(&state)`
3. Вывести каждый proposal:
   - Тип предложения
   - `reason` из payload
   - `suggestion` из payload

**Acceptance criteria:**
- [ ] `adiyutant suggest` на пустой БД предлагает `morning_checkin` и `create_plan`
- [ ] После `adiyutant checkin morning "..."` — предложение `morning_checkin` исчезает
- [ ] Все proposal имеют тип и осмысленный текст
- [ ] Команда не падает при отсутствии DailyLog

---

### Stage 3 — Integration: `main.rs`

**Файл:** `AdiyutantCore/adiyutant_cli/src/main.rs`

Собрать все subcommand-ы в единый `match` в `main()`.

**Структура `main()`:**
```rust
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Today => {
            let store = init_store_or_exit();
            commands::daily::cmd_today(&store);
        }
        Commands::Log => {
            let store = init_store_or_exit();
            commands::daily::cmd_log(&store);
        }
        Commands::Checkin(args) => {
            let store = init_store_or_exit();
            commands::daily::cmd_checkin(&store, args);
        }
        Commands::Habit { cmd } => {
            let store = init_store_or_exit();
            commands::habit::handle(&store, cmd);
        }
        // ... аналогично для Timer, Reminder, Alarm, Context, Suggest
    }
}

fn init_store_or_exit() -> adiyutant_store::SqliteStore {
    match common::init_store() {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Error initializing store: {e}");
            std::process::exit(1);
        }
    }
}
```

**Acceptance criteria:**
- [ ] `cargo check -p adiyutant_cli` — 0 errors
- [ ] Все команды из Stage 2a-2e доступны и работают
- [ ] `adiyutant --help` показывает все подкоманды с описаниями
- [ ] `adiyutant today --help` (если есть аргументы) показывает usage

---

## 5. Required Test Levels

| Level | Name | Applicable? | Details |
|---|---|---|---|
| 1 | Static checks | ✅ | `cargo fmt --all -- --check`, `cargo check --workspace` |
| 2 | Unit tests | ✅ | Аргументы команд (парсинг `--level`, `--at`), mapping enum-ов, `db_path()` |
| 3 | Component tests | ❌ | Нет UI-компонентов, CLI — это точка входа |
| 4 | Integration tests | ✅ | Запуск CLI binary с `--db :memory:`, проверка вывода команд |
| 5 | Stand smoke tests | ✅ | `cargo run -p adiyutant_cli -- <command>` с реальным `SqliteStore` (файловая БД) |
| 6 | UI automation | ❌ | CLI, не GUI |
| 7 | User scenarios | ✅ | Полный сценарий: `today` → `checkin morning` → `habit add` → `habit done` → `suggest` → `today` |
| 8 | Regression pack | ✅ | `cargo test --workspace` — тесты Фаз 1-4 проходят без изменений |
| 9 | Acceptance review | ✅ | Финальная сверка по чеклисту |

---

## 6. Test Stand

### Database

| Параметр | Значение |
|---|---|
| **Тип** | SQLite 3.x (bundled через `rusqlite`) |
| **Режим интеграционных тестов** | In-memory (`:memory:`) — `ADIYUTANT_DB_PATH=:memory:` или прямой вызов `SqliteStore::new_in_memory()` |
| **Режим smoke-тестов** | Файловая БД в `~/.adiyutant/adiyutant.db` |
| **Миграция** | `store.migrate()` вызывается при каждом запуске (идемпотентен) |
| **Жизненный цикл для тестов** | In-memory БД создаётся и уничтожается в каждом тесте |
| **Жизненный цикл для smoke** | Файловая БД сохраняется между запусками. Для чистого состояния — удалить файл. |

### Seed Data

Не требуется для тестов. Каждый тест создаёт данные через CLI-команды.

### Environment Variables

| Переменная | Назначение | Значение по умолчанию |
|---|---|---|
| `ADIYUTANT_DB_PATH` | Путь к SQLite-файлу | `~/.adiyutant/adiyutant.db` |

Никаких секретов, токенов или API-ключей не требуется.

### Smoke Commands

```bash
# Проверка инициализации
ADIYUTANT_DB_PATH=/tmp/adiyutant-test.db cargo run -p adiyutant_cli -- today

# Полный сценарий (см. Stage 7 — User scenarios)
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- checkin morning "спал 6ч, энергия 5"
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- habit add speech
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- habit done speech --level min
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- today
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- suggest
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- timer start "focus" --minutes 25
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- context add core "My Core" "# Values"
ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db cargo run -p adiyutant_cli -- context list
```

### Reset / Cleanup

```bash
rm -f /tmp/adiyutant-test.db /tmp/adiyutant-scenario.db
cargo clean -p adiyutant_cli
cargo test -p adiyutant_cli
```

---

## 7. Files

### Создать (7 файлов)

| # | File | Stage |
|---|---|---|
| 1 | `AdiyutantCore/adiyutant_cli/src/common.rs` | 1 |
| 2 | `AdiyutantCore/adiyutant_cli/src/commands/mod.rs` | 1 |
| 3 | `AdiyutantCore/adiyutant_cli/src/commands/daily.rs` | 2a |
| 4 | `AdiyutantCore/adiyutant_cli/src/commands/habit.rs` | 2b |
| 5 | `AdiyutantCore/adiyutant_cli/src/commands/time.rs` | 2c |
| 6 | `AdiyutantCore/adiyutant_cli/src/commands/context.rs` | 2d |
| 7 | `AdiyutantCore/adiyutant_cli/src/commands/suggest.rs` | 2e |

### Изменить (2 файла)

| # | File | Change |
|---|---|---|
| 1 | `AdiyutantCore/adiyutant_cli/Cargo.toml` | Добавить `dirs` |
| 2 | `AdiyutantCore/adiyutant_cli/src/main.rs` | Полная замена: clap-диспетчер с подкомандами |

### Не изменять

- `AdiyutantCore/adiyutant_core/` — все файлы (доменные модели, TodayState, LocalRuleAgentGateway, error, id, datetime)
- `AdiyutantCore/adiyutant_store/` — все файлы (Store trait, SqliteStore, NoopStore, schema, тесты)
- `AdiyutantCore/Cargo.toml` — workspace dependencies
- Корневые документы проекта (кроме ROADMAP.md — опционально обновить статус)

---

## 8. Acceptance Criteria (Final)

### 8.1. Build & Static

- [ ] `cargo fmt --all -- --check` — без расхождений
- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo build -p adiyutant_cli` — бинарный файл `adiyutant` собирается
- [ ] `cargo clippy --workspace -- -D warnings` — без ошибок (опционально, если установлен)

### 8.2. Tests

- [ ] `cargo test --workspace` — все тесты зелёные, включая:
  - Тесты Фаз 1-4 (регрессия) — 66 + 23 = 89 тестов
  - Новые тесты CLI (unit + integration)
- [ ] Интеграционные тесты CLI используют in-memory SQLite

### 8.3. CLI Commands

- [ ] `adiyutant --help` — выводит все подкоманды с описаниями
- [ ] `adiyutant today` — показывает состояние дня (пустое или заполненное)
- [ ] `adiyutant checkin morning "текст"` — создаёт DailyLog + CheckIn (если DailyLog нет — создаёт)
- [ ] `adiyutant checkin evening "текст"` — добавляет evening check-in
- [ ] `adiyutant checkin shutdown "текст"` — добавляет shutdown check-in
- [ ] `adiyutant log` — показывает daily log
- [ ] `adiyutant habit add "имя"` — создаёт привычку
- [ ] `adiyutant habit list` — показывает список привычек
- [ ] `adiyutant habit done "имя" --level base` — отмечает выполнение
- [ ] `adiyutant habit skip "имя"` — отмечает пропуск
- [ ] `adiyutant timer start "имя" --minutes N` — сохраняет таймер
- [ ] `adiyutant reminder add "заголовок" --at "правило"` — сохраняет напоминание
- [ ] `adiyutant reminder list` — показывает напоминания
- [ ] `adiyutant alarm add "заголовок" --at "HH:MM"` — сохраняет будильник
- [ ] `adiyutant alarm list` — показывает будильники
- [ ] `adiyutant context add <тип> "заголовок" "содержимое"` — сохраняет документ
- [ ] `adiyutant context list` — показывает список документов
- [ ] `adiyutant context show "заголовок"` — выводит содержимое
- [ ] `adiyutant suggest` — выводит локальные предложения

### 8.4. Data Persistence

- [ ] Данные сохраняются между запусками (файловая SQLite)
- [ ] `ADIYUTANT_DB_PATH` позволяет указать альтернативный путь к БД
- [ ] Директория `~/.adiyutant/` создаётся автоматически
- [ ] При отсутствии БД — создаётся новая с полной схемой

### 8.5. Boundaries

- [ ] CLI не добавляет новых методов в `adiyutant_core`
- [ ] CLI не изменяет `adiyutant_store`
- [ ] CLI не вызывает LLM
- [ ] CLI не использует API-ключи
- [ ] CLI не требует интернета
- [ ] Все ошибки выводятся в stderr, успех — в stdout
- [ ] Код возврата: 0 при успехе, 1 при ошибке

---

## 9. Risks

| Risk | Mitigation |
|---|---|
| `dirs` crate нестабилен на некоторых платформах | Fallback на `"."` при ошибке `home_dir()` |
| Парсинг `--at "HH:MM"` может падать на невалидных строках | `NaiveTime::parse_from_str` с человекочитаемой ошибкой |
| `TodayState` сборка требует много запросов к БД | Для MVP допустимо (нет индексов). Оптимизация — Фаза 6. |
| `habit done` ищет привычку по имени (не по id) | `list_habits()` + `find` по имени. Имена не уникальны — берём первое совпадение. |
| `suggest` дублирует логику сбора `TodayState` из `today` | Вынести сборку `TodayState` из Store в общую функцию `build_today_state(store)` в `common.rs` |
| Конфликт модулей при параллельной разработке subcommand-ов | Каждый subcommand — отдельный файл. `main.rs` и `common.rs` правит только Stage 1 и 3. |
| Зависимость `dirs` может требовать лицензионной проверки | `dirs` — MIT/Apache-2.0, совместимо с Rust экосистемой. |

---

## 10. Out of Scope Reminder

- ❌ TUI / curses / интерактивный режим
- ❌ JSON-вывод
- ❌ Plan CRUD через CLI (только просмотр в `today`)
- ❌ ActionProposal CRUD через CLI (только генерация в `suggest`)
- ❌ CalendarEvent команды
- ❌ Редактирование / удаление сущностей (кроме habit done/skip)
- ❌ Фильтрация HabitEvent по дате на уровне Store (фильтруем в CLI)
- ❌ Изменения в `adiyutant_core` или `adiyutant_store`
- ❌ Конфигурационные файлы
- ❌ Platform alarms / notifications
- ❌ LLM, Agent Gateway (кроме LocalRule), backend, sync

---

## 11. User Scenario (Acceptance Test)

Полный сценарий для ручной или автоматизированной проверки:

```bash
# Setup
export ADIYUTANT_DB_PATH=/tmp/adiyutant-scenario.db
rm -f $ADIYUTANT_DB_PATH

# 1. Пустой today
cargo run -p adiyutant_cli -- today
# → "No daily log for today"

# 2. Morning check-in
cargo run -p adiyutant_cli -- checkin morning "спал 6 часов, энергия 5, хочу работать над Rust core"
# → ✅ Morning check-in recorded.

# 3. Today после check-in
cargo run -p adiyutant_cli -- today
# → Показывает: mode none, check-ins 1 (morning: yes), habits 0, no plan

# 4. Добавить привычки
cargo run -p adiyutant_cli -- habit add speech
cargo run -p adiyutant_cli -- habit add exercise
cargo run -p adiyutant_cli -- habit list
# → Показывает 2 привычки

# 5. Отметить привычку
cargo run -p adiyutant_cli -- habit done speech --level min
# → ✅ speech done (min)

# 6. Today после привычек
cargo run -p adiyutant_cli -- today
# → Показывает: habits 2 tracked, 1 done today

# 7. Предложения
cargo run -p adiyutant_cli -- suggest
# → create_plan (нет morning_checkin, т.к. уже сделан, нет recovery — sleep_score не указан)

# 8. Таймер
cargo run -p adiyutant_cli -- timer start "focus" --minutes 25
# → ⏱️ Timer "focus" started

# 9. Будильник
cargo run -p adiyutant_cli -- alarm add "wake up" --at "07:00"

# 10. Напоминание
cargo run -p adiyutant_cli -- reminder add "вечерний speech" --at "21:00"

# 11. Контекст
cargo run -p adiyutant_cli -- context add core "My Core" "# Values\n\nBe kind."
cargo run -p adiyutant_cli -- context list

# 12. Evening check-in
cargo run -p adiyutant_cli -- checkin evening "день прошёл продуктивно"

# 13. Shutdown
cargo run -p adiyutant_cli -- checkin shutdown "собрал ядро, завтра продолжить storage"

# 14. Итоговый today
cargo run -p adiyutant_cli -- today
# → Полная картина дня: 3 check-ins, 2 habits (1 done), 1 timer, 1 alarm, 1 reminder, 1 context

# Cleanup
rm -f $ADIYUTANT_DB_PATH
```

---

## 12. Evidence Table (to be filled by executor)

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static checks | `cargo fmt --all -- --check` | | |
| Static checks | `cargo check --workspace` | | |
| Build | `cargo build -p adiyutant_cli` | | |
| Unit tests (CLI) | `cargo test -p adiyutant_cli` | | |
| Integration tests (CLI + in-memory DB) | `cargo test -p adiyutant_cli -- --nocapture` | | |
| Regression (Phase 1-4) | `cargo test --workspace` | | |
| Smoke: today | `ADIYUTANT_DB_PATH=... cargo run ... -- today` | | |
| Smoke: full scenario | User scenario (Section 11) | | |
| Clippy (optional) | `cargo clippy --workspace -- -D warnings` | | |
| CLI --help | `cargo run -p adiyutant_cli -- --help` | | |
