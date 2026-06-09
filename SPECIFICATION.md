# SPECIFICATION.md — Adiyutant

Статус: черновик спецификации 0.1  
Фаза проекта: **MVP 0.1** (3 Rust crates, SQLite, CLI, 103 tests)  
Главный фокус ближайшей разработки: `AdiyutantCore` реализован, переходим к UI/Agent Gateway

---

## 1. Краткое описание

**Adiyutant** — локальный органайзер с встроенным ИИ-агентом.

Продукт объединяет:

- будильник;
- таймер;
- ежедневник;
- календарь;
- привычки;
- напоминания;
- личный контекст пользователя;
- чат с ИИ-агентом;
- работу агента с инструментами;
- будущую синхронизацию между устройствами.

Ключевая идея:

> Adiyutant — не просто чат-бот.  
> Adiyutant — органайзер, в который встроен ИИ-агент.

Органайзер должен быть полезен без ИИ.  
ИИ должен усиливать рабочие процессы, а не владеть моделью данных.

---

## 2. Цель продукта

Adiyutant должен помогать пользователю управлять днём, режимом, задачами и личной системой без постоянного ручного микроменеджмента.

Основные сценарии:

- проснуться по будильнику;
- открыть утренний check-in;
- зафиксировать сон, энергию, состояние и план дня;
- получить подсказку от ИИ-агента;
- вести задачи и события дня;
- использовать таймеры и напоминания;
- вечером закрыть день;
- сохранить историю;
- синхронизировать данные между устройствами на следующих этапах.

---

## 3. Принцип архитектуры

Главный принцип:

```text
Local-first Core first.
Server later.
AI Gateway outside the Core.
```

Расшифровка:

- сначала создаётся локальное переносимое ядро;
- ядро должно работать без сервера;
- ядро должно работать без LLM;
- UI-клиенты являются оболочками вокруг ядра;
- сервер добавляется позже для синхронизации, аккаунтов, облачной истории и AI relay;
- API-ключи LLM-провайдеров не должны храниться в мобильном клиенте.

---

## 4. Состав solution

Планируемая структура solution:

```text
Adiyutant/
  README.md
  SPECIFICATION.md
  ARCHITECTURE.md
  ROADMAP.md
  SOLUTION_MAP.md
  AGENTS.md

  docs/
    core-boundary.md
    agent-gateway.md
    sync-protocol.md
    security-notes.md
    adr/

  AdiyutantCore/
  AdiyutantAndroid/
  AdiyutantWeb/
  AdiyutantDesktop/
```

### 4.1. AdiyutantCore

Назначение:

- переносимое Rust-ядро;
- доменные модели;
- локальная SQLite-база;
- бизнес-логика органайзера;
- подготовка состояния для UI;
- подготовка данных для синхронизации;
- CLI для проверки ядра.

### 4.2. AdiyutantAndroid

Назначение:

- Android-оболочка;
- мобильный UI;
- реальные будильники, уведомления, permission’ы;
- будущая интеграция с Rust Core через FFI/UniFFI или другой механизм.

### 4.3. AdiyutantWeb

Назначение:

- web/PWA-интерфейс;
- dashboard;
- возможно облачный кабинет;
- интеграция с Core пока не утверждена.

Варианты будущей интеграции:

- через backend API;
- через WebAssembly;
- как отдельный web-клиент поверх sync-сервера.

### 4.4. AdiyutantDesktop

Назначение:

- desktop-оболочка;
- работа с локальным Core;
- системный tray;
- локальные уведомления;
- будущий агентный режим.

Возможные технологии:

- Tauri;
- WPF;
- другой desktop UI.

---

## 5. Граница Core

### 5.1. Core отвечает за

```text
DailyLog
CheckIn
Habit
HabitEvent
ReminderDefinition
AlarmDefinition
TimerDefinition
CalendarEvent
ContextDocument
Plan
ActionProposal
LocalChange
SyncState
```

Core должен уметь:

- создавать дневной журнал;
- принимать check-in’ы;
- хранить привычки;
- отмечать события привычек;
- хранить определения будильников, таймеров и напоминаний;
- хранить календарные события;
- формировать состояние текущего дня;
- формировать предложения действий;
- готовить локальные изменения для будущего sync;
- работать через CLI без UI.

### 5.2. Core не отвечает за

```text
Android AlarmManager
desktop tray
browser notifications
push notifications
platform permissions
OAuth
billing
admin panel
хранение LLM API keys
прямые вызовы конкретных LLM provider API
```

Core хранит смысл события.  
Платформа исполняет событие.

Пример:

```text
Core: будильник должен сработать в 07:00.
Android: реально ставит системный alarm.
Desktop: реально ставит desktop notification / tray event.
Web: использует браузерные notification API, если доступно.
```

---

## 6. ИИ-агент

ИИ-агент является внешним усилителем, а не владельцем данных.

Правило:

```text
Agent suggests.
Core validates.
UI confirms.
Storage persists.
```

ИИ может:

- структурировать свободный текст;
- предлагать план дня;
- анализировать состояние;
- предлагать режим дня;
- помогать закрывать день;
- работать с инструментами через gateway;
- позже использовать MCP-like инструменты.

ИИ не должен:

- напрямую менять базу без проверки;
- владеть источником истины;
- хранить API-ключи в мобильном клиенте;
- быть обязательным для работы органайзера.

---

## 7. Agent Gateway

Для связи с ИИ вводится внешний слой: **Agent Gateway**.

Возможные реализации:

```text
MockAgentGateway
LocalRuleAgentGateway
OpenClawAgentGateway
CustomServerAgentGateway
```

### 7.1. MockAgentGateway

Используется в раннем MVP и тестах.

- не вызывает LLM;
- возвращает фиксированные ответы;
- позволяет тестировать Core и UI без сервера.

### 7.2. LocalRuleAgentGateway

Простые локальные правила без LLM.

Примеры:

```text
если sleep_hours < 6 → day_mode = recovery
если вечер и speech не выполнен → предложить минимальный speech
если перегруз → предложить укоротить план
```

### 7.3. OpenClawAgentGateway

Возможная будущая реализация через OpenClaw.

Назначение:

- агентный runtime;
- работа с инструментами;
- MCP-like сценарии;
- маршрутизация задач к LLM.

OpenClaw не должен быть жёсткой зависимостью Core.

### 7.4. CustomServerAgentGateway

Собственный сервер Adiyutant.

Назначение:

- хранение API-ключей провайдеров;
- AI relay;
- маршрутизация запросов к DeepSeek/Qwen/OpenAI/Gemini/Ollama/другим провайдерам;
- лимиты;
- логирование;
- безопасность;
- связь с sync.

---

## 8. Сервер

Сервер не является обязательным для MVP 0.1.

Будущий сервер нужен для:

- аккаунтов;
- устройств;
- синхронизации;
- облачного бэкапа;
- истории между устройствами;
- AI relay;
- хранения API-ключей;
- OpenClaw-интеграции;
- администрирования;
- будущей SaaS-модели.

Возможные варианты:

```text
1. OpenClaw as Agent Backend
2. Custom Adiyutant Server
3. Hybrid: custom sync server + OpenClaw agent runtime
```

Предпочтительная долгосрочная модель:

```text
Adiyutant Server = sync / accounts / devices / AI relay
OpenClaw = optional agent runtime / tool layer
```

---

## 9. Безопасность API-ключей

Правило:

> LLM provider API keys must not be stored in mobile clients.

Запрещено:

- хранить DeepSeek/Qwen/OpenAI/Gemini ключи в APK;
- передавать provider API keys в мобильное приложение;
- вызывать внешние LLM напрямую из телефона с главным ключом пользователя, если ключ нельзя безопасно защитить.

Разрешённые варианты:

- локальный mock-agent без ключей;
- локальные правила без LLM;
- запросы через домашний ПК/VPS;
- запросы через OpenClaw;
- запросы через собственный Adiyutant Server;
- device token вместо provider API key.

Телефон должен хранить максимум:

```text
device_id
device_token
user_id
local SQLite database
local app settings
```

---

## 10. MVP 0.1

MVP 0.1 должен быть локальным.

Цель:

> Проверить локальное ядро органайзера без сервера, облака и внешнего ИИ.

Минимальный состав:

- Rust Core;
- SQLite local DB;
- CLI interface;
- DailyLog;
- CheckIn;
- Habit;
- HabitEvent;
- ReminderDefinition;
- TimerDefinition;
- AlarmDefinition;
- ContextDocument;
- простые локальные правила;
- тесты ядра.

Примеры CLI-команд:

```bash
adiyutant today
adiyutant checkin morning "спал 6 часов, энергия 5, хочу работать над Rust core"
adiyutant habit add speech
adiyutant habit done speech --level min
adiyutant timer start "focus" --minutes 25
adiyutant reminder add "вечерний speech" --at "21:00"
adiyutant shutdown "собрал ядро, завтра продолжить storage"
```

---

## 11. MVP 0.2

Цель:

> Добавить первый UI-клиент вокруг Core.

Приоритетный вариант:

- Android shell или Desktop shell;
- Core остаётся источником доменной логики;
- UI не дублирует правила;
- реальные будильники и уведомления исполняются платформой.

---

## 12. MVP 0.3

Цель:

> Добавить внешний Agent Gateway без хранения API-ключей в телефоне.

Варианты:

- OpenClaw adapter;
- local desktop relay;
- VPS relay;
- простой custom AI gateway.

Минимальный сценарий:

```text
Mobile/Desktop UI
  -> Adiyutant Core
  -> AgentGateway
  -> OpenClaw or CustomServer
  -> LLM provider
```

---

## 13. MVP 0.4

Цель:

> Добавить sync между устройствами.

Минимальные сущности sync:

```text
Device
DeviceToken
LocalChange
RemoteChange
SyncCursor
EntityRevision
ConflictRecord
```

Принцип:

```text
локальный клиент работает всегда;
сервер синхронизирует изменения;
конфликты документируются и решаются явно;
сервер не ломает локальную работу.
```

---

## 14. Предварительная доменная модель

### 14.1. DailyLog

```text
id
date
mode
sleep_score
energy
mood
raw_notes
ai_summary
created_at
updated_at
```

### 14.2. CheckIn

```text
id
daily_log_id
type: morning / day / evening / shutdown
raw_input
structured_data
agent_response
created_at
```

### 14.3. Habit

```text
id
name
description
frequency
target
is_active
created_at
updated_at
```

### 14.4. HabitEvent

```text
id
habit_id
date_time
status: done / skipped / partial
level: min / light / base / full / custom
comment
```

### 14.5. ReminderDefinition

```text
id
title
message
schedule_rule
next_fire_at
enabled
created_at
updated_at
```

### 14.6. AlarmDefinition

```text
id
title
time
repeat_rule
enabled
platform_binding_id
created_at
updated_at
```

### 14.7. TimerDefinition

```text
id
title
duration_seconds
mode: focus / rest / custom
created_at
```

### 14.8. CalendarEvent

```text
id
title
description
start_at
end_at
location
source
created_at
updated_at
```

### 14.9. ContextDocument

```text
id
type: core / goals / rules / routine / health / work / custom
title
content_markdown
version
is_active
created_at
updated_at
```

### 14.10. ActionProposal

```text
id
source
proposal_type
payload_json
status: pending / accepted / rejected / applied
created_at
applied_at
```

---

## 15. Нефункциональные требования

### 15.1. Local-first

- приложение должно работать без интернета;
- базовые функции не должны зависеть от сервера;
- локальная база является первичным рабочим состоянием клиента.

