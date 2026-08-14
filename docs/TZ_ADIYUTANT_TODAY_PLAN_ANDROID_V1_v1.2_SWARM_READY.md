# TZ: Adiyutant Today + Plan v1 — Android / Kotlin / Jetpack Compose

| Поле | Значение |
|---|---|
| Статус | Ready for implementation — ревизия 1.2: swarm-ready execution, корректный timer anchor, lossless-cancel semantics, сохранение interrupted fact |
| Дата | 2026-08-14 |
| Автор | Architect |
| Источники | `AdiyutantAndroid/design/App.jsx` (поведенческий source of truth), `AdiyutantAndroid/design/DESIGN.md` (визуальный/UX), `AdiyutantAndroid/design/MEMORY.md` (архитектурные решения и инварианты) |
| Целевой код | `AdiyutantAndroid/app/src/main/kotlin/com/adiyutant/app/` |
| Версия | v1 — behavioral parity с прототипом, in-memory, без backend |
| Язык UI | Русский |
| Запрещено | менять `App.jsx`, `DESIGN.md`, `MEMORY.md`, Rust Core, экраны Проекты/Время/Настройки |

> **Правило переноса.** App.jsx — поведенческая спецификация, но не архитектурный образец. Механический перевод JSX → Compose запрещён. Переносятся: доменные инварианты, переходы состояний, контракты листов (sheets), производные вычисления. Структура файлов и способ хранения — нативные решения Kotlin из этого ТЗ.

---

## Execution strategy

- **🟢 Staged Swarm, максимум 2 параллельных writer-а.** Общий `PlannerCore`/`PlannerState` и shell нельзя редактировать конкурентно: сначала замораживаются контракты, затем экраны расходятся по независимым зонам владения.
- **Wave A — single writer / Phase 1:** агент A владеет `domain/` + `data/`; реализует модели, reducer, validators, Clock/TimeMath и unit-тесты. Выходной gate: заморожены публичные контракты `PlannerState`, `PlannerEvent`, `ScheduleRequest`, `SheetState`; Phase 1 tests зелёные.
- **Wave B — single integrator / Phase 2 + Phase 3A:** интегратор владеет `ui/planner/`, `AppShell`, `AppNavGraph` и общими UI-контрактами; реализует hoisting VM и общий scaffold листов. Агент B затем единолично реализует `ui/common/` (`ScheduleSheet`, `TaskDetailsSheet` и базовые controls). Gate: `assembleDebug` зелёный, API `ui/common/` заморожен.
- **Wave C — parallel, максимум 2 потока:** агент B владеет только `ui/feature/plan/` (Phase 3B), агент C — только `ui/feature/today/` (Phase 4). Оба считают `domain/`, `ui/planner/`, `ui/common/`, `AppShell` и `AppNavGraph` read-only. Любая необходимость менять замороженный контракт возвращается интегратору отдельным change request, а не правится параллельно.
- **Wave D — single integrator / Phase 5:** merge Plan + Today, cross-screen flows, C1–C10, Agent navigation, DoElse/EOD integration. После merge обязательны unit + `assembleDebug` + smoke.
- **Wave E — visual polish / Phase 6:** допустимы два параллельных потока по экранным пакетам при неизменных domain/state contracts.
- **Wave F — single integrator / Phase 7:** полный build, regression, emulator acceptance и evidence table. Тесты каждой фазы пишутся вместе с кодом, а не откладываются на конец.

## Execution checklist

- [x] 0. Контекст сверен: App.jsx, DESIGN.md, MEMORY.md, текущий Android-проект, ADR-0002/0003/0004, ROADMAP; файл `docs/adr/ADR-0004-android-local-planner-domain.md` реально существует
- [x] 1. Phase 1 — domain-модель, `PlannerCore` (reducer), `ScheduleValidator`, `TimeMath`, `Clock`, seed-данные, in-memory репозиторий
- [x] 2. Phase 2 — `PlannerViewModel` (StateFlow), hoisting в `AppShell`, изменение сигнатуры `AppNavGraph`, host общих листов, обновление smoke-теста
- [x] 2.1. Review fixes Phase 0–2 (после ревью): инвариант END_DAY cancel восстанавливает `EndDaySheet`, план-факт разделение закрыто тестами, единая формула elapsed, smoke-тест обновлён
- [x] 3A. **Phase 3A — Shared UI gate (ACCEPTED AND FROZEN, 2026-08-14).** `ui/common/` (AdiyutantSheet, Label, SegmentedControl, EmojiScale, MoodPicker, KpiMini, Chip, ScheduleSheet, TaskDetailsSheet, ScheduleSheetLogic) реализован; публичные сигнатуры заморожены; `collectAsStateWithLifecycle` во всех потребителях state; `rememberSaveable` в ScheduleSheet привязан к identity `ScheduleRequest` (form-state изоляция). Сквозные контракты `PlannerState`/`PlannerEvent`/`ScheduleRequest`/`PlannerCore`/`PlannerRepository`/`PlannerViewModel` остаются замороженными от Phase 2.1.
- [ ] 3B. Phase 3B — Plan v1: День/Неделя/Бэклог, DayPickerStrip, Workload, Timeline, PlanAgentCard; **read-only** для domain/, data/, ui/planner/, ui/common/, ui/shell/, ui/navigation/, core/. Любая правка замороженного файла — отдельный change request
- [ ] 4. Phase 4 — Today v1: hero state machine, KPI, Check-in, Mode, Progress, Top 3, Quick Note, Agent, все Today-листы; после gate Phase 3A может выполняться параллельно с Phase 3B; **read-only** те же каталоги. `PlannerEvent.OpenCompletionSheet(blockId)` уже реализован и принят в Phase 2.1 (PlannerCore.kt:188–194; принимает только при наличии активной PLANNED-сессии для этого блока; `progressPercent < 100`); `completionConfirmed` принимается из `BlockEndedSheet` или `CompletionSheet` (PlannerCore.kt:201–205); покрыто `PlannerCoreStateMachineTest` (6 тестов). Phase 4 использует frozen event/API без дополнительных prerequisites
- [ ] 5. Phase 5 — кросс-экранные сценарии: «Делаю другое» (двухшаговый), End of Day → ScheduleSheet, Agent-навигация, синхронизация Today ↔ Plan
- [ ] 6. Phase 6 — визуальная доводка по DESIGN.md (токены, градиенты, отступы, insets, touch targets ≥44dp, hardcoded `–` в ScheduleSheet, accessibility-state confirm-кнопки)
- [x] 7. Unit-тесты (`./gradlew testDebugUnitTest`) — **118 / 118, 0 failures, 0 errors, 0 skipped** на момент закрытия Phase 3A. Распределение: PlannerCoreStateMachine 28, ScheduleSheetContract 11, EndOfDay 11, TimeMath 11, DoSomethingElse 10, ScheduleSheetLogic 9, AgentInsight 9, ElapsedTime 7, SingleEntityRule 6, QuickNote 6, WorkloadDerivation 5, AppShellSmoke 3, FakeCorePort 2
- [x] 8. `./gradlew compileDebugKotlin` — **PASS**; `./gradlew assembleDebug` — **PASS** (APK собран). `./gradlew build` (полный, со всеми вариантами и проверками) — **НЕ ВЫПОЛНЕН** (Phase 7)
- [ ] 9. Smoke на эмуляторе: запуск, 5 вкладок, навигация не сбрасывает активную сессию — **НЕ ВЫПОЛНЕН** (эмулятор недоступен в окружении; остаётся Phase 7). Реальный Compose UI test (scrim, BackHandler LIFO, form-state isolation) — **НЕ ВЫПОЛНЕН**
- [ ] 10. User-сценарии по чек-листу приёмки (раздел 14) — сценарии C1–C10 на эмуляторе; ожидает Phase 7
- [ ] 11. Регрессия: `FakeCorePortTest` зелёный; `AppShellSmokeTest` зелёный (но это JVM-статический тест, **НЕ реальный Compose UI smoke** — для Phase 7 требуется androidTest)
- [ ] 12. Документация: таблица доказательств заполнена (см. §14.3, ниже); дрейфы зафиксированы в ADR-0004
- [ ] 13. Финальный acceptance review по evidence table — Phase 7

**Phase 3A — принято и заморожено.** См. §13.1 (структура файлов) и §15 (Phase 3A gate). Дальнейшие изменения `ui/common/*`, `domain/*`, `data/*`, `ui/planner/*`, `ui/shell/*`, `ui/navigation/*`, `core/*` — только через integrator с явным change request.

**Известно stale:** finding W3 из пред-Phase 3A ревью (отмена ScheduleSheet с purpose=END_DAY не восстанавливает EndDaySheet) — **НЕ подтверждается**. `PlannerCore.cancelSheet` (PlannerCore.kt:437–449) уже обрабатывает `sheet.request.purpose == SchedulePurpose.END_DAY` отдельной веткой: возвращает `EndDaySheet`, оставляет `dayPhase = END_OF_DAY`, очищает `pendingDoElse`. Покрыто `EndOfDayTest.cancelEndDayScheduleRestoresEndDaySheetLossless` и `cancelEndDayScheduleKeepsActiveSessionAndSourceBlock`.

---

## 1. Цель реализации

Перенести поведение двух экранов интерактивного прототипа (App.jsx: Today + Plan) в нативный Android-клиент на Kotlin / Jetpack Compose, сохранив все доменные инварианты и кросс-экранное поведение:

- **План** строит план (намерение): задачи, даты, время, бэклог.
- **Сегодня** исполняет план (факт): чек-ин, KPI, state-machine hero, запуск/пауза/завершение, «Делаю другое», заметки, итоги дня.
- Today и Plan используют **один общий state**, переживающий переключение вкладок.

**Входит в scope:**

- Domain-модель: Task → ScheduledBlock → WorkSession (+ CheckIn, QuickNote, день/исполнение).
- Машина состояний дня с 9 каноническими состояниями.
- Plan v1: День / Неделя / Бэклог, workload, timeline, ScheduleSheet, TaskDetailsSheet, Agent.
- Today v1: header, KPI, hero, Agent, Top 3, Quick Note, Check-in, End of Day.
- Единый ScheduleSheet с контрактом schedule/reschedule (общий для обоих экранов).
- In-memory хранилище с seed-данными, эквивалентными прототипу.
- Unit-тесты доменных инвариантов и переходов.

**Сознательно НЕ входит** (детали — раздел «Out of scope»): persistence, backend, sync, notifications, бизнес-логика Проектов/Времени/Настроек, интеграция Rust Core, новые AI-возможности, создание задач через UI.

## 2. Текущая Android-архитектура

Существующее состояние (Phase 9 bootstrap выполнен, см. `ROADMAP.md`):

| Элемент | Файл | Роль |
|---|---|---|
| Стек | ADR-0002 | Kotlin + Compose + Material 3 (как инфраструктура) + Navigation Compose + ViewModel/StateFlow |
| Вход | `MainActivity.kt` → `AdiyutantTheme { AppShell() }` | edge-to-edge, dark-тема |
| Shell | `ui/shell/AppShell.kt` | `Scaffold` + `BottomNavBar` + `AppNavGraph` |
| Навигация | `ui/navigation/AppNavGraph.kt`, `Destinations.kt` | 5 маршрутов-вкладок: today / plan / projects / time / settings |
| Тема | `ui/theme/Color.kt`, `Type.kt`, `Shape.kt`, `AdiyutantTheme.kt` | токены UI Kit есть; **не хватает `AccentRed`/`AccentRedDeep`** (добавить, раздел 12) |
| Граница Core | `core/CorePort.kt` + `core/fake/FakeCorePort.kt` + `core/model/` | шов под будущий Rust Core (ADR-0002/0003) |
| DI | `di/ServiceLocator.kt` | ручной контейнер |
| Экраны | `ui/feature/today/` (stub + `TodayViewModel` на CorePort), `ui/feature/plan/`, `projects/`, `time/`, `settings/` (stub) | стабы |
| Тесты | `FakeCorePortTest`, `AppShellSmokeTest` | должны остаться зелёными |
| Сборка | `app/build.gradle.kts` | compileSdk 36, minSdk 26 (java.time без desugaring), Compose BOM 2025.08.01, lifecycle 2.9.1, navigation 2.9.0, JUnit 4 |

**Куда встраиваются новые компоненты** (детали — раздел 13):

- Новые пакеты `domain/`, `data/` — чистая доменная логика без зависимостей от Android (кроме `java.time`).
- Новый пакет `ui/planner/` — общий ViewModel и его UI-state.
- Новый пакет `ui/common/` — общие листы и контролы (включая `ScheduleSheet`, `TaskDetailsSheet`).
- `ui/feature/today/` и `ui/feature/plan/` переписываются на общий state; остальные feature-пакеты не трогаются.
- `AppShell` и `AppNavGraph` меняются минимально: hoisting ViewModel на уровень Activity + host общих листов.

