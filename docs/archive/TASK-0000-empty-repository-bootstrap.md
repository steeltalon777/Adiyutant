# TASK-0000 — Empty Repository Bootstrap

Статус: готово к выдаче AI coding agent  
Целевая модель: DeepSeek Flash  
Тип задачи: repository bootstrap / documentation / structure  
Приоритет: critical  
Фаза: Phase 0 — Repository Foundation  
Источник: `SPECIFICATION.md`, `ADIYUTANT_ITERATIVE_ROADMAP.md`  

---

## 1. Краткая цель

Создать начальный каркас пустого репозитория **Adiyutant**.

На этой итерации НЕ нужно писать бизнес-логику, Rust-код, Android-код, Web-код, Desktop-код, backend или интеграции.

Задача итерации — подготовить репозиторий так, чтобы:

- структура solution была понятна человеку;
- AI coding agents могли безопасно ориентироваться в проекте;
- текущий статус проекта был честно обозначен как `bootstrap`;
- будущая архитектура была описана как `Planned`, а не как уже реализованная;
- были созданы корневые документы и папки будущих проектов.

---

## 2. Контекст проекта

**Adiyutant** — локальный органайзер с встроенным ИИ-агентом.

Планируемые функции продукта:

- будильник;
- таймер;
- ежедневник;
- календарь;
- привычки;
- напоминания;
- личный контекст пользователя;
- чат с ИИ-агентом;
- будущая работа агента с инструментами;
- будущая синхронизация между устройствами.

Главная архитектурная идея:

```text
Local-first Core first.
Server later.
AI Gateway outside the Core.
```

Расшифровка:

- сначала создаётся локальное переносимое ядро `AdiyutantCore`;
- ядро будет написано на Rust;
- ядро должно работать без сервера;
- ядро должно работать без внешнего LLM;
- UI-клиенты являются оболочками вокруг ядра;
- backend/server будет добавлен позже для sync, аккаунтов, cloud backup и AI relay;
- LLM API keys не должны храниться в мобильном клиенте;
- OpenClaw может быть одним из будущих Agent Gateway, но не является обязательной зависимостью MVP.

---

## 3. Главное ограничение итерации

Эта итерация создаёт только **структуру и документационный каркас**.

Запрещено:

- создавать Rust workspace;
- создавать Cargo.toml;
- писать Rust-код;
- писать Android-код;
- писать Web-код;
- писать Desktop-код;
- создавать backend;
- добавлять зависимости;
- добавлять LLM API вызовы;
- создавать OpenClaw plugin;
- реализовывать sync;
- реализовывать базу данных;
- создавать тесты к несуществующему коду;
- утверждать, что что-либо уже реализовано, если этого нет в коде.

Разрешено:

- создать директории;
- создать `.gitkeep` в пустых директориях;
- создать документационные файлы;
- создать минимальный `.gitignore`;
- создать `docs/adr/`;
- создать пустые/минимальные placeholder-документы с честным статусом `bootstrap`;
- создать `TASKS.md` с отметкой текущей итерации.

---

## 4. Входное состояние

Ожидается пустой или почти пустой git-репозиторий.

Возможные состояния:

### Вариант A — репозиторий полностью пустой

Нужно создать всю структуру.

### Вариант B — часть файлов или папок уже существует

Нужно:

- не удалять существующие файлы;
- не перезаписывать содержимое без необходимости;
- если файл существует, аккуратно обновить его;
- если структура отличается, привести её к целевой, не ломая существующее.

---

## 5. Целевая структура после итерации

В корне репозитория должна появиться такая структура:

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
  .gitignore

  docs/
    core-boundary.md
    agent-gateway.md
    sync-protocol.md
    adr/
      .gitkeep

  AdiyutantCore/
    README.md
    .gitkeep

  AdiyutantAndroid/
    README.md
    .gitkeep

  AdiyutantWeb/
    README.md
    .gitkeep

  AdiyutantDesktop/
    README.md
    .gitkeep
