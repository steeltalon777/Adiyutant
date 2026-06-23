# Scope: Phase 8.4 — Portable Bundle and Projects/Roadmaps

**Date:** 2026-06-22  
**Decision Makers:** User, Architect  
**Status:** Planned scope; implementation TZs are not created yet.

## Problem

Adiyutant is approaching Android work, but the Core still lacks two foundations that should shape the Android navigation and onboarding before UI code hardens around a simple daily todo list: a safe portable import/export format for Life Core settings, and first-class Project/Roadmap entities that can feed daily planning. If Android starts now, it will likely be designed around Today/Plan/Timers only, then later require navigation, onboarding, DTO, and storage rework when bundle import and project navigation arrive.

The desired workflow is:

1. User discusses Life Core, routines, rules, projects, and roadmaps with ChatGPT or another assistant.
2. Assistant helps prepare a portable Adiyutant bundle.
3. User imports the bundle into Adiyutant.
4. Core parses and validates the bundle strictly.
5. App shows preview/diff/import plan.
6. User confirms apply.
7. Adiyutant applies only Core-supported sections through Core-controlled logic, not direct AI database writes. In 8.4A this excludes project/roadmap persistence; in 8.4B it includes it.

## Current Evidence

- Current Rust workspace has three crates under `AdiyutantCore/`: `adiyutant_core`, `adiyutant_store`, `adiyutant_cli`.
- Current Core already has `ContextDocument`, `ChecklistTemplate`, `ChecklistRun`, `DayPlan`, `PlanItem`, `TaskCheckpoint`, `JournalEntry`, DTOs, service facade, and SQLite persistence.
- Current Core does **not** have first-class `Project`, `Roadmap`, `RoadmapPhase`, `RoadmapItem`, or portable bundle models.
- Current `PlanItem` is a daily/runtime entity; roadmap source links should not be embedded directly into it.
- Existing storage uses versioned SQLite migrations and embedded JSON only for embedded collections per ADR-0001.

## Selected Approach

Phase 8.4 is split into two separate implementation TZs:

1. **Phase 8.4A — Portable Bundle Contract**  
   Build the portable bundle format and safe import/export pipeline for existing Core domains. Project and roadmap bundle sections are parsed/validated/previewed only, then explicitly ignored on apply until 8.4B.

2. **Phase 8.4B — Projects/Roadmaps MVP**  
   Add first-class Project/Roadmap domain, persistence, service/DTO/CLI support, bundle apply/export support for project sections, and the `RoadmapItem → PlanItem` take-item cycle.

This keeps the data-transfer contract separate from the new domain model. The two TZs may be reviewed together, but implementation and acceptance must remain separate.

## Core Decisions

### Decision 1 — Bundle format

**Adiyutant settings/import format = Adiyutant Bundle v1.**

- Canonical format: `Adiyutant Bundle v1`.
- Transport format: `.adiyutant.zip`.
- MVP development/debug format: unpacked directory bundle with the same structure.
- Internal parsed model: `AdiyutantBundle`.
- A single giant YAML file is not canonical.

Planned directory structure:

```text
adiyutant-pack/
  manifest.yaml
  life-core.yaml
  routines.yaml
  rules.yaml
  checklists.yaml
  planning.yaml
  context/
    personal.md
    work.md
    health.md
  projects/
    adiyutant.yaml
    warehouse.yaml
  roadmaps/
    adiyutant-roadmap.yaml
    warehouse-roadmap.yaml
```

### Decision 2 — Manifest is authoritative

`manifest.yaml` is required and must explicitly declare bundle identity, schema version, title, timestamps, capabilities, and sections.

Example shape:

```yaml
schema_version: "adiyutant.bundle.v1"
bundle_id: "max-core-2026-06"
title: "Max Life Core"
created_at: "2026-06-22T10:00:00Z"

capabilities:
  projects: "preview-only"
  roadmaps: "preview-only"

sections:
  life_core: "life-core.yaml"
  routines: "routines.yaml"
  rules: "rules.yaml"
  checklists: "checklists.yaml"
  planning: "planning.yaml"
  context_dir: "context/"
  projects_dir: "projects/"
  roadmaps_dir: "roadmaps/"
```

Core must not guess which files are important when `manifest.yaml` is present. Unknown top-level files are warnings, not implicit sections.

### Decision 3 — Phase 8.4A applies existing domains only

Phase 8.4A applies:

- `life-core.yaml` as Life Core / profile context documents.
- `context/*.md` as `ContextDocument` records.
- `rules.yaml` as human-readable Life Core rules/context.
- `checklists.yaml` as `ChecklistTemplate` / checklist items.
- `routines.yaml` and `planning.yaml` as typed context/preferences if a suitable model exists; otherwise as context documents/preferences records according to the TZ design.

