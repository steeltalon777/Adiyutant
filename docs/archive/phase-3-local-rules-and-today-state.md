# TZ: Phase 3 — Local Rules and Today State

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-3-local-rules-and-today-state.md`

## Execution Strategy

- [ ] Sequential execution recommended
- **Reason:** `TodayState` зависит от доменных моделей (Фаза 2). `LocalRuleAgentGateway` зависит от `TodayState`. Оба модуля — в `adiyutant_core`. `adiyutant_store` и `adiyutant_cli` не изменяются.

## Execution Checklist

- [x] 0. Context verified
- [x] 1. `today_state.rs` — TodayState struct + builder
- [x] 2. `local_rule_gateway.rs` — LocalRuleAgentGateway + rules
- [x] 3. Export both from `lib.rs` + `model/mod.rs` (?)
- [x] 4. Static checks: `cargo fmt`, `cargo check`
- [x] 5. Unit tests: `cargo test`
- [x] 6. Integration: `cargo check --workspace`
- [x] 7. Final acceptance review

## Check Rules

- Все пункты отмечаются только после реализации и верификации.
- Если проверка не прошла — пункт остаётся незаполненным с причиной.

---

## 1. Source Documents

| Документ | Роль |
|---|---|
| `ROADMAP.md` — Phase 3 | Источник scope |
| `SPECIFICATION.md` — §5.1 (Core отвечает), §7.2 (LocalRuleAgentGateway), §7.1 (MockAgentGateway) | Поведение |
| `ARCHITECTURE.md` — Core Boundary | Что Core owns |
| `docs/core-boundary.md` | Границы Core |
| `AGENTS.md` | Правила репозитория |

---

## 2. Goal

Получить полезное поведение без внешнего ИИ:

- **TodayState** — read-агрегатор состояния текущего дня, собирающий данные из всех доменных моделей в одну структуру для CLI/UI
- **LocalRuleAgentGateway** — набор локальных правил (без LLM), которые анализируют TodayState и генерируют `ActionProposal`

Оба модуля живут в `adiyutant_core` (новые файлы). `adiyutant_store` не трогается — работаем in-memory (SQLite — Фаза 4).

---

## 3. Scope

### In scope

- `adiyutant_core/src/today_state.rs` — TodayState структура и конструктор
- `adiyutant_core/src/local_rule_gateway.rs` — LocalRuleAgentGateway + набор правил
- Экспорт из `adiyutant_core/src/lib.rs`
- Unit tests для TodayState и LocalRuleAgentGateway

### Out of scope

- SQLite — Фаза 4
- Репозитории — Фаза 4
- `adiyutant_store` — не трогать
- `adiyutant_cli` — не трогать
- LLM, Agent Gateway, backend, sync
- Серьёзный rule engine (простые if-правила, не petri net / decision tree)

---

## 4. Implementation Stages

### Stage 1 — `today_state.rs`

**Файл:** `AdiyutantCore/adiyutant_core/src/today_state.rs`

TodayState — read-only snapshot состояния текущего дня.

```rust
use crate::datetime::AdiyutantDateTime;
use crate::model::action_proposal::ActionProposal;
use crate::model::alarm::AlarmDefinition;
use crate::model::check_in::{CheckIn, CheckInType};
use crate::model::context_document::ContextDocument;
use crate::model::daily_log::DailyLog;
use crate::model::habit::Habit;
use crate::model::habit_event::HabitEvent;
use crate::model::plan::Plan;
use crate::model::reminder::ReminderDefinition;
use crate::model::timer::TimerDefinition;

#[derive(Debug, Clone)]
pub struct TodayState {
    pub date: chrono::NaiveDate,
    pub daily_log: Option<DailyLog>,
    pub check_ins: Vec<CheckIn>,
    pub habits: Vec<Habit>,
    pub habit_events: Vec<HabitEvent>,
    pub plan: Option<Plan>,
    pub alarms: Vec<AlarmDefinition>,
    pub reminders: Vec<ReminderDefinition>,
    pub timers: Vec<TimerDefinition>,
    pub context_documents: Vec<ContextDocument>,
    pub generated_at: AdiyutantDateTime,
}
```

**Builder:**

```rust
#[derive(Debug, Default)]
pub struct TodayStateBuilder {
    date: Option<chrono::NaiveDate>,
    daily_log: Option<DailyLog>,
    check_ins: Vec<CheckIn>,
    habits: Vec<Habit>,
    habit_events: Vec<HabitEvent>,
    plan: Option<Plan>,
    alarms: Vec<AlarmDefinition>,
    reminders: Vec<ReminderDefinition>,
    timers: Vec<TimerDefinition>,
    context_documents: Vec<ContextDocument>,
}

