---
schemaVersion: 1
scope: workspace
updatedAt: "2026-08-14T07:55:00.000Z"
workspaceName: "Adiyutant"
---

# Project Memory

## Project Overview

- Adiyutant is a dark AI-assistant mobile app, reference frame 390×875, with five primary tabs: **Сегодня, План, Проекты, Время, Настройки**.
- Core product rule: **«Сегодня исполняет план. План строит план. Время показывает факт.»**
- `today-screen.html` is the original visual source of truth for Today.
- `adiyutant-ui-kit.html` is the reusable component-library reference.
- `DESIGN.md` is the authoritative design-system and interaction-pattern document.
- UI language is Russian.

## Current State

- `App.jsx` is the unified interactive prototype and current executable behavioral specification.
- Current repaired version is ~112 KB / ~2179 lines and contains Today + Plan in one app shell with shared state.
- **Plan v1 is frozen/stable** for behavior and should not be redesigned while implementing other screens.
- **Today v1 is ready for freeze** after the final repair pass: shared execution state, stable task identity, factual sessions, two-step «Делаю другое», safe End-of-Day rescheduling, working Energy → Check-in, and lossless ScheduleSheet cancel behavior.
- Проекты, Время and Настройки remain stub/future screens.
- Persistence is not implemented; state is currently in-memory.

## Product Semantics

- **План** = what and when the user intends to do.
- **Сегодня** = what to do now and what is actually happening.
- **Время** = factual time and planned-vs-actual analysis; this is the logical next product surface after Today + Plan.
- Agent is contextual and embedded in screens; there is no separate Agent bottom tab.

## Domain Model

Stable identity contract:

```text
Task.taskId
    ↓
ScheduledBlock.blockId
    ↓
WorkSession
```

Rules:

- `taskId` is permanent logical identity.
- `blockId` is a schedule placement and can move/change.
- Top 3, completion, future Projects and analytics reference `taskId`.
- `WorkSession` represents factual execution separately from planning.
- A date-only task (`plannedDate` with no time) is not a `ScheduledBlock`.
- A logical task must not exist simultaneously as a scheduled block and an unscheduled/date-only entry with the same `taskId`.
- Reschedule preserves `taskId`.

## Shared App State

The app shell owns state required by both Today and Plan so it survives tab switching. Current concepts include:

- `unscheduled`
- `scheduledMap`
- `planUi`
- `scheduleSheet`
- `taskSheet`
- `activeSession`
- `factSessions`
- `dayCheckIn`
- `dayMode`
- `dayEnergy`
- `dayStatus`
- `todayTop`
- `quickNotesToday`
- `endDayReview`

Today ↔ Plan navigation must never reset the active execution session or shared schedule state.

## Plan v1 — Frozen Behavior

- Modes: **День | Неделя | Бэклог**.
- Default mode: **Неделя**.
- Weekday strip changes selected date without navigating away.
- Day uses the same selected date with a detailed timeline.
- `ScheduleSheet` is shared and supports `schedule` + `reschedule`.
- Same-date reschedule = **Изменить время**.
- Cross-date reschedule = **Перенести**.
- ScheduleSheet supports selectable free slots and a real manual time picker with workday/overlap validation.
- Date-aware unscheduled tasks use `plannedDate`.
- Workload is derived from actual blocks + date-only tasks, not stored as independent contradictory totals.
- `Нужно распределить` shows only tasks for the selected date.
- Backlog groups:
  - Без даты
  - Есть дата, нет времени
  - Отложенные
- `TaskDetailsSheet` actions: Изменить время / Перенести / Завершить.
- Plan Agent CTA must perform a visible action; no dead controls.

## Today v1 — Frozen Behavior

### Structure

- Header with date and Check-in / Finish Day action.
- KPI row: Режим дня / Прогресс / Энергия.
- State-machine hero card.
- Event-driven Agent card, hidden when no actionable insight exists.
- Главное на сегодня (Top 3).
- Быстрая заметка.

### State machine

Canonical states:

```text
day_not_started
→ upcoming
→ ready
→ active ⇄ paused
→ block_ended_unresolved
→ between_blocks
→ end_of_day
→ day_completed
```

The implementation may render a smaller set of hero variants while preserving these semantics.

### Check-in / KPIs

- Check-in stores energy, focus, mood/wellbeing and optional obstacle.
- First check-in may start the day.
- Energy KPI opens/reuses Check-in.
- Day mode choices: Focus / Обычный / Лёгкий / Восстановление.
- Progress is derived from current tasks/Top 3/minutes.

### Factual execution

- Start creates `activeSession` for a scheduled block.
- Pause freezes elapsed factual time.
- Resume continues from accumulated duration.
- Elapsed is derived from timestamps + accumulated duration; it is not a second independent source of truth.
- Complete marks the block completed and records a factual session.
- Active session survives Today → Plan → Today.

### «Делаю другое»

Two-step flow:

1. Choose actual activity:
   - Другая задача
   - Незапланированная
   - Перерыв
   - Личное
   - Другое
2. Choose what happens to original planned block:
   - Оставить в плане
   - Сдвинуть
   - Перенести

For `Другая задача`, user selects a real existing Today task.

Unplanned/break/personal factual sessions are visible in the Today hero and can be paused/completed. Completing them does not automatically complete the original planned task.

### Block unresolved / missed

Planned end time never auto-completes.

Resolution options:

- Завершил
- Ещё работаю
- Не делал

`Не делал` records no factual work and offers:

- Перенести
- В бэклог
- Оставить

### Top 3

- Maximum 3 primary outcomes.
- References stable `taskId`.
- Inline start/completion behavior.
- `Показать все` is Today-only; do not navigate to Plan merely to display all Today tasks.

### Quick Note

- Types: Заметка / Идея / Проблема / Итог.
- Store local timestamp and attach task/project/session context where available.

### End of Day

Unfinished task actions:

- Завтра → date-only task unless time is explicitly selected.
- В бэклог → no date.
- На дату → selected date, no time.
- Выбрать время → shared `ScheduleSheet`.

Important invariant: a time-based End-of-Day move does **not** delete the original block before successful ScheduleSheet confirmation. Cancel is lossless. Confirmation removes/moves only after target date/time is known and preserves `taskId`.

## Shared Scheduling Rules

- Today and Plan use the same `ScheduleSheet` behavior.
- `sourceDate` is where the block currently lives.
- `initialDate` is only the proposed target/date shown when the sheet opens.
- Same-date confirmation updates the existing placement.
- Cross-date confirmation removes the old placement and adds the block on target date.
- `taskId` is preserved.
- Scheduling a date-only task removes its unscheduled representation.
- Cancel must not mutate state.

## Design Direction

- Deep dark palette; main bg `#0A121B`.
- Explicit high-contrast text colors, especially headings.
- Four main semantic accents: blue primary, green success/energy, orange warning/move attention, purple AI/deep work.
- Reuse existing cards, sheets, segmented controls, chips, progress patterns and navigation.
- Do not redesign stable screens during behavioral fixes.
- Agent appears only for actionable insights; no motivational filler.

## Artifacts

- `App.jsx` — current executable interactive prototype; Today + Plan are implemented.
- `DESIGN.md` — authoritative design/interaction system, updated after Today v1 final repair.
- `MEMORY.md` — workspace decisions/current status.
- `today-screen.html` — legacy/original Today visual source.
- `adiyutant-ui-kit.html` — component library reference.

## Validation / Repair History

- Plan was iteratively corrected for date-aware unscheduled tasks, workload math, selectable slots, schedule/reschedule semantics, real manual time picker, backlog semantics and working Agent actions.
- Today was rebuilt as a shared-state execution dashboard and then audited after OpenCoDesign hit provider/output limits.
- Goose audit found pause/resume timing, End-of-Day duplicate-identity risks and dead controls; those areas were corrected.
- Final manual repair pass additionally fixed remaining Today wiring/runtime issues, Energy KPI routing, two-step «Делаю другое», unplanned-session hero behavior and lossless End-of-Day time rescheduling.
- Current prototype is the handoff source for native implementation; future agents should inspect the current `App.jsx` rather than rely on older generated reports.

## Implementation Handoff: Android / Kotlin

The next implementation stage may use the current JSX prototype as a behavioral specification for a native Android app.

Recommended target:

- Kotlin
- Jetpack Compose
- Navigation Compose
- shared ViewModel / StateFlow state holder
- local/in-memory repository first
- no backend required for initial Today + Plan port

Do **not** mechanically translate JSX to Kotlin. Preserve behavior and domain invariants, then express them natively.

Recommended conceptual modules:

```text
ui/today
ui/plan
ui/common
navigation
domain
data
```

Core native models should map to:

- `Task`
- `ScheduledBlock`
- `WorkSession`
- `CheckIn`
- `QuickNote`
- day/execution state

First milestone for the Kotlin port: **Today + Plan build successfully with one shared state model and cross-screen behavior preserved**. Pixel-perfect polish comes after the behavioral loop works.

## Open Questions

- Final scope/content of Проекты.
- Final scope/content of Время, beyond factual timeline + planned-vs-actual analytics.
- Final scope/content of Настройки, including Agent initiative/permissions and integrations.
- Whether original HTML assets remain in the production repository or move to `/docs/legacy` after Kotlin migration.
- Whether free windows should proactively suggest a short task or remain passive by default.

## Next Steps

1. Freeze/commit current JSX prototype as the reference checkpoint.
2. Send `App.jsx`, `DESIGN.md`, `MEMORY.md` (plus optional HTML references) to the devstand.
3. Implement native **Сегодня + План** in Kotlin/Jetpack Compose using shared ViewModel/StateFlow.
4. Build and fix compile/runtime issues before visual micro-polish.
5. After Today + Plan are stable natively, design/implement **Время** from real `WorkSession` data.
6. Then move to Проекты and Настройки.

## Recent History

- 2026-08-14: Initial design tokens and component system extracted; Plan prototype created.
- 2026-08-14: Plan v1 expanded to Day/Week/Backlog with interactive scheduling and task details.
- 2026-08-14: Plan data-model corrections added date-aware tasks, derived workload, true reschedule/move semantics, manual time picker and semantic backlog groups.
- 2026-08-14: Today rebuilt from `today-screen.html` as interactive execution dashboard with shared state, state-machine hero, KPIs, Agent, Top 3, quick notes and End-of-Day flow.
- 2026-08-14: OpenCoDesign stalled on output/context limits during final repair.
- 2026-08-14: Goose audit identified several remaining behavioral/runtime issues.
- 2026-08-14: Final manual repair produced the current JSX handoff candidate; documentation updated for Kotlin migration.
