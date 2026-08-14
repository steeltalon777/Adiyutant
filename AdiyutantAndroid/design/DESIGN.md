# Adiyutant Design System

---
version: alpha-0.2
name: Adiyutant Design System
updatedAt: "2026-08-14"
---

# Adiyutant Design System

## Overview

Adiyutant is a dark mobile AI-assistant app built around a 390×875 reference frame with iOS-style chrome. The product has five primary tabs: **Сегодня, План, Проекты, Время, Настройки**.

Core product principle:

> **«Сегодня исполняет план. План строит план. Время показывает факт.»**

- **Сегодня** answers: what should I do now, and what is actually happening with my day?
- **План** answers: what am I going to do, and when?
- **Время** will answer: where did the time actually go?
- **Проекты** provides project structure and progress context.
- **Настройки** controls behavior, integrations, notifications and interface preferences.

The UI is dark-first, compact, stateful and action-oriented. The Agent is contextual infrastructure, not a separate bottom-tab destination.

## Design Principles

1. **Execution and planning are different surfaces.** Today must not become a second calendar; Plan must not become a live timer.
2. **Plan and fact are separate.** A scheduled block describes intent; a work session describes reality.
3. **One logical task, one identity.** Rescheduling must never clone the task.
4. **Actionable Agent only.** Hide the Agent card when there is no concrete insight or action.
5. **No dead controls.** Any visible button must perform the action it promises.
6. **Cancel must be lossless.** Closing a sheet must never silently delete or move a task.
7. **Free time is valid.** Do not auto-fill every free window.
8. **Explicit text colors on dark surfaces.** Headings and labels must never rely on browser/default black text.
9. **Reuse patterns.** New screens should extend existing tokens, sheets, cards and interaction rules rather than invent parallel systems.

## Colors

| Token | Hex | Usage |
|---|---|---|
| bg | `#0A121B` | Main app background |
| bgDeep | `#071019` | Outer/deep background |
| surface | `#111A24` | Cards, primary surfaces |
| surface2 | `#151F2A` | Secondary surfaces / controls |
| surface3 | `#172230` | Tertiary surfaces |
| fg | `#F4F7FB` | Primary text |
| muted | `#B5BFCC` | Secondary text |
| muted2 | `#7D8997` | Tertiary text |
| border | `#253240` | Borders/dividers |
| accent | `#2F80FF` | Primary action / selection |
| accentSoft | `#183D77` | Soft blue surface tint |
| accentDeep | `#0F2C5C` | Deep blue hero background |
| accentGreen | `#31D06D` | Success / energy / completed |
| accentGreenDeep | `#0F3B23` | Green surface tint |
| accentOrange | `#FFAD3D` | Warning / importance / move attention |
| accentOrangeDeep | `#3D2A0F` | Orange surface tint |
| accentPurple | `#A660FF` | AI / insights / deep work |
| accentPurpleDeep | `#26183D` | Purple surface tint |
| accentRed | `#FF4D4D` | Danger / overload / alert |
| accentRedDeep | `#3D1414` | Red surface tint |

### Accent semantics

- **Blue**: navigation, primary action, selected state.
- **Green**: completed, success, energy, factual execution success.
- **Orange**: warning, overload, reschedule attention.
- **Purple**: Agent insight, AI action, deep-work context.
- **Red**: destructive or severe overload only; avoid using it for normal lateness.

## Typography

| Token | fontFamily | fontSize | fontWeight | lineHeight | letterSpacing |
|---|---|---|---|---|---|
| display | `SF Pro Display, Inter, system-ui, sans-serif` | — | — | — | — |
| body | `SF Pro Text, Inter, system-ui, sans-serif` | — | — | — | — |
| mono | `ui-monospace, SF Mono, Menlo, monospace` | — | — | — | — |
| h1 | `SF Pro Display, Inter, system-ui, sans-serif` | 26px | 700 | 1.05 | -0.025em |
| h2 | `SF Pro Display, Inter, system-ui, sans-serif` | 17px | 600 | 1.2 | normal |
| h3 | `SF Pro Text, Inter, system-ui, sans-serif` | 14px | 600 | 1.25 | normal |
| bodyText | `SF Pro Text, Inter, system-ui, sans-serif` | 14px | 400 | 1.4 | normal |
| meta | `SF Pro Text, Inter, system-ui, sans-serif` | 11px | 500 | 1.3 | 0.04em |
| button | `SF Pro Text, Inter, system-ui, sans-serif` | 14px | 600 | 1 | -0.01em |
| mini | `SF Pro Text, Inter, system-ui, sans-serif` | 10px | 500 | 1.2 | 0.06em |

## Rounded

| Token | Value |
|---|---|
| card | 16px |
| cardSm | 12px |
| btn | 10px |
| btnSm | 8px |
| pill | 999px |