Phase 8.4A does **not** apply:

- `projects/*`
- `roadmaps/*`

In 8.4A, project/roadmap sections are valid bundle sections but must be reported as:

```text
status = preview_only
action = ignored_on_apply
```

### Decision 4 — Bundle rules are not executable rule-engine mappings

`rules.yaml` contains human Life Core / personal rules and context. It is not a mapping into current `LocalRuleGateway` behavior.

Current rule separation:

```text
Bundle rules ≠ LocalRuleGateway rules
```

Future work may add Rule Engine v2:

```text
human rules → structured rules → local suggestions
```

That future engine is out of scope for Phase 8.4.

### Decision 5 — Import modes

Phase 8.4A MVP import/export modes:

- `validate`
- `preview`
- `apply --mode append`
- `apply --mode merge`
- `export`

No destructive `replace` mode in MVP.

Definitions:

- `preview` = parse + validate + compute import plan + no writes.
- `apply` = parse + validate + compute the same import plan + writes.
- `preview` and `apply` must use the same internal planner.
- `append` adds new entities; slug/id conflicts do not update existing entities and must be reported as warning/error according to section semantics.
- `merge` updates existing entities by stable slug/id and adds absent entities; it must not delete absent entities.

### Decision 6 — Strict validation

Validation policy:

| Case | Result |
|---|---|
| Missing `manifest.yaml` | Error |
| Unsupported `schema_version` | Error |
| Duplicate slug/id | Error |
| Missing required field | Error |
| Invalid enum/status value | Error |
| Unknown top-level file | Warning |
| Unknown optional field | Warning, ignored |
| Unknown required-like section not declared in manifest | Warning unless explicitly required by schema |

The bundle format should be tolerant of assistant-generated extra notes, but strict about identifiers, schema version, required fields, and enum values.

### Decision 7 — Zip security

`.adiyutant.zip` import must be treated as untrusted input.

Mandatory MVP checks:

- Reject absolute paths.
- Reject paths containing `..` path traversal.
- Reject symlinks.
- Reject hardlinks if the zip library exposes them.
- Enforce max file count: 200.
- Enforce max uncompressed size: 10 MB.
- Enforce max single YAML file size: 1 MB.
- Enforce max single Markdown file size: 2 MB.
- Allow only `.yaml`, `.yml`, and `.md` extensions.
- Allow only declared/root-approved files and directories.
- Reject hidden executable-ish junk unless explicitly allowed by the bundle contract.

### Decision 8 — Planning bundle semantics

`planning.yaml` is for preferences, templates, and candidate hints. It is not a concrete `DayPlan` import format.

Allowed direction:

```yaml
default_mode: "manual_review"
daily_capacity:
  deep_tasks_max: 3
  light_tasks_max: 5

templates:
  - id: "normal-workday"
    title: "Normal workday"
    blocks:
      - kind: "morning"
        start: "06:30"
      - kind: "deep_work"
        start: "09:00"
        duration_minutes: 120

candidate_hints:
  - "Prefer project tasks before random backlog"
  - "If sleep score < 6, use recovery mode"
```

Forbidden in 8.4A: importing a dated task list as an actual persisted `DayPlan`.

### Decision 9 — Stable slug identity

Bundle YAML uses stable human/AI-readable slug/id strings. Core creates internal UUIDs.

Phase 8.4A requires stable merge keys for supported entities:

- `ChecklistTemplate` needs a stable slug with uniqueness.
- `ChecklistItem` identity should be stable within a template, either as item slug in embedded JSON or a future normalized model if chosen by the TZ.
- `ContextDocument` needs a stable source/external slug or import mapping; `title + doc_type` is acceptable only as a documented temporary fallback if the TZ explicitly rejects schema changes for that entity.

Phase 8.4B uniqueness requirements:

```text
projects.slug UNIQUE
roadmaps(project_id, slug) UNIQUE
roadmap_phases(roadmap_id, slug) UNIQUE
roadmap_items(phase_id, slug) UNIQUE
```

### Decision 10 — Import reports and history

Every validate/preview/apply operation returns a report DTO.

Minimum DTO concepts:

- `BundleValidationReportDto`
- `BundlePreviewDto`
- `BundleApplyReportDto`

Minimum fields:

- `bundle_id`
- `schema_version`
- `status`: `valid`, `valid_with_warnings`, `invalid`, `applied`, `failed`
- `errors[]`
- `warnings[]`
- `sections[]`
- `actions[]`

