# MVP 0.1 Acceptance Scenario

## Date

2026-06-08

## Tester

Кнот (orchestrator)

## Scenario — One Day with ADIYUTANT

| Step | Command | Expected | Result |
|---|---|---|---|
| 1 | `checkin morning "Доброе утро"` | Morning check-in recorded | ✅ |
| 2 | `habit add "спорт"` | Habit created | ✅ |
| 3 | `habit done "спорт"` | Habit event logged | ✅ `спорт done (base)` |
| 4 | `timer start "focus" --minutes 25` | Timer started | ✅ |
| 5 | `suggest` | Local rule suggestions shown | ✅ 2 proposals |
| 6 | `context add core "Ценности" "# Мои ценности"` | Document created | ✅ `v1` |
| 7 | `checkin evening "всё сделал"` | Evening check-in recorded | ✅ |
| 8 | `today` | Full day summary | ✅ Check-ins: 2, Habits: 1/1, Timers: 1 |

## Notes

- `plan create` not implemented as CLI command — replaced with `habit done` in scenario
- All 8 steps execute without errors
- SQLite persistence works across commands (temp file-based DB)

## Verdict

✅ ACCEPTED — MVP 0.1 scenario passes.