## Spacing

Base spacing unit: **4px**.

| Token | Value |
|---|---|
| xs | 4px |
| sm | 8px |
| md | 12px |
| page | 16px |
| sectionGap | 16px |
| cardPad | 14px |

## Device & Navigation

### Reference Device

- Logical content frame: **390×875px**.
- iOS-style status bar, Dynamic Island and home indicator are presentation chrome only.
- Demo/debug controls must stay **outside** the production phone frame.
- Touch targets should be **≥44px** where practical.

### Bottom Navigation

Order:

1. Сегодня
2. План
3. Проекты
4. Время
5. Настройки

Active tab uses `accent`; inactive tabs use `muted2`.

The tab bar changes screens but must not reset shared task, schedule or active execution state.

## Core Data & Interaction Semantics

These semantics are part of the design contract and must survive the future Kotlin/Compose implementation.

### Identity model

```text
Task.taskId
    ↓
ScheduledBlock.blockId
    ↓
WorkSession
```

- `taskId` is stable logical identity.
- `blockId` represents one placement in the schedule and may change/move.
- `WorkSession` is factual execution and may be planned or unplanned.
- Top 3, completion, projects and future analytics reference `taskId`, not a transient block id.

### Date-only task

A task may have `plannedDate` without a time.

```text
Task(plannedDate = date) + no ScheduledBlock
```

This means **Есть дата, нет времени**. It is not a scheduled block.

### Single-entity rule

A logical task must never exist simultaneously as both:

- a scheduled block with time, and
- an unscheduled/date-only entry with the same `taskId`.

Scheduling consumes the date-only/unscheduled representation. Moving back to backlog removes the scheduled placement.

### Plan vs fact

- Scheduling/rescheduling changes intent.
- Start/Pause/Resume/Complete changes factual execution.
- Unplanned work may create a factual session without completing the planned task.

## Components

### Buttons

- Primary: ~46px height, radius 10px, `accent`, white text.
- Secondary: ~46px height, radius 10px, `surface`/`surface2`, visible border.
- Outline: transparent, visible border.
- Danger: `accentRed` only for destructive actions.
- Small: ~34px height, radius 8px.
- Icon button: minimum visual target approximately 32–46px depending on density.

### Cards

- Standard: radius 16px, padding 14px, `surface`, subtle border.
- Compact: radius 12px, padding 10–12px.
- AI/Agent card: purple family, clear action, no filler state.
- Hero cards may use deep tinted gradients but must preserve text contrast.

### Controls

- Checkbox: ~22×22px, radius ~7px.
- Switch: ~46×28px.
- Radio: ~20×20px.
- Segmented control: `surface2` base, active segment visually raised/selected.
- Selected state must be visually clear without relying only on text color.

### Progress

- Linear: 6–8px, pill radius.
- Circular: ~46×46px, 4px stroke.
- Progress values should be derived from current state where possible, not duplicated static numbers.

### Chip

- Height ~28px.
- Pill radius.
- Use for time windows, status, free slots and compact metadata.

### Bottom Sheet

Shared modal interaction pattern for Today and Plan.

- Rounded top corners.
- Dimmed backdrop.
- Scrollable when content exceeds viewport.
- Explicit primary/secondary actions.
- Closing/canceling is lossless.

### ScheduleSheet

`ScheduleSheet` is the **single scheduling/rescheduling surface** shared by Today and Plan.

Modes:

- `schedule`: place a date-only/backlog task into a time slot.
- `reschedule`: modify an existing scheduled block.

Required behavior:

- source date and target date are distinct concepts;
- same-date reschedule = **Изменить время**;
- cross-date reschedule = **Перенести**;
- selectable suggested/free slots;
- manual time picker with overlap/workday validation;
- date selection when moving across days;
- optional date lock when the flow requires same-day movement;
- own block excluded from overlap calculation during reschedule;
- cancel does not mutate schedule;
- confirmation preserves `taskId`.

### TaskDetailsSheet

Actions:

- **Изменить время** → same-date `ScheduleSheet`.
- **Перенести** → cross-date `ScheduleSheet`.
- **Завершить** → marks the logical task/block completed.

### Agent Card

- Purple semantic family.
- Appears only when there is an actionable insight.
- Must include a concrete CTA when shown.
- Examples: overload, low energy + heavy block, missed block, actual work disrupting the remaining plan.
- Hide completely when no action is warranted.

## Screen: Сегодня

### Role

Execution dashboard. It continuously answers **«Что мне делать сейчас?»** and records the difference between plan and fact.

### Visual hierarchy

1. Header: Сегодня + date + Check-in / Finish Day action.
2. KPI row: Режим дня / Прогресс / Энергия.
3. Current-state hero card.
4. Event-driven Agent card when actionable.
5. Главное на сегодня (Top 3).
6. Быстрая заметка.