Phase 8.4A must persist apply history to SQLite:

```text
import_runs
  id
  bundle_id
  bundle_title
  schema_version
  mode
  status
  started_at
  finished_at
  summary_json
```

This enables future Settings → Import/Export → Last import report.

### Decision 11 — Apply transactionality

Import apply must be all-or-nothing for supported sections.

Required pipeline:

```text
parse → validate entire bundle → preview entire bundle → apply supported sections in transaction
```

Rules:

- Apply only proceeds when preview has no errors.
- Unsupported preview-only sections are skipped with explicit report actions.
- If any supported section write fails, the supported-section apply must roll back.
- For 8.4A, projects/roadmaps skipped-on-apply are warnings/report actions, not write failures.

### Decision 12 — Export scope

Phase 8.4A export includes only supported sections:

```text
manifest.yaml
life-core.yaml
routines.yaml
rules.yaml
checklists.yaml
planning.yaml
context/*.md
```

Phase 8.4A export must not attempt to export projects/roadmaps before 8.4B. It may advertise capabilities:

```yaml
capabilities:
  projects: "preview-only"
  roadmaps: "preview-only"
```

After 8.4B, capabilities become:

```yaml
capabilities:
  projects: "apply-export"
  roadmaps: "apply-export"
```

### Decision 13 — RoadmapItem to PlanItem link

Phase 8.4B uses a separate link table, not fields inside `PlanItem`.

Planned table:

```text
roadmap_plan_links
  id
  roadmap_item_id
  plan_item_id
  link_type: "taken_into_day"
  created_at
```

Reason: `PlanItem` is a daily/runtime entity. Project strategy and daily execution are separate ownership boundaries; the link belongs between them.

### Decision 14 — Android sequencing

Before Android Phase 9.1, Core should support:

```text
RoadmapItem → take into DayPlan → PlanItem → checkpoint/journal/day flow
```

Android Phase 9.1 may still use fake/golden JSON and read-only project screens, but navigation and screen design should assume the take-item operation exists in Core.

## In Scope

### Phase 8.4A — Portable Bundle Contract

- Adiyutant Bundle v1 contract.
- Directory bundle parser.
- `.adiyutant.zip` parser with security checks.
- Manifest validation.
- Validate/preview/apply/export pipeline.
- Import modes: append and merge.
- No destructive replace/delete mode.
- Apply existing supported sections:
  - Life Core / profile context.
  - Human-readable rules as context.
  - Checklists as templates/items.
  - Routines/planning as typed context/preferences if model exists; otherwise as context/preferences records according to TZ decision.
  - Markdown context documents.
- Projects/roadmaps parse + validate + preview-only.
- Import report DTOs and persisted `import_runs` history.
- CLI:
  - `adiyutant import validate <path>`
  - `adiyutant import preview <path>`
  - `adiyutant import apply <path> --mode append|merge`
  - `adiyutant export bundle <path>`
- Documentation:
  - `docs/contracts/adiyutant-bundle-v1.md`
  - `docs/contracts/examples/adiyutant-bundle-basic/`
- Tests for directory bundles, zip security, validation errors, preview/apply reports, export, and SQLite-backed apply.

### Phase 8.4B — Projects/Roadmaps MVP

- Domain models:
  - `Project`
  - `Roadmap`
  - `RoadmapPhase`
  - `RoadmapItem`
  - `RoadmapPlanLink`
- SQLite tables and migration:
  - `projects`
  - `roadmaps`
  - `roadmap_phases`
  - `roadmap_items`
  - `roadmap_plan_links`
- Store trait methods and `SqliteStore` implementation.
- DTOs for project/roadmap screens and take-item result.
- Service methods for listing/showing projects/roadmaps/items and taking roadmap item into a day plan.
- Bundle apply/export support for `projects/*` and `roadmaps/*`.
- `roadmap take-item` creates both `PlanItem` and `RoadmapPlanLink` transactionally.
- CLI:
  - `adiyutant project list`
  - `adiyutant project show <slug>`
  - `adiyutant roadmap list [project_slug]`
  - `adiyutant roadmap show <slug>`
  - `adiyutant roadmap items <roadmap_or_phase_slug>`
  - `adiyutant roadmap take-item <item_slug> --date today|tomorrow`
- Smoke scenario:
  - import bundle
  - project list
  - roadmap show
  - roadmap take-item
  - plan list

## Out of Scope

