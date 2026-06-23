# TZ: Phase 8.4B — Projects/Roadmaps MVP

## Execution Strategy

- [ ] 🟢 Parallel execution recommended
- **Reason:** This TZ depends on 8.4A, but inside 8.4B the domain/storage, bundle schema/docs, and DTO/contract work can start independently after a short parent setup. Service/CLI take-item integration is sequential after domain and storage are available.

## Execution Checklist

- [ ] 0. Context verified
- [ ] 1. Architecture boundaries confirmed
- [ ] 2. Implementation stage 1 complete — domain models, DTO contract, and migration plan
- [ ] 3. Implementation stage 2 complete — persistence and bundle project/roadmap apply/export integrated
- [ ] 4. Implementation stage 3 complete — service methods, take-item transaction, and CLI commands
- [ ] 5. Unit/component tests complete
- [ ] 6. Integration tests with real dependencies complete
- [ ] 7. Stand smoke tests complete
- [ ] 8. UI automation tests complete or explicitly marked not applicable
- [ ] 9. User scenario tests complete
- [ ] 10. Regression checks complete
- [ ] 11. Documentation updated
- [ ] 12. Final acceptance review complete

## Check Rules

- Architect creates checklist items and acceptance criteria.
- Executor agents may check implementation and test items only after implementing the assigned work and attaching evidence.
- QA verifier may check final acceptance only after reviewing command output, smoke evidence, and unchecked/skipped items.
- Failed or unavailable checks stay unchecked with a blocker note.
- This TZ must not be started until Phase 8.4A is implemented and accepted, unless the user explicitly authorizes parallel speculative work.

## Dependency