**Архитектурное решение R1 (дрейф против ADR-0002 — зафиксировать):** Rust Core сейчас не содержит домена Task/ScheduledBlock/WorkSession. MEMORY.md явно предписывает для первого милстоуна Kotlin-порта «local/in-memory repository first», модули `domain`/`data` и нативные модели `Task`, `ScheduledBlock`, `WorkSession`, `CheckIn`, `QuickNote`. Поэтому v1 строит домен **локально в Android** (`domain/` + `data/`), а `CorePort`/`FakeCorePort`/`TodaySnapshot`/`CurrentActivity` **не трогаются** — это отдельный шов Rust-интеграции (ADR-0003). Интерфейс `data/PlannerRepository` — будущая точка замены in-memory реализации на Core-backed. Дрейф против буквы ADR-0002 («UI потребляет только CorePort») зафиксирован отдельным документом **ADR-0004** (решение владельца продукта; см. `docs/adr/ADR-0004-android-local-planner-domain.md`).

**Последствия R1:** существующий `TodayViewModel` (stub на CorePort) **удаляется**, `TodayScreen` переписывается. `FakeCorePortTest` не зависит от экранов — остаётся зелёным. `AppShellSmokeTest` обновляется под новые экраны (Phase 2).

## 3. Domain model

### 3.1 Общая схема идентичности (обязательный инвариант)

```text
Task.taskId           — стабильная логическая сущность (никогда не меняется)
    ↓
ScheduledBlock.blockId — одно размещение во времени (может менять дату/время, сохраняя blockId)
    ↓
WorkSession           — фактическое выполнение (факт ≠ план)
```

- `reschedule` **никогда** не меняет `taskId`.
- Top 3, completion, будущие Проекты и аналитика ссылаются на `taskId`, а не на `blockId`.
- `Task` с `plannedDate`, но без выбранного времени — это **не** `ScheduledBlock` («Есть дата, нет времени»).

### 3.2 Task

```kotlin
data class Task(
    val taskId: String,          // стабильная логическая идентичность, уникальна
    val title: String,
    val project: String,         // "" = без проекта
    val durationMin: Int,        // плановая длительность, > 0
    val priority: Priority,      // HIGH | MEDIUM | LOW
    val plannedDate: LocalDate?, // null → «Без даты»; задана и нет блока → «Есть дата, нет времени»
    val postponed: Boolean,      // true → группа «Отложенные»
)
```

Инварианты:

- `taskId` уникален во всём каталоге; перенос никогда не клонирует задачу.
- **Single-entity rule:** если существует `ScheduledBlock(taskId)` ⇒ у `Task` `plannedDate == null` (дата выводится из блока). Одна логическая задача не существует одновременно как date-only Task и как ScheduledBlock. Любая операция планирования/снятия планирования обязана поддерживать это правило атомарно.
- `postponed == true` ⇒ `plannedDate == null`.
- `title`, `project`, `durationMin`, `priority` переносятся между представлениями без изменений.

**Отличие от JSX (нормализация, не изменение поведения):** в прототипе поля задачи дублируются в блоках (`title`, `project` внутри `INIT_SCHEDULED`) и есть две коллекции. В Kotlin — **единый каталог `tasks`** + отдельный список `blocks`; UI-поля блока резолвятся через `taskId`. Это устраняет класс ошибок дублирования; поведение эквивалентно.

Производные представления (функции от state, см. 4.3):

- date-only задача на дату D: `task.plannedDate == D && нет блока для taskId`.
- Бэклог «Без даты»: `plannedDate == null && !postponed && нет блока для taskId`.
- Бэклог «Есть дата, нет времени»: `plannedDate != null && !postponed && нет блока для taskId`.
- Бэклог «Отложенные»: `postponed == true && нет блока для taskId`.

**Инвариант проекций (обязательный):** Backlog и date-only проекции ОБЯЗАНЫ исключать любую задачу, у которой есть `ScheduledBlock` с тем же `taskId`. Так как в едином каталоге `Task` остаётся даже после планирования (с `plannedDate == null`), без этого условия все scheduled-задачи появятся в бэклоге («фантомные задачи»).

### 3.3 ScheduledBlock

```kotlin
data class ScheduledBlock(
    val blockId: String,       // уникальная идентичность размещения
    val taskId: String,        // → Task (title/project НЕ копируются)
    val date: LocalDate,
    val start: LocalTime,
    val end: LocalTime,        // end > start
    val progressPercent: Int,  // 0..100; 100 = завершено
    val accent: BlockAccent,   // BLUE | PURPLE | GREEN | ORANGE | NONE (полоса в timeline)
)
```

Инварианты:

- `blockId` уникален.
- `end > start`; рабочий день 09:00–19:00 (валидация при планировании; seed-данные соответствуют).
- На одной дате блоки не пересекаются (валидация при планировании).
- Same-date reschedule обновляет существующий блок **на месте** (`blockId` сохраняется).
- Cross-date move удаляет старое размещение и создаёт на целевой дате блок с **тем же `blockId`** (JSX: `movedBlock={...block, time, end}`).
- `progressPercent` — состояние завершённости задачи в v1 (single-entity: у задачи одно размещение, поэтому прогресс блока = прогресс задачи).
- **Инвариант завершения (обязательный):** `ScheduledBlock` с `progressPercent == 100` НЕ ДОЛЖЕН иметь активную PLANNED-сессию, ссылающуюся на его `blockId` (включая сессию на паузе). Reducer обязан отклонять (no-op) любые события, нарушающие этот инвариант (`TaskDetailsComplete`, `TopThreeComplete` при активной сессии этого блока); UI обязан направлять такие случаи в `CompletionSheet` (Today) или CTA «Открыть в Сегодня» (Plan, см. 6.6).

**Генерация `blockId`** (детерминированно, без UUID): `"s_" + taskId + "_" + yyyyMMdd(date)`. Коллизий нет: задача может иметь максимум одно размещение. `blockId` непрозрачен — формат не должен разбираться кодом.

`BlockAccent` выводится функцией `accentFor(priority, project)` (см. 4.4); в seed может задаваться явно (обед = NONE).

### 3.4 WorkSession (завершённый факт)

```kotlin
data class WorkSession(
    val sessionId: String,       // уникальна
    val taskId: String?,         // null для незапланированной работы
    val blockId: String?,
    val parentBlockId: String?,  // исходный плановый блок при «Делаю другое»
    val kind: SessionKind,       // PLANNED_COMPLETED | PLANNED_INTERRUPTED | UNPLANNED | BREAK | PERSONAL | OTHER | STOPPED_AT_DAY_END
    val label: String?,          // для UNPLANNED/BREAK/PERSONAL/OTHER
    val startedAtMs: Long,
    val endedAtMs: Long,
    val durationSec: Long,       // зафиксировано в момент завершения
    val ease: Ease?,             // EASY | NORMAL | HARD, только для PLANNED_COMPLETED
)
```

Инварианты:

- **Plan ≠ Fact.** WorkSession описывает реальность; ScheduledBlock — намерение. Плановое окончание блока **никогда** не создаёт WorkSession автоматически и не завершает задачу.
- Незапланированная работа создаёт сессию **без** `taskId`/`blockId` и **не** завершает исходный плановый блок.
- `durationSec` вычисляется один раз в момент завершения из timestamps + accumulated (см. 4.5); не хранится как параллельный источник истины.
- Если пользователь уходит с уже запущенной PLANNED-сессии через «Делаю другое», отработанный кусок **не теряется**: при подтверждении переключения создаётся `WorkSession(kind=PLANNED_INTERRUPTED, progress не меняется)`. Отмена DoElse/ScheduleSheet оставляет исходную active session нетронутой.

### 3.5 ActiveSession (текущая сессия)

```kotlin
data class ActiveSession(
    val kind: ActiveKind,        // PLANNED (из блока) | UNPLANNED_WORK (unplanned/break/personal/other)
    val taskId: String?,         // PLANNED: taskId блока; иначе null
    val blockId: String?,        // PLANNED: blockId; иначе null
    val parentBlockId: String?,  // исходный блок «Делаю другое»
    val label: String?,          // «Незапланированная работа» / «Перерыв» / «Личное» / custom
    val startedAtMs: Long,       // immutable: фактический старт всей сессии, НЕ меняется после Resume
    val runningSinceMs: Long?,   // старт текущего активного сегмента; null, когда paused=true
    val accumulatedMs: Long,     // сумма завершённых активных сегментов до runningSinceMs
    val paused: Boolean,
    val plannedStart: LocalTime?, // копия time/end блока (для PLANNED)
    val plannedEnd: LocalTime?,
)
```

Elapsed (единственная формула, см. 4.5):

```kotlin
fun elapsedMs(session: ActiveSession, nowMs: Long): Long =
    session.accumulatedMs +
        if (session.paused || session.runningSinceMs == null) 0L
        else (nowMs - session.runningSinceMs)
```

Инварианты active session:

- В `PlannerState` одновременно может существовать **не более одной** `ActiveSession`.
- `StartBlock` / `TopThreeStart` — no-op, если `activeSession != null`, блок отсутствует или `progressPercent == 100`. UI также скрывает/блокирует старт при активной сессии, reducer служит последней защитой.
- `startedAtMs` — неизменяемый timestamp первого старта сессии и позже копируется в `WorkSession.startedAtMs`; Pause/Resume меняют только `runningSinceMs`, `accumulatedMs`, `paused`.

### 3.6 CheckIn

```kotlin
data class CheckIn(
    val energy: Int,       // 0..3 (·, ⚡, ⚡⚡, ⚡⚡⚡)
    val focus: Int,        // 0..3 (туман, средний, острый, пик)
    val mood: Mood,        // BAD | OK | GOOD
    val obstacle: String?, // необязательно
    val atMs: Long,
)
```

Первый чек-ин может начать день; последующие обновляют текущее состояние. Значение energy синхронизируется с KPI `dayEnergy`.

### 3.7 QuickNote

```kotlin
data class QuickNote(
    val noteId: String,
    val text: String,           // непустой
    val kind: NoteKind,         // NOTE | IDEA | PROBLEM | SUMMARY
    val atMs: Long,
    val contextLabel: String?,  // заголовок текущего блока или label активной сессии
)
```

Хранится не более 5 (последняя первой), отображается 3.

### 3.8 DayPhase / HeroState (день и hero)

`DayPhase` — сохраняемое состояние дня (4 значения):

```kotlin
enum class DayPhase { DAY_NOT_STARTED, RUNNING, END_OF_DAY, DAY_COMPLETED }
```

`HeroState` — производное состояние hero (9 канонических значений из MEMORY.md):

```kotlin
enum class HeroState {
    DAY_NOT_STARTED, UPCOMING, READY, ACTIVE, PAUSED,
    BLOCK_ENDED_UNRESOLVED, BETWEEN_BLOCKS, END_OF_DAY, DAY_COMPLETED,
}
```

Вывод `deriveHeroState(state, nowMs)` (детали — раздел 7):

1. `activeSession != null` → `PAUSED`, если `paused`; `BLOCK_ENDED_UNRESOLVED`, если открыт BlockEndedSheet; иначе `ACTIVE`.
2. `dayPhase == DAY_NOT_STARTED` → `DAY_NOT_STARTED`.
3. `dayPhase == END_OF_DAY` → `END_OF_DAY`.
4. `dayPhase == DAY_COMPLETED` → `DAY_COMPLETED`.
5. `dayPhase == RUNNING`: есть незавершённый блок, чьё окно содержит `now` → `READY`; иначе есть незавершённый блок со `start > now` → `UPCOMING`; иначе `BETWEEN_BLOCKS`.

**Нормализация против JSX (важно):** в прототипе `upcoming`/`ready` достигаются только через demo-контролы (сравнение с замороженным 14:30 и `dayStatus==='upcoming'/'ready'`), а в реальном потоке после чек-ина hero всегда падает в `between`. Это артефакт прототипа. Нативное поведение: `UPCOMING`/`READY` выводятся из **реального** времени `now` (инъекция `Clock`); demo-контролы не переносятся. Визуально `UPCOMING` рендерит ту же карточку, что и `BETWEEN_BLOCKS` (как в JSX — отдельного варианта нет), но логически это разные состояния, оба тестируются.

### 3.9 Остальные перечисления

```kotlin
enum class DayMode { FOCUS, NORMAL, LIGHT, RECOVERY }      // Фокус/Обычный/Лёгкий/Восстановление
enum class Ease { EASY, NORMAL, HARD }                      // Легко/Нормально/Тяжело
enum class Mood { BAD, OK, GOOD }                           // Плохо/Нормально/Хорошо
enum class NoteKind { NOTE, IDEA, PROBLEM, SUMMARY }        // Заметка/Идея/Проблема/Итог
enum class Priority { HIGH, MEDIUM, LOW }
enum class BlockAccent { BLUE, PURPLE, GREEN, ORANGE, NONE }
enum class PlanMode { DAY, WEEK, BACKLOG }                  // День/Неделя/Бэклог, default WEEK
enum class ActiveKind { PLANNED, UNPLANNED_WORK }
enum class SessionKind { PLANNED_COMPLETED, PLANNED_INTERRUPTED, UNPLANNED, BREAK, PERSONAL, OTHER, STOPPED_AT_DAY_END }
enum class DoElseKind { OTHER_TASK, UNPLANNED, BREAK, PERSONAL, OTHER }
enum class DoElseHandling { LEAVE, SHIFT, MOVE }
enum class NotDoneAction { MOVE, TO_BACKLOG, LEAVE }
enum class EndDayAction { KEEP_TOMORROW, TO_BACKLOG, TO_DATE, CHOOSE_TIME }
data class EndDayReview(val atMs: Long, val energy: Int)
```