- Android implementation.
- Cloud/backend sync.
- Direct LLM provider calls from Core.
- Executable rule engine / `LocalRuleGateway` v2.
- Destructive import replace/delete modes.
- Full project-management CRUD UI.
- Jira/Trello clone features.
- Multi-user collaboration.
- Concrete dated DayPlan import from `planning.yaml`.
- Storing provider API keys in client projects.

## Success Criteria

1. Phase 8.4A and 8.4B are represented by two separate TZ files after this scope is accepted.
2. Phase 8.4A can validate, preview, apply append/merge, and export Adiyutant Bundle v1 for supported existing Core domains.
3. Phase 8.4A reports projects/roadmaps as preview-only and does not persist them.
4. Phase 8.4A records apply history in `import_runs` and returns structured report DTOs.
5. Phase 8.4A rejects unsafe zip bundles and invalid schema/slug/enum/required-field data.
6. Phase 8.4B adds persistent Project/Roadmap domain with stable slug uniqueness.
7. Phase 8.4B supports `RoadmapItem → PlanItem` through `roadmap_plan_links`, not through a field on `PlanItem`.
8. Phase 8.4B CLI smoke proves `import bundle → project list → roadmap show → roadmap take-item → plan list`.
9. Android Phase 9.1 can be designed around Today/Projects/Plan/Timers/Settings without needing to reinvent Core boundaries.

## Assumptions

| Assumption | Status | Validation |
|---|---|---|
| Bundle import/export is needed before Android to avoid wrong onboarding/navigation assumptions. | Reasonable | User workflow requires ChatGPT-assisted bundle generation and later Android import. |
| A directory bundle plus `.adiyutant.zip` transport is simpler long-term than one giant YAML. | Reasonable | Git/debug readability and future section growth favor multiple files. |
| Current Core has enough existing entities for 8.4A to apply useful data without Project/Roadmap domain. | Validated | Existing `ContextDocument` and `ChecklistTemplate` models are present. |
| Project/Roadmap domain does not exist yet. | Validated | Code survey found no first-class project/roadmap models. |
| `RoadmapItem → PlanItem` should be a link table. | Reasonable | Preserves ownership boundary between project strategy and daily execution. |
| All-or-nothing apply is feasible in local SQLite. | Reasonable | Existing store already uses composite transactional operations; exact transaction seam must be designed in TZ. |
| Existing `ContextDocument`/`ChecklistTemplate` may need slug/source identity additions. | Reasonable | Merge semantics require stable non-UUID keys; current models primarily use UUID/title/doc_type. |
| Zip library support for symlink/hardlink detection is available or can be approximated safely. | Unknown | Must be validated during 8.4A TZ/tooling selection. |

## Alternatives Considered

| Approach | Verdict | Reason |
|---|---|---|
| Do nothing; start Android Phase 9 immediately | Rejected | Android would likely harden around Today/Plan/Timers and later require navigation/onboarding/domain rework. |
| Single YAML as canonical import format | Rejected | It will become unreadable and hard to maintain in Git/ChatGPT workflows as sections grow. |
| Zip with arbitrary files | Rejected | Unsafe and hard to validate; manifest-driven bundle is required. |
| 8.4A and 8.4B as one TZ | Rejected | Mixes data-transfer contract and new domain/persistence risks into one oversized assignment. |
| Put project/roadmap data into `ContextDocument` temporarily | Rejected | Creates migration debt and blurs domain ownership. |
| Add roadmap source fields directly to `PlanItem` | Rejected | Couples daily runtime planning to project strategy and makes future one-to-many links awkward. |
| Build Project/Roadmap in memory first, persistence later | Rejected | Android/Core contract would drift when persistence and links are added. |
| Make bundle rules executable immediately | Rejected | Turns YAML into a rule programming language; Rule Engine v2 should be separate future work. |

## First Slice

The first implementation TZ should be:

```text
docs/tz/phase-8.4A-portable-bundle-contract.md
```

It should cover only the bundle contract and supported existing-domain import/export. It must include architecture review, zip security acceptance tests, report/history requirements, transactional apply, and explicit preview-only handling for projects/roadmaps.

The second implementation TZ should be:

```text
docs/tz/phase-8.4B-projects-roadmaps-mvp.md
```

It should depend on 8.4A and add Project/Roadmap persistence, DTOs, service methods, CLI smoke, bundle apply/export for project sections, and the take-item cycle.

## Next Step

After this scope is accepted, create two separate TZ files:

1. `docs/tz/phase-8.4A-portable-bundle-contract.md`
2. `docs/tz/phase-8.4B-projects-roadmaps-mvp.md`

Each TZ must include its own execution strategy, checklist, test ladder, real stand requirements, acceptance criteria, and architecture review gate before implementation.
