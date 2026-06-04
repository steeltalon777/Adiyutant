# ADIYUTANT_ITERATIVE_ROADMAP.md

Статус: черновик 0.1  
Источник: `SPECIFICATION.md`  
Цель: провести проект от пустого репозитория до локального MVP 0.1 через короткие итерации, каждая из которых может быть превращена в отдельное ТЗ для AI coding agent.

---

## 0. Главный фокус MVP

MVP 0.1 — это локальный `AdiyutantCore` на Rust.

MVP 0.1 должен доказать, что органайзер может работать без:

- backend;
- sync;
- Android UI;
- Desktop UI;
- Web UI;
- внешнего LLM;
- OpenClaw;
- API-ключей провайдеров.

Главная формула MVP:

```text
AdiyutantCore on Rust + SQLite + CLI.
```

Core должен уметь:

- создавать дневной журнал;
- принимать check-in’ы;
- хранить привычки;
- отмечать события привычек;
- хранить определения будильников, таймеров и напоминаний;
- хранить документы контекста;
- формировать состояние текущего дня;
- создавать простые ActionProposal через локальные правила;
- сохранять данные в SQLite;
- проверяться через CLI и тесты.

---

## 1. Правила итерационной разработки

Каждая итерация должна быть маленькой и завершённой.

На каждую итерацию создаётся отдельное ТЗ для агента:

```text
TASK-000X-<short-name>.md
```

Каждая итерация должна заканчиваться:

- работающим кодом или документацией;
- проверкой команд;
- обновлением README/INDEX/AI docs, если изменилась структура;
- коротким summary в `TASKS.md` или `MEMORY.md`, если решение стало стабильным.

---

## 2. Общие правила для AI coding agent

Агенту запрещено:

- добавлять backend до отдельного решения;
- добавлять Android/Web/Desktop реализацию до завершения Core MVP;
- хранить LLM API keys в клиенте;
- вызывать внешние LLM из Core напрямую;
- дублировать доменную логику в UI-проектах;
- менять несколько проектов без прямого указания в ТЗ;
- добавлять сложный rule engine, MCP, sync или OpenClaw plugin в MVP 0.1.

Агенту разрешено:

- создавать и менять документацию;
- создавать Rust workspace;
- писать доменные модели;
- писать SQLite storage;
- писать CLI;
- писать unit/integration tests;
- добавлять простые локальные правила без LLM;
- обновлять roadmap/index/context docs по факту изменений.

Обязательные проверки для Core:

```bash
cargo fmt
cargo check
cargo test
```

Если появляется CLI:

```bash
cargo run -p adiyutant_cli -- --help
cargo run -p adiyutant_cli -- today
```

---

## 3. Ветка и коммиты

Рекомендуемая схема:

```text
main        стабильная ветка
/dev        рабочая ветка MVP
/task-000X  короткие task-ветки, если удобно
```

Минимальная схема коммитов:

```text
docs: bootstrap ai-friendly docs
chore: add rust workspace skeleton
feat(core): add domain primitives
feat(store): add sqlite schema
feat(cli): add today command
test(core): add domain tests
```

---

# 4. Глобальная дорожная карта до MVP 0.1

---

## Phase 0 — Repository Foundation

Цель: создать репозиторий, который сразу понятен человеку и AI-агенту.

### ITER-0000 — Empty repository bootstrap

Тип: docs / structure  
Приоритет: critical

Задача:

- создать пустой git-репозиторий;
- создать базовую структуру solution;
- зафиксировать, что проект находится в bootstrap state.

Ожидаемая структура:

```text
Adiyutant/
  README.md
  SPECIFICATION.md
  ROADMAP.md
  TASKS.md
  SOLUTION_MAP.md
  ARCHITECTURE.md
  INDEX.md
  AI_CONTEXT.md
  AI_ENTRY_POINTS.md
  MEMORY.md
  AGENTS.md
  SECURITY_NOTES.md
  GLOSSARY.md

  docs/
    core-boundary.md
    agent-gateway.md
    sync-protocol.md
    adr/

  AdiyutantCore/
  AdiyutantAndroid/
  AdiyutantWeb/
  AdiyutantDesktop/
```

Acceptance criteria:

- репозиторий инициализирован;
- есть корневые документы;
- есть пустые проектные папки;
- в документах явно указано, что реализация ещё не подтверждена кодом.

---

### ITER-0001 — AI-friendly documentation baseline