```

Примечание:

- Если git не отслеживает пустые папки, использовать `.gitkeep`.
- Не создавать вложенные исходники внутри проектных папок.
- Rust workspace появится только в следующей итерации.

---

## 6. Требования к содержанию файлов

### 6.1. Общие правила для всех документов

Каждый документ должен:

- быть кратким, но полезным;
- явно отличать текущее состояние от планов;
- использовать статусы `Current`, `Planned`, `Unknown`, где это уместно;
- не утверждать наличие реализации;
- не использовать маркетинговый стиль;
- не писать “production-ready”, “enterprise-grade”, “scalable”, если это не подтверждено кодом;
- не выдумывать детали будущей реализации.

---

## 7. Корневые файлы

### 7.1. `README.md`

Назначение:

Первый входной документ для человека и AI agent.

Обязательное содержание:

```text
# Adiyutant
```

Разделы:

1. `Project Overview`
2. `Current State`
3. `Repository Type`
4. `Planned Solution Projects`
5. `Architecture Direction`
6. `MVP Focus`
7. `What Is Not Implemented Yet`
8. `Documentation Map`
9. `Development Priority`

Ключевые тезисы:

- Adiyutant is a local-first organizer with an embedded AI agent.
- Current state: bootstrap.
- First MVP focus: `AdiyutantCore`.
- `AdiyutantCore` will be a Rust local-first core.
- Android/Web/Desktop are planned shells.
- Backend/sync/server are planned later.
- AI Gateway is external to Core.
- API keys must not be stored in mobile clients.

---

### 7.2. `SPECIFICATION.md`

Назначение:

Высокоуровневая спецификация продукта.

Если файл уже существует — не удалять, только при необходимости привести к общей структуре.

Минимальные разделы:

1. `Product Definition`
2. `Product Goal`
3. `Architecture Principle`
4. `Solution Composition`
5. `Core Boundary`
6. `Agent Boundary`
7. `MVP 0.1 Scope`
8. `Out of Scope for MVP 0.1`
9. `Open Questions`

Важно:

- Не описывать реализованные функции как готовые.
- Всё будущее писать как `Planned`.

---

### 7.3. `ROADMAP.md`

Назначение:

Глобальная дорога от пустого репозитория до MVP 0.1.

Минимальные разделы:

1. `MVP Goal`
2. `Phase 0 — Repository Foundation`
3. `Phase 1 — Rust Core Workspace`
4. `Phase 2 — Domain Model`
5. `Phase 3 — Local Rules and Today State`
6. `Phase 4 — SQLite Storage`
7. `Phase 5 — CLI MVP`
8. `Phase 6 — Stabilization`

Для текущей итерации указать:

```text
Current iteration: TASK-0000 — Empty Repository Bootstrap
```

---

### 7.4. `TASKS.md`

Назначение:

Текущие и будущие задачи проекта.

Минимальное содержание:

```md
# TASKS

## Current

- [ ] TASK-0000 — Empty Repository Bootstrap

## Next

- [ ] TASK-0001 — AI-friendly documentation baseline
- [ ] TASK-0002 — ADR baseline
- [ ] TASK-0003 — Rust workspace skeleton

## Done