## 4. Shared state architecture

### 4.1 Архитектурные решения (R2–R6)

- **R2. Единый state holder.** Один `PlannerViewModel` (StateFlow) на Activity scope; Today и Plan получают один и тот же экземпляр. `PlannerViewModel` тонкий: `dispatch(event) { _state.value = core.reduce(_state.value, event) }`. Вся бизнес-логика — в чистом синхронном `PlannerCore.reduce(state, event): PlannerState` (без корутин, без Android) → покрывается JVM-тестами без эмулятора.
- **R3. Immutable state + event-driven updates.** UI никогда не мутирует домен. Все изменения — через `PlannerEvent` (sealed). Поля формы открытого листа хранятся локально в composable (`rememberSaveable`); в VM уходит только подтверждённый результат.
- **R4. Elapsed — производная величина.** `startedAtMs` неизменяем и хранит начало всей сессии; текущий running-сегмент якорится `runningSinceMs`. `elapsedMs = accumulatedMs + (nowMs - runningSinceMs)` при работе и только `accumulatedMs` при паузе. Таймер в UI — локальный тикер (см. 4.6); параллельного счётчика секунд нет.
- **R5. Cancel всегда lossless для бизнес-данных.** Закрытие листа не меняет `tasks`, `blocks`, `factSessions`, `quickNotesToday` и не обрывает `activeSession`. Допустимы только транзитные workflow-изменения: `sheet=null`, сброс `pendingDoElse`; для отмены EndDaySheet — восстановление `dayPhase=RUNNING`. Планирование/факт меняются только в confirm-ветках reducer'а.
- **R6. Редьюсер защищает инварианты.** `PlannerCore` повторно валидирует confirm-данные (время, пересечения, single-entity) через `ScheduleValidator`; невалидный confirm — no-op. Дополнительно reducer запрещает вторую одновременную active session и запуск завершённого блока. Тесты бьют по инвариантам напрямую через события.

### 4.2 Что в общем state (VM) — обязательно

```kotlin
data class PlannerState(
    val todayDate: LocalDate,                    // зафиксирован при старте (ограничение 4.7)
    val tasks: List<Task>,                       // единый каталог
    val blocks: List<ScheduledBlock>,            // все размещения
    val activeSession: ActiveSession?,           // переживает переключение вкладок
    val factSessions: List<WorkSession>,         // завершённые факт-сессии
    val dayCheckIn: CheckIn?,
    val dayPhase: DayPhase,
    val dayMode: DayMode,
    val dayEnergy: Int,                          // 0..3, последний чек-ин
    val todayTop: List<String>,                  // taskId, максимум 3
    val quickNotesToday: List<QuickNote>,        // максимум 5
    val endDayReview: EndDayReview?,
    val planUi: PlanUiState,                     // mode/selectedDate/viewMonth — переживает вкладки
    val sheet: SheetState?,                      // единственный активный лист VM-уровня
    val pendingDoElse: PendingDoElse?,           // двухшаговый «Делаю другое» → ждёт ScheduleSheet
)

data class PlanUiState(
    val mode: PlanMode = PlanMode.WEEK,
    val selectedDate: LocalDate,                 // default = today
    val viewMonth: YearMonth,                    // для недельной сетки
)

sealed interface SheetState {
    data object CheckInSheet : SheetState
    data object ModeSheet : SheetState
    data object ProgressSheet : SheetState
    data class DoElseSheet(val blockId: String) : SheetState
    data class BlockEndedSheet(val blockId: String) : SheetState
    data class NotDoneSheet(val blockId: String) : SheetState
    data class CompletionSheet(val blockId: String) : SheetState
    data object QuickNoteSheet : SheetState
    data object EndDaySheet : SheetState
    data class ScheduleSheet(val request: ScheduleRequest) : SheetState
    data class TaskDetailsSheet(val blockId: String, val sourceDate: LocalDate) : SheetState
}

data class PendingDoElse(val blockId: String, val selection: DoElseSelection)
```

### 4.3 Производные функции (чистые; НЕ хранить дубликаты)

```kotlin
fun blocksOn(state, date): List<ScheduledBlock>      // = state.blocks.filter { it.date == date }
fun dateOnlyTasksOn(state, date): List<Task>         // plannedDate==date && нет блока
fun backlogGroups(state): BacklogGroups              // noDate / hasDate / postponed
fun workload(state, date): Workload                  // plannedMin / withoutTimeMin / totalMin / overload
fun deriveHeroState(state, nowMs): HeroState
fun topThreeItems(state): List<ScheduledBlock>       // todayTop → блоки (отсутствующие отбрасываются)
fun kpi(state): Kpi                                  // doneToday/totalToday/topDone/topTotal/doneMin/blockMin
fun agentInsight(state, nowMs): AgentInsight?        // overload / lowEnergy / pastDue (разделы 6, 7)
fun elapsedMs(session, nowMs): Long
```

Workload-математика (строго из JSX):

- `plannedMin = Σ computeBlockDuration(b)` по блокам даты;
- `withoutTimeMin = Σ task.durationMin` по date-only задачам даты;
- `totalMin = plannedMin + withoutTimeMin`;
- `overload = totalMin > 480` (8ч);
- `computeBlockDuration(b) = end - start` (обе заданы), иначе `task.durationMin ?: 30`.

`backlogGroups` (с обязательным исключением задач, имеющих блок — инвариант 3.2):

```kotlin
val hasBlock: (Task) -> Boolean = { t -> state.blocks.any { it.taskId == t.taskId } }

noDate    = state.tasks.filter { it.plannedDate == null && !it.postponed && !hasBlock(it) }
hasDate   = state.tasks.filter { it.plannedDate != null && !it.postponed && !hasBlock(it) }
postponed = state.tasks.filter { it.postponed && !hasBlock(it) }
```

### 4.4 Цвет блока (производный)

```kotlin
fun accentFor(priority: Priority, project: String): BlockAccent =
    when {
        project == "Обучение" || project == "Self Development" -> BlockAccent.PURPLE
        priority == Priority.HIGH -> BlockAccent.BLUE
        priority == Priority.LOW  -> BlockAccent.ORANGE
        else                      -> BlockAccent.BLUE
    }
```

### 4.5 Elapsed и тайминг (точные правила)

- **Start:** `ActiveSession(kind=PLANNED, taskId, blockId, startedAtMs=now, runningSinceMs=now, accumulatedMs=0, paused=false, plannedStart/End из блока)`. `startedAtMs` после этого не меняется.
- **Pause:** `accumulatedMs += (now - runningSinceMs!!); runningSinceMs = null; paused = true`.
- **Resume:** `paused = false; runningSinceMs = now` (`startedAtMs` и `accumulatedMs` не трогаются).
- **Complete (PLANNED):** `durationSec = floor(elapsedMs(session, now)/1000)`; создаётся `WorkSession(kind=PLANNED_COMPLETED, startedAtMs=session.startedAtMs, endedAtMs=now, ease, ...)`; `activeSession = null`.
- **Interrupt by DoElse (PLANNED):** при подтверждённом переходе на другую активность создать `WorkSession(kind=PLANNED_INTERRUPTED, startedAtMs=session.startedAtMs, endedAtMs=now, durationSec=elapsed, ease=null)`; прогресс исходного блока не менять; затем запустить альтернативную сессию.
- **Complete (UNPLANNED_WORK):** то же, `kind` из типа активности, `taskId/blockId = null`, `parentBlockId` сохраняется; исходный плановый блок **не** завершается.
- Форматирование: `<60с → "45с"`, иначе `"Nм"` (JSX `fmtElapsed`).

### 4.6 Локальный UI-state (допустимо держать вне VM)

- Поля формы открытого `ScheduleSheet` (date/start/end + ошибка валидации) — `rememberSaveable` внутри листа; в VM уходит только confirm.
- Поля CheckIn/Mode/Completion/QuickNote листов до confirm.
- `expanded` для «Показать все» в Top 3.
- Тикер `now` для hero (обновление 1 раз/с, пока есть активная сессия; при паузе отображение замёрзшее, т.к. `elapsedMs` при `paused` не зависит от now).
- Позиция скролла, локальные анимации.

### 4.7 Как синхронизируются Today и Plan

- Оба экрана рендерятся из одного `PlannerState` → синхронизация мгновенная и безусловная; специального механизма нет.
- Переключение вкладок не трогает `activeSession`, `dayPhase`, `planUi`, листы VM-уровня — сессия и лист переживают переход.
- Навигация для Agent CTA: VM о навигации не знает; `AppShell` передаёт в экраны колбэк `onNavigate(Destinations)` (аналог JSX `onNavigate`).
- Ограничение v1: `todayDate` фиксируется при старте приложения (`Clock.today()`); смена даты в полночь без перезапуска не обрабатывается (задокументировано).

### 4.8 Каталог событий (`PlannerEvent`)

```kotlin
sealed interface PlannerEvent {
    data class CheckInSubmitted(val energy: Int, val focus: Int, val mood: Mood, val obstacle: String?) : PlannerEvent
    data class DayModeSelected(val mode: DayMode) : PlannerEvent
    data class StartBlock(val blockId: String) : PlannerEvent
    data object PauseSession : PlannerEvent
    data object ResumeSession : PlannerEvent
    data object FinishActiveSession : PlannerEvent          // «Завершил»/«Завершить»
    data object BlockEndedStillWorking : PlannerEvent       // «Ещё работаю»
    data object BlockEndedNotDone : PlannerEvent            // «Не делал»
    data class CompletionConfirmed(val ease: Ease) : PlannerEvent
    data class NotDoneAction(val action: NotDoneAction) : PlannerEvent // MOVE | TO_BACKLOG | LEAVE
    data class DoElseRequested(val blockId: String) : PlannerEvent
    data class DoElseConfirmed(val selection: DoElseSelection, val handling: DoElseHandling) : PlannerEvent // LEAVE | SHIFT | MOVE
    data class OpenScheduleSheet(val request: ScheduleRequest) : PlannerEvent
    data class ScheduleConfirmed(val request: ScheduleRequest, val date: LocalDate, val start: LocalTime, val end: LocalTime) : PlannerEvent
    data object ScheduleCancelled : PlannerEvent
    data class OpenTaskDetails(val blockId: String, val sourceDate: LocalDate) : PlannerEvent
    data class TaskDetailsReschedule(val sameDateOnly: Boolean) : PlannerEvent // Изменить время (true) / Перенести (false)
    data class TaskDetailsComplete(val blockId: String) : PlannerEvent
    data object CloseSheet : PlannerEvent
    data class TopThreeStart(val blockId: String) : PlannerEvent
    data class TopThreeComplete(val blockId: String) : PlannerEvent
    data class QuickNoteAdded(val text: String, val kind: NoteKind) : PlannerEvent
    data object OpenEndOfDay : PlannerEvent
    data class EndDayTaskHandled(val blockId: String, val action: EndDayAction, val targetDate: LocalDate?) : PlannerEvent
        // EndDayAction: KEEP_TOMORROW | TO_BACKLOG | TO_DATE | CHOOSE_TIME
    data class EndDayEnergySelected(val energy: Int) : PlannerEvent
    data object EndDayFinished : PlannerEvent
    data class PlanModeSelected(val mode: PlanMode) : PlannerEvent
    data class PlanDateSelected(val date: LocalDate) : PlannerEvent
    data class ViewMonthShifted(val deltaMonths: Int) : PlannerEvent
    data class PostponedReturned(val taskId: String) : PlannerEvent
}

data class DoElseSelection(val kind: DoElseKind, val label: String, val targetBlockId: String?)
    // DoElseKind: OTHER_TASK | UNPLANNED | BREAK | PERSONAL | OTHER
```

## 5. Navigation

Пять вкладок (существующий порядок и строки `strings.xml` — не менять):

1. Сегодня (`today`) — полная реализация.
2. План (`plan`) — полная реализация.
3. Проекты (`projects`) — заглушка `StubScreen` (оставить).
4. Время (`time`) — заглушка `StubScreen` (оставить).
5. Настройки (`settings`) — заглушка `StubScreen` (оставить).

Правила:

- Navigation Compose сохраняется (ADR-0002). Изменения: `AppNavGraph(navController, planner: PlannerViewModel, onNavigate: (Destinations) -> Unit)`; маршруты не меняются.
- `PlannerViewModel` создаётся **один раз** в `AppShell` (Activity scope: `viewModel(factory = ...)` внутри activity-контента) и передаётся в граф → переключение вкладок не пересоздаёт state и не сбрасывает активную сессию (обязательное acceptance-требование).
- Общие листы (`ScheduleSheet`, `TaskDetailsSheet`) рендерятся в `AppShell` поверх `NavHost` (по `state.sheet`); Today-специфичные листы рендерит `TodayScreen` внутри своей ветки. `TaskDetailsSheet` дополнительно получает `onNavigate` для CTA «Открыть в Сегодня» (см. 6.6).
- Back (системный) закрывает открытый лист **без бизнес-мутаций** (`BackHandler` → `CloseSheet`/`ScheduleCancelled`): transient workflow state может закрыться/сброситься по R5, но tasks/blocks/fact/active session не меняются.
- Agent CTA вызывают `onNavigate` напрямую (без маршрута через VM).

## 6. План v1

### 6.1 Структура экрана

Header: «План» + месяц/год `viewMonth` + chevrons ← → (сдвиг месяца) + сегмент-контрол **День | Неделя | Бэклог** (default **Неделя**).

### 6.2 Действия и переходы (INPUT → STATE TRANSITION → RESULT IN UI)

| # | Действие | Вход | Переход состояния | Результат в UI |
|---|---|---|---|---|
| P1 | Выбор дня в ленте | тап по чипу даты (-3…+10 от today) | `PlanDateSelected(date)` → `planUi.selectedDate = date` | подсветка выбранного дня; контент День/Неделя пересчитывается; режим НЕ меняется |
| P2 | Переключение режима | сегмент-контрол | `PlanModeSelected(m)` → `planUi.mode = m` | рендер соответствующего view; режим сохраняется при смене вкладок |
| P3 | Сдвиг месяца | chevron | `ViewMonthShifted(±1)` | месяц/год в header и недельная сетка |
| P4 | Клик по дню в недельной сетке | ячейка месяца | `PlanDateSelected(iso)` + `PlanModeSelected(DAY)` | переход в режим День с выбранной датой |
| P5 | Клик по блоку в timeline | блок | `OpenTaskDetails(blockId, selectedDate)` | `TaskDetailsSheet` |
| P6 | «В план» (date-only задача дня) | кнопка-календарь | `OpenScheduleSheet(SCHEDULE, taskId, sourceDate=selectedDate, initialDate=plannedDate ?: selectedDate, lockDate=false)` | `ScheduleSheet` |
| P7 | «В план» (бэклог: Без даты / Есть дата) | кнопка | то же, `initialDate = task.plannedDate ?: selectedDate` | `ScheduleSheet` |
| P8 | «Вернуть» (Отложенные) | кнопка | `PostponedReturned(taskId)` → `postponed=false`, `plannedDate=today` | задача переходит в «Есть дата, нет времени» |
| P9 | Agent: «Разгрузить» | CTA | `PlanModeSelected(DAY)` | открыт режим День (видимое действие, не dead control) |
| P10 | Agent: «Открыть» (не закрыт блок) | CTA | `PlanModeSelected(DAY)` | режим День; виден незакрытый блок |

### 6.3 День (Day view)

- `DayPickerStrip`: горизонтальная лента дней -3…+10 от today; выбранный — accent-подложка, today — точка-индикатор; не уводит из режима.
- Заголовок: день недели (полное русское название) + «14 августа».
- `WorkloadSummary` (производная, см. 4.3) — три KPI-mini:
  - «Запланировано» — `fmtMinsExact(plannedMin)` (blue);
  - «Без времени» — `fmtMinsExact(withoutTimeMin)` (orange);
  - «Нагрузка» — `fmtMinsExact(totalMin)` (green); **«Перегруз»** (red), если `totalMin > 480`.
- Timeline (см. 12.7): блоки отсортированы по start; top = `(startMin − 540) × (40/60)` dp от базы 09:00; height = `max(28, dur × 40/60)` dp; завершённые приглушены + зачёркнуты; прогресс-бар при `0 < progress < 100`; клик → TaskDetailsSheet; пусто → «Пустой день — добавьте задачу».
- Секция «Без времени · N» (`Нужно распределить`): только date-only задачи выбранной даты; каждая — title, `project · ~duration`, кнопка-календарь → ScheduleSheet (P6).

### 6.4 Неделя (Week view, default)

- `DayPickerStrip` присутствует и здесь.
- Сетка месяца `viewMonth`, неделя с понедельника; ячейки: дата, до 3 цветных точек блоков; сегодня — accent-обводка; выбранная — accent-подложка; прошедшие — opacity 0.55. Клик → выбор даты + переход в режим День (P4).
- Подпись «Неделя · кликните день для детального плана».

### 6.5 Бэклог

Три группы (заголовок + счётчик), каждая строка: title, `project · ~duration`, для «Есть дата» — дата чипом:

1. **Без даты** — `plannedDate == null && !postponed && нет блока`; кнопка «В план» → ScheduleSheet.
2. **Есть дата, нет времени** — `plannedDate != null && !postponed && нет блока`; кнопка «В план».
3. **Отложенные** — `postponed && нет блока`; зачёркнутые, opacity 0.7, чип «Отложено» (orange), кнопка «Вернуть» (P8).

Все три группы обязаны исключать задачи, имеющие ScheduledBlock (инвариант 3.2) — scheduled-задача не появляется в бэклоге ни при каких условиях.

### 6.6 Timeline и завершение в Plan

- «Завершить» из TaskDetailsSheet: `TaskDetailsComplete` → `progressPercent = 100` + закрыть лист. **Не создаёт WorkSession** (факт без запущенной сессии не фиксируется — JSX-поведение).
- **Блок с активной сессией (решение владельца, НЕ JSX-поведение):** если у блока есть активная PLANNED-сессия (в т.ч. на паузе), кнопка «Завершить» в TaskDetailsSheet заменяется CTA **«Открыть в Сегодня»**: закрыть лист + `onNavigate(TODAY)`. Plan не управляет фактической сессией — завершение происходит в Today (BlockEnded → Завершил). Reducer дополнительно отклоняет `TaskDetailsComplete` (no-op), если активная PLANNED-сессия ссылается на этот `blockId` (защита инварианта из 3.3).

### 6.7 Plan Agent

Плавающая карточка (purple) внизу; правила вывода (порядок проверки):

1. `plannedMin(today) > 480` → «Перегруз дня» / «Больше 8ч запланировано. Могу разнести задачи.» / CTA «Разгрузить» → режим День.
2. Иначе первый незавершённый блок today с `end < now` (сортировка по start) → «Не закрыт блок» / «{title} — оцените, успеваете?» / CTA «Открыть» → режим День.
3. Иначе карточка скрыта (нет filler-контента).

CTA всегда выполняет видимое действие (P9/P10). Без LLM, без сетевых вызовов.

## 7. Сегодня v1

### 7.1 Структура экрана (визуальная иерархия)

1. Header: «Сегодня» + «Пт · 14 августа» + кнопка «Чек-ин» (если чек-ина нет) или «Завершить день» (если чек-ин есть).
2. KPI row: Режим дня / Прогресс / Энергия.
3. Hero-карточка (state machine).
4. Agent-карточка (только при actionable insight).
5. Главное на сегодня (Top 3).
6. Быстрая заметка.

### 7.2 Машина состояний (полная таблица)

| Состояние | Условия входа | Hero-карточка | Доступные действия | Переходы |
|---|---|---|---|---|
| `DAY_NOT_STARTED` | старт приложения, нет чек-ина | градиент accentDeep; «Скоро старт» / «Доброе утро» / «Перед началом — короткий чек-ин: энергия, фокус, препятствие.» | Чек-ин (hero CTA и header) | Чек-ин submit → `RUNNING` (hero пересчитывается: READY/UPCOMING/BETWEEN по часам) |
| `UPCOMING` | день запущен, ближайший незавершённый блок в будущем | карточка как у BETWEEN_BLOCKS (JSX-поведение): «Свободное окно» / «Перерыв» / «Можно отдохнуть или взять короткую задачу.» | Старт из Top 3 | `StartBlock` → ACTIVE; время дошло до окна блока → READY (автоматически, производный) |
| `READY` | `now` внутри окна [start; end] незавершённого блока, нет сессии | градиент accentDeep; «Сейчас» / title / «HH:MM–HH:MM · project» / «Пора начинать» | «Начать»; «Делаю другое» | Начать → ACTIVE; Делаю другое → DoElseSheet (отмена без изменений); окно прошло → BETWEEN (авто) |
| `ACTIVE` | сессия запущена, не на паузе | градиент accentDeep, border accent; «Сейчас» / elapsed (tab-nums) / title или label / «HH:MM–HH:MM · project» либо «Вне плана · фактическое время» | Пауза; Завершил; «Делаю другое» (только PLANNED) | Пауза → PAUSED; Завершил (PLANNED) → BLOCK_ENDED_UNRESOLVED; Завершил (UNPLANNED_WORK) → запись факт-сессии, `activeSession=null` → BETWEEN; Делаю другое → DoElseSheet |
| `PAUSED` | `paused == true` | градиент surface2; «На паузе» (orange) / title / «Возобновите или завершите — время не идёт.» | Продолжить; Завершить | Продолжить → ACTIVE; Завершить → как Завершил в ACTIVE |
| `BLOCK_ENDED_UNRESOLVED` | открыт BlockEndedSheet («Завершил» у PLANNED-сессии) | hero остаётся ACTIVE, поверх — лист «Блок закончился» (title, time–end · project) | Завершил; Ещё работаю; Не делал | Завершил → CompletionSheet → confirm → `progressPercent=100`, запись WorkSession(PLANNED_COMPLETED, ease, durationSec=elapsed), `activeSession=null` → BETWEEN/UPCOMING/READY (производно); Ещё работаю → лист закрыт, сессия идёт за план-окончание (ACTIVE, overrun); Не делал → `activeSession=null` **без** факт-записи → NotDoneSheet |
| `BETWEEN_BLOCKS` | день RUNNING, нет сессии, нет READY/UPCOMING блока | градиент surface2; «Свободное окно» / «Перерыв» / «Можно отдохнуть или взять короткую задачу.» | Старт из Top 3 (если нет активной сессии) | `StartBlock` → ACTIVE; «Завершить день» → END_OF_DAY |
| `END_OF_DAY` | `dayPhase=END_OF_DAY` (открыт EndDaySheet) | градиент accentPurpleDeep; «День почти закончен» / «Подведём итоги» / «Незакрытое перенесём в завтра или в бэклог.» | Действия End of Day (раздел 11) | Отмена листа → RUNNING (hero по часам); «Завершить день» → DAY_COMPLETED |
| `DAY_COMPLETED` | подтверждён End of Day | surface; «День завершён» (green) / «Хороший день» / «Завтра — новый план.» | — | терминальное (до перезапуска приложения) |

Ключевые правила (обязательные):

- **Плановое окончание блока никогда не завершает задачу автоматически** — наступление времени окончания не переключает ACTIVE; оверран только через «Ещё работаю».
- Активная сессия переживает Today → Plan → Today.
- Время на паузе не накапливается; elapsed — производная (4.5).
- «Не делал» не записывает факт-сессию.

### 7.3 Check-in

Лист «Чек-ин»: Энергия (0–3: ·, ⚡, ⚡⚡, ⚡⚡⚡), Фокус (0–3: туман, средний, острый, пик), Самочувствие (Плохо/Нормально/Хорошо), «Что может помешать?» (необязательное поле).

Submit: `CheckInSubmitted` → сохранить CheckIn, `dayEnergy = energy`; если `dayPhase == DAY_NOT_STARTED` → `RUNNING` (кнопка — «Начать день», иначе «Сохранить»). Повторное открытие (KPI Энергия) предзаполняет значения последнего чек-ина.

### 7.4 Режим дня / KPI

KPI row (3 кнопки-карточки):

- **Режим**: label «Режим», значение «Фокус/Обычный/Лёгкий/Восстановление» → `ModeSheet` (4 опции с цветами: Фокус-purple, Обычный-blue, Лёгкий-green, Восстановление-orange; выбор → `DayModeSelected` + закрыть).
- **Прогресс**: «doneToday/totalToday · topDone/topTotal» → `ProgressSheet`: Задачи done/total; Главное topDone/topTotal; Время `fmtMinsExact(doneMin) / fmtMinsExact(blockMin)` (по сегодняшним блокам, 4.3).
- **Энергия**: «·» или «⚡⚡⚡» по `dayEnergy` → открывает/переиспользует `CheckInSheet` (не режим).

### 7.5 Top 3 («Главное на сегодня»)

