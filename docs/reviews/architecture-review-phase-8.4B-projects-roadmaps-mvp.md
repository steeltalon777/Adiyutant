# Architecture Review — Phase 8.4B Projects/Roadmaps MVP

**Date:** 2026-06-22  
**Reviewer:** Architect

## Verdict

Approved with conditions.

The plan is acceptable because it makes Projects/Roadmaps first-class persisted domain objects, keeps daily `PlanItem` ownership separate through a link table, and limits MVP CLI to list/show/take-item instead of building a project-management clone. No blockers remain in the TZ after the conditions below are included as implementation requirements.

## 🔴 Blockers

None.

## 🟡 Warnings

### 1. Slug selectors are ambiguous by design

- **Checklist item:** Public API surface — minimal but clear API.
- **Issue:** Required uniqueness is scoped: roadmaps are unique per project, items are unique per phase. CLI commands that accept a single slug can match multiple records.
- **Impact:** The CLI/service could take the wrong roadmap item into a day plan.
- **Recommendation:** Support unqualified slugs only when unambiguous. Add documented qualified selector syntax and tests for ambiguity errors.

### 2. Migration ordering depends on 8.4A

- **Checklist item:** Database migrations planned.
- **Issue:** 8.4B must add a migration after 8.4A's import/slugs migration.
- **Impact:** Starting 8.4B first can cause migration version conflicts and broken bundle capabilities.
- **Recommendation:** Treat 8.4A acceptance as a hard predecessor unless user explicitly authorizes speculative parallel work.

### 3. Foreign keys may not be enforced automatically

- **Checklist item:** Data & State — source of truth and integrity.
- **Issue:** SQLite `REFERENCES` clauses require `PRAGMA foreign_keys=ON`; current code survey did not confirm it is enabled.
- **Impact:** Link rows could reference missing roadmap items or plan items.
- **Recommendation:** Enable/test foreign-key enforcement or add equivalent integrity tests before accepting the migration.

### 4. Take-item duplicate behavior must be deterministic

- **Checklist item:** Failure Modes — partial or repeated user action.
- **Issue:** Users can run `roadmap take-item` twice for the same item/date.
- **Impact:** Duplicate plan items clutter the day and obscure project progress.
- **Recommendation:** Define behavior before implementation: return existing link/plan item or reject with conflict. Add integration tests.

### 5. Roadmap scope can easily expand into Jira/Trello

- **Checklist item:** Complexity — simplest solution that works.
- **Issue:** Full CRUD, assignments, workflow automation, comments, and boards are tempting but not needed for Android readiness.
- **Impact:** 8.4B becomes too large and delays Android.
- **Recommendation:** Keep MVP to imported data, read/list/show, and take-item cycle.

## 🔵 Notes

### 1. Journal entry on take-item is optional

- **Checklist item:** Cohesion — day flow ownership.
- **Issue:** Existing `add_plan_item` creates a checkpoint but does not necessarily create a journal entry.
- **Recommendation:** Do not force a journal entry on take-item unless tests prove it improves consistency. Existing start/done/checkpoint flows can own journal progression.

### 2. Embedded JSON is acceptable for criteria/dependencies/links

- **Checklist item:** Complexity — avoid premature normalization.
- **Issue:** Acceptance criteria, dependency slugs, and links are embedded collections.
- **Recommendation:** Use JSON columns for MVP, consistent with ADR-0001, unless querying those fields becomes required.