### 15.2. Portable Core

- доменная логика должна быть переносимой;
- Core не должен зависеть от Android/WPF/Web;
- UI не должен дублировать бизнес-правила.

### 15.3. Testability

- Core должен проверяться через CLI и unit tests;
- бизнес-логика должна тестироваться без UI;
- storage должен тестироваться отдельно.

### 15.4. Security

- API-ключи не хранятся в мобильном клиенте;
- device token не равен provider API key;
- будущий сервер должен разделять user auth, device auth и provider credentials.

### 15.5. AI-safe behavior

- ИИ не должен менять состояние без подтверждения там, где действие критично;
- предложения ИИ должны быть представлены как ActionProposal;
- Core должен валидировать результат ИИ.

---

## 16. Что не входит в MVP 0.1

Не делать в первом этапе:

- backend;
- облачный sync;
- OpenClaw plugin;
- LLM provider integration;
- Android UI;
- WPF/Tauri UI;
- Web UI;
- billing;
- SaaS;
- MCP;
- полноценный календарный сервер;
- сложный rule engine;
- marketplace шаблонов.

---

## 17. Рекомендуемая среда разработки

### 17.1. Основная среда для Core

Рекомендуется разрабатывать `AdiyutantCore` на Ubuntu/Linux.

Причины:

- Rust CLI/storage/backend-разработка проще в Linux-среде;
- SQLite, shell-скрипты, cargo, CI-подобные проверки работают ближе к будущему серверному окружению;
- проще готовить Docker/backend/sync в следующих этапах;
- меньше конфликтов с путями, правами и нативными зависимостями;
- удобнее автоматизация через bash/justfile/make.

### 17.2. Windows использовать для

- WPF/desktop shell;
- проверки кроссплатформенности;
- Visual Studio / Rider / Windows UI;
- тестов сборки под Windows;
- разработки Android, если Android Studio удобнее установлен там.

### 17.3. Практическое правило

```text
Core / CLI / SQLite / future server  -> Ubuntu
WPF / Windows desktop shell          -> Windows
Android                              -> Ubuntu or Windows, где стабильнее Android Studio
Web                                  -> Ubuntu preferred, Windows acceptable
```

Для ближайших выходных:

> Разрабатывать AdiyutantCore лучше на Ubuntu.

---

## 18. Первый технический план

### День 1

- создать/проверить структуру solution;
- создать `AdiyutantCore` Rust workspace;
- добавить crates:
  - `adiyutant_core`;
  - `adiyutant_store`;
  - `adiyutant_cli`;
- описать первые модели;
- сделать `cargo check`;
- добавить первые unit tests.

### День 2

- подключить SQLite;
- сделать миграции или начальный schema bootstrap;
- реализовать команды CLI:
  - `today`;
  - `checkin morning`;
  - `habit add`;
  - `habit done`;
  - `shutdown`;
- добавить README для Core;
- зафиксировать ADR по Core-first подходу.

---

## 19. Acceptance Criteria для MVP 0.1

MVP 0.1 считается успешным, если:

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

## 20. Открытые вопросы

- Как Android будет вызывать Rust Core: UniFFI, C ABI или другой способ?
- Что выбрать для Desktop: Tauri или WPF?
- Как Web будет интегрироваться с Core: backend API или WASM?
- Будет ли OpenClaw первым Agent Gateway?
- Когда добавлять собственный sync server?
- Как решать конфликты между устройствами?
- Где будет выполняться LLM: домашний ПК, VPS, OpenClaw, custom server или локальная модель?

---

## 21. Главная формула проекта

```text
Adiyutant = organizer + local-first core + optional AI agent.

Core owns data and rules.
Platform owns UI and OS integration.
Agent Gateway owns LLM and tools.
Server owns sync, accounts, relay, and cloud state.
```

Первый кирпич проекта — не сервер и не UI.

Первый кирпич проекта:

```text
AdiyutantCore on Rust + SQLite + CLI.
```