- `todayTop` — максимум 3 `taskId`; элементы резолвятся в блоки сегодняшнего дня; отсутствующие отбрасываются.
- Строка: чекбокс (завершение), title (зачёркнут при 100%), «HH:MM–HH:MM», кнопка «Начать» (только если нет активной сессии и блок не завершён) или метка «идёт» (если это блок активной сессии).
- Чекбокс: если у блока активная сессия → `CompletionSheet` (ease); иначе мгновенно `TopThreeComplete` → `progressPercent=100` (без факт-сессии — JSX-поведение).
- «Показать все»/«Скрыть» — Today-only раскрытие списка всех незавершённых блоков today. **Не навигирует в Plan.** `expanded` — локальный UI-state.

### 7.6 Quick Note

Список (до 3 записей) или пустая карточка «Здесь появятся заметки дня»; «+ Добавить» → `QuickNoteSheet`: чипы типа (Заметка/Идея/Проблема/Итог), поле текста, контекст («Контекст: {текущий блок или label сессии}»), «Сохранить» disabled при пустом тексте. Сохранение: `QuickNoteAdded` → в начало списка, cap 5.

### 7.7 Agent (Today)

Правила вывода (порядок проверки; карточка скрыта, если нет insight):

1. `plannedMin(today) > 480` → «Перегруз дня» / «Больше 8ч запланировано. Могу разнести задачи.» / CTA «Разгрузить» → `onNavigate(plan)`.
2. `dayCheckIn?.energy == 0` И существует незавершённый блок (первый по времени; **нормализация:** в JSX захардкожен `t5`, здесь — первый незавершённый) → «Энергия низкая» / «Сложный блок впереди — перенести или облегчить?» / CTA «Посмотреть» → `onNavigate(plan)`.
3. Иначе `null` — карточка не рендерится.

### 7.8 «Делаю другое» (двухшаговый поток)

Лист с тремя стадиями: **pick → (task) → handling**.

1. **pick** — «Что делаете вместо?»: Другая задача / Незапланированная / Перерыв / Личное / Другое. «Другая задача» → стадия task (список других незавершённых блоков today с временем; пусто → «Других незавершённых задач сегодня нет.», кнопка «← Назад»). Остальные → стадия handling.
2. **handling** — карточка «Фактически сейчас: {label}» + опциональное поле «Уточнить название» (для unplanned/personal/other) + «Что сделать с исходным блоком?»:
   - **Оставить в плане** → `DoElseConfirmed(selection, LEAVE)`;
   - **Сдвинуть** → `DoElseConfirmed(selection, SHIFT)`;
   - **Перенести** → `DoElseConfirmed(selection, MOVE)`.

Переходы в reducer'е (`DoElseConfirmed`):

- `LEAVE`: если сейчас активна PLANNED-сессия исходного `blockId`, сначала зафиксировать её отработанный кусок как `PLANNED_INTERRUPTED` (progress не менять); затем `startAlternativeSession(originalBlockId, selection)`; `dayPhase = RUNNING`; лист закрыт. Если DoElse открыт из READY без active session, interrupt-helper — no-op.
- `SHIFT`: исходную active session пока **не трогать**; закрыть DoElseSheet; `pendingDoElse = {blockId, selection}`; открыть `ScheduleSheet(reschedule, sourceDate=today, initialDate=today, lockDate=true, purpose=DO_ELSE)`.
- `MOVE`: то же, но `initialDate=tomorrow, lockDate=false, purpose=DO_ELSE`.
- Подтверждение ScheduleSheet (purpose=DO_ELSE): атомарно применить перенос; затем, если исходная PLANNED active session всё ещё относится к `pendingDoElse.blockId`, закрыть её как `PLANNED_INTERRUPTED`; затем `startAlternativeSession`; `pendingDoElse=null`.
- Отмена ScheduleSheet: `pendingDoElse=null`, расписание не тронуто, новая сессия не запускается, **исходная active session продолжает жить без потери факта**.

`startAlternativeSession(originalBlockId, selection)`:

- `OTHER_TASK` → `ActiveSession(kind=PLANNED, taskId/blockId выбранного блока, label=title, parentBlockId=originalBlockId, startedAtMs=now, runningSinceMs=now, accumulatedMs=0, paused=false)` — сессия привязана к **реальной** задаче со стабильными id.
- прочие → `ActiveSession(kind=UNPLANNED_WORK, taskId=null, blockId=null, label=selection.label, parentBlockId=originalBlockId, startedAtMs=now, runningSinceMs=now, accumulatedMs=0, paused=false)`.

Завершение unplanned/break/personal сессии (`FinishActiveSession`): запись `WorkSession(kind=UNPLANNED/BREAK/PERSONAL/OTHER, parentBlockId, label, durationSec=elapsed)`, `activeSession=null`, исходный плановый блок **не** завершается. В hero unplanned-сессия видна как «{label} / Вне плана · фактическое время».

### 7.9 «Блок закончился» / «Не делал»

`BlockEndedSheet` (вход — «Завершил» у PLANNED-сессии):

- «Завершил» → `CompletionSheet` (ease: Легко/Нормально/Тяжело) → confirm → переходы из таблицы 7.2.
- «Ещё работаю» → закрыть лист, сессия продолжает идти (оверран плана).
- «Не делал» → `NotDoneSheet` (без факт-записи):
  - «Перенести» → `NotDoneAction(MOVE)` → закрыть лист + `ScheduleSheet(reschedule, sourceDate=today, lockDate=false, purpose=PLAIN)`.
  - «В бэклог» → `NotDoneAction(TO_BACKLOG)`: удалить блок из today, задача `plannedDate=null`, `postponed=false` (upsert по taskId, 11.2), сессия уже закрыта.
  - «Оставить» → `NotDoneAction(LEAVE)` → только закрыть лист.

### 7.10 End of Day

Полный контракт — раздел 11.

## 8. Plan vs Fact (правила)

1. `ScheduledBlock` описывает **намерение**; `WorkSession` — **факт**. Они живут в разных коллекциях и не выводятся друг из друга.
2. `Start/Pause/Resume/Complete` меняют факт; `schedule/reschedule` меняют намерение.
3. **Незапланированная работа не завершает planned task автоматически.** Сессия `UNPLANNED_WORK` имеет `taskId=null`; `parentBlockId` — только ссылка для контекста.
4. **Плановое окончание блока ≠ выполнение.** После `end` блока сессия может продолжаться («Ещё работаю»); завершение — только явное действие пользователя.
5. Завершение блока без запущенной сессии (Top 3 чекбокс, TaskDetailsSheet «Завершить») ставит `progressPercent=100` **без** WorkSession — факт без сессии не фиксируется.
6. `WorkSession` для PLANNED завершения фиксирует `durationSec=elapsed` и `ease`.
7. «Не делал» — ни прогресса, ни факт-сессии; только судьба блока (перенос/бэклог/оставить).

## 9. Cross-screen сценарии (обязательные acceptance criteria)

| # | Сценарий | Шаги | Ожидание |
|---|---|---|---|
| C1 | schedule in Plan → visible in Today | Plan: date-only задача today → «В план» → свободный слот → «Запланировать» | блок появился в Plan timeline и в Today (KPI, hero-вывод, Top 3-логика); задача исчезла из «Без времени» и бэклога (single-entity) |
| C2 | complete in Today → completed in Plan | Today: Top 3 чекбокс / BlockEnded → Завершил → ease | `progressPercent=100`; в Plan timeline блок приглушён+зачёркнут; KPI Прогресс вырос |
| C3 | reschedule in Today → updated in Plan | Today: «Не делал» → Перенести → ScheduleSheet (завтра, время) → подтвердить | блок удалён из today, появился на целевой дате в Plan; `taskId` тот же |
| C4 | move to backlog in Today → visible in Plan Backlog | Today: «Не делал» → В бэклог (или EOD → В бэклог) | блок исчез из today; в Plan → Бэклог → «Без даты» появилась задача с тем же `taskId` |
| C5 | active session survives Today → Plan → Today | запустить блок в Today → вкладка План → вкладка Сегодня | сессия жива, elapsed продолжает накапливаться, hero ACTIVE |
| C6 | taskId survives all scheduling operations | пройти C3, затем перенос обратно, затем EOD «Выбрать время» | `taskId` неизменен на каждом шаге; нет дублей задачи ни в одном представлении |
| C7 | ScheduleSheet из Today и из Plan — одно поведение | открыть из «Не делал» (Today) и из TaskDetailsSheet (Plan) | идентичные режимы, валидация, cancel-lossless, сохранение blockId/taskId |
| C8 | EOD «Выбрать время» не теряет блок при отмене | Today → Завершить день → «Выбрать дату и время» → ScheduleSheet → Cancel | блок остался в today; сессия (если была) продолжает идти |
| C9 | Agent CTA навигация | Today: Перегруз → «Разгрузить» | вкладка План открыта; в Plan можно действовать; при возврате в Today состояние дня не сброшено |
| C10 | Завершить в Plan при активной сессии | запустить блок в Today → Plan → TaskDetailsSheet этого блока | «Завершить» заменён CTA «Открыть в Сегодня»; переход в Today; сессия продолжает идти; `progressPercent` не изменён |

## 10. ScheduleSheet contract

### 10.1 Request

```kotlin
enum class ScheduleMode { SCHEDULE, RESCHEDULE }
enum class SchedulePurpose { PLAIN, DO_ELSE, END_DAY }

data class ScheduleRequest(
    val mode: ScheduleMode,
    val taskId: String?,            // SCHEDULE
    val blockId: String?,           // RESCHEDULE
    val sourceDate: LocalDate,      // RESCHEDULE: где блок сейчас; SCHEDULE: selectedDate (контекст)
    val initialDate: LocalDate,     // только предлагаемая целевая дата при открытии
    val lockDate: Boolean,          // запрет смены даты
    val purpose: SchedulePurpose,
)
```

Семантика: **sourceDate ≠ initialDate.** `sourceDate` — где блок живёт сейчас; `initialDate` — предложение, пользователь может менять (если `!lockDate`).

### 10.2 Режимы

- **SCHEDULE** — из date-only/backlog задачи: создать новое размещение; consume unscheduled-представление.
- **RESCHEDULE** — существующий блок: same-date confirm = **Изменить время** (обновить на месте); cross-date confirm = **Перенести** (удалить старое, создать на новой дате).

### 10.3 Правила (обязательные)

1. **Cancel lossless:** закрытие (назад/подложка/системный back) не меняет бизнес-данные (`tasks`, `blocks`, `factSessions`, `activeSession`). Допустимо закрыть `sheet` и сбросить transient `pendingDoElse`; новая сессия не запускается, исходная active session продолжает жить.
2. **Не удалять исходный блок до успешного подтверждения:** при cross-date confirm reducer применяет удаление и добавление атомарно в одном `reduce`-шаге.
3. Same-date confirm обновляет существующий блок (`blockId` сохраняется).
4. Cross-date confirm: удалить из `sourceDate`, добавить на целевую дату с тем же `blockId` (см. 3.3), `taskId` неизменен.
5. SCHEDULE confirm: новый блок (`blockId = "s_${taskId}_${yyyyMMdd(date)}"`), `progressPercent=0`, `accent=accentFor(...)`; у задачи `plannedDate=null` (single-entity) — **независимо от** целевой даты.
6. Manual time validation (в листе и повторно в reducer):
   - формат HH:MM;
   - `start ≥ 09:00`, `end ≤ 19:00`, `end > start`;
   - пересечение с любым блоком целевой даты — ошибка; **собственный блок исключается** из расчёта пересечений в режиме RESCHEDULE;
   - сообщения: «Время должно быть в пределах 09:00–19:00.» / «Этот интервал пересекается с другим блоком.»
7. Overlap: `conflict = other.start < candidateEnd && candidateStart < other.end`.
8. Confirm disabled, пока невалидно. Label: SCHEDULE → «Запланировать»; RESCHEDULE same-date → «Сохранить»; cross-date → «Перенести».

### 10.4 UI листа

- Заголовок: «Запланировать» (SCHEDULE) / «Изменить расписание» (RESCHEDULE).
- Карточка задачи: title, `project · ~duration`.
- «Дата»: чипы `[sourceDate, today, tomorrow, today+2]` (дедуп, без null); при `lockDate` — только текущая дата, чипы неактивны. Смена даты сбрасывает выбранное время.
- «Свободные слоты»: шаг 30 мин от 09:00 до (19:00 − duration); исключить конфликты; показать ≤8 чипов «HH:MM–HH:MM»; пусто → «Нет свободных слотов».
- «Время вручную»: кнопки −1ч, −15, поле start, «–», поле end, +15, +1ч. Установка start → `end = start + duration`. End редактируется только если `> start`. Первое нажатие ± при пустом времени → `09:00` + duration.
- Ошибка валидации под полями (red, 11sp).
- Confirm (full-width primary).

### 10.5 Точки входа