### KPI behavior

- **Режим дня**: Focus / Обычный / Лёгкий / Восстановление.
- **Прогресс**: task completion, Top 3 completion, planned/completed minutes.
- **Энергия**: opens/reuses Check-in, not the mode selector.

### Check-in

Captures:

- energy;
- focus;
- mood/wellbeing;
- optional obstacle/blocker.

First check-in may start the day; later check-ins update current state.

### Hero state machine

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

Hero presentation may simplify these into visual variants such as `pre`, `ready`, `active`, `paused`, `between`, `eod`, `done`.

Rules:

- scheduled end time never auto-completes a task;
- active factual session survives Today ↔ Plan navigation;
- paused time does not accrue;
- elapsed time is derived from timestamps/accumulated duration, not an independent source of truth.

### Start / Pause / Resume / Complete

- Start creates an active factual session for the selected scheduled block.
- Pause freezes accumulated factual duration.
- Resume continues the same session.
- Complete records a factual session and marks the task/block completed.
- Completion review may record perceived difficulty: Легко / Нормально / Тяжело.

### Делаю другое

Two-stage flow:

1. Choose factual activity:
   - Другая задача
   - Незапланированная
   - Перерыв
   - Личное
   - Другое
2. Choose treatment of the original planned block:
   - Оставить в плане
   - Сдвинуть
   - Перенести

For **Другая задача**, select a real existing Today task.

Unplanned/break/personal work creates a factual session that must remain visible in the hero and must be independently pausable/completable. Completing it must not automatically complete the original planned task.

### Блок закончился / Не делал

Planned end time does not imply completion.

User can resolve the block as:

- Завершил
- Ещё работаю
- Не делал

`Не делал` records no work time and offers:

- Перенести
- В бэклог
- Оставить

### Главное на сегодня

- Up to 3 primary outcomes.
- References stable `taskId`.
- Inline completion/start actions.
- `Показать все` expands/opens a Today-only task list; do not navigate to Plan merely to show all Today tasks.

### Quick Note

Types:

- Заметка
- Идея
- Проблема
- Итог

Where possible attach current task/project/session context and timestamp.

### End of Day

End-day review shows summary and resolves unfinished work.

For each unfinished task:

- Завтра: date-only task unless a time is explicitly chosen.
- В бэклог: no planned date.
- На дату: chosen date, no time.
- Выбрать время: use shared `ScheduleSheet`.

The original scheduled block must not be removed until a time-based reschedule is successfully confirmed. Cancel leaves it intact.

## Screen: План

### Role

Planning workspace. It answers **«Что и когда я собираюсь делать?»**. It is not merely a calendar.

### Modes

Segmented control:

```text
День | Неделя | Бэклог
```

Default: **Неделя**.

### Day / Week behavior

- Weekday strip changes `selectedDate` without leaving the mode.
- Day uses the same selected date with a more detailed timeline.
- Workload is derived from scheduled blocks + date-aware unscheduled tasks.
- Free slots are informative; they are not automatically filled.
- `Нужно распределить` contains only tasks whose `plannedDate` matches the selected date and that have no scheduled time.

### Workload

Display at least:

- Запланировано
- Без времени
- Перегруз when > available capacity

Avoid duplicated static workload numbers; derive from current state.

### Timeline

- Scheduled blocks are clickable.
- Completion state is visible.
- Current-time marker may be shown for Today.
- Clicking a block opens `TaskDetailsSheet`.

### Backlog semantics

Three groups:

1. **Без даты**: `plannedDate == null`, not postponed.
2. **Есть дата, нет времени**: `plannedDate != null`, no scheduled block.
3. **Отложенные**: explicit postponed/deferred state.

### Agent in Plan

Agent suggestions may help resolve overload or expose the problematic day/block. CTA must always perform a visible action.

## Screens: Проекты / Время / Настройки

These screens are not yet final and must reuse the system above.

### Проекты

Planned direction: project cockpit with stage, progress, next milestone, active tasks, blockers and contextual Agent insight.

### Время

Planned direction: factual timeline and analytics derived from `WorkSession`, including planned-vs-actual comparisons, deep work and unplanned work.

### Настройки

Planned direction: profile/day defaults, check-in, Agent initiative/permissions, planning behavior, integrations, notifications, privacy and appearance.

## Prototype / Implementation Notes

- `App.jsx` is the current executable behavioral specification, not the final production architecture.
- `today-screen.html` remains the visual reference for Today.
- `adiyutant-ui-kit.html` remains the component reference.
- The future Android implementation should reproduce behavior natively in Kotlin/Jetpack Compose rather than mechanically translating JSX.
- Preserve the shared domain semantics and state transitions even if component/file structure changes.
