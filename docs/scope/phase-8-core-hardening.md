# Scope: Phase 8 — Core Android Readiness Hardening

**Date:** 2026-06-19
**Decision Makers:** Architect, based on ChatGPT audit of v0.2.0/v0.3.0 dev branch

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-8.1-doc-sync-and-error-fix.md`

## Problem

Код находится в состоянии v0.3.0 (183 теста, CoreErrorDto, composite-транзакции, UI contract), но документация рассинхронизирована (`TASKS.md` — bootstrap, `pipeline-state.json` — PENDING RE-REVIEW, `AI_CONTEXT.md` — MVP 0.1), а service layer содержит 3 места игнорирования ошибок Store через `let _ = ...`. Следующий агент (Android или рефакторинг) увидит противоречивую картину и либо начнёт переделывать сделанное, либо построит UI на нестабильном контракте. Пользователь может потерять данные (checkpoint не создался, а UI показывает «задача создана»).

## In Scope (Phase 8 — три слайса)

### Slice 1: Doc Sync + Error Fix (этот документ)

- Синхронизировать `TASKS.md` с реальным состоянием (Phase 0-7 done, Phase 8 in progress)
- Обновить `pipeline-state.json`: добавить Phase 7, снять PENDING RE-REVIEW с Phase 6, установить current_phase
- Обновить `AI_CONTEXT.md`: 3 крейта, 15 команд, 15 таблиц, 148+ тестов, v0.3.0
- Убрать `let _ = self.store...` в `service.rs` — заменить на `?` или composite-транзакции
- Добавить composite Store-методы для add_plan_item + checkpoint и answer_checkpoint + next_cp + journal
- `cargo fmt`, `clippy --all-targets -D warnings`, `cargo test --workspace`

### Slice 2: DTO Contract Completion (отдельный TZ)

- Создать `HabitDto`, `TimerDto`, `ReminderDto`, `AlarmDto`, `ContextDocumentDto`, `SuggestionDto`
- Перевести все tuple-возвраты facade на DTO
- `get_suggestions() → Vec<SuggestionDto>`, `mark_habit_done() → HabitEventDto`

### Slice 3: Facade Split + Contract Tests (отдельный TZ)

- Разбить `service.rs` на подмодули: `service/today.rs`, `service/planning.rs`, `service/checkpoints.rs`, `service/checklists.rs`, `service/journal.rs`
- Golden JSON снэпшоты в `docs/contracts/examples/`
- `CurrentActivity` привязать к `planned_start/planned_end`

## Out Of Scope

- Android/Kotlin проект (после Phase 8)
- Новые фичи (Planning v2, Habits v2, etc.)
- LLM интеграция
- Sync/Backend
- Удаление legacy `Plan` (только документировать как deprecated)
- UX-документы (после Slice 2)

## Success Criteria

1. `TASKS.md` отражает реальное состояние: Phase 0-7 done, Phase 8 in progress
2. `pipeline-state.json` содержит Phase 7 со статусом done, Phase 6 без PENDING RE-REVIEW, current_phase указывает на Phase 8
3. `AI_CONTEXT.md` соответствует v0.3.0 (15 команд, 15 таблиц, 148+ тестов)
4. Ноль `let _ = self.store...` в service.rs — все ошибки Store либо propagated через `?`, либо обёрнуты в composite-транзакции
5. `cargo test --workspace` — все тесты проходят
6. `cargo clippy --all-targets -D warnings` — без warnings

## Assumptions

| Assumption | Status | Validation |
|---|---|---|
| Теги v0.2.0 и v0.3.0 существуют в git | Validated | `git tag -l` подтверждает оба тега |
| Phase 7.1 composite-методы в Store trait можно расширить двумя новыми | Reasonable | Store trait имеет 5 composite-методов; добавление 2 новых — паттерн, а не нововведение |
| `let _ =` в checkpoint/journal — единственные места игнорирования ошибок | Validated | Grep по `let _ = self.store` нашёл ровно 3 места |
| `AI_CONTEXT.md` устарел, но остальные doc-файлы в порядке | Partially validated | README.md и ARCHITECTURE.md обновлены в v0.2.0; AI_CONTEXT.md — нет |
| Документация в `docs/reports/` не требует обновления (build reports) | Reasonable | Build reports — исторические артефакты; достаточно добавить audit report |

## Alternatives Considered

| Approach | Verdict | Reason |
|---|---|---|
| Do nothing, jump to Android | Отклонено | Агент запутается в документации, данные потеряются, Android привяжется к tuple-API |
| Doc-only hardening (без error fix) | Отклонено | Решает путаницу, но не потерю данных — пользователь увидит «задача создана», а checkpoint не создался |
| All-at-once (все 10 пунктов аудита сразу) | Отклонено | Слишком большой объём для одного агента, высокий риск ошибок |
| Phased hardening (выбрано) | Принято | Три независимых слайса, каждый — законченная ценность, минимальный риск |

## Selected Approach

**Phased hardening: Slice 1 (Doc Sync + Error Fix) → Slice 2 (DTO Contract) → Slice 3 (Facade Split + Contract Tests).**

Slice 1 — минимальный слайс, который сильнее всего снижает риск хаоса для следующего агента. Сначала документы и error semantics, потом контракт, потом структура.

## First Slice

**Doc Sync + Error Fix:**
- TASKS.md, pipeline-state.json, AI_CONTEXT.md — синхронизация с v0.3.0
- 3 места `let _ = self.store...` → `?` или composite
- 2 новых composite Store-метода
- Полный прогон тестов и линтера

**Исключено из первого слайса:**
- Новые DTO (HabitDto, etc.)
- Распил service.rs
- Golden JSON снэпшоты
- Изменение сигнатур публичных методов (tuple → DTO)

## Next Step

Создать TZ: `docs/tz/phase-8.1-doc-sync-and-error-fix.md` и выполнить Slice 1.