| Источник | request |
|---|---|
| Plan: «В план» | SCHEDULE, taskId, sourceDate=selectedDate, initialDate=task.plannedDate ?: selectedDate, lockDate=false |
| Plan: TaskDetails «Изменить время» | RESCHEDULE, sourceDate=block.date, initialDate=block.date, lockDate=true |
| Plan: TaskDetails «Перенести» | RESCHEDULE, sourceDate=block.date, initialDate=(block.date==today ? tomorrow : block.date), lockDate=false |
| Today: «Не делал» → Перенести | RESCHEDULE, sourceDate=today, initialDate=today, lockDate=false |
| Today: «Делаю другое» → Сдвинуть | RESCHEDULE, sourceDate=today, initialDate=today, lockDate=true, purpose=DO_ELSE |
| Today: «Делаю другое» → Перенести | RESCHEDULE, sourceDate=today, initialDate=tomorrow, lockDate=false, purpose=DO_ELSE |
| Today: EOD «Выбрать дату и время» | RESCHEDULE, sourceDate=today, initialDate=tomorrow, lockDate=false, purpose=END_DAY |

Reducer `ScheduleConfirmed` (единая логика):

```text
if request.purpose == DO_ELSE && pendingDoElse != null:
    применить перенос (п. 3–5);
    если activeSession — PLANNED и activeSession.blockId == pendingDoElse.blockId:
        записать WorkSession(PLANNED_INTERRUPTED, startedAt=session.startedAtMs, endedAt=now, duration=elapsed); progress не менять
    затем startAlternativeSession(pendingDoElse.blockId, pendingDoElse.selection);
    pendingDoElse = null; sheet = null
else:
    применить перенос; sheet = null
```

## 11. End of Day

### 11.1 Вход и общая структура

- Вход: header «Завершить день» (доступен только после чек-ина) → `OpenEndOfDay` → `dayPhase=END_OF_DAY`, `sheet=EndDaySheet`.
- Лист «Итоги дня»: KPI-mini — Сделано `done/total` (green), Заметок `N` (blue), Факт-сессий `factSessions.size + (activeSession? 1:0)` (purple).
- «Незакрытое»: все блоки today с `progressPercent < 100`; каждая строка раскрывается («Действие»/«Скрыть») с действиями: **Завтра / В бэклог / На дату / Выбрать дату и время**.
- «Энергия в конце»: шкала 0–3 → `EndDayEnergySelected` (обновляет `dayEnergy`).
- Кнопка «Завершить день» → `EndDayFinished`.

### 11.2 Правила обработки незакрытых задач

Общие helpers (внутри reducer, атомарно):

- `removeBlockFromToday(state, blockId)`;
- `upsertTask(state, taskId, plannedDate)`: удалить любую существующую запись с этим `taskId` из каталога и добавить одну с `plannedDate` (title/project/duration/priority из исходных данных задачи, `postponed=false`). **Нет клонирования.**

| Действие | Transition | Результат |
|---|---|---|
| Завтра | `EndDayTaskHandled(blockId, KEEP_TOMORROW)` | блок удалён из today; задача `plannedDate = tomorrow` — **date-only, без времени**, никакого фиктивного времени |
| В бэклог | `TO_BACKLOG` | блок удалён; `plannedDate = null`, `postponed=false` → группа «Без даты» |
| На дату | `TO_DATE(targetDate)` | блок удалён; `plannedDate = targetDate` (date-only). UI: выбор из чипов следующих 7 дней (Завтра, Пт · 15, …), по умолчанию tomorrow. **Нормализация:** JSX жёстко берёт +2 дня — здесь пользователь выбирает дату |
| Выбрать дату и время | `CHOOSE_TIME` | блок **остаётся на месте**; открывается ScheduleSheet (RESCHEDULE, sourceDate=today, initialDate=tomorrow, lockDate=false, purpose=END_DAY). Блок удаляется/перемещается только после confirm |

- **No data loss on Cancel:** закрытие листа EndDaySheet (`CloseSheet`) → `dayPhase=RUNNING` (возврат), все блоки нетронуты; отмена ScheduleSheet — блок остался в today.
- **Согласованность с активной сессией (нормализация против JSX):** если действие убирает блок из today, а этот блок владеет активной PLANNED-сессией — сессия завершается немедленно и записывается как `WorkSession(kind=STOPPED_AT_DAY_END, taskId/blockId, durationSec=elapsed)`, `activeSession=null`. (JSX оставляет «осиротевшую» сессию, деградирующую до «Незапланированная работа» — артефакт, в нативном поведении не воспроизводится.) Для CHOOSE_TIME сессия продолжается до подтверждения переноса.

### 11.3 Завершение дня

`EndDayFinished`:

1. Если есть `activeSession` (любого kind): записать `WorkSession(kind=STOPPED_AT_DAY_END, label, parentBlockId, durationSec=elapsed)`, `activeSession=null`.
2. `endDayReview = EndDayReview(now, dayEnergy)`.
3. `dayPhase = DAY_COMPLETED`; лист закрыт; hero → «День завершён».

## 12. Visual implementation

**Принцип:** переносить токены и визуальную иерархию, не пиксельные стили JSX. Токены DESIGN.md уже продублированы в `ui/theme/Color.kt`, `Type.kt`, `Shape.kt`. **Единственное обязательное дополнение темы:**

```kotlin
// Color.kt — добавить (токены DESIGN.md, отсутствуют сейчас):
val AccentRed = Color(0xFFFF4D4D)      // danger / severe overload
val AccentRedDeep = Color(0xFF3D1414)  // red surface tint
```

### 12.1 Цвета (семантика)

| Семантика | Токены |
|---|---|
| Фон приложения | `Bg` `#0A121B`; внешний/глубокий `BgDeep` |
| Поверхности | `Surface` / `Surface2` / `Surface3` |
| Текст | `Fg` (заголовки), `Muted` (вторичный), `Muted2` (третичный/лейблы) — явные цвета, никогда не дефолтный чёрный |
| Бордеры | `Border` `#253240` |
| Blue (primary/выбранное) | `Accent`, `AccentSoft`, `AccentDeep` |
| Green (успех/энергия/завершено) | `AccentGreen`, `AccentGreenDeep` |
| Orange (внимание/перенос) | `AccentOrange`, `AccentOrangeDeep` |
| Purple (AI/глубокая работа) | `AccentPurple`, `AccentPurpleDeep` |
| Red (danger/перегруз) | `AccentRed`, `AccentRedDeep` — только перегруз/ошибки, не для обычного опоздания |

### 12.2 Типографика

`Type.kt` уже мапит роли DESIGN.md: h1 26/700 → `displayLarge`; h2 17/600 → `titleMedium`; h3 14/600 → `titleSmall`; body 14/400 → `bodyLarge`; meta 11/500 → `labelSmall`; button 14/600; mono — `labelMedium` (tab-nums для таймеров/времени). Заголовки секций: 12sp/600/`Muted2`/uppercase/tracking — переиспользовать паттерн `Label`.

### 12.3 Отступы и радиусы

База 4dp: xs 4, sm 8, md 12, page 16, sectionGap 16, cardPad 14. Радиусы: card 16, cardSm 12, btn 10, btnSm 8, pill 999. Touch targets ≥44dp.

### 12.4 Компоненты

- **Кнопки:** primary ~46dp/accent/белый текст/radius 10; secondary `Surface2`+`Border`; danger `AccentRed`; small ~34dp/radius 8. Disabled: `Surface2` + `Muted2`.
- **Карточки:** стандартная radius 16, pad 14, `Surface`+тонкий border; compact radius 12, pad 10–12; Agent — purple-семейство с обязательным CTA; hero — deep-градиенты (12.5).
- **Сегмент-контрол:** `Surface2`-основа, radius 10, pad 3, активный сегмент `Surface3`, текст `Fg`/600.
- **Чипы:** ~28dp, pill, для времени/статусов/слотов.
- **Checkbox** 22×22/radius ~7; **радио** 20×20; прогресс linear 6–8dp pill, circular ~46dp/4dp.
- **Bottom Sheet:** общий scaffold `AdiyutantSheet` (см. 13): подложка `rgba(0,0,0,0.55)`, скругление верха 18dp, handle 36×4 `Border`, maxHeight ~85%, скролл, `BackHandler` → lossless close, insets `navigationBarsPadding`.

### 12.5 Hero-карточки (градиенты из JSX)

| Вариант | Фон | Border |
|---|---|---|
| DAY_NOT_STARTED | `linear 135°: AccentDeep → Bg` | `Accent` 55% |
| READY | `linear 135°: AccentDeep → Bg` | `Accent` 66% |
| ACTIVE | `linear 135°: AccentDeep → Bg` | `Accent` + внутреннее свечение |
| PAUSED | `linear 135°: Surface2 → Bg` | `Border`; лейбл orange |
| BETWEEN/UPCOMING | `linear 135°: Surface2 → Bg` | `Border` |
| END_OF_DAY | `linear 135°: AccentPurpleDeep → Bg` | `AccentPurple` 55% |
| DAY_COMPLETED | `Surface` | `Border`; лейбл green |

Лейбл-надпись: 11sp/600/uppercase/tracking, цвет по состоянию (blue/orange/purple/green/muted). Title 17/700 `Fg`. Методанные 12sp `Muted`. Elapsed — tab-nums.

### 12.6 Tab bar / header / Agent

- Tab bar: существующий `BottomNavBar` (NavigationBar) — визуально не менять; active `Accent`, inactive `Muted2`.
- Today header: градиент `radial(120% 80% at 0% 0%, Surface → Bg)`, нижний border; «Сегодня» 11sp uppercase `Muted2`; дата 22/700 `Fg` («Пт · 14 августа»); кнопки: «Чек-ин» (accent 13% + border accent 40%), «Завершить день» (`Surface2`+border).
- Plan header: «План» 11sp + месяц/год 22/700; chevrons 34×34 radius 10.
- Agent-карточка: `AccentPurpleDeep`, radius 14, pad 14, иконка-звезда в круге `AccentPurple` 33%, title 12/600 `Fg`, body 11 `Muted`, CTA-чип с border `AccentPurple`. Скрыта при `null`.
- Status bar: системный (не рисовать fake «9:41» из JSX).

### 12.7 Timeline

- Левая рейка 50dp; линия-ось на 18dp, 1dp `Border`; время слева 11sp `Muted2` tab-nums.
- Блок: `Surface2` (незавершён) / `Surface`+opacity 0.55+зачёркнуто (завершён); цветная полоса 3dp слева (`BlockAccent`); title 13/600 `Fg`; sub 10sp `Muted2` (`project · HH:MM–HH:MM`); прогресс-бар 2dp при `0<progress<100`.
- Позиция: top = `(startMin − 540) × (40/60)` dp; height = `max(28, dur × 40/60)` dp; шаг 40dp/час.
- Текущая линия времени — опционально, **вне scope v1**.

### 12.8 Reference frame

Логический фрейм 390×875 — пропорциональная база. Android-раскладка адаптивная (dp), пиксели JSX трактуются как dp при макете, без жёсткой фиксации ширины. Компоненты тянутся на ширину контента с паддингом page 16.

## 13. Kotlin / Compose implementation guidance

### 13.1 Структура файлов (обязательная)