Тип: docs  
Приоритет: critical

Задача:

- заполнить минимальные AI-friendly документы;
- отделить текущий статус от планируемой архитектуры;
- зафиксировать правила для агентов.

Файлы:

```text
README.md
ARCHITECTURE.md
INDEX.md
AI_CONTEXT.md
AI_ENTRY_POINTS.md
MEMORY.md
AGENTS.md
SOLUTION_MAP.md
```

Главные тезисы:

- `AdiyutantCore` — главный фокус MVP;
- Android/Web/Desktop — будущие оболочки;
- backend/server — позже;
- Agent Gateway — внешний слой;
- Core не хранит API-ключи и не вызывает провайдеров напрямую.

Acceptance criteria:

- AI agent понимает, с чего начинать;
- все текущие утверждения либо `Current`, либо `Planned`, либо `Unknown`;
- нет заявлений о несуществующем коде.

---

### ITER-0002 — Architecture decisions baseline

Тип: docs / ADR  
Приоритет: high

Задача:

Создать первые ADR:

```text
docs/adr/0001-local-first-rust-core.md
docs/adr/0002-core-as-domain-source-of-truth.md
docs/adr/0003-platform-shells-around-core.md
docs/adr/0004-sqlite-as-local-storage.md
docs/adr/0005-cli-first-core-validation.md
```

Acceptance criteria:

- каждое ADR содержит Context / Decision / Consequences / Alternatives / Confidence;
- решения не притворяются реализованным кодом;
- `INDEX.md` содержит ссылки на ADR.

---

## Phase 1 — Rust Core Workspace

Цель: получить рабочий Rust workspace без доменной сложности.

### ITER-0003 — Create Rust workspace skeleton

Тип: code / structure  
Приоритет: critical

Задача:

Создать workspace:

```text
AdiyutantCore/
  Cargo.toml
  crates/
    adiyutant_core/
    adiyutant_store/
    adiyutant_cli/
```

Crates:

- `adiyutant_core` — чистая доменная логика;
- `adiyutant_store` — SQLite/local persistence;
- `adiyutant_cli` — первый интерфейс проверки Core.

Acceptance criteria:

```bash
cd AdiyutantCore
cargo check
cargo test
```

должны проходить.

---

### ITER-0004 — Core coding standards and error model

Тип: code / foundation  
Приоритет: high

Задача:

- добавить базовые зависимости;
- создать единый error model;
- создать базовые ID/time/value primitives;
- добавить первые unit tests.

Возможные зависимости:

```text
chrono
serde
serde_json
uuid
thiserror
```

Модули `adiyutant_core`:

```text
src/lib.rs
src/error.rs
src/ids.rs
src/time.rs
src/types.rs
```

Acceptance criteria:

- есть `AdiyutantError` / `Result<T>`;
- есть типы ID или обёртки под UUID/string;
- есть базовые тесты;
- `cargo fmt`, `cargo check`, `cargo test` проходят.

---

## Phase 2 — Domain Model MVP

Цель: описать доменную модель MVP без SQLite и UI.

### ITER-0005 — DailyLog and CheckIn domain

Тип: code / domain  
Приоритет: critical

Задача:

Добавить доменные сущности:

```text
DailyLog
CheckIn
CheckInType
DayMode
```

Минимальные use cases:

```text
create_daily_log(date)
add_checkin(daily_log, checkin_type, raw_input)
set_day_mode(daily_log, mode)
```

Acceptance criteria:

- можно создать дневной журнал в памяти;
- можно добавить morning/day/evening/shutdown check-in;
- есть тесты на создание и валидацию;
- Core не зависит от store/CLI.

---

### ITER-0006 — Habit and HabitEvent domain

Тип: code / domain  
Приоритет: critical

Задача:

Добавить:

```text
Habit
HabitEvent
HabitStatus
HabitLevel
```

Минимальные use cases:

```text
create_habit(name)
mark_habit_done(habit, level, comment)
mark_habit_skipped(habit, comment)
```

Acceptance criteria:

- привычка создаётся;
- событие привычки создаётся;
- уровни `min/light/base/full/custom` поддерживаются;
- есть тесты.

---

### ITER-0007 — Time objects: reminders, alarms, timers

Тип: code / domain  
Приоритет: high

Задача:

Добавить определения:

```text
ReminderDefinition
AlarmDefinition
TimerDefinition
TimerMode
```