impl TodayStateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn date(mut self, date: chrono::NaiveDate) -> Self {
        self.date = Some(date);
        self
    }

    pub fn daily_log(mut self, log: DailyLog) -> Self {
        self.daily_log = Some(log);
        self
    }

    pub fn check_in(mut self, ci: CheckIn) -> Self {
        self.check_ins.push(ci);
        self
    }

    pub fn check_ins(mut self, cis: Vec<CheckIn>) -> Self {
        self.check_ins = cis;
        self
    }

    // ... аналогично для других полей

    pub fn build(self) -> TodayState {
        let date = self.date.unwrap_or_else(|| chrono::Utc::now().date_naive());
        TodayState {
            date,
            daily_log: self.daily_log,
            check_ins: self.check_ins,
            habits: self.habits,
            habit_events: self.habit_events,
            plan: self.plan,
            alarms: self.alarms,
            reminders: self.reminders,
            timers: self.timers,
            context_documents: self.context_documents,
            generated_at: AdiyutantDateTime::now(),
        }
    }
}
```

**Утилитарные методы TodayState:**

```rust
impl TodayState {
    /// Вернуть утренний check-in, если есть
    pub fn morning_check_in(&self) -> Option<&CheckIn> {
        self.check_ins.iter().find(|c| c.check_in_type == CheckInType::Morning)
    }

    /// Вернуть evening / shutdown check-in
    pub fn evening_check_in(&self) -> Option<&CheckIn> {
        self.check_ins.iter().find(|c| c.check_in_type == CheckInType::Evening || c.check_in_type == CheckInType::Shutdown)
    }

    /// Есть ли план на сегодня
    pub fn has_plan(&self) -> bool {
        self.plan.as_ref().map_or(false, |p| !p.items.is_empty())
    }

    /// Сколько habit-событий сегодня
    pub fn habit_event_count(&self) -> usize {
        self.habit_events.len()
    }

    /// Количество незавершённых пунктов плана
    pub fn pending_plan_items(&self) -> usize {
        self.plan.as_ref().map_or(0, |p| {
            p.items.iter().filter(|i| !matches!(i.status, crate::model::plan::PlanItemStatus::Done)).count()
        })
    }
}
```

**Unit tests:**
- Builder создаёт TodayState с датой по умолчанию (сегодня)
- Builder принимает daily_log
- Builder принимает check_ins
- morning_check_in() возвращает правильный check-in
- has_plan() возвращает false для пустого плана
- pending_plan_items() считает незавершённые

---

### Stage 2 — `local_rule_gateway.rs`

**Файл:** `AdiyutantCore/adiyutant_core/src/local_rule_gateway.rs`

**Архитектурное решение:** `LocalRuleAgentGateway` — in-memory engine, который принимает `TodayState` и возвращает список `ActionProposal`.

```rust
use crate::model::action_proposal::{ActionProposal, ProposalSource};
use crate::today_state::TodayState;
use serde_json::json;

/// Локальный rule engine без LLM.
/// Анализирует TodayState и генерирует ActionProposal на основе правил.
pub struct LocalRuleAgentGateway;

#[derive(Debug)]
pub struct RuleResult {
    pub proposals: Vec<ActionProposal>,
}