```text
AdiyutantAndroid/app/src/main/kotlin/com/adiyutant/app/
├── core/                                   # НЕ ТРОГАТЬ (шов ADR-0002/0003)
├── domain/
│   ├── model/
│   │   ├── Task.kt
│   │   ├── ScheduledBlock.kt
│   │   ├── WorkSession.kt                  # + ActiveSession, SessionKind, ActiveKind
│   │   ├── CheckIn.kt
│   │   ├── QuickNote.kt
│   │   ├── PlannerState.kt                 # + PlanUiState, SheetState, ScheduleRequest, PendingDoElse, EndDayReview
│   │   ├── PlannerEvent.kt                 # + DoElseSelection, DoElseKind, EndDayAction, NotDoneAction
│   │   └── Enums.kt                        # DayPhase, HeroState, DayMode, Ease, Mood, NoteKind, Priority, BlockAccent, PlanMode, ScheduleMode, SchedulePurpose
│   ├── PlannerCore.kt                      # чистый reducer reduce(state, event) + производные функции (4.3)
│   ├── ScheduleValidator.kt                # workday/overlap/время (чистые функции)
│   ├── TimeMath.kt                         # minToTime/timeToMin/addMin/computeBlockDuration/форматы (fmtMins, fmtMinsExact, fmtElapsed, RU-локали)
│   └── Clock.kt                            # fun interface Clock { fun nowMs(): Long; fun today(): LocalDate }
├── data/
│   ├── PlannerRepository.kt                # интерфейс — будущий шов под Rust Core
│   ├── InMemoryPlannerRepository.kt        # хранит PlannerState в памяти
│   └── SampleSeedData.kt                   # seed = INIT_* из App.jsx, даты относительны today
├── di/
│   └── ServiceLocator.kt                   # + plannerCore: PlannerCore, начальный PlannerState
├── ui/
│   ├── planner/
│   │   └── PlannerViewModel.kt             # StateFlow<PlannerState>, dispatch(PlannerEvent), factory
│   ├── common/
│   │   ├── AdiyutantSheet.kt               # scaffold bottom sheet (12.4)
│   │   ├── Label.kt                        # заголовок секции
│   │   ├── SegmentedControl.kt
│   │   ├── EmojiScale.kt                   # шкала 0..3 / 0..N
│   │   ├── MoodPicker.kt
│   │   ├── KpiMini.kt
│   │   ├── Chip.kt
│   │   ├── ScheduleSheet.kt                # общий, contract раздел 10
│   │   └── TaskDetailsSheet.kt
│   ├── feature/
│   │   ├── today/
│   │   │   ├── TodayScreen.kt              # + рендер Today-листов
│   │   │   ├── TodayHeader.kt
│   │   │   ├── KpiRow.kt
│   │   │   ├── HeroCard.kt
│   │   │   ├── AgentCard.kt
│   │   │   ├── TopThreeSection.kt
│   │   │   ├── QuickNoteSection.kt
│   │   │   └── sheets/
│   │   │       ├── CheckInSheet.kt
│   │   │       ├── ModeSheet.kt
│   │   │       ├── ProgressSheet.kt
│   │   │       ├── DoSomethingElseSheet.kt
│   │   │       ├── BlockEndedSheet.kt
│   │   │       ├── NotDoneSheet.kt
│   │   │       ├── CompletionSheet.kt
│   │   │       ├── QuickNoteSheet.kt
│   │   │       └── EndDayReviewSheet.kt
│   │   ├── plan/
│   │   │   ├── PlanScreen.kt
│   │   │   ├── PlanHeader.kt
│   │   │   ├── DayView.kt
│   │   │   ├── WeekView.kt
│   │   │   ├── BacklogView.kt
│   │   │   ├── DayPickerStrip.kt
│   │   │   ├── WorkloadSummary.kt
│   │   │   ├── DayTimeline.kt
│   │   │   └── PlanAgentCard.kt
│   │   ├── projects/ · time/ · settings/   # НЕ ТРОГАТЬ (заглушки)
│   │   └── StubScreen.kt                   # НЕ ТРОГАТЬ
│   ├── navigation/
│   │   └── AppNavGraph.kt                  # сигнатура: (navController, planner, onNavigate)
│   └── shell/
│       └── AppShell.kt                     # hoisting PlannerViewModel + host ScheduleSheet/TaskDetailsSheet
├── ui/theme/                               # только добавить AccentRed/AccentRedDeep в Color.kt
```

Удаляется: `ui/feature/today/TodayViewModel.kt` (stub на CorePort; заменён `ui/planner/PlannerViewModel`).

Тесты:

```text
AdiyutantAndroid/app/src/test/kotlin/com/adiyutant/app/domain/
├── PlannerCoreStateMachineTest.kt
├── ScheduleSheetContractTest.kt
├── EndOfDayTest.kt
├── DoSomethingElseTest.kt
├── ElapsedTimeTest.kt
├── SingleEntityRuleTest.kt
├── WorkloadDerivationTest.kt
├── AgentInsightTest.kt
├── QuickNoteTest.kt
└── TimeMathTest.kt
```

### 13.2 Правила реализации

- **Kotlin + Jetpack Compose + ViewModel + StateFlow + Navigation Compose** (ADR-0002); никаких новых DI-фреймворков; новые зависимости — только при крайней необходимости (`kotlinx-coroutines-test` допустим, если VM-тесты реально потребуют; приоритет — тестировать `PlannerCore` на чистом JUnit).
- **Не писать бизнес-логику в composable.** Composable: рендер state → события. Вся логика переходов — `PlannerCore` (чистый, синхронный, с инъекцией `Clock`).
- **Не дублировать доменную логику в UI-проекте** (правило AGENTS.md). Производные величины — только функции из 4.3.
- **Строки:** весь пользовательский текст — в `res/values/strings.xml` (новые ключи вида `today_*`, `plan_*`, `sheet_*`); в тестах допускаются литералы.
- **Форматы дат/времени:** статические русские массивы (WEEKDAYS_SHORT, WEEKDAYS_LONG, MONTHS, MONTHS_LONG — как в JSX) для детерминизма вне зависимости от локали устройства; время HH:MM, диапазон с en-dash «–».
- **Compose-паттерны:** `collectAsStateWithLifecycle` для state; тикер hero через `LaunchedEffect(activeSession != null)` с `delay(1000)`; `rememberSaveable` для форм листов; `BackHandler` в `AdiyutantSheet`.
- **Не переводить механически:** JSX-стили и структуру компонентов не копировать; сохранять поведение, тексты, порядок элементов и доменные правила.
- После каждого изменения Kotlin-кода: `./gradlew testDebugUnitTest` + `./gradlew assembleDebug` (в конце фазы).

### 13.3 Seed-данные

Эквивалент `INIT_UNSCHEDULED` + `INIT_SCHEDULED` из App.jsx **с датами, относительными к `Clock.today()`**:

- today: блоки t1–t6 (s1–s6) с временами/прогрессом как в JSX (09:00–17:30, progress 100/100/45/0/0/0), плюс date-only tu1, tu2 (plannedDate=today);
- tomorrow: t7–t9 (s7–s9); +2 дня: t10 (s10); -1 день: t11–t14 (s11–s14, все 100, обед s13 accent=NONE); -2 дня: t15–t16 (s15–s16, 100);
- unscheduled: tu3 (plannedDate=+1), tu4 (null), tu5 (plannedDate=+2), tu6 (null, postponed);
- `todayTop = ["t3","t4","t5"]`; `dayPhase = DAY_NOT_STARTED`; `dayMode = NORMAL`; `planUi(mode=WEEK, selectedDate=today, viewMonth=месяц today)`; `dayEnergy = 0`; `activeSession = null`; пустые factSessions/quickNotes.

## 14. Testing / acceptance criteria

### 14.1 Test ladder

| Уровень | Проверка | Как выполняется | Статус |
|---|---|---|---|
| 1. Static | `./gradlew assembleDebug`, `./gradlew compileDebugKotlin` | локально | обязателен |
| 2. Unit | `./gradlew testDebugUnitTest` — domain-инварианты, машина состояний, планирование, Plan vs Fact (чистый JVM) | локально | обязателен |
| 3. Component | Compose UI-тесты (androidTest): Today рендерит seed; переключение вкладок не сбрасывает сессию | эмулятор | обязателен минимум (обновлённый `AppShellSmokeTest`) |
| 4. Integration | нет (in-memory, без БД/сервисов) | — | неприменим |
| 5. Stand smoke | `./gradlew build`, установка APK на эмулятор, запуск, навигация | эмулятор API 26+ | обязателен |
| 6. UI automation | сценарии C1–C10 (ручной чек-лист; автотесты опционально) | эмулятор | обязателен ручной чек-лист |
| 7. User scenarios | раздел 9 + End of Day | эмулятор | обязателен |
| 8. Regression | `FakeCorePortTest` + обновлённый `AppShellSmokeTest` + все unit | локально | обязателен |
| 9. Acceptance review | evidence table + подпись чек-листов | — | обязателен |

Stand: Android-эмулятор (API ≥ 26), сборка из `AdiyutantAndroid/`, seed в памяти, сброс — перезапуск приложения. Секретов/переменных окружения нет.

### 14.2 Acceptance checklist

**Domain invariants:**

- [ ] `taskId` стабилен при всех операциях планирования/переноса/EOD (C6)
- [ ] same-date reschedule сохраняет `blockId`; cross-date move сохраняет `blockId`
- [ ] Single-entity: после SCHEDULE задача не существует как date-only; после TO_BACKLOG блок удалён
- [ ] `postponed == true` ⇒ `plannedDate == null`
- [ ] Нет дубликатов `taskId` ни в одном представлении (каталог/блоки/бэклог)
- [ ] Блоки одной даты не пересекаются (после любого confirm)

**State transitions (Today):**

- [ ] DAY_NOT_STARTED → RUNNING только через чек-ин
- [ ] READY выводится из реального времени внутри окна блока; UPCOMING — из будущего блока
- [ ] Start создаёт единственную PLANNED-сессию; повторный Start/TopThreeStart при `activeSession != null`, завершённом или отсутствующем блоке — no-op; Pause замораживает elapsed; Resume продолжает
- [ ] Плановое окончание блока НЕ завершает задачу автоматически
- [ ] Завершил (PLANNED) → ease → progress=100 + WorkSession(PLANNED_COMPLETED, durationSec=elapsed)
- [ ] «Ещё работаю» продолжает сессию за план-окончание
- [ ] «Не делал» не создаёт факт-сессию
- [ ] «Делаю другое» LEAVE стартует альтернативную сессию сразу; SHIFT/MOVE — только после confirm ScheduleSheet; если исходная PLANNED-сессия уже шла, её факт сохраняется как `PLANNED_INTERRUPTED`; cancel не теряет/не обрывает исходную сессию
- [ ] Завершение unplanned-сессии не завершает исходный блок
- [ ] End of Day: Завтра → date-only; В бэклог → без даты; На дату → date-only на дате; Выбрать время → блок на месте до confirm

**Today ↔ Plan:**

- [ ] C1–C10 из раздела 9 проходят на эмуляторе
- [ ] Активная сессия переживает Today → Plan → Today (C5)
- [ ] Переключение вкладок не сбрасывает `planUi` (режим/дата) и открытый ScheduleSheet

**Scheduling:**

- [ ] ScheduleSheet: свободные слоты без конфликтов; свой блок исключён из overlap при reschedule
- [ ] Manual time: границы 09:00–19:00, end > start; сообщения об ошибках точные
- [ ] Same-date confirm = обновление на месте; cross-date = удаление+добавление атомарно
- [ ] SCHEDULE удаляет date-only представление независимо от целевой даты
- [ ] Confirm disabled при невалидном времени/пересечении

**Timer:**

- [ ] `startedAtMs` остаётся неизменным от первого Start до завершения; elapsed = accumulatedMs + (now − runningSinceMs) при работе; только accumulatedMs при паузе
- [ ] Ряд пауз/резюмов сохраняет суммарное время без потерь/удвоений

**Pause/Resume / Completion:**

- [ ] Пауза/резюм меняют только `paused`/`runningSinceMs`/`accumulatedMs`; `startedAtMs` никогда не перезаписывается
- [ ] CompletionSheet записывает ease; завершение без сессии (Top 3 чекбокс, TaskDetails) — без WorkSession
- [ ] Блок с активной PLANNED-сессией не завершается из Plan: CTA «Открыть в Сегодня», reducer `TaskDetailsComplete` — no-op (C10)
- [ ] Инвариант завершения (3.3): блок с `progressPercent == 100` никогда не имеет активной PLANNED-сессии

**Backlog:**

- [ ] Группы Без даты / Есть дата, нет времени / Отложенные корректны
- [ ] Проекции бэклога исключают задачи с блоком: scheduled-задача не появляется ни в одной группе (нет фантомных задач)
- [ ] «Вернуть» переводит Отложенные → Есть дата, нет времени (plannedDate=today)

**End of Day:**

- [ ] Сводка: Сделано/Заметок/Факт-сессий корректны (активная сессия считается +1)
- [ ] Обработка блока активной сессии закрывает сессию как STOPPED_AT_DAY_END (нормализация)
- [ ] `EndDayFinished` закрывает активную сессию, ставит endDayReview, DAY_COMPLETED
- [ ] Отмена EndDaySheet возвращает `dayPhase=RUNNING`, закрывает transient sheet и не меняет tasks/blocks/factSessions/activeSession

**Cancellation:**

- [ ] Закрытие любого листа (back/подложка) lossless
- [ ] Отмена ScheduleSheet при purpose=DO_ELSE: нет сессии, нет изменений расписания
- [ ] Отмена EOD ScheduleSheet: блок остался в today

**No duplicate taskId:**

- [ ] upsert при EOD/бэклоге не создаёт второй записи с тем же `taskId`

**Survival:**

- [ ] Поворот экрана (config change) не сбрасывает состояние (VM Activity scope)
- [ ] Перезапуск процесса — сброс на seed (принятое ограничение v1)

### 14.3 Evidence table (заполняет исполнитель)