Важно:

- Core хранит смысл;
- платформы исполняют реальные alarm/notification/timer;
- Core не вызывает Android AlarmManager / desktop tray / browser APIs.

Acceptance criteria:

- можно создать reminder/alarm/timer definition;
- есть базовая валидация времени/длительности;
- есть тесты;
- документация `docs/core-boundary.md` обновлена.

---

### ITER-0008 — ContextDocument domain

Тип: code / domain  
Приоритет: medium

Задача:

Добавить:

```text
ContextDocument
ContextDocumentType
```

Типы:

```text
core / goals / rules / routine / health / work / custom
```

Acceptance criteria:

- можно создать active/inactive context document;
- поддерживается version;
- есть тесты;
- документ может хранить markdown string.

---

### ITER-0009 — Plan and ActionProposal domain

Тип: code / domain  
Приоритет: high

Задача:

Добавить:

```text
Plan
PlanItem
ActionProposal
ActionProposalStatus
ActionProposalType
```

Принцип:

```text
Agent suggests.
Core validates.
UI confirms.
Storage persists.
```

Acceptance criteria:

- ActionProposal может быть `pending/accepted/rejected/applied`;
- нельзя применить rejected proposal;
- есть тесты жизненного цикла proposal;
- Core пока не вызывает LLM.

---

## Phase 3 — Local Rules and Today State

Цель: получить полезное поведение без внешнего ИИ.

### ITER-0010 — TodayState aggregator

Тип: code / use case  
Приоритет: critical

Задача:

Создать агрегатор текущего дня:

```text
TodayState
build_today_state(...)
```

TodayState должен включать:

- date;
- day mode;
- check-ins;
- active habits;
- habit events for date;
- reminders/timers/alarms summaries;
- pending action proposals.

Acceptance criteria:

- TodayState собирается из in-memory данных;
- есть тесты на пустой день;
- есть тесты на день с check-in и habit event.

---

### ITER-0011 — LocalRuleAgentGateway MVP

Тип: code / local agent  
Приоритет: high

Задача:

Создать локальные правила без LLM:

```text
LocalRuleAgentGateway
```

Минимальные правила:

```text
если sleep_hours < 6 -> propose recovery mode
если evening/shutdown и speech не выполнен -> propose minimal speech
если raw_input содержит сильный перегруз -> propose reduce plan
```

Важно:

- правила создают `ActionProposal`;
- правила не меняют состояние напрямую;
- это не LLM-интеграция.

Acceptance criteria:

- LocalRuleAgentGateway возвращает ActionProposal;
- состояние Core не меняется без apply/confirm;
- есть тесты.

---

## Phase 4 — SQLite Storage

Цель: сделать локальное состояние постоянным.

### ITER-0012 — SQLite schema bootstrap

Тип: code / persistence  
Приоритет: critical

Задача:

В `adiyutant_store` добавить SQLite schema bootstrap для MVP-сущностей:

```text
daily_logs
checkins
habits
habit_events
reminder_definitions
alarm_definitions
timer_definitions
context_documents
action_proposals
```

Требования:

- схема создаётся при первом запуске;
- хранение JSON допускается для flexible fields;
- миграционный механизм может быть простым на MVP.

Acceptance criteria:

- можно открыть SQLite DB;
- таблицы создаются;
- повторный запуск не ломает схему;
- есть тест на in-memory DB.

---

### ITER-0013 — Repositories for DailyLog and CheckIn

Тип: code / persistence  
Приоритет: critical

Задача:

Добавить репозитории:

```text
DailyLogRepository
CheckInRepository
```

Функции:

```text
get_or_create_today(date)
insert_checkin(...)
list_checkins_for_day(...)
```

Acceptance criteria:

- DailyLog сохраняется в SQLite;
- CheckIn сохраняется в SQLite;
- можно прочитать день обратно;
- есть integration tests на временной DB.

---

### ITER-0014 — Repositories for habits

Тип: code / persistence  
Приоритет: high

Задача:

Добавить:

```text
HabitRepository
HabitEventRepository
```

Acceptance criteria:

- можно создать habit;
- можно отметить habit event;
- можно получить активные habits;
- можно получить habit events for date;
- есть тесты.

---

### ITER-0015 — Repositories for reminders, alarms, timers, context, proposals

Тип: code / persistence  
Приоритет: medium

Задача:

Добавить хранение:

```text
ReminderDefinition
AlarmDefinition
TimerDefinition
ContextDocument
ActionProposal
```

Acceptance criteria:

- все MVP-сущности сохраняются и читаются;
- есть smoke tests;
- поля с JSON корректно сериализуются/десериализуются.

---

## Phase 5 — CLI MVP

Цель: получить первый рабочий интерфейс к Core.

### ITER-0016 — CLI skeleton and database path

Тип: code / CLI  
Приоритет: critical

Задача:

Создать CLI на `clap`.

Команды:

```text
adiyutant --help
adiyutant today
```

Настройки:

- путь к DB через аргумент `--db`;
- default local path для разработки;
- in-memory или temp DB для тестов.

Acceptance criteria:

```bash
cargo run -p adiyutant_cli -- --help
cargo run -p adiyutant_cli -- today
```

работают.

---

### ITER-0017 — CLI daily/checkin commands

Тип: code / CLI  
Приоритет: critical

Задача:

Добавить команды:

```bash
adiyutant today
adiyutant checkin morning "..."
adiyutant checkin day "..."
adiyutant checkin evening "..."
adiyutant shutdown "..."
```

Acceptance criteria:

- команда `today` создаёт/показывает день;
- check-in сохраняется;
- shutdown сохраняется как check-in type `shutdown`;
- данные переживают повторный запуск CLI.

---

### ITER-0018 — CLI habits commands

Тип: code / CLI  
Приоритет: critical

Задача:

Добавить:

```bash
adiyutant habit add speech
adiyutant habit list
adiyutant habit done speech --level min
adiyutant habit skip speech --comment "..."
```

Acceptance criteria:

- habit создаётся;
- повторное создание с тем же именем обрабатывается явно;
- habit event сохраняется;
- `today` показывает habit status.

---

### ITER-0019 — CLI timer/reminder/alarm commands

Тип: code / CLI  
Приоритет: high

Задача:

Добавить:

```bash
adiyutant timer start "focus" --minutes 25
adiyutant reminder add "вечерний speech" --at "21:00"
adiyutant alarm add "morning" --time "07:00" --repeat weekdays
```

Важно:

- CLI только сохраняет definitions;
- реальное срабатывание платформенных уведомлений не входит в Core MVP.

Acceptance criteria:

- definitions сохраняются;
- `today` отображает summaries;
- документация явно объясняет, что реальное исполнение — задача платформ.

---

### ITER-0020 — CLI context commands

Тип: code / CLI  
Приоритет: medium

Задача:

Добавить:

```bash
adiyutant context add "Life OS" ./docs/life-os.md --type core
adiyutant context list
adiyutant context show "Life OS"
```

Acceptance criteria:

- markdown читается из файла;
- context document сохраняется;
- можно посмотреть список и содержимое;
- есть обработка ошибок файла.

---

### ITER-0021 — CLI local suggestions

Тип: code / CLI / local agent  
Приоритет: high

Задача:

Добавить команду:

```bash
adiyutant suggest
```

Команда должна:

- собрать TodayState;
- прогнать LocalRuleAgentGateway;
- сохранить pending ActionProposal;
- показать предложения пользователю.

Дополнительно:

```bash
adiyutant proposal list
adiyutant proposal accept <id>
adiyutant proposal reject <id>
```

Acceptance criteria:

- предложения создаются без LLM;
- accepted proposal может менять состояние только через явный apply;
- rejected proposal не применяется.

---

## Phase 6 — MVP Hardening

Цель: довести локальный Core до состояния, которое можно считать MVP 0.1.

### ITER-0022 — Test coverage pass

Тип: test / quality  
Приоритет: critical

Задача:

Добавить тесты:

- domain tests;
- storage integration tests;
- CLI smoke tests;
- error handling tests;
- JSON serialization tests.

Acceptance criteria:

```bash
cargo fmt
cargo check
cargo test
```

проходят стабильно.

---

### ITER-0023 — Documentation sync with implemented Core

Тип: docs  
Приоритет: critical

Задача:

Обновить документацию по факту кода:

```text
README.md
ARCHITECTURE.md
INDEX.md
AI_CONTEXT.md
AI_ENTRY_POINTS.md
MEMORY.md
AdiyutantCore/README.md
AdiyutantCore/ARCHITECTURE.md
```

Правила:

- не писать, что не реализовано;
- отметить planned/unknown отдельно;
- указать реальные CLI-команды;
- указать реальные file paths.

