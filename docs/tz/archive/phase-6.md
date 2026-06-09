# TZ: Phase 6 — Stabilization

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-6.md`

## Execution Strategy

- [ ] 🟡 Sequential execution recommended
- **Reason:** Code fixes (clippy), doc sync, integration tests, and acceptance scenario form a linear pipeline. Doc sync depends on code being stable. Acceptance scenario depends on everything else.

## Execution Checklist

- [x] 0. Context verified — baseline tests pass, fmt clean
- [x] 1. Fix 4 clippy warnings in Phase 3 code
- [x] 2. Add automated CLI integration/smoke tests
- [x] 3. Sync ARCHITECTURE.md — remove "bootstrap" language
- [x] 4. Sync remaining docs — verify README, AGENTS.md, SPECIFICATION.md reflect reality
- [x] 5. Static checks: `cargo fmt`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`
- [x] 6. Unit tests: `cargo test --workspace` — all tests pass
- [x] 7. Integration tests: CLI binary smoke test with real SQLite
- [x] 8. Regression: `cargo test --workspace` — 100+ tests green
- [x] 9. MVP acceptance scenario — execute and document
- [x] 10. MVP 0.1 release marker — tag or document
- [x] 11. Final acceptance review

## Check Rules

- Architect создаёт checklist и acceptance criteria.
- Executor agent отмечает пункты только после реализации и верификации.
- Если проверка не прошла — пункт остаётся незаполненным с причиной.
- QA verifier проверяет финальную приёмку только после review evidence.

---

## 1. Source Documents

| Документ | Роль |
|---|---|
| `ROADMAP.md` — Phase 6 | Источник scope |
| `ARCHITECTURE.md` | Требует синхронизации |
| `AGENTS.md` | Обновлён в аудите, проверить |
| `SPECIFICATION.md` | Проверить на drift |
| `docs/reports/phase-3-review.md` | Clippy warnings identified |
| `docs/reports/phase-5-review.md` | Integration tests deferred |
| `AdiyutantCore/adiyutant_core/src/local_rule_gateway.rs` | 3 clippy warnings |
| `AdiyutantCore/adiyutant_core/src/today_state.rs` | 1 clippy warning |

---

## 2. Goal

Довести локальное Core до качества MVP 0.1:

- **Test coverage pass** — исправить clippy warnings, добавить CLI integration тесты
- **Documentation sync** — синхронизировать ARCHITECTURE.md, SPECIFICATION.md и README с реализованным кодом
- **MVP acceptance scenario** — определить и выполнить end-to-end сценарий приёмки
- **MVP 0.1 release marker** — отметить релиз

---

## 3. Scope

### In scope

| # | Что | Где |
|---|---|---|
| 1 | Fix `collapsible_if` ×2 | `local_rule_gateway.rs` |
| 2 | Fix `manual_range_contains` | `local_rule_gateway.rs` |
| 3 | Fix `unnecessary_map_or` → `is_some_and` | `today_state.rs` |
| 4 | CLI integration tests: `tests/cli_smoke.rs` | `adiyutant_cli/` |
| 5 | Sync `ARCHITECTURE.md` | Project root |
| 6 | Sync `SPECIFICATION.md` (if exists) | Project root |
| 7 | Verify `AGENTS.md` is current | Project root |
| 8 | Document MVP acceptance scenario | `docs/mvp-acceptance.md` |
| 9 | Execute acceptance scenario | — |
| 10 | MVP 0.1 marker | ROADMAP, git tag |

### Out of scope

- ❌ New features beyond stabilization
- ❌ GUI/Web/Android — будущие фазы после MVP
- ❌ External LLM integration
- ❌ Performance optimization
- ❌ CI/CD pipeline setup

---

## 4. MVP Acceptance Scenario

Базовый сценарий пользователя (один день с ADIYUTANT):

1. `adiyutant checkin morning "..."` — начать день
2. `adiyutant habit add "спорт"` — добавить привычку
3. `adiyutant habit done "спорт"` — отметить выполнение привычки (plan create отложен до фазы с Plans)
4. `adiyutant timer start "focus" --minutes 25` — запустить таймер
5. `adiyutant suggest` — получить предложения
6. `adiyutant context add core "Ценности" "# Мои ценности"` — добавить документ
7. `adiyutant checkin evening "всё сделал"` — завершить день
8. `adiyutant today` — увидеть итоги дня

Каждый шаг должен выполниться без ошибок и произвести осмысленный вывод.

---

## 5. Test Ladder

| Level | What | Applicable |
|---|---|---|
| L1 — Unit | Existing 100 tests | ✅ regress |
| L2 — Integration | CLI binary smoke (real SQLite) | ✅ new |
| L3 — Static | cargo fmt, cargo check, cargo clippy -D warnings | ✅ required |
| L4 — Stand | MVP acceptance scenario | ✅ required |
| L5 — UI | N/A | ❌ no GUI |

---

## 6. Acceptance Criteria

- `cargo clippy --workspace -- -D warnings` — 0 errors
- `cargo test --workspace` — all tests pass (100+)
- `cargo fmt --all -- --check` — clean
- CLI acceptance scenario — all 8 steps pass
- ARCHITECTURE.md — no "bootstrap" or unimplemented claims
- AGENTS.md — reflects current stack
- MVP 0.1 marker in ROADMAP

## Output

Write only the TZ file. Report "DONE" when finished.