**Phase 3A checkpoint (2026-08-14):**

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Unit tests | `./gradlew testDebugUnitTest --rerun-tasks` | **PASS — 118/118, 0 failures, 0 errors, 0 skipped** | `AdiyutantAndroid/app/build/test-results/testDebugUnitTest/*.xml` (13 suites: PlannerCoreStateMachine 28, ScheduleSheetContract 11, EndOfDay 11, TimeMath 11, DoSomethingElse 10, ScheduleSheetLogic 9, AgentInsight 9, ElapsedTime 7, SingleEntityRule 6, QuickNote 6, WorkloadDerivation 5, AppShellSmoke 3, FakeCorePort 2) |
| Kotlin compile | `./gradlew compileDebugKotlin` | **PASS** | `BUILD SUCCESSFUL`, 0 warnings |
| Debug APK | `./gradlew assembleDebug` | **PASS** | `BUILD SUCCESSFUL`, `app/build/outputs/apk/debug/app-debug.apk` собран |
| Full build | `./gradlew build` | **NOT DONE** (Phase 7) | — |
| Stand smoke (emulator) | manual | **NOT DONE** (Phase 7 — эмулятор недоступен в среде) | — |
| Compose UI test (scrim/BackHandler LIFO/form isolation) | androidTest | **NOT DONE** (Phase 7) | — |
| User scenarios C1–C10 | emulator | **NOT DONE** (Phase 7) | — |
| Regression — `FakeCorePortTest` | JVM unit | **PASS** (2/2) | `app/build/test-results/testDebugUnitTest/TEST-com.adiyutant.app.core.fake.FakeCorePortTest.xml` |
| Regression — `AppShellSmokeTest` | JVM unit | **PASS** (3/3) | `app/build/test-results/testDebugUnitTest/TEST-com.adiyutant.app.ui.shell.AppShellSmokeTest.xml`. **Замечание:** это JVM-тест со статическими проверками (Destinations.routes, размеры seed), **НЕ реальный Compose UI smoke**. Реальный shell-render test — Phase 7. |

**Phase 2.1 freeze (commit `e51e55b`, tag `adiyutant-planner-phase2.1`):**

| Check | Result |
|---|---|
| Unit tests | PASS — 109/109 |
| Domain contracts frozen | `PlannerState`, `PlannerEvent`, `ScheduleRequest`, `PlannerCore`, `PlannerRepository`, `PlannerViewModel` не менялись после freeze |
| ТЗ checklist 7 | 100 тестов → 109 (после добавления `cancelEndDayScheduleRestoresEndDaySheetLossless`, `cancelEndDayScheduleKeepsActiveSessionAndSourceBlock`, `plainScheduleCancelClosesSheetWithoutEndDayRestore`, и др.) |

**Phase 3A residual / known issues:**

- **NOT DONE:** Phase 3B (Plan v1), Phase 4 (Today v1), Phase 5 (cross-screen), Phase 6 (visual polish), Phase 7 (acceptance/build + emulator smoke + TalkBack/accessibility).
- **Stale W3:** finding пред-Phase 3A ревью про восстановление `EndDaySheet` после cancel ScheduleSheet(purpose=END_DAY) — **НЕ подтверждается**. `PlannerCore.cancelSheet` уже обрабатывает purpose=END_DAY (PlannerCore.kt:442–444); покрыто `EndOfDayTest.cancelEndDayScheduleRestoresEndDaySheetLossless`.
- **OpenCompletionSheet contract:** `PlannerEvent.OpenCompletionSheet(blockId)` реализован и принят в Phase 2.1 (PlannerCore.kt:188–194). Guard: блок существует, `progressPercent < 100`, активная сессия PLANNED для этого `blockId`. `completionConfirmed` принимается из `BlockEndedSheet` или `CompletionSheet` (PlannerCore.kt:201–205). Покрыто `PlannerCoreStateMachineTest.openCompletionSheetOpensForValidUnfinishedBlock`, `openCompletionSheetIsNoOpForUnknownBlock`, `openCompletionSheetIsNoOpForCompletedBlock`, `openCompletionSheetIsNoOpForIncompleteBlockWithoutSession`, `openCompletionSheetIsNoOpForIncompleteBlockWithOtherActiveSession`, `completionConfirmedWorksFromCompletionSheet`. Phase 4 использует frozen event/API без дополнительных prerequisites.

## 15. Implementation phases

| Фаза | Содержание | Критерий выхода |
|---|---|---|
| **Phase 1 — domain/state** | Все модели `domain/model/`, `PlannerCore` со всеми переходами (разделы 3, 4, 7.2, 10, 11), `ScheduleValidator`, `TimeMath`, `Clock`, `data/` (репозиторий + seed), расширение `ServiceLocator` | Unit-тесты Phase 1 зелёные: инварианты, машина состояний, ScheduleSheet contract, EOD, elapsed, single-entity |
| **Phase 2 — navigation shell** *(ACCEPTED 2026-08-14, freeze `e51e55b`, tag `adiyutant-planner-phase2.1`)* | `PlannerViewModel`, удаление старого `TodayViewModel`, hoisting в `AppShell`, `AppNavGraph(navController, planner, onNavigate)`, host общих листов (пустые заглушки), обновление `AppShellSmokeTest`, новые строки в `strings.xml` | `assembleDebug` зелёный; приложение запускается; 5 вкладок; smoke-тест зелёный (109 unit-тестов на момент freeze) |
| **Phase 2.1 — review fixes** *(в составе freeze `e51e55b`)* | `PlannerCore.cancelSheet` восстанавливает `EndDaySheet` для ScheduleSheet(purpose=END_DAY); `dayPhase` сохраняется END_OF_DAY; `pendingDoElse` сбрасывается; Plan/Fact разделение закрыто тестами | `cancelEndDayScheduleRestoresEndDaySheetLossless`, `cancelEndDayScheduleKeepsActiveSessionAndSourceBlock`, `plainScheduleCancelClosesSheetWithoutEndDayRestore` |
| **Phase 3A — shared UI gate** *(ACCEPTED AND FROZEN 2026-08-14)* | Один writer реализует `ui/common/` (AdiyutantSheet, Label, SegmentedControl, EmojiScale, MoodPicker, KpiMini, Chip), `ScheduleSheet`, `TaskDetailsSheet`; публичные параметры/события фиксируются и после gate считаются read-only для экранных агентов | `assembleDebug` зелёный; ScheduleSheet/TaskDetails contracts покрыты тестами (118/118 unit); API `ui/common/` заморожен; `collectAsStateWithLifecycle` во всех 3 потребителях state; `ScheduleSheet.rememberSaveable` ключуется по identity `ScheduleRequest` (form-state isolation) |
| **Phase 3B — Plan** | `PlanScreen` + все view (Header, Day, Week, Backlog, DayPickerStrip, Workload, Timeline), `PlanAgentCard`; может выполняться параллельно с Phase 4 после gate 3A | Сценарии P1–P10 и ScheduleSheet-контракт работают на эмуляторе; проекции бэклога без фантомных задач; Plan-юниты зелёные |
| **Phase 4 — Today** | `TodayScreen` + Header, KpiRow, HeroCard (все 9 вариантов), AgentCard, TopThreeSection, QuickNoteSection, 9 Today-листов; может выполняться параллельно с Phase 3B после gate 3A | Машина состояний (7.2) и все Today-действия работают; Today-юниты зелёные |
| **Phase 5 — integration** | Двухшаговый «Делаю другое» с pendingDoElse, EOD → ScheduleSheet (purpose=END_DAY), закрытие сессий при EOD (нормализация), Agent-навигация, CTA «Открыть в Сегодня» (TaskDetailsSheet при активной сессии), проверка C1–C10 | C1–C10 проходят на эмуляторе |
| **Phase 6 — visual polish** | `AccentRed`/`AccentRedDeep`, градиенты hero, отступы/радиусы по DESIGN.md, insets, touch targets ≥44dp, a11y contentDescription | Визуальное соответствие чек-листу раздела 12 |
| **Phase 7 — tests/build** | Полный прогон unit + build, ручной чек-лист 14.2, evidence table, фиксация дрейфов в документации | Все чекбоксы 14.2 выполнены или явно отмечены как skip с причиной; `./gradlew build` зелёный |

## 16. Out of scope (явно)

- **Backend / сеть:** никаких серверов, API, синхронизации.
- **Persistence:** in-memory; перезапуск приложения возвращает seed (допустимо для v1).
- **Уведомления/напоминания.**
- **Проекты / Время / Настройки:** остаются заглушками; никакой бизнес-логики.
- **Rust Core интеграция** (ADR-0003) и расширение `CorePort` — отдельный трек; файлы `core/` не трогаются.
- **Новые AI-возможности:** Agent — только два локальных правила (Today: перегруз/низкая энергия; Plan: перегруз/просроченный блок). Никаких LLM-вызовов, никаких ключей.
- **Создание/редактирование задач через UI** (в прототипе нет; только планирование/перенос/завершение существующих).
- **Текущая линия времени в timeline** (опционально в дизайне, не в v1).
- **Пиксель-перфект JSX:** приоритет — поведенческий паритет (MEMORY.md: polish после поведенческого цикла).
- **Rollover даты в полночь** (todayDate фиксируется при старте).
- **Demo-контролы прототипа** (кнопки Hero-состояний вне рамки телефона).

---

## Сводка

### Зафиксированные архитектурные решения

- **R1.** Домен Task/ScheduledBlock/WorkSession реализуется локально в Android (`domain/` + `data/`), in-memory; `PlannerRepository` — будущий шов под Rust Core; `CorePort` не трогается. Дрейф против ADR-0002 зафиксирован в **ADR-0004** (создан по решению владельца).
- **R2.** Единый `PlannerViewModel` (StateFlow, Activity scope) + чистый синхронный reducer `PlannerCore.reduce(state, event)` — вся бизнес-логика тестируется на JVM без эмулятора.
- **R3.** Immutable `PlannerState`, event-driven (`PlannerEvent` sealed); UI не мутирует домен; поля форм листов — локальный `rememberSaveable`, в VM — только confirm.
- **R4.** `startedAtMs` неизменяем и хранит начало всей factual session; elapsed выводится из `runningSinceMs + accumulatedMs`, пауза не накапливает. Это сохраняет корректный `WorkSession.startedAtMs` для будущего экрана «Время».
- **R5.** Cancel любого листа lossless для бизнес-данных: допускаются только закрытие transient workflow-state (`sheet`, `pendingDoElse`) и возврат EndDay `dayPhase` в RUNNING; tasks/blocks/fact/active session не теряются.
- **R6.** Single-entity rule: единый каталог `Task` + `blocks`; задача существует либо как date-only, либо как блок; `taskId` стабилен всегда; `blockId` сохраняется при same-date reschedule и cross-date move; детерминированная генерация blockId. **Проекции бэклога и date-only ОБЯЗАНЫ исключать задачи, имеющие ScheduledBlock** (защита от фантомных задач в бэклоге).
- **R7.** Нормализации против JSX (подтверждены владельцем): UPCOMING/READY — от реальных часов (`Clock`), demo-контролы не переносятся; hardcoded `t5` в Today-Agent заменён на «первый незавершённый блок»; EOD «На дату» — чипы следующих 7 дней; EOD закрывает осиротевшую сессию как STOPPED_AT_DAY_END.
- **R8.** Plan vs Fact разделены: плановое окончание ≠ завершение; незапланированная работа не завершает план; завершение без сессии не создаёт факт-сессию.
- **R9.** Agent — только локальные правила, CTA всегда выполняет видимое действие, карточка скрыта при отсутствии insight.
- **R10.** Navigation Compose сохраняется; общий VM + общие листы на уровне shell; пользовательские строки в `strings.xml`.
- **R11 (решение владельца).** Инвариант завершения: блок с `progressPercent == 100` не может иметь активную PLANNED-сессию. Plan не завершает блок с активной сессией — в TaskDetailsSheet показывается CTA «Открыть в Сегодня»; reducer отклоняет `TaskDetailsComplete` как no-op. JSX-поведение (молчаливое завершение с «висящей» сессией) НЕ сохраняется.
- **R12.** В state может быть только одна `ActiveSession`; reducer запрещает второй Start. При подтверждённом «Делаю другое» уже накопленный факт исходной PLANNED-сессии сохраняется как `PLANNED_INTERRUPTED`, а не исчезает при замене active session.
- **R13.** Реализация swarm-safe: domain/state и shared UI проходят single-writer gates; затем Plan и Today допускают максимум два параллельных writer-а с жёстким file ownership; merge/integration и acceptance выполняет один интегратор.

### Выявленные риски

1. **Дрейф домена против ADR-0002** (Android-локальный домен vs «UI потребляет только CorePort») — смягчён швом `PlannerRepository` и ADR-0004; миграция на Core не потребует изменения UI.
2. **Нормализации против JSX** (реальные часы, обобщённый Agent, EOD-чипы дат, закрытие осиротевших сессий, CTA «Открыть в Сегодня») — осознанные отклонения от буквы прототипа; решения приняты владельцем, при ревью финальной реализации стоит пройтись по списку ещё раз на эмуляторе.
3. **In-memory:** потеря состояния при убийстве процесса; midnight-rollover не обрабатывается — принято для v1, задокументировано.
4. **Зависимости:** для VM-тестов может понадобиться `kotlinx-coroutines-test` — допустимо, но приоритет на чистых JUnit-тестах `PlannerCore`.

### Открытые вопросы (не блокируют реализацию)

1. Финализация Проектов/Времени/Настроек — вне scope, остаётся в ROADMAP.
2. Следующий продукт после Today + Plan — экран «Время» на реальных данных `WorkSession` (по MEMORY.md Next Steps).