Acceptance criteria:

- документация соответствует коду;
- AI agent может войти в проект без догадок;
- нет расхождения “описано, но не существует”.

---

### ITER-0024 — MVP acceptance scenario

Тип: acceptance / demo  
Приоритет: critical

Задача:

Создать финальный сценарий проверки MVP:

```bash
adiyutant today
adiyutant checkin morning "спал 6 часов, энергия 5, хочу работать над Rust core"
adiyutant habit add speech
adiyutant habit done speech --level min
adiyutant timer start "focus" --minutes 25
adiyutant reminder add "вечерний speech" --at "21:00"
adiyutant suggest
adiyutant shutdown "собрал ядро, завтра продолжить storage"
adiyutant today
```

Acceptance criteria:

- сценарий проходит на чистой DB;
- данные сохраняются;
- `today` показывает понятное состояние;
- предложения создаются без внешнего ИИ;
- API-ключи нигде не требуются.

---

### ITER-0025 — MVP 0.1 release marker

Тип: release / docs  
Приоритет: critical

Задача:

Зафиксировать MVP 0.1:

- добавить `CHANGELOG.md`;
- отметить версию `0.1.0-local-core`;
- обновить README;
- описать known limitations;
- создать tag.

Acceptance criteria:

```bash
git tag v0.1.0-local-core
```

MVP считается готовым, если выполнены критерии из `SPECIFICATION.md`:

- `cargo check` проходит;
- `cargo test` проходит;
- CLI создаёт дневной журнал;
- CLI добавляет check-in;
- CLI показывает состояние текущего дня;
- CLI добавляет привычку;
- CLI отмечает событие привычки;
- CLI закрывает день;
- данные сохраняются в SQLite;
- Core не зависит от Android/Web/Desktop;
- API-ключи нигде не используются и не требуются.

---

# 5. Что не входит в этот roadmap

До MVP 0.1 не делать:

- Android UI;
- Desktop UI;
- Web UI;
- backend;
- sync server;
- OpenClaw plugin;
- внешние LLM provider calls;
- MCP;
- billing;
- SaaS;
- сложный календарный сервер;
- полноценный platform alarm executor.

Эти направления идут после локального Core MVP.

---

# 6. Следующий roadmap после MVP 0.1

После `v0.1.0-local-core` возможны отдельные дороги:

## Path A — Android Shell MVP 0.2

Цель:

- первый мобильный UI;
- реальные Android alarms/notifications;
- вызов Core через UniFFI/C ABI/иной механизм.

## Path B — Desktop Shell MVP 0.2

Цель:

- desktop UI;
- tray;
- локальные уведомления;
- удобный daily dashboard.

## Path C — Agent Gateway MVP 0.3

Цель:

- OpenClaw adapter или custom local relay;
- LLM без API-ключей в телефоне;
- ActionProposal от внешнего агента.

## Path D — Sync MVP 0.4

Цель:

- device auth;
- sync journal;
- conflict records;
- multi-device history.

---

# 7. Рекомендуемый первый набор ТЗ

Чтобы не распухнуть, первые ТЗ лучше делать в таком порядке:

```text
TASK-0000-bootstrap-repo.md
TASK-0001-ai-friendly-docs.md
TASK-0002-adr-baseline.md
TASK-0003-rust-workspace.md
TASK-0004-core-primitives.md
TASK-0005-daily-checkin-domain.md
TASK-0006-habits-domain.md
TASK-0012-sqlite-schema.md
TASK-0016-cli-skeleton.md
TASK-0017-cli-checkins.md
```

После этого уже видно, насколько агент бодр, а где ему пора выдать шлем и страховку.

---

# 8. Definition of Done для каждой итерации

Итерация считается завершённой, если:

- задача выполнена в рамках указанной области;
- не затронуты запрещённые зоны;
- `cargo fmt` проходит;
- `cargo check` проходит;
- `cargo test` проходит, если есть Rust-код;
- документация обновлена, если изменилась структура или поведение;
- uncertainty записана явно, если решение не принято;
- результат можно показать в 3–5 командах или коротком diff summary.

---

# 9. Главный принцип

Не строить сразу всю систему.

Сначала:

```text
локальное ядро, которое реально работает.
```

Потом:

```text
оболочки, агент, сервер и синк.
```

Если Core не работает из CLI, то Android, Web, Desktop и OpenClaw только красиво умножат хаос.