impl LocalRuleAgentGateway {
    /// Запустить все правила на данном TodayState.
    /// Возвращает список ActionProposal с источником LocalRule.
    pub fn evaluate(state: &TodayState) -> RuleResult {
        let mut proposals = Vec::new();
        let mut push = |pt: &str, payload: serde_json::Value| {
            proposals.push(ActionProposal::new(
                ProposalSource::LocalRule,
                pt.to_string(),
                payload,
            ));
        };

        // Правило 1: Нет morning check-in → предложить
        if state.morning_check_in().is_none() {
            push(
                "morning_checkin",
                json!({
                    "reason": "No morning check-in yet today",
                    "suggestion": "Do a morning check-in to start the day"
                }),
            );
        }

        // Правило 2: Sleep score < 6 → recovery mode
        if let Some(log) = &state.daily_log {
            if let Some(score) = log.sleep_score {
                if score < 6 {
                    push(
                        "set_recovery_mode",
                        json!({
                            "reason": "Low sleep score detected",
                            "sleep_score": score,
                            "suggestion": "Consider setting day mode to recovery"
                        }),
                    );
                }
            }
        }

        // Правило 3: No plan → предложить создать план
        if !state.has_plan() {
            push(
                "create_plan",
                json!({
                    "reason": "No plan for today yet",
                    "suggestion": "Create a plan to organize the day"
                }),
            );
        }

        // Правило 4: Есть привычки, но нет событий → предложить выполнить
        if !state.habits.is_empty() && state.habit_events.is_empty() {
            push(
                "do_habit",
                json!({
                    "reason": "Active habits exist but no events logged today",
                    "habit_count": state.habits.len(),
                    "suggestion": "Log at least one habit event today"
                }),
            );
        }

        // Правило 5: Evening and plan not done → предложить shutdown
        if state.evening_check_in().is_none() && state.pending_plan_items() > 0 {
            push(
                "evening_checkin",
                json!({
                    "reason": "Evening approaching with pending plan items",
                    "pending_items": state.pending_plan_items(),
                    "suggestion": "Review the day and do an evening check-in"
                }),
            );
        }

        // Правило 6: No evening check-in and it's late → предложить shutdown
        if state.evening_check_in().is_none() {
            // Проверяем, если есть shutdown-level event или время > 20:00
            // Упрощённо: нет evening/shutdown check-in, значит напоминаем
            let now = chrono::Utc::now();
            let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(0);
            if hour >= 20 || hour < 6 {
                push(
                    "shutdown",
                    json!({
                        "reason": "It's late and no shutdown check-in done",
                        "suggestion": "Do a shutdown check-in to close the day"
                    }),
                );
            }
        }

        RuleResult { proposals }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::check_in::CheckInType;
    use crate::model::daily_log::DailyLog;
    use crate::model::habit::Habit;
    use crate::today_state::TodayStateBuilder;

    #[test]
    fn no_morning_checkin_proposes_checkin() {
        let state = TodayStateBuilder::new()
            .date(chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap())
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(result.proposals.iter().any(|p| p.proposal_type == "morning_checkin"));
    }

    #[test]
    fn low_sleep_proposes_recovery_mode() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let mut log = DailyLog::new(date);
        log.sleep_score = Some(4);
        let state = TodayStateBuilder::new()
            .date(date)
            .daily_log(log)
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(result.proposals.iter().any(|p| p.proposal_type == "set_recovery_mode"));
    }

    #[test]
    fn no_plan_proposes_create_plan() {
        let state = TodayStateBuilder::new()
            .date(chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap())
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(result.proposals.iter().any(|p| p.proposal_type == "create_plan"));
    }