Empty for now.
```

Если агент завершает задачу в рамках этой итерации, он может отметить TASK-0000 как done в финальном изменении:

```md
- [x] TASK-0000 — Empty Repository Bootstrap
```

---

### 7.5. `SOLUTION_MAP.md`

Назначение:

Показать, какие проекты есть в solution и за что они отвечают.

Обязательные разделы:

1. `Solution Overview`
2. `Projects`
3. `Current Responsibility Map`
4. `Planned Responsibility Map`
5. `Unknown / Not Decided`

Содержимое:

```text
AdiyutantCore      Planned Rust local-first domain core
AdiyutantAndroid   Planned Android UI shell
AdiyutantWeb       Planned Web/PWA client
AdiyutantDesktop   Planned Desktop shell
Backend            Not present yet
OpenClaw Adapter   Not present yet
```

---

### 7.6. `ARCHITECTURE.md`

Назначение:

Архитектура текущего состояния и планируемого направления.

Обязательные разделы:

1. `System Overview`
2. `Current State`
3. `Planned High-Level Architecture`
4. `Core Boundary`
5. `Platform Boundary`
6. `Agent Gateway Boundary`
7. `Backend Boundary`
8. `Known Unknowns`

Важно:

Текущий статус должен быть явно указан:

```text
Current state: bootstrap repository only.
No application code is implemented yet.
```

---

### 7.7. `INDEX.md`

Назначение:

Быстрая навигационная карта репозитория.

Обязательные разделы:

1. `Repository Overview`
2. `Root Documents`
3. `Project Directories`
4. `Docs Directory`
5. `Architecture Decisions`
6. `Current Priority`
7. `Open Uncertainties`

---

### 7.8. `AI_CONTEXT.md`

Назначение:

Правила понимания проекта для AI coding agents.

Обязательные тезисы:

- This repository is in bootstrap state.
- Do not assume implementation exists.
- Core is the planned source of domain logic.
- UI clients are planned shells.
- Backend is not present yet.
- Agent Gateway is external to Core.
- Do not store LLM API keys in mobile clients.
- Do not add backend, sync, LLM, MCP, OpenClaw or UI code unless a specific task asks for it.

---

### 7.9. `AI_ENTRY_POINTS.md`

Назначение:

Где AI agent должен начинать чтение.

Для текущего состояния указать:

```text
There are no code entry points yet.
```

Обязательные разделы:

1. `Documentation Entry Points`
2. `Planned Core Entry Points`
3. `Planned Android Entry Points`
4. `Planned Web Entry Points`
5. `Planned Desktop Entry Points`
6. `Not Implemented Yet`

---

### 7.10. `MEMORY.md`

Назначение:

Стабильные проектные факты.

Важно:

Это не changelog и не фантазии.

Минимальные стабильные факты:

- Adiyutant is an organizer with an embedded AI agent.
- MVP 0.1 is local-first.
- First implementation target is `AdiyutantCore`.
- Core must work without backend and without external LLM.
- API keys must not be stored in mobile clients.
- Backend is planned later for sync, account management, cloud backup and AI relay.
- OpenClaw may become one possible Agent Gateway, but is not required for MVP 0.1.

---

### 7.11. `AGENTS.md`

Назначение:

Жёсткие правила работы AI coding agents в репозитории.

Обязательные разделы:

1. `Repository Status`
2. `Primary Priority`
3. `Allowed Changes`
4. `Forbidden Changes`
5. `Documentation Rules`
6. `Core Boundary Rules`
7. `Security Rules`
8. `Verification`

Ключевые правила:

```text
Do not invent implemented architecture.
Do not add source code during TASK-0000.
Do not add backend during TASK-0000.
Do not add LLM provider calls.
Do not store API keys in client projects.
Do not duplicate future Core domain logic in UI projects.
If a document conflicts with an ADR, ADR wins.
If code conflicts with docs in future, document the drift.
```

---

### 7.12. `SECURITY_NOTES.md`

Назначение:

Ранние правила безопасности и приватности.

Минимальные тезисы:

- Mobile clients must not store LLM provider API keys.
- Secrets must not be committed.
- Local data may contain private daily notes and personal context.
- Future sync must consider encryption, device tokens and user consent.
- AI requests must avoid sending unnecessary private context.

---

### 7.13. `GLOSSARY.md`

Назначение:

Доменные термины.

Минимальные термины:

```text
Adiyutant
AdiyutantCore
Check-in
DailyLog
Habit
HabitEvent
ReminderDefinition
AlarmDefinition
TimerDefinition
CalendarEvent
ContextDocument
ActionProposal
Agent Gateway
Sync
Local-first
Platform Shell
```

---

## 8. Файлы в `docs/`

### 8.1. `docs/core-boundary.md`

Назначение:

Зафиксировать границу Core.

Разделы:

1. `Purpose`
2. `Core Owns`
3. `Core Does Not Own`
4. `Platform Owns`
5. `Backend Owns Later`
6. `Agent Gateway Owns Later`
7. `Open Questions`

Ключевая формула:

```text
Core stores meaning.
Platforms execute OS-specific behavior.
Agent Gateway handles AI/provider/tool execution.
Backend handles sync/account/cloud concerns later.
```

---

### 8.2. `docs/agent-gateway.md`

Назначение:

Зафиксировать, что ИИ-агент находится вне Core.

Разделы:

1. `Purpose`
2. `Current State`
3. `Planned Role`
4. `Possible Implementations`
5. `Security Rule`
6. `MVP 0.1 Decision`
7. `Open Questions`

Ключевые тезисы:

- Agent Gateway is not implemented in MVP 0.1.
- Core must not call LLM providers directly.
- Mobile client must not store provider API keys.
- Possible future implementations: custom server, OpenClaw adapter, local relay, mock gateway.

---

### 8.3. `docs/sync-protocol.md`

Назначение:

Ранний документ по будущему sync.

На TASK-0000 не проектировать детальный протокол.

Минимальное содержание:

- sync is planned later;
- MVP 0.1 works without sync;
- local client is primary for MVP;
- future backend may manage accounts, devices, cloud backup and conflict handling;
- conflict resolution is unknown.

---

### 8.4. `docs/adr/`

Создать директорию.

На TASK-0000 не создавать полноценные ADR, если нет отдельного указания.

Создать только:

```text
docs/adr/.gitkeep
```

ADR baseline будет отдельной итерацией `TASK-0002`.

---

## 9. Проектные папки

### 9.1. `AdiyutantCore/README.md`

Минимальное содержание:

```md
# AdiyutantCore

Status: Planned / not implemented yet.

AdiyutantCore is planned as the Rust local-first domain core for Adiyutant.

It will own domain models, local storage, local rules, CLI validation, and future sync state preparation.

No Rust workspace is created in TASK-0000.
```

---

### 9.2. `AdiyutantAndroid/README.md`

Минимальное содержание:

```md
# AdiyutantAndroid

Status: Planned / not implemented yet.

AdiyutantAndroid is planned as the Android UI shell for Adiyutant.

It should not duplicate Core domain logic.

Future Core integration method is unknown.
```

---

### 9.3. `AdiyutantWeb/README.md`

Минимальное содержание:

```md
# AdiyutantWeb