Required predecessor:

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-8.4A-portable-bundle-contract.md`

Primary scope reference:

- `/home/makc/AI_sandbox/ADIYUTANT/docs/scope/phase-8.4-portable-bundle-and-projects-roadmaps.md`

Phase 8.4B extends the 8.4A bundle pipeline. It changes `projects/*` and `roadmaps/*` from preview-only to apply/export supported sections.

## Goal

Add first-class Project/Roadmap domain to Core, persist it in SQLite, expose stable DTO/service/CLI operations, support bundle apply/export for project sections, and implement the critical cycle:

```text
RoadmapItem → take into DayPlan → PlanItem → checkpoint/journal/day flow
```

This makes Adiyutant a project navigator feeding daily planning, not only a daily checklist/timer tool.

## Non-Goals

- No Android implementation.
- No full project-management UI.
- No Jira/Trello clone features.
- No multi-user collaboration or sync.
- No executable rule engine.
- No destructive bundle replace/delete mode.
- No manual full CRUD CLI beyond the required read/take-item smoke commands.
- No direct LLM/provider calls.

## Domain Model Requirements

### Project

```text
Project
  id
  slug
  title
  description
  status: active / paused / done / archived
  priority
  why
  created_at
  updated_at
```

### Roadmap

```text
Roadmap
  id
  slug
  project_id
  title
  horizon: week / month / quarter / custom
  status
  created_at
  updated_at
```

### RoadmapPhase

```text
RoadmapPhase
  id
  slug
  roadmap_id
  title
  order_index
  status: planned / active / done / skipped
```

### RoadmapItem

```text
RoadmapItem
  id
  slug
  phase_id
  title
  description
  status: planned / active / blocked / done / skipped
  priority
  acceptance_criteria
  depends_on
  links
  created_at
  updated_at
```

`acceptance_criteria`, `depends_on`, and `links` may use embedded JSON collections in SQLite, consistent with ADR-0001, unless the executor documents a stronger reason to normalize them now.

### RoadmapPlanLink

```text
RoadmapPlanLink
  id
  roadmap_item_id
  plan_item_id
  link_type: taken_into_day
  created_at
```

Do not add `project_id`, `roadmap_id`, `phase_id`, or `roadmap_item_id` fields directly to `PlanItem` in this TZ. The link table owns the relationship between project strategy and daily execution.

## SQLite Requirements

Add a new migration after 8.4A's migration version.

Required tables:

```text
projects
roadmaps
roadmap_phases
roadmap_items
roadmap_plan_links
```

Required uniqueness:

```text
projects.slug UNIQUE
roadmaps(project_id, slug) UNIQUE
roadmap_phases(roadmap_id, slug) UNIQUE
roadmap_items(phase_id, slug) UNIQUE
```

Required referential direction:

```text
roadmaps.project_id → projects.id
roadmap_phases.roadmap_id → roadmaps.id
roadmap_items.phase_id → roadmap_phases.id
roadmap_plan_links.roadmap_item_id → roadmap_items.id
roadmap_plan_links.plan_item_id → plan_items.id
```

If SQLite foreign keys are not currently enabled for the connection, the executor must either enable them safely or document and test equivalent referential integrity behavior. Do not assume `REFERENCES` clauses alone are enforced.

## Store Requirements

Add methods to `Store` and implement them in `SqliteStore` for:

- Insert/update/get/list projects.
- Get project by slug.
- Insert/update/get/list roadmaps.
- List roadmaps by project.
- Get roadmap by project+slug and/or unambiguous slug selector.
- Insert/update/get/list roadmap phases.
- Insert/update/get/list roadmap items.
- Resolve roadmap item by selector with ambiguity handling.
- Insert/list roadmap-plan links.
- Composite transactional take-item operation.

Required composite operation behavior:

```text
take roadmap item into date:
  get/create DailyLog for target date
  create PlanItem
  create initial StartCheck TaskCheckpoint
  create RoadmapPlanLink
  optionally update RoadmapItem status to active if it was planned
  commit all or rollback all
```

Journal behavior:

- Taking an item into a day does not have to create a journal entry if it follows existing `add_plan_item` semantics.
- The resulting `PlanItem` must still enter the existing checkpoint/start/done/journal flow when the user starts/completes/answers checkpoints.
- If executor adds a journal entry on take-item, it must be covered by tests and not duplicate later task-start journal entries.

## DTO Requirements

Add stable flat DTOs, with strings for ids/dates/times and no `Id<T>` in DTOs.

Minimum DTOs:

```text
ProjectDto
RoadmapDto
RoadmapPhaseDto
RoadmapItemDto
RoadmapPlanLinkDto
RoadmapTakeItemResultDto
ProjectDetailDto
RoadmapDetailDto
```

Minimum fields:

- Project DTO: `id`, `slug`, `title`, `description`, `status`, `priority`, `why`, `created_at`, `updated_at`.
- Roadmap DTO: `id`, `slug`, `project_slug`, `title`, `horizon`, `status`, `created_at`, `updated_at`.
- Phase DTO: `id`, `slug`, `roadmap_slug`, `title`, `order_index`, `status`.
- Item DTO: `id`, `slug`, `phase_slug`, `title`, `description`, `status`, `priority`, `acceptance_criteria`, `depends_on`, `links`, `created_at`, `updated_at`.
- Take result: `roadmap_item`, `plan_item`, `link`, `target_date`.

Update Android/Core contract docs or add a Phase 8.4B contract supplement with these DTOs and golden examples.

## Service Requirements

Add facade methods on `AdiyutantCoreService` or equivalent service boundary.

Required capabilities:

```text
list_projects() -> Vec<ProjectDto>
get_project(slug) -> ProjectDetailDto
list_roadmaps(project_slug?) -> Vec<RoadmapDto>
get_roadmap(selector) -> RoadmapDetailDto
list_roadmap_items(selector) -> Vec<RoadmapItemDto>
take_roadmap_item(item_selector, target_date) -> RoadmapTakeItemResultDto
```

Selector rules:

- Unqualified slugs are allowed only when unambiguous.
- If a roadmap or item slug matches multiple records, return `InvalidInput` with enough information for the CLI/user to choose a qualified selector.
- Qualified selectors may use a documented syntax such as `project/roadmap`, `roadmap/phase/item`, or another explicit form. Document the chosen syntax in CLI help and contract docs.

Target date rules for take-item:

- CLI MVP supports `today` and `tomorrow`.
- Service may accept an explicit `NaiveDate` or date string internally, but CLI must expose only the MVP values unless the executor documents a safe extension.
- Taking the same roadmap item into the same day twice should not silently duplicate work. It must either return an existing link/plan item or return a clear conflict report/error. The chosen behavior must be documented and tested.

## Bundle Integration Requirements

Extend Phase 8.4A bundle support:

- `projects/*` becomes apply/export supported.
- `roadmaps/*` becomes apply/export supported.
- Export manifest capabilities become:

```yaml
capabilities:
  projects: "apply-export"
  roadmaps: "apply-export"
```

Bundle apply behavior:

- Project/roadmap imports use stable slugs, not external UUIDs.
- `append` creates missing project/roadmap data and reports conflicts without overwriting existing slug-matched data.
- `merge` updates existing slug-matched data and creates missing records without deleting absent records.
- Project/roadmap apply participates in the same transaction model as 8.4A supported sections.
- Roadmap item dependency references by slug must be validated; unresolved dependencies are errors or explicit warnings according to documented schema semantics. For MVP, prefer errors for dependencies within the same roadmap that cannot be resolved.

## CLI Requirements

Add commands:

```bash
adiyutant project list
adiyutant project show <slug>
adiyutant roadmap list [project_slug]
adiyutant roadmap show <slug_or_selector>
adiyutant roadmap items <roadmap_or_phase_selector>
adiyutant roadmap take-item <item_slug_or_selector> --date today|tomorrow
```

CLI output requirements:

- Deterministic summaries suitable for smoke tests.
- Non-zero exit on ambiguous selectors, invalid slugs, missing records, and failed take-item.
- Show slug, title, status, and relevant parent context.
- `take-item` output must include roadmap item slug, created/reused plan item id, target date, and link id.

Full manual CRUD is out of scope. Creating/updating projects and roadmaps in MVP happens through bundle import.

## Exact Files / Areas in Scope

Expected source areas for implementation agents:

- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/model/project.rs` new file
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/model/roadmap.rs` new file or equivalent split files
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/model/mod.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/dto.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/store.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/service/` new project/roadmap modules
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/bundle/` 8.4A bundle extension
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_store/src/lib.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/src/main.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/src/commands/project.rs` new file
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/src/commands/roadmap.rs` new file
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/src/commands/mod.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/tests/cli_smoke.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/docs/contracts/adiyutant-bundle-v1.md`
- `/home/makc/AI_sandbox/ADIYUTANT/docs/contracts/android-core-api.md` or a new 8.4B supplement under `docs/contracts/`
- `/home/makc/AI_sandbox/ADIYUTANT/docs/contracts/examples/adiyutant-bundle-basic/` or a new projects bundle example

## Parallel Work Units

### Stage 0 — Parent setup

Owner: primary executor/orchestrator.

Responsibilities:

- Verify 8.4A is accepted.
- Confirm migration version after 8.4A.
- Define selector syntax for ambiguous slugs.
- Reserve shared files (`dto.rs`, `store.rs`, `lib.rs`, CLI `main.rs`) for parent integration or explicit single ownership.

### Stage 1 — Parallel units

#### Unit A — Domain models and DTO contract

Owned files/areas:

- New project/roadmap model files.
- DTO contract docs/examples.

Deliverables:

- Domain enums and constructors.
- DTO mappings.
- Contract docs/golden JSON for Projects/Roadmaps read-only screens and take-item result.

Verification:

- Unit tests for enum round-trips and DTO serialization.

#### Unit B — SQLite persistence and Store methods

Owned files/areas:

- `store.rs`
- `adiyutant_store/src/lib.rs`
- store tests.

Deliverables:

- New migration.
- CRUD/list/lookup methods.
- Unique constraints.
- Link table.
- Composite take-item transaction.

Verification:

- In-memory SQLite migration and persistence tests.
- Duplicate slug constraint tests.
- Transaction rollback test for take-item.

#### Unit C — Bundle schema extension for projects/roadmaps

Owned files/areas:

- `adiyutant_core/src/bundle/` project/roadmap section support.
- `docs/contracts/adiyutant-bundle-v1.md`
- bundle examples.

Deliverables:

- Parse/validate/apply/export support for `projects/*` and `roadmaps/*`.
- Capability update from preview-only to apply-export.

Verification:

- Bundle validation tests for project/roadmap sections, dependency references, duplicate slugs, append/merge semantics.

### Stage 2 — Service, CLI, and integration

Owner: primary executor/orchestrator or one assigned implementer.

Responsibilities:

- Wire service methods.
- Implement selector ambiguity behavior.
- Implement CLI commands.
- Add full smoke scenario.
- Run all checks.

## Test Strategy

### Static checks

Required:

```bash
cargo fmt
cargo check --workspace
cargo clippy --all-targets -- -D warnings
```

Run from:

```text
/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore
```

### Unit tests

Required coverage:

- Project/Roadmap enum parsing/round-trips.
- DTO mapping and serialization.
- Selector parsing and ambiguity detection.
- Bundle validation for project/roadmap files.
- `append` vs `merge` behavior for project/roadmap sections.

Command:

```bash
cargo test -p adiyutant_core
```

### Component tests

Required coverage:

- `list_projects`, `get_project`, `list_roadmaps`, `get_roadmap`, `list_roadmap_items` service methods.
- `take_roadmap_item` creates/reuses or rejects duplicate same-day links according to documented behavior.
- Take-item result contains roadmap item, plan item, link, and target date.
- Bundle preview/apply now includes project/roadmap actions.

Command:

```bash
cargo test --workspace roadmap
```

### Integration tests with real dependencies

Required:

- SQLite in-memory tests for new migration.
- Unique slug constraint tests.
- Foreign-key/referential integrity behavior tests.
- Composite take-item transaction rollback test.
- Bundle import creates project/roadmap rows and export reproduces supported sections.

Command:

```bash
cargo test -p adiyutant_store
```

### Real stand smoke tests

Required because this touches runtime CLI behavior and SQLite persistence.

Stand:

- Database: SQLite file in `/tmp` or test temp directory.
- Lifecycle: fresh DB per smoke run.
- Env var: `ADIYUTANT_DB_PATH`.
- Seed data: project/roadmap bundle example imported through 8.4A/8.4B import command.
- Services: no long-running services.
- Health check: `adiyutant project list` after import.
- Cleanup: remove temp DB and temp export bundle.

Smoke sequence:

```bash
cargo run -p adiyutant_cli -- import apply ../docs/contracts/examples/adiyutant-bundle-basic --mode append
cargo run -p adiyutant_cli -- project list
cargo run -p adiyutant_cli -- project show adiyutant
cargo run -p adiyutant_cli -- roadmap list adiyutant
cargo run -p adiyutant_cli -- roadmap show adiyutant-roadmap
cargo run -p adiyutant_cli -- roadmap items adiyutant-roadmap
cargo run -p adiyutant_cli -- roadmap take-item <known_item_slug_or_selector> --date tomorrow
cargo run -p adiyutant_cli -- plan list
```

Executor must replace `<known_item_slug_or_selector>` with a slug from the example bundle and report exact commands.

### UI automation

Not applicable in this TZ. No Android/web/desktop UI is implemented. Leave checklist item unchecked or mark with explicit N/A evidence in the completion report according to project workflow.

### User scenario tests

Required scenario:

```text
Import Adiyutant project bundle → inspect Projects/Roadmap → take roadmap item into tomorrow → verify DayPlan contains created PlanItem → start/done flow still works
```

Evidence must show:

- Imported project visible by slug.
- Roadmap phases/items visible.
- `roadmap_plan_links` relationship exists after take-item.
- Plan item appears in the selected day plan.
- Existing plan item lifecycle commands still work.

### Regression checks

Required:

```bash
cargo test --workspace
```

Also rerun existing CLI smoke tests and any Phase 8.4A bundle tests.

### Acceptance review

Executor completion report must include evidence table:

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static | `cargo fmt`, `cargo check --workspace`, `cargo clippy --all-targets -- -D warnings` | pass/fail | log/summary |
| Unit | `cargo test -p adiyutant_core` | pass/fail | log/summary |
| DB integration | `cargo test -p adiyutant_store` | pass/fail | migration/link note |
| Workspace regression | `cargo test --workspace` | pass/fail | log/summary |
| Stand smoke | CLI import/project/roadmap/take-item sequence | pass/fail | DB path + command output |
| UI automation | N/A | skipped | no UI in TZ |

## Acceptance Criteria

1. Persistent Project/Roadmap domain exists with the required models, DTOs, Store methods, and SQLite tables.
2. Slug uniqueness is enforced according to project/roadmap/phase/item ownership boundaries.
3. Project/roadmap bundle sections are apply/export supported and manifest capabilities are updated to `apply-export`.
4. `append` and `merge` project/roadmap behavior follows 8.4A import semantics and never deletes absent records.
5. CLI can list/show projects and roadmaps imported from a bundle.
6. `roadmap take-item` creates a `PlanItem`, initial checkpoint, and `RoadmapPlanLink` transactionally.
7. No roadmap source fields are added directly to `PlanItem`.
8. Ambiguous slug selectors return clear errors instead of choosing arbitrary records.
9. Duplicate same-day take-item behavior is deterministic and tested.
10. Bundle import/export, SQLite migration, and take-item rollback are covered by integration tests.
11. Real stand smoke proves `import bundle → project list → roadmap show → roadmap take-item → plan list`.
12. Existing planning/checkpoint/journal regression tests still pass.
13. Existing `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test --workspace` pass.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Slug ambiguity across projects/roadmaps | Require explicit selector syntax and ambiguity errors. |
| 8.4B starts before 8.4A import pipeline stabilizes | Keep 8.4A as required predecessor. |
| Roadmap becomes a Jira clone | Restrict CLI to list/show/take-item; no full CRUD in MVP. |
| Daily planning gets coupled to project strategy | Use `roadmap_plan_links`, not `PlanItem` source fields. |
| Duplicate take-item creates repeated daily work | Define and test deterministic duplicate behavior. |
| Foreign keys are declared but not enforced | Enable/test SQLite FK behavior or document equivalent integrity checks. |

## Architecture Review

- Review file: `/home/makc/AI_sandbox/ADIYUTANT/docs/reviews/architecture-review-phase-8.4B-projects-roadmaps-mvp.md`
- Verdict: Approved with conditions.
- Blockers: none after this TZ includes selector ambiguity handling, link-table ownership, migration/foreign-key checks, take-item transactionality, and strict dependency on 8.4A.

Executor must read the review before implementation and keep its warnings tracked in the completion report.
