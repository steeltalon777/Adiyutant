# TASK-0001: MVP 0.1 Independent Audit

**Дата:** 2026-06-08 (перепроверено 2026-06-09)  
**Аудитор:** QA agent (reviewer mode)  
**Объём:** Полный код, тесты, CLI, документация  
**Ветка:** `dev`  
**HEAD:** `2aa6071` (+ 1 fix `error.rs` clippy all-targets)

---

## Итоговый вердикт

| Пункт | Результат |
|---|---|
| Реальная структура workspace соответствует отчёту | ✅ |
| Все команды CLI работают | ✅ |
| CLI не падает на пустой базе | ✅ |
| SQLite schema идемпотентна | ✅ |
| CRUD не теряет данные между запусками | ✅ |
| TodayState собирает реальные данные из Store | ✅ |
| LocalRuleAgentGateway независим от UI/LLM | ✅ |
| Core не содержит платформенной логики | ✅ |
| Store не содержит бизнес-решений | ✅ |
| CLI не дублирует доменную логику | ✅ |
| Документация не обещает несуществующее | ✅ |
| Нет секретов, ключей, мусора | ✅ |

**MVP 0.1 Core пригоден как фундамент для Android shell.**

---

## 1. Workspace structure

| Отчёт | Факт |
|---|---|
| 3 крейта | `adiyutant_core`, `adiyutant_store`, `adiyutant_cli` |
| Core: 18 файлов (включая model/) | 18 .rs файлов |
| Store: 1 файл | `lib.rs` (2 088 строк) |
| CLI: 8 файлов + tests | `main.rs` + 7 модулей + `cli_smoke.rs` |

✅ Структура точно соответствует `ARCHITECTURE.md`.

## 2. CLI commands — manual smoke

Проверено на чистой SQLite-базе (`/tmp/adiyutant_manual_test.db`):

```
1. today (empty)          ✅ ── Today: 2026-06-09 — shows empty state
2. checkin morning        ✅ morning check-in recorded
3. habit add speech       ✅ Habit "speech" created
4. habit done speech      ✅ speech done (min)
5. reminder add           ✅ Reminder added (rule: закрыть день)
6. alarm add              ✅ Alarm "подъём" set for 06:30:00
7. suggest                ✅ 2 proposals: create_plan, shutdown
8. checkin shutdown       ✅ shutdown check-in recorded
9. today (full)           ✅ Check-ins: 2, Habits: 1/1, Alarms: 1, Reminders: 1
```

## 3. Empty database stability

`adiyutant today` на свежей БД → корректный вывод без паники.  
`adiyutant suggest` на пустой БД → предложения morning_checkin + create_plan.

## 4. SQLite persistence

Check-in создан в одном запуске: `checkin morning "test"`  
Второй запуск `today` → показывает Check-ins: 1 ✅

Данные переживают перезапуск процесса.

## 5. Clippy — all targets

Первичный прогон `--all-targets` нашёл 1 warning в тестовом коде:
- `error.rs:28` — `used unwrap() on Ok value`
- **Исправлено** (добавлен `#[allow]` на тест)

После фикса: `cargo clippy --workspace --all-targets -- -D warnings` — **0 errors**.

## 6. Static checks

| Проверка | Результат |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo check --workspace` | ✅ 0 errors |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 errors |
| `cargo test --workspace` | ✅ 103 tests pass |

## 7. Architectural boundaries

### Core (`adiyutant_core`)
- ✅ 10 доменных сущностей в `model/`
- ✅ `TodayState` — read-агрегатор
- ✅ `LocalRuleAgentGateway` — 6 правил, единственная ссылка на «LLM»: комментарий «without LLM»
- ❌ Нет вызовов LLM-провайдеров (grep: openai, gemini, deepseek, etc. — 0 совпадений)
- ❌ Нет платформенных зависимостей (grep: android, jni, wpf, tauri — 0 совпадений)

### Store (`adiyutant_store`)
- ✅ Только CRUD + schema DDL
- ❌ Нет валидации, бизнес-правил, проверок условий
- ✅ `schedule_rule`/`repeat_rule` — имена колонок, не бизнес-логика

### CLI (`adiyutant_cli`)
- ✅ Импортирует доменные типы из `adiyutant_core::model::*`
- ✅ Не переопределяет модели (не дублирует логику)
- ✅ `suggest` использует `LocalRuleAgentGateway::evaluate()` через Core
- ✅ `today` собирает `TodayState` через `common::build_today_state()`

## 8. Secrets & security

| Проверка | Результат |
|---|---|
| `.env` файлы | 0 |
| `.pem`/`.key` файлы | 0 |
| `api_key`/`api-key`/`Bearer` в коде | 0 |
| `sk-` токены | 0 |
| `API_KEY` в `.rs`/`.toml` | 0 |
| Упоминания API-ключей в документах | Только предупреждения «НЕ хранить ключи в клиенте» |

## 9. Documentation truthfulness

| Документ | Статус |
|---|---|
| `README.md` | MVP 0.1, Core → Implemented, метрики точны |
| `ARCHITECTURE.md` | Реальная структура, без bootstrap |
| `SPECIFICATION.md` | Фаза: MVP 0.1 |
| `AGENTS.md` | Актуальный стек, тесты, верификация |
| `AI_CONTEXT.md` | Реализованные компоненты перечислены |
| `SOLUTION_MAP.md` | Core → Implemented, breakdown по крейтам |
| `ROADMAP.md` | Phase 0–6 все Done |
| `docs/mvp-acceptance.md` | 8/8 шагов |

❗ `CalendarEvent` присутствует в `SPECIFICATION.md` §14.8 как доменная модель — но в текущем Core не реализован. Это нормально: спецификация описывает *продукт*, а не *реализованное*. README и ARCHITECTURE аккуратно не обещают CalendarEvent.

❗ `ADIYUTANT_ITERATIVE_ROADMAP.md` — исторический документ, содержит «bootstrap state». Не блокирует.

## 10. Найденные и исправленные в процессе

| # | Проблема | Статус |
|---|---|---|
| 1 | `clippy --all-targets` нашёл `unnecessary_literal_unwrap` в `error.rs` | ✅ Исправлено |

---

## Решение

**✅ MVP 0.1 Core прошёл независимый аудит. Фундамент пригоден.**

Рекомендация: `git tag v0.1.0-core-mvp` на HEAD после коммита фикса `error.rs`.