Status: Planned / not implemented yet.

AdiyutantWeb is planned as a Web/PWA client or dashboard.

Future integration with Core is unknown.
Possible options: backend API, WebAssembly, or sync-server-based client.
```

---

### 9.4. `AdiyutantDesktop/README.md`

Минимальное содержание:

```md
# AdiyutantDesktop

Status: Planned / not implemented yet.

AdiyutantDesktop is planned as a desktop UI shell for Adiyutant.

Possible future options: Tauri, WPF, or another desktop shell.

It should not duplicate Core domain logic.
```

---

## 10. `.gitignore`

Создать минимальный `.gitignore`.

Должен включать:

```gitignore
# OS / editors
.DS_Store
Thumbs.db
.vscode/
.idea/

# Environment / secrets
.env
.env.*
*.pem
*.key
secrets/

# Rust future artifacts
target/
Cargo.lock

# Node / web future artifacts
node_modules/
dist/
build/

# .NET future artifacts
bin/
obj/

# Android future artifacts
.gradle/
local.properties
*.apk
*.aab

# Local databases
*.sqlite
*.sqlite3
*.db

# Logs
*.log
logs/
```

Примечание:

`Cargo.lock` можно будет пересмотреть позже. Для бинарного Rust workspace его часто стоит коммитить, но на TASK-0000 Rust workspace ещё нет. Пока допустимо держать в ignore как временное правило или добавить комментарий `review later`.

---

## 11. Проверки после выполнения

AI agent должен проверить:

### 11.1. Структура

Команда для Linux/macOS/Git Bash:

```bash
find . -maxdepth 3 -type f | sort
find . -maxdepth 3 -type d | sort
```

Команда для PowerShell:

```powershell
Get-ChildItem -Recurse -Depth 3 | Select-Object FullName
```

### 11.2. Git status

```bash
git status --short
```

### 11.3. Проверка отсутствия исходного кода

Убедиться, что не созданы:

```text
Cargo.toml
*.rs
*.kt
*.cs
package.json
manage.py
pyproject.toml
Dockerfile
docker-compose.yml
```

Если какие-то из этих файлов уже существовали до задачи, не удалять их, но указать в финальном отчёте.

---

## 12. Acceptance Criteria

Итерация считается выполненной, если:

- [ ] создана корневая документационная структура;
- [ ] созданы директории `docs/`, `docs/adr/`;
- [ ] созданы директории `AdiyutantCore/`, `AdiyutantAndroid/`, `AdiyutantWeb/`, `AdiyutantDesktop/`;
- [ ] во всех пустых директориях есть `.gitkeep`;
- [ ] создан `README.md` в корне;
- [ ] создан `SPECIFICATION.md`;
- [ ] создан `ROADMAP.md`;
- [ ] создан `TASKS.md`;
- [ ] создан `SOLUTION_MAP.md`;
- [ ] создан `ARCHITECTURE.md`;
- [ ] создан `INDEX.md`;
- [ ] создан `AI_CONTEXT.md`;
- [ ] создан `AI_ENTRY_POINTS.md`;
- [ ] создан `MEMORY.md`;
- [ ] создан `AGENTS.md`;
- [ ] создан `SECURITY_NOTES.md`;
- [ ] создан `GLOSSARY.md`;
- [ ] создан `docs/core-boundary.md`;
- [ ] создан `docs/agent-gateway.md`;
- [ ] создан `docs/sync-protocol.md`;
- [ ] создан `.gitignore`;
- [ ] документы явно указывают `Current state: bootstrap` или аналогичную формулировку;
- [ ] документы не утверждают наличие реализованного кода;
- [ ] не создана реализация Core;
- [ ] не создан backend;
- [ ] не созданы LLM/API интеграции;
- [ ] не созданы platform UI implementations.

---

## 13. Definition of Done

После завершения агент должен вывести краткий отчёт:

```text
TASK-0000 completed.

Created:
- list of created root docs
- list of created docs files
- list of created project directories

Updated:
- list of updated existing files, if any

Not created by design:
- Rust workspace
- backend
- UI implementations
- LLM integration

Verification:
- git status checked
- repository structure checked
- no source code added

Notes / uncertainties:
- any existing files or deviations
```

---

## 14. Рекомендуемый commit

После ручной проверки человеком:

```bash
git add .
git commit -m "docs: bootstrap adiyutant repository"
```

AI agent не обязан сам делать commit, если это не разрешено отдельно.

---

## 15. Важное напоминание для AI agent

This task is documentation and structure only.

Do not be helpful by adding implementation.

Do not create Rust workspace.

Do not create backend.

Do not create UI code.

Do not create LLM integration.

Do not invent completed architecture.

The correct result is a clean bootstrap repository, not a half-implemented application.