    #[test]
    fn habits_without_events_proposes_do_habit() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let habit = Habit::new("read".into());
        let state = TodayStateBuilder::new()
            .date(date)
            .habit(habit)
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(result.proposals.iter().any(|p| p.proposal_type == "do_habit"));
    }

    #[test]
    fn morning_checkin_present_skips_proposal() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let mut log = DailyLog::new(date);
        let checkin = crate::model::check_in::CheckIn::new(
            log.id, CheckInType::Morning, "good morning".into(),
        );
        let state = TodayStateBuilder::new()
            .date(date)
            .daily_log(log)
            .check_in(checkin)
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(!result.proposals.iter().any(|p| p.proposal_type == "morning_checkin"));
    }

    #[test]
    fn normal_sleep_does_not_propose_recovery() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let mut log = DailyLog::new(date);
        log.sleep_score = Some(8);
        let state = TodayStateBuilder::new()
            .date(date)
            .daily_log(log)
            .build();
        let result = LocalRuleAgentGateway::evaluate(&state);
        assert!(!result.proposals.iter().any(|p| p.proposal_type == "set_recovery_mode"));
    }
}
```

**Unit tests (минимум 7):**
1. No morning check-in → `morning_checkin` proposal
2. Low sleep → `set_recovery_mode` proposal
3. No plan → `create_plan` proposal
4. Active habits without events → `do_habit` proposal
5. Morning check-in present → no `morning_checkin` proposal
6. Normal sleep → no recovery proposal
7. Все proposals имеют `ProposalSource::LocalRule`

---

### Stage 3 — Export from `lib.rs`

**Файл:** `AdiyutantCore/adiyutant_core/src/lib.rs`

Добавить две строки:

```rust
pub mod today_state;
pub mod local_rule_gateway;
```

**Acceptance criteria:**
- [ ] `cargo check` проходит
- [ ] `cargo test` — все тесты зелёные
- [ ] `today_state::TodayState`, `today_state::TodayStateBuilder` публичны
- [ ] `local_rule_gateway::LocalRuleAgentGateway`, `local_rule_gateway::RuleResult` публичны

---

## 5. Required Test Levels

| Level | Name | Applicable? | Details |
|---|---|---|---|
| 1 | Static checks | ✅ | `cargo fmt --check`, `cargo check --workspace` |
| 2 | Unit tests | ✅ | `cargo test` — минимум 10 тестов |
| 3 | Component tests | ❌ | Чистые функции, нет компонентов |
| 4 | Integration tests | ❌ | Нет real dependencies (SQLite позже) |
| 5 | Stand smoke tests | ❌ | CLI не использует TodayState напрямую (Фаза 5) |
| 6 | UI automation | ❌ | CLI, не GUI |
| 7 | User scenarios | ❌ | Нет UI |
| 8 | Regression pack | ✅ | Тесты Фазы 1 и Фазы 2 должны проходить без изменений |
| 9 | Acceptance review | ✅ | Финальная сверка по чеклисту |

---

## 6. Test Stand

Не требуется. Все проверки — локальный `cargo`.

**Требования к окружению:**
- Rust toolchain stable (edition 2024)
- `cargo`, `rustc`, `rustfmt`

---

## 7. Files

### Создать (3 файла)

| # | File | Stage |
|---|---|---|
| 1 | `AdiyutantCore/adiyutant_core/src/today_state.rs` | 1 |
| 2 | `AdiyutantCore/adiyutant_core/src/local_rule_gateway.rs` | 2 |

### Изменить (1 файл)

| # | File | Change |
|---|---|---|
| 1 | `AdiyutantCore/adiyutant_core/src/lib.rs` | Добавить `pub mod today_state;` и `pub mod local_rule_gateway;` |

### Не изменять

- `model/*` — все модели Фазы 2
- `adiyutant_store/` — Фаза 4
- `adiyutant_cli/` — Фаза 5
- Корневые документы проекта

---

## 8. Acceptance Criteria (Final)

- [ ] `cargo check --workspace` — 0 errors
- [ ] `cargo test --workspace` — все тесты пройдены, включая тесты Фаз 1 и 2 (регрессия)
- [ ] `cargo fmt --all -- --check` — без расхождений
- [ ] `TodayState` создаётся через `TodayStateBuilder`
- [ ] `TodayState` имеет accessor-методы (`morning_check_in()`, `has_plan()`, `pending_plan_items()`)
- [ ] `LocalRuleAgentGateway::evaluate()` принимает `&TodayState`, возвращает `RuleResult`
- [ ] Правила генерируют `ActionProposal` с `ProposalSource::LocalRule`
- [ ] Все proposals имеют осмысленный `reason` и `suggestion`
- [ `adiyutant_store` и `adiyutant_cli` не изменены и компилируются
- [ ] Нет SQLite, LLM, backend, sync-кода

---

## 9. Risks

| Risk | Mitigation |
|---|---|
| `TodayStateBuilder` слишком громоздкий | Использовать `#[derive(Default)]` и опциональные поля |
| Правила в `LocalRuleAgentGateway` могут быть хрупкими | Каждое правило — отдельная функция с чётким условием |
| `lib.rs` экспортирует модули, но они не используются в CLI | Нормально — CLI будет использовать их в Фазе 5 |
| Часовой пояс для правила shutdown | Используем `chrono::Utc::now()`. Позже — local timezone |

---

## 10. Out of Scope Reminder

- ❌ SQLite — Фаза 4
- ❌ Репозитории, CRUD — Фаза 4
- ❌ CLI-команды — Фаза 5
- ❌ LLM, Agent Gateway (кроме LocalRule), backend, sync
- ❌ `TodayState` без builder (не нужен конструктор с 10+ аргументами)
- ❌ Изменения в `adiyutant_store` или `adiyutant_cli`
